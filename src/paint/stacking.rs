use crate::css::values::{CssValue, ZIndex};
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::style::ComputedStyle;
use std::collections::HashMap;

/// Sibling paint entry carrying a reference to the LayoutBox, its computed z-index, and its original document-order index.
pub struct SiblingPaintEntry<'a> {
    pub layout_box: &'a LayoutBox,
    pub z_index: ZIndex,
    pub doc_index: usize,
}

/// Helper to get the computed `z-index` for a LayoutBox.
/// If the box has no node or style, defaults to `ZIndex::Auto`.
pub fn get_z_index(layout_box: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> ZIndex {
    layout_box
        .node
        .and_then(|node_id| styles.get(&node_id))
        .and_then(|style| style.get("z-index"))
        .and_then(|val| match val {
            CssValue::ZIndex(z) => Some(*z),
            _ => None,
        })
        .unwrap_or(ZIndex::Auto)
}

/// Takes a slice of sibling layout boxes and returns them in CSS painting order:
/// a stable sort where lower z-index paints first, higher paints last, and `auto`/0 keep document order among equals.
///
/// If all siblings have `z-index: auto` (which is the overwhelmingly common case),
/// this function returns references to the siblings in their original document order without performing any sorting.
pub fn sort_siblings<'a>(
    children: &'a [LayoutBox],
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Vec<&'a LayoutBox> {
    // If empty, return empty
    if children.is_empty() {
        return Vec::new();
    }

    // Check if there are any non-auto z-indices.
    // If not, we can just return references to the children in document order.
    let mut has_non_auto = false;
    let mut entries = Vec::with_capacity(children.len());

    for (index, child) in children.iter().enumerate() {
        let z = get_z_index(child, styles);
        if !matches!(z, ZIndex::Auto) {
            has_non_auto = true;
        }
        entries.push(SiblingPaintEntry {
            layout_box: child,
            z_index: z,
            doc_index: index,
        });
    }

    if !has_non_auto {
        // Fast path: no non-auto z-index found, return in original document order.
        return children.iter().collect();
    }

    // Stable sort by z-index value (Auto / Index(0) are both 0).
    entries.sort_by(|a, b| {
        let val_a = match a.z_index {
            ZIndex::Auto => 0,
            ZIndex::Index(v) => v,
        };
        let val_b = match b.z_index {
            ZIndex::Auto => 0,
            ZIndex::Index(v) => v,
        };

        // Use stable sort: if values are equal, compare original document index.
        match val_a.cmp(&val_b) {
            std::cmp::Ordering::Equal => a.doc_index.cmp(&b.doc_index),
            other => other,
        }
    });

    entries.into_iter().map(|e| e.layout_box).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{Dom, NodeData};
    use crate::geom::Rect;

    // Helper to construct a mock LayoutBox for testing
    fn make_mock_box(node_id: Option<NodeId>) -> LayoutBox {
        LayoutBox {
            node: node_id,
            rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            children: Vec::new(),
            text: None,
        }
    }

    // Helper to construct mock style map with z-index values
    fn make_mock_styles(data: &[(NodeId, ZIndex)]) -> HashMap<NodeId, ComputedStyle> {
        let mut map = HashMap::new();
        for &(id, z) in data {
            let mut style = ComputedStyle::default();
            style.insert("z-index".to_string(), CssValue::ZIndex(z));
            map.insert(id, style);
        }
        map
    }

    #[test]
    fn test_zorder_stable_for_all_auto() {
        let mut dom = Dom::new();
        let id1 = dom.create_node(NodeData::Text("1".into()));
        let id2 = dom.create_node(NodeData::Text("2".into()));
        let id4 = dom.create_node(NodeData::Text("4".into()));

        let box1 = make_mock_box(Some(id1));
        let box2 = make_mock_box(Some(id2));
        let box3 = make_mock_box(None);
        let box4 = make_mock_box(Some(id4));

        let children = vec![box1, box2, box3, box4];
        let styles = make_mock_styles(&[
            (id1, ZIndex::Auto),
            (id2, ZIndex::Auto),
            (id4, ZIndex::Auto),
        ]);

        let sorted = sort_siblings(&children, &styles);
        assert_eq!(sorted.len(), 4);
        assert_eq!(sorted[0].node, Some(id1));
        assert_eq!(sorted[1].node, Some(id2));
        assert_eq!(sorted[2].node, None);
        assert_eq!(sorted[3].node, Some(id4));
    }

    #[test]
    fn test_zorder_sorts_ascending() {
        let mut dom = Dom::new();
        let id1 = dom.create_node(NodeData::Text("1".into()));
        let id2 = dom.create_node(NodeData::Text("2".into()));
        let id3 = dom.create_node(NodeData::Text("3".into()));
        let id4 = dom.create_node(NodeData::Text("4".into()));

        let box1 = make_mock_box(Some(id1)); // z-index: 2
        let box2 = make_mock_box(Some(id2)); // z-index: -1
        let box3 = make_mock_box(Some(id3)); // z-index: auto (treated as 0)
        let box4 = make_mock_box(Some(id4)); // z-index: 1

        let children = vec![box1, box2, box3, box4];
        let styles = make_mock_styles(&[
            (id1, ZIndex::Index(2)),
            (id2, ZIndex::Index(-1)),
            (id3, ZIndex::Auto),
            (id4, ZIndex::Index(1)),
        ]);

        let sorted = sort_siblings(&children, &styles);
        assert_eq!(sorted.len(), 4);
        assert_eq!(sorted[0].node, Some(id2)); // -1
        assert_eq!(sorted[1].node, Some(id3)); // 0 (auto)
        assert_eq!(sorted[2].node, Some(id4)); // 1
        assert_eq!(sorted[3].node, Some(id1)); // 2
    }

    #[test]
    fn test_zorder_stable_tiebreak() {
        let mut dom = Dom::new();
        let id1 = dom.create_node(NodeData::Text("1".into()));
        let id2 = dom.create_node(NodeData::Text("2".into()));
        let id3 = dom.create_node(NodeData::Text("3".into()));
        let id4 = dom.create_node(NodeData::Text("4".into()));

        let box1 = make_mock_box(Some(id1)); // z-index: 5 (first)
        let box2 = make_mock_box(Some(id2)); // z-index: 5 (second)
        let box3 = make_mock_box(Some(id3)); // z-index: -2
        let box4 = make_mock_box(Some(id4)); // z-index: -2

        let children = vec![box1, box2, box3, box4];
        let styles = make_mock_styles(&[
            (id1, ZIndex::Index(5)),
            (id2, ZIndex::Index(5)),
            (id3, ZIndex::Index(-2)),
            (id4, ZIndex::Index(-2)),
        ]);

        let sorted = sort_siblings(&children, &styles);
        assert_eq!(sorted.len(), 4);
        assert_eq!(sorted[0].node, Some(id3)); // -2 (first)
        assert_eq!(sorted[1].node, Some(id4)); // -2 (second)
        assert_eq!(sorted[2].node, Some(id1)); // 5 (first)
        assert_eq!(sorted[3].node, Some(id2)); // 5 (second)
    }
}
