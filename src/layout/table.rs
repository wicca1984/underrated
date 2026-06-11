use crate::css::values::{CssValue, DisplayValue, LengthUnit};
use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;
use crate::layout::{
    LayoutBox, get_layoutable_children, get_px, layout_node, resolve_margins_and_width,
};
use crate::style::ComputedStyle;
use std::collections::HashMap;

struct TableRowInfo {
    node: Option<NodeId>,
    cells: Vec<NodeId>,
}

pub fn layout_table_container(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    depth: usize,
) -> Option<LayoutBox> {
    let style = styles.get(&node)?;

    // Get box model values
    let margin_top = get_px(style, "margin-top", 0.0);
    let padding_left = get_px(style, "padding-left", 0.0);
    let padding_right = get_px(style, "padding-right", 0.0);
    let padding_top = get_px(style, "padding-top", 0.0);
    let padding_bottom = get_px(style, "padding-bottom", 0.0);

    let border_left = get_px(style, "border-left-width", 0.0);
    let border_right = get_px(style, "border-right-width", 0.0);
    let border_top = get_px(style, "border-top-width", 0.0);
    let border_bottom = get_px(style, "border-bottom-width", 0.0);

    // Resolve horizontal margins and width
    let (resolved_margin_left, resolved_margin_right, content_width, _auto_width) =
        resolve_margins_and_width(
            style,
            containing_width,
            false, // is_inline is false for display: table
            border_left,
            border_right,
            padding_left,
            padding_right,
        );
    let _ = resolved_margin_right;

    let border_box_x = offset_x + resolved_margin_left;
    let border_box_y = offset_y + margin_top;

    // Gather table rows
    let rows = gather_table_rows(dom, styles, node);
    if rows.is_empty() {
        // Empty table
        let border_box_height = padding_top + padding_bottom + border_top + border_bottom;
        return Some(LayoutBox {
            node: Some(node),
            rect: Rect::new(
                border_box_x,
                border_box_y,
                content_width + padding_left + padding_right + border_left + border_right,
                border_box_height,
            ),
            children: Vec::new(),
            text: None,
        });
    }

    let num_cols = rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
    if num_cols == 0 {
        // Empty table
        let border_box_height = padding_top + padding_bottom + border_top + border_bottom;
        return Some(LayoutBox {
            node: Some(node),
            rect: Rect::new(
                border_box_x,
                border_box_y,
                content_width + padding_left + padding_right + border_left + border_right,
                border_box_height,
            ),
            children: Vec::new(),
            text: None,
        });
    }

    // Determine the width of each column
    let mut col_widths = vec![0.0_f32; num_cols];
    for (col_idx, width) in col_widths.iter_mut().enumerate() {
        *width = get_col_max_preferred_width(dom, styles, &rows, col_idx, content_width, depth);
    }

    let sum_col_widths: f32 = col_widths.iter().sum();
    let has_definite_width = matches!(style.get("width"), Some(CssValue::Length(_, _)));

    let final_content_width = if has_definite_width {
        content_width
    } else {
        sum_col_widths.min(content_width)
    };

    if sum_col_widths > 0.0 {
        if final_content_width > sum_col_widths {
            // Distribute remaining space equally to match table's final width
            let remaining = final_content_width - sum_col_widths;
            let share = remaining / num_cols as f32;
            for w in &mut col_widths {
                *w += share;
            }
        } else if final_content_width < sum_col_widths {
            // Scale down columns proportionally
            let scale = final_content_width / sum_col_widths;
            for w in &mut col_widths {
                *w *= scale;
            }
        }
    }

    let mut table_children = Vec::new();
    let mut row_cursor_y = border_box_y + border_top + padding_top;

    // Lay out rows and cells
    for row_info in rows {
        let mut row_cells_boxes = Vec::new();
        let mut max_cell_height = 0.0_f32;

        for (col_idx, &cell_node) in row_info.cells.iter().enumerate() {
            if col_idx >= num_cols {
                break;
            }
            let col_width = col_widths[col_idx];
            let col_offset_x: f32 = col_widths[0..col_idx].iter().sum();
            let cell_x = border_box_x + border_left + padding_left + col_offset_x;
            let cell_y = row_cursor_y;

            if let Some(mut cell_box) = layout_node(
                dom,
                styles,
                cell_node,
                col_width,
                cell_x,
                cell_y,
                depth + 2, // deep enough for children of cells
            ) {
                // Ensure cell fills its assigned column width
                cell_box.rect.size.width = col_width;
                if cell_box.rect.size.height > max_cell_height {
                    max_cell_height = cell_box.rect.size.height;
                }
                row_cells_boxes.push(cell_box);
            }
        }

        // Align all cells to have the same row height
        for cell_box in &mut row_cells_boxes {
            cell_box.rect.size.height = max_cell_height;
        }

        let row_box = LayoutBox {
            node: row_info.node,
            rect: Rect::new(
                border_box_x + border_left + padding_left,
                row_cursor_y,
                final_content_width,
                max_cell_height,
            ),
            children: row_cells_boxes,
            text: None,
        };
        table_children.push(row_box);
        row_cursor_y += max_cell_height;
    }

    let final_content_height = row_cursor_y - (border_box_y + border_top + padding_top);
    let border_box_height =
        final_content_height + padding_top + padding_bottom + border_top + border_bottom;

    Some(LayoutBox {
        node: Some(node),
        rect: Rect::new(
            border_box_x,
            border_box_y,
            final_content_width + padding_left + padding_right + border_left + border_right,
            border_box_height,
        ),
        children: table_children,
        text: None,
    })
}

