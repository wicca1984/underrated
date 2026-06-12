use crate::dom::Dom;
use crate::infra::NodeId;
use crate::layout::{LayoutBox, find_box_rect, layout_document};
use crate::paint::invalidate::DirtyRegion;
use crate::style::CategorizedComputedStyle;
use std::collections::HashMap;

/// Explicit batched dirty-flush: consumes accumulated layout-dirty nodes, performs ONE
/// batch relayout, and returns the new layout tree together with the paint DirtyRegion
/// covering the dirty nodes. Returns `None` when there is no dirty state (no relayout performed).
pub fn flush_dirty(
    dom: &mut Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    viewport_width: f32,
) -> Option<(LayoutBox, DirtyRegion)> {
    if !dom.has_dirty() {
        return None;
    }

    let nodes = dom.take_dirty();
    let layout = layout_document(dom, styles, viewport_width);

    let mut region = DirtyRegion::new();
    for n in nodes {
        if let Some(rect) = find_box_rect(&layout, n) {
            region.add(rect);
        }
    }

    Some((layout, region))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_stylesheet;
    use crate::dom::NodeData;
    use crate::style::compute_styles;

    #[test]
    fn test_flush_no_dirty() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let stylesheet = parse_stylesheet("div { display: block; width: 100px; height: 100px; }");
        let styles = compute_styles(&dom, &stylesheet);

        // Initially no dirty state.
        assert!(!dom.has_dirty());
        let result = flush_dirty(&mut dom, &styles, 500.0);
        assert!(result.is_none());
        assert!(!dom.has_dirty()); // should stay false and not drain anything
    }

    #[test]
    fn test_flush_with_dirty() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let stylesheet = parse_stylesheet("div { display: block; width: 100px; height: 100px; }");
        let styles = compute_styles(&dom, &stylesheet);

        // Mark dirty
        dom.mark_dirty(div);
        assert!(dom.has_dirty());

        let result = flush_dirty(&mut dom, &styles, 500.0);
        assert!(result.is_some());
        let (layout, region) = result.unwrap();

        // Check region bounds
        assert!(!region.is_empty());
        let rect = find_box_rect(&layout, div);
        assert!(rect.is_some());
        assert_eq!(region.bounds(), rect);

        // has_dirty should be cleared now
        assert!(!dom.has_dirty());

        // A second flush immediately after should be None (idempotent)
        let second_result = flush_dirty(&mut dom, &styles, 500.0);
        assert!(second_result.is_none());
    }

    #[test]
    fn test_flush_multiple_dirty() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(div1, div2);

        let stylesheet = parse_stylesheet(
            "
            div { display: block; width: 100px; height: 50px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        // Mark both dirty
        dom.mark_dirty(div1);
        dom.mark_dirty(div2);

        let result = flush_dirty(&mut dom, &styles, 500.0);
        assert!(result.is_some());
        let (layout, region) = result.unwrap();

        let rect1 = find_box_rect(&layout, div1).unwrap();
        let rect2 = find_box_rect(&layout, div2).unwrap();

        let union_rect = rect1.union(rect2);
        assert_eq!(region.bounds(), Some(union_rect));
    }
}
