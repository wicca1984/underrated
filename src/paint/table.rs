//! Presentational `<table border>` gridline rendering.
use super::DisplayItem;
use crate::css::values::Color;
use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;

/// Returns the 4 edge SolidRect items for a bordered table cell/table box, or empty.
pub fn table_border_items(dom: &Dom, node_id: NodeId, name: &str, rect: Rect) -> Vec<DisplayItem> {
    let is_table_or_cell = name.eq_ignore_ascii_case("table")
        || name.eq_ignore_ascii_case("td")
        || name.eq_ignore_ascii_case("th");
    if !is_table_or_cell {
        return vec![];
    }

    // Find nearest ancestor (inclusive) `<table>`
    let mut current_node = Some(node_id);
    let mut table_node = None;
    while let Some(curr) = current_node {
        if let Some(NodeData::Element { name: el_name, .. }) = dom.data(curr)
            && el_name.eq_ignore_ascii_case("table")
        {
            table_node = Some(curr);
            break;
        }
        current_node = dom.parent(curr);
    }

    let Some(t_node) = table_node else {
        return vec![];
    };

    let border_attr = dom.get_attribute(t_node, "border");
    let has_border = match border_attr {
        Some(val) => {
            let trimmed = val.trim();
            !trimmed.is_empty() && trimmed != "0"
        }
        None => false,
    };

    if !has_border {
        return vec![];
    }

    let x = rect.origin.x;
    let y = rect.origin.y;
    let w = rect.size.width;
    let h = rect.size.height;

    if w <= 0.0 || h <= 0.0 {
        return vec![];
    }

    // TODO(spec): honor numeric border width / border-collapse
    let color = Color::Rgba(128, 128, 128, 255);
    let mut items = Vec::new();

    // Top border strip
    items.push(DisplayItem::SolidRect {
        rect: Rect::new(x, y, w, 1.0),
        color: color.clone(),
    });

    // Bottom border strip
    if h > 1.0 {
        items.push(DisplayItem::SolidRect {
            rect: Rect::new(x, y + h - 1.0, w, 1.0),
            color: color.clone(),
        });
    }

    // Left border strip
    let middle_h = h - 2.0;
    if middle_h > 0.0 {
        items.push(DisplayItem::SolidRect {
            rect: Rect::new(x, y + 1.0, 1.0, middle_h),
            color: color.clone(),
        });
    }

    // Right border strip
    if w > 1.0 && middle_h > 0.0 {
        items.push(DisplayItem::SolidRect {
            rect: Rect::new(x + w - 1.0, y + 1.0, 1.0, middle_h),
            color,
        });
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_stylesheet;
    use crate::layout::layout_document;
    use crate::paint::build_display_list;
    use crate::style::compute_styles;

    #[test]
    fn test_paint_table_border_present() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let table = dom.create_node(NodeData::Element {
            name: "table".into(),
            attrs: vec![("border".into(), "1".into())],
        });
        dom.append_child(doc, table);

        let tr = dom.create_node(NodeData::Element {
            name: "tr".into(),
            attrs: vec![],
        });
        dom.append_child(table, tr);

        let td = dom.create_node(NodeData::Element {
            name: "td".into(),
            attrs: vec![],
        });
        dom.append_child(tr, td);

        let text = dom.create_node(NodeData::Text("cell".into()));
        dom.append_child(td, text);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // e.g. count of Color::Rgba(128,128,128,255) SolidRects >= 8, i.e. 4 per box
        let gray_borders = items
            .iter()
            .filter(|item| match item {
                DisplayItem::SolidRect { color, .. } => *color == Color::Rgba(128, 128, 128, 255),
                _ => false,
            })
            .count();

        assert!(
            gray_borders >= 8,
            "Expected at least 8 gray border SolidRect segments (4 for table, 4 for td), got {}",
            gray_borders
        );
    }

    #[test]
    fn test_paint_table_border_absent() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let table = dom.create_node(NodeData::Element {
            name: "table".into(),
            attrs: vec![],
        });
        dom.append_child(doc, table);

        let tr = dom.create_node(NodeData::Element {
            name: "tr".into(),
            attrs: vec![],
        });
        dom.append_child(table, tr);

        let td = dom.create_node(NodeData::Element {
            name: "td".into(),
            attrs: vec![],
        });
        dom.append_child(tr, td);

        let text = dom.create_node(NodeData::Text("cell".into()));
        dom.append_child(td, text);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let gray_borders = items
            .iter()
            .filter(|item| match item {
                DisplayItem::SolidRect { color, .. } => *color == Color::Rgba(128, 128, 128, 255),
                _ => false,
            })
            .count();

        assert_eq!(
            gray_borders, 0,
            "Expected 0 gray border SolidRect segments, got {}",
            gray_borders
        );
    }
}