fn gather_table_rows(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
) -> Vec<TableRowInfo> {
    let mut rows = Vec::new();
    let mut implicit_row_cells = Vec::new();

    fn traverse(
        dom: &Dom,
        styles: &HashMap<NodeId, ComputedStyle>,
        node: NodeId,
        rows: &mut Vec<TableRowInfo>,
        implicit_row_cells: &mut Vec<NodeId>,
    ) {
        let layoutable_children = get_layoutable_children(dom, styles, node);
        for &child in &layoutable_children {
            let style = styles.get(&child);
            let display = style.and_then(|s| s.get("display"));
            let is_row = matches_display(display, "table-row")
                || matches_display(display, "table-row-group")
                || is_table_row_element(dom, child);
            let is_cell =
                matches_display(display, "table-cell") || is_table_cell_element(dom, child);

            if is_row {
                if !implicit_row_cells.is_empty() {
                    rows.push(TableRowInfo {
                        node: None,
                        cells: std::mem::take(implicit_row_cells),
                    });
                }
                let mut row_cells = Vec::new();
                gather_row_cells(dom, styles, child, &mut row_cells);
                rows.push(TableRowInfo {
                    node: Some(child),
                    cells: row_cells,
                });
            } else if is_cell {
                implicit_row_cells.push(child);
            } else {
                traverse(dom, styles, child, rows, implicit_row_cells);
            }
        }
    }

    traverse(dom, styles, node, &mut rows, &mut implicit_row_cells);

    if !implicit_row_cells.is_empty() {
        rows.push(TableRowInfo {
            node: None,
            cells: implicit_row_cells,
        });
    }

    rows
}

fn gather_row_cells(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    row_node: NodeId,
    cells: &mut Vec<NodeId>,
) {
    let layoutable_children = get_layoutable_children(dom, styles, row_node);
    for &child in &layoutable_children {
        let style = styles.get(&child);
        let display = style.and_then(|s| s.get("display"));
        let is_cell = matches_display(display, "table-cell") || is_table_cell_element(dom, child);
        if is_cell {
            cells.push(child);
        } else {
            gather_row_cells(dom, styles, child, cells);
        }
    }
}

fn matches_display(display: Option<&CssValue>, expected: &str) -> bool {
    if let Some(disp) = display {
        match disp {
            CssValue::Keyword(kw) => kw == expected,
            CssValue::Display(dv) => matches!(
                (dv, expected),
                (DisplayValue::Table, "table")
                    | (DisplayValue::TableRow, "table-row")
                    | (DisplayValue::TableCell, "table-cell")
            ),
            _ => false,
        }
    } else {
        false
    }
}

fn is_table_row_element(dom: &Dom, node: NodeId) -> bool {
    if let Some(NodeData::Element { name, .. }) = dom.data(node) {
        name == "tr"
    } else {
        false
    }
}

fn is_table_cell_element(dom: &Dom, node: NodeId) -> bool {
    if let Some(NodeData::Element { name, .. }) = dom.data(node) {
        name == "td" || name == "th"
    } else {
        false
    }
}

