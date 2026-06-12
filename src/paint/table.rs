//! Presentational `<table border>` gridline rendering.
use super::DisplayItem;
use crate::css::values::Color;
use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;

fn is_last_cell_in_row(dom: &Dom, cell_id: NodeId) -> bool {
    if let Some(parent_id) = dom.parent(cell_id) {
        let siblings = dom.children(parent_id);
        if let Some(pos) = siblings.iter().position(|&id| id == cell_id) {
            for &sibling in &siblings[pos + 1..] {
                if let Some(NodeData::Element { name, .. }) = dom.data(sibling)
                    && (name.eq_ignore_ascii_case("td") || name.eq_ignore_ascii_case("th"))
                {
                    return false;
                }
            }
        }
    }
    true
}

fn is_last_row(dom: &Dom, cell_id: NodeId) -> bool {
    if let Some(row_id) = dom.parent(cell_id)
        && let Some(NodeData::Element { name: row_name, .. }) = dom.data(row_id)
        && row_name.eq_ignore_ascii_case("tr")
    {
        let mut current = Some(cell_id);
        let mut table_node = None;
        while let Some(curr) = current {
            if let Some(NodeData::Element { name, .. }) = dom.data(curr)
                && name.eq_ignore_ascii_case("table")
            {
                table_node = Some(curr);
                break;
            }
            current = dom.parent(curr);
        }

        if let Some(t_node) = table_node {
            let descendants = dom.descendants(t_node);
            let mut rows = Vec::new();
            for desc in descendants {
                if let Some(NodeData::Element { name, .. }) = dom.data(desc)
                    && name.eq_ignore_ascii_case("tr")
                {
                    rows.push(desc);
                }
            }
            if let Some(pos) = rows.iter().position(|&r| r == row_id) {
                return pos == rows.len() - 1;
            }
        }
    }
    true
}

fn cell_has_rendered_content(dom: &Dom, node_id: NodeId) -> bool {
    for &child in dom.children(node_id) {
        if let Some(data) = dom.data(child) {
            match data {
                NodeData::Element { .. } => {
                    return true;
                }
                NodeData::Text(s) if !s.trim().is_empty() => {
                    return true;
                }
                _ => {}
            }
        }
        if cell_has_rendered_content(dom, child) {
            return true;
        }
    }
    false
}

fn parse_empty_cells_from_style_string(style: &str) -> Option<String> {
    for decl in style.split(';') {
        let parts: Vec<&str> = decl.split(':').collect();
        if parts.len() == 2 {
            let name = parts[0].trim();
            let val = parts[1].trim();
            if name.eq_ignore_ascii_case("empty-cells") {
                if val.eq_ignore_ascii_case("hide") {
                    return Some("hide".to_string());
                } else if val.eq_ignore_ascii_case("show") || val.eq_ignore_ascii_case("initial") {
                    return Some("show".to_string());
                } else if val.eq_ignore_ascii_case("inherit") {
                    return Some("inherit".to_string());
                }
            }
        }
    }
    None
}

fn get_pragmatic_empty_cells(dom: &Dom, node_id: NodeId) -> String {
    let mut current = Some(node_id);
    while let Some(curr) = current {
        if let Some(crate::dom::NodeData::Element { attrs, .. }) = dom.data(curr)
            && let Some((_, style_val)) = attrs.iter().find(|(name, _)| name == "style")
            && let Some(val) = parse_empty_cells_from_style_string(style_val)
            && val != "inherit"
        {
            return val;
        }
        current = dom.parent(curr);
    }
    "show".to_string()
}