fn get_col_max_preferred_width(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    rows: &[TableRowInfo],
    col_idx: usize,
    content_width: f32,
    depth: usize,
) -> f32 {
    let mut max_w = 0.0_f32;
    for row in rows {
        if let Some(&cell_node) = row.cells.get(col_idx) {
            let mut width = 0.0_f32;
            if let Some(cs) = styles.get(&cell_node)
                && let Some(CssValue::Length(val, LengthUnit::Px)) = cs.get("width")
            {
                width = *val;
            }
            if width == 0.0
                && let Some(cell_box) =
                    layout_node(dom, styles, cell_node, content_width, 0.0, 0.0, depth + 1)
            {
                width = cell_box.rect.size.width;
            }
            if width > max_w {
                max_w = width;
            }
        }
    }
    max_w
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::values::{CssValue, DisplayValue, LengthUnit};
    use crate::dom::{Dom, NodeData};
    use crate::style::ComputedStyle;
    use std::collections::HashMap;

    fn style_with_display(display: &str) -> ComputedStyle {
        let mut s = ComputedStyle::default();
        let val = match display {
            "table" => CssValue::Display(DisplayValue::Table),
            "table-row" => CssValue::Display(DisplayValue::TableRow),
            "table-cell" => CssValue::Display(DisplayValue::TableCell),
            _ => CssValue::Keyword(display.to_string()),
        };
        s.insert("display".to_string(), val);
        s
    }

    #[test]
    fn test_basic_table_layout() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Create table element
        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Create row 1
        let row1_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row1_node);

        // Create cell 1.1
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell11_node);

        // Create cell 1.2
        let cell12_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell12_node);

        // Create row 2
        let row2_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row2_node);

        // Create cell 2.1
        let cell21_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell21_node);

        // Setup styles
        let mut styles = HashMap::new();

        // Table style: width 200px
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        styles.insert(table_node, table_style);

        // Rows
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        // Cell 1.1: width 50px
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        // Cell 1.2: width 80px
        let mut cell12_style = style_with_display("table-cell");
        cell12_style.insert("width".to_string(), CssValue::Length(80.0, LengthUnit::Px));
        cell12_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell12_node, cell12_style);

        // Cell 2.1: width 40px
        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(25.0, LengthUnit::Px));
        styles.insert(cell21_node, cell21_style);

        // Document layout
        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 10.0, 20.0, 0)
            .expect("should layout table");

        // Table box checks
        assert_eq!(table_box.rect.origin.x, 10.0);
        assert_eq!(table_box.rect.origin.y, 20.0);
        assert_eq!(table_box.rect.size.width, 200.0);

        // Column widths should be:
        // Col 0: max(50, 40) = 50px
        // Col 1: max(80) = 80px
        // Total columns preferred = 130px.
        // Table content width is 200px.
        // The remaining 70px is shared equally: +35px each.
        // Col 0 final: 50 + 35 = 85px
        // Col 1 final: 80 + 35 = 115px

        // Table has 2 row children
        assert_eq!(table_box.children.len(), 2);

        // Row 1
        let r1 = &table_box.children[0];
        assert_eq!(r1.node, Some(row1_node));
        assert_eq!(r1.rect.origin.y, 20.0);
        // Row 1 height should be max(30, 40) = 40px
        assert_eq!(r1.rect.size.height, 40.0);
        assert_eq!(r1.children.len(), 2);

        let cell11_box = &r1.children[0];
        assert_eq!(cell11_box.node, Some(cell11_node));
        assert_eq!(cell11_box.rect.origin.x, 10.0);
        assert_eq!(cell11_box.rect.size.width, 85.0);
        assert_eq!(cell11_box.rect.size.height, 40.0); // stretched!

        let cell12_box = &r1.children[1];
        assert_eq!(cell12_box.node, Some(cell12_node));
        assert_eq!(cell12_box.rect.origin.x, 10.0 + 85.0);
        assert_eq!(cell12_box.rect.size.width, 115.0);
        assert_eq!(cell12_box.rect.size.height, 40.0);

        // Row 2
        let r2 = &table_box.children[1];
        assert_eq!(r2.node, Some(row2_node));
        assert_eq!(r2.rect.origin.y, 20.0 + 40.0);
        // Row 2 height should be max(25) = 25px
        assert_eq!(r2.rect.size.height, 25.0);
        assert_eq!(r2.children.len(), 1);

        let cell21_box = &r2.children[0];
        assert_eq!(cell21_box.node, Some(cell21_node));
        assert_eq!(cell21_box.rect.origin.x, 10.0);
        assert_eq!(cell21_box.rect.size.width, 85.0);
        assert_eq!(cell21_box.rect.size.height, 25.0);
    }

    #[test]
    fn test_implicit_rows_table_layout() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Create table element
        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Create cell 1 directly under table (creates implicit row)
        let cell1_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, cell1_node);

        // Setup styles
        let mut styles = HashMap::new();

        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        styles.insert(table_node, table_style);

        let mut cell1_style = style_with_display("table-cell");
        cell1_style.insert("width".to_string(), CssValue::Length(60.0, LengthUnit::Px));
        cell1_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell1_node, cell1_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // Implicit row check
        assert_eq!(table_box.children.len(), 1);
        let implicit_row = &table_box.children[0];
        assert_eq!(implicit_row.node, None); // anonymous
        assert_eq!(implicit_row.children.len(), 1);

        let cell1_box = &implicit_row.children[0];
        assert_eq!(cell1_box.node, Some(cell1_node));
        assert_eq!(cell1_box.rect.size.width, 100.0); // stretched to fit 100% table width
    }
}