/// Returns the 4 edge SolidRect items for a bordered table cell/table box, or empty.
pub fn table_border_items(
    dom: &Dom,
    node_id: NodeId,
    name: &str,
    rect: Rect,
    collapse: bool,
) -> Vec<DisplayItem> {
    let is_table_or_cell = name.eq_ignore_ascii_case("table")
        || name.eq_ignore_ascii_case("td")
        || name.eq_ignore_ascii_case("th");
    if !is_table_or_cell {
        return vec![];
    }

    if name.eq_ignore_ascii_case("td") || name.eq_ignore_ascii_case("th") {
        let empty_cells_val = get_pragmatic_empty_cells(dom, node_id);
        // TODO(spec): thread computed empty-cells via style
        if empty_cells_val == "hide" && !cell_has_rendered_content(dom, node_id) {
            return vec![];
        }
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

    // TODO(spec): honor numeric border width.
    // border-collapse: collapse has been resolved via spec S-102 (collapsed borders
    // draw each interior grid line once and a closed outer frame without doubling).
    let color = Color::Rgba(128, 128, 128, 255);
    let mut items = Vec::new();

    if collapse {
        if name.eq_ignore_ascii_case("table") {
            // Under collapse mode, the table node itself doesn't need to paint its own separate border items
            // because the cells' borders (including outer edges on the last row and column) completely
            // frame the table. This avoids outer edge doubling.
            return vec![];
        }

        let is_last_cell = is_last_cell_in_row(dom, node_id);
        let is_last_row_cell = is_last_row(dom, node_id);

        // Top border strip
        items.push(DisplayItem::SolidRect {
            rect: Rect::new(x, y, w, 1.0),
            color: color.clone(),
        });

        // Left border strip (goes from y to y + h for a clean seamless vertical line)
        items.push(DisplayItem::SolidRect {
            rect: Rect::new(x, y, 1.0, h),
            color: color.clone(),
        });

        // Right border strip (only if it is the last cell in its row)
        if is_last_cell {
            items.push(DisplayItem::SolidRect {
                rect: Rect::new(x + w - 1.0, y, 1.0, h),
                color: color.clone(),
            });
        }

        // Bottom border strip (only if it is in the last row)
        if is_last_row_cell {
            items.push(DisplayItem::SolidRect {
                rect: Rect::new(x, y + h - 1.0, w, 1.0),
                color,
            });
        }
    } else {
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

    #[test]
    fn test_border_collapse_conflict_resolution() {
        // Build a 2x2 table under border-collapse: separate model (default)
        let mut dom_sep = Dom::new();
        let doc_sep = dom_sep.document();
        let table_sep = dom_sep.create_node(NodeData::Element {
            name: "table".into(),
            attrs: vec![("border".into(), "1".into())],
        });
        dom_sep.append_child(doc_sep, table_sep);

        for _ in 0..2 {
            let tr = dom_sep.create_node(NodeData::Element {
                name: "tr".into(),
                attrs: vec![],
            });
            dom_sep.append_child(table_sep, tr);
            for _ in 0..2 {
                let td = dom_sep.create_node(NodeData::Element {
                    name: "td".into(),
                    attrs: vec![],
                });
                dom_sep.append_child(tr, td);
                let text = dom_sep.create_node(NodeData::Text("cell".into()));
                dom_sep.append_child(td, text);
            }
        }

        let stylesheet_sep = parse_stylesheet("");
        let styles_sep = compute_styles(&dom_sep, &stylesheet_sep);
        let layout_sep = layout_document(&dom_sep, &styles_sep, 800.0);
        let display_list_sep = build_display_list(&layout_sep, &dom_sep, &styles_sep);
        let sep_borders = display_list_sep
            .0
            .iter()
            .filter(|item| match item {
                DisplayItem::SolidRect { color, .. } => *color == Color::Rgba(128, 128, 128, 255),
                _ => false,
            })
            .count();

        // Build a 2x2 table under border-collapse: collapse model
        let mut dom_col = Dom::new();
        let doc_col = dom_col.document();
        let table_col = dom_col.create_node(NodeData::Element {
            name: "table".into(),
            attrs: vec![("border".into(), "1".into())],
        });
        dom_col.append_child(doc_col, table_col);

        for _ in 0..2 {
            let tr = dom_col.create_node(NodeData::Element {
                name: "tr".into(),
                attrs: vec![],
            });
            dom_col.append_child(table_col, tr);
            for _ in 0..2 {
                let td = dom_col.create_node(NodeData::Element {
                    name: "td".into(),
                    attrs: vec![],
                });
                dom_col.append_child(tr, td);
                let text = dom_col.create_node(NodeData::Text("cell".into()));
                dom_col.append_child(td, text);
            }
        }

        // Apply border-collapse: collapse stylesheet
        let stylesheet_col = parse_stylesheet("table { border-collapse: collapse; }");
        let styles_col = compute_styles(&dom_col, &stylesheet_col);
        let layout_col = layout_document(&dom_col, &styles_col, 800.0);
        let display_list_col = build_display_list(&layout_col, &dom_col, &styles_col);
        let col_borders = display_list_col
            .0
            .iter()
            .filter(|item| match item {
                DisplayItem::SolidRect { color, .. } => *color == Color::Rgba(128, 128, 128, 255),
                _ => false,
            })
            .count();

        assert!(
            col_borders > 0,
            "Expected some border display items for collapse model, got 0"
        );
        assert!(
            col_borders < sep_borders,
            "Expected fewer border segments under collapse model (got {}) than separate model (got {})",
            col_borders,
            sep_borders
        );
    }

    #[test]
    fn test_empty_cells_hide_non_empty() {
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
            attrs: vec![("style".into(), "empty-cells: hide;".into())],
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

        // The non-empty cell should STILL produce its borders, so total borders >= 8
        assert!(
            gray_borders >= 8,
            "Expected at least 8 borders, got {}",
            gray_borders
        );
    }

    #[test]
    fn test_empty_cells_hide_empty() {
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
            attrs: vec![("style".into(), "empty-cells: hide;".into())],
        });
        dom.append_child(tr, td);

        // No children or only whitespace text in the td!
        let text = dom.create_node(NodeData::Text("   ".into()));
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

        // The empty cell should suppress its borders. Only the table itself might have borders (4 segments).
        // So the count should be < 8 (specifically, 4 for the table, 0 for the td).
        assert!(
            gray_borders < 8,
            "Expected empty cell to suppress borders, total borders was {}",
            gray_borders
        );
        assert_eq!(gray_borders, 4);
    }

    #[test]
    fn test_empty_cells_show_empty() {
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
            attrs: vec![("style".into(), "empty-cells: show; padding: 10px;".into())],
        });
        dom.append_child(tr, td);

        // No children or only whitespace text in the td!
        let text = dom.create_node(NodeData::Text("   ".into()));
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

        // The empty cell should STILL produce its borders because of show, so total borders >= 8
        assert!(
            gray_borders >= 8,
            "Expected at least 8 borders, got {}",
            gray_borders
        );
    }

    #[test]
    fn test_empty_cells_inheritance() {
        let mut dom = Dom::new();
        let doc = dom.document();
        // table has style="empty-cells: hide;"
        let table = dom.create_node(NodeData::Element {
            name: "table".into(),
            attrs: vec![
                ("border".into(), "1".into()),
                ("style".into(), "empty-cells: hide;".into()),
            ],
        });
        dom.append_child(doc, table);

        let tr = dom.create_node(NodeData::Element {
            name: "tr".into(),
            attrs: vec![],
        });
        dom.append_child(table, tr);

        // td has no style, inherits empty-cells: hide from table
        let td = dom.create_node(NodeData::Element {
            name: "td".into(),
            attrs: vec![],
        });
        dom.append_child(tr, td);

        let text = dom.create_node(NodeData::Text("   ".into()));
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

        // Inherited empty-cells: hide should suppress the empty cell borders (total borders == 4)
        assert_eq!(gray_borders, 4);
    }
}
