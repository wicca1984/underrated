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

struct CellPlacement {
    node: NodeId,
    col_idx: usize,
    colspan: usize,
    row_idx: usize,
    rowspan: usize,
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

    // Determine cell placements (slot-occupancy model)
    let mut occupied = HashMap::<(usize, usize), bool>::new();
    let mut cell_placements = HashMap::<NodeId, CellPlacement>::new();

    for (r, row_info) in rows.iter().enumerate() {
        let mut curr_col = 0;
        for &cell_node in &row_info.cells {
            while occupied.get(&(r, curr_col)).copied().unwrap_or(false) {
                curr_col += 1;
            }

            let colspan = parse_span_attribute(dom, cell_node, "colspan").clamp(1, 1000);
            let remaining_rows = rows.len() - r;
            let rowspan = parse_span_attribute(dom, cell_node, "rowspan").clamp(1, remaining_rows);

            let placement = CellPlacement {
                node: cell_node,
                col_idx: curr_col,
                colspan,
                row_idx: r,
                rowspan,
            };
            cell_placements.insert(cell_node, placement);

            // Mark slot occupancy
            for dr in 0..rowspan {
                for dc in 0..colspan {
                    occupied.insert((r + dr, curr_col + dc), true);
                }
            }

            curr_col += colspan;
        }
    }

    let num_cols = occupied
        .keys()
        .map(|&(_r, c)| c)
        .max()
        .map(|c| c + 1)
        .unwrap_or(0);

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

    // First, handle all cells with colspan == 1 to establish baseline column widths
    for placement in cell_placements.values() {
        if placement.colspan == 1 {
            let cell_pref_w =
                get_cell_preferred_width(dom, styles, placement.node, content_width, depth + 1);
            if cell_pref_w > col_widths[placement.col_idx] {
                col_widths[placement.col_idx] = cell_pref_w;
            }
        }
    }

    // Next, handle all cells with colspan > 1 (sorted by colspan ascending)
    let mut colspanning_cells: Vec<&CellPlacement> =
        cell_placements.values().filter(|p| p.colspan > 1).collect();
    colspanning_cells.sort_by_key(|p| p.colspan);

    for placement in colspanning_cells {
        let cell_pref_w =
            get_cell_preferred_width(dom, styles, placement.node, content_width, depth + 1);
        let current_combined: f32 = col_widths
            [placement.col_idx..(placement.col_idx + placement.colspan)]
            .iter()
            .sum();
        if cell_pref_w > current_combined {
            let deficit = cell_pref_w - current_combined;
            let share = deficit / placement.colspan as f32;
            for w in col_widths
                .iter_mut()
                .skip(placement.col_idx)
                .take(placement.colspan)
            {
                *w += share;
            }
        }
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

    // Pre-layout every cell to determine height based on final column widths
    let mut cell_boxes = HashMap::new();
    for (&cell_node, placement) in &cell_placements {
        let cell_width: f32 = col_widths
            [placement.col_idx..(placement.col_idx + placement.colspan)]
            .iter()
            .sum();
        if let Some(mut cell_box) = layout_node(
            dom,
            styles,
            cell_node,
            cell_width,
            0.0,
            0.0,
            depth + 2, // deep enough for children of cells
        ) {
            cell_box.rect.size.width = cell_width;
            cell_boxes.insert(cell_node, cell_box);
        }
    }

    // Determine row heights
    let mut row_heights = vec![0.0_f32; rows.len()];

    // First, establish baseline row heights using rowspan == 1 cells
    for placement in cell_placements.values() {
        if placement.rowspan == 1
            && let Some(cell_box) = cell_boxes.get(&placement.node)
        {
            let h = cell_box.rect.size.height;
            if h > row_heights[placement.row_idx] {
                row_heights[placement.row_idx] = h;
            }
        }
    }

    // Next, resolve cells with rowspan > 1 (sorted by rowspan ascending)
    let mut rowspanning_cells: Vec<&CellPlacement> =
        cell_placements.values().filter(|p| p.rowspan > 1).collect();
    rowspanning_cells.sort_by_key(|p| p.rowspan);

    for placement in rowspanning_cells {
        if let Some(cell_box) = cell_boxes.get(&placement.node) {
            let h = cell_box.rect.size.height;
            let current_combined: f32 = row_heights
                [placement.row_idx..(placement.row_idx + placement.rowspan)]
                .iter()
                .sum();
            if h > current_combined {
                let deficit = h - current_combined;
                let share = deficit / placement.rowspan as f32;
                for rh in row_heights
                    .iter_mut()
                    .skip(placement.row_idx)
                    .take(placement.rowspan)
                {
                    *rh += share;
                }
            }
        }
    }

    let mut row_y_offsets = vec![0.0_f32; rows.len()];
    let mut curr_y = border_box_y + border_top + padding_top;
    for r in 0..rows.len() {
        row_y_offsets[r] = curr_y;
        curr_y += row_heights[r];
    }

    let mut table_children = Vec::new();

    // Lay out rows and place cells
    for (r, row_info) in rows.into_iter().enumerate() {
        let mut row_cells_boxes = Vec::new();

        for &cell_node in &row_info.cells {
            if let Some(placement) = cell_placements.get(&cell_node)
                && let Some(mut cell_box) = cell_boxes.remove(&cell_node)
            {
                let col_idx = placement.col_idx;
                let col_offset_x: f32 = col_widths[0..col_idx].iter().sum();
                let cell_x = border_box_x + border_left + padding_left + col_offset_x;
                let cell_y = row_y_offsets[r];

                let cell_width: f32 = col_widths[col_idx..(col_idx + placement.colspan)]
                    .iter()
                    .sum();
                let cell_height: f32 = row_heights[r..(r + placement.rowspan)].iter().sum();

                cell_box.rect.origin.x = cell_x;
                cell_box.rect.origin.y = cell_y;
                cell_box.rect.size.width = cell_width;
                cell_box.rect.size.height = cell_height;

                row_cells_boxes.push(cell_box);
            }
        }

        let row_box = LayoutBox {
            node: row_info.node,
            rect: Rect::new(
                border_box_x + border_left + padding_left,
                row_y_offsets[r],
                final_content_width,
                row_heights[r],
            ),
            children: row_cells_boxes,
            text: None,
        };
        table_children.push(row_box);
    }

    let final_content_height = curr_y - (border_box_y + border_top + padding_top);
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

fn parse_span_attribute(dom: &Dom, node: NodeId, name: &str) -> usize {
    if let Some(s) = dom.get_attribute(node, name) {
        if let Ok(val) = s.trim().parse::<i32>() {
            if val <= 0 { 1 } else { val as usize }
        } else {
            1
        }
    } else {
        1
    }
}

fn get_cell_preferred_width(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    cell_node: NodeId,
    content_width: f32,
    depth: usize,
) -> f32 {
    let mut width = 0.0_f32;
    if let Some(cs) = styles.get(&cell_node)
        && let Some(CssValue::Length(val, LengthUnit::Px)) = cs.get("width")
    {
        width = *val;
    }
    if width == 0.0
        && let Some(cell_box) = layout_node(dom, styles, cell_node, content_width, 0.0, 0.0, depth)
    {
        width = cell_box.rect.size.width;
    }
    width
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

    #[test]
    fn test_colspan_table_layout() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Row 1
        let row1_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row1_node);

        // Cell 1.1: colspan="2"
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![("colspan".to_string(), "2".to_string())],
        });
        dom.append_child(row1_node, cell11_node);

        // Row 2
        let row2_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row2_node);

        // Cell 2.1
        let cell21_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell21_node);

        // Cell 2.2
        let cell22_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell22_node);

        let mut styles = HashMap::new();
        styles.insert(table_node, style_with_display("table"));
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(25.0, LengthUnit::Px));
        styles.insert(cell21_node, cell21_style);

        let mut cell22_style = style_with_display("table-cell");
        cell22_style.insert("width".to_string(), CssValue::Length(60.0, LengthUnit::Px));
        cell22_style.insert("height".to_string(), CssValue::Length(25.0, LengthUnit::Px));
        styles.insert(cell22_node, cell22_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        assert_eq!(table_box.rect.size.width, 100.0);
        assert_eq!(table_box.children.len(), 2);

        // Row 1
        let r1 = &table_box.children[0];
        assert_eq!(r1.children.len(), 1);
        let cell11_box = &r1.children[0];
        assert_eq!(cell11_box.rect.size.width, 100.0);

        // Row 2
        let r2 = &table_box.children[1];
        assert_eq!(r2.children.len(), 2);
        assert_eq!(r2.children[0].rect.size.width, 40.0);
        assert_eq!(r2.children[1].rect.size.width, 60.0);
    }

    #[test]
    fn test_rowspan_table_layout() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Row 1
        let row1_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row1_node);

        // Cell 1.1: rowspan="2"
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![("rowspan".to_string(), "2".to_string())],
        });
        dom.append_child(row1_node, cell11_node);

        // Cell 1.2
        let cell12_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell12_node);

        // Row 2
        let row2_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row2_node);

        // Cell 2.1
        let cell21_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell21_node);

        let mut styles = HashMap::new();
        styles.insert(table_node, style_with_display("table"));
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        let mut cell12_style = style_with_display("table-cell");
        cell12_style.insert("width".to_string(), CssValue::Length(80.0, LengthUnit::Px));
        cell12_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell12_node, cell12_style);

        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(80.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(20.0, LengthUnit::Px));
        styles.insert(cell21_node, cell21_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        assert_eq!(table_box.rect.size.width, 130.0);
        assert_eq!(table_box.children.len(), 2);

        // Row 1
        let r1 = &table_box.children[0];
        assert_eq!(r1.children.len(), 2);
        let cell11_box = &r1.children[0];
        assert_eq!(cell11_box.rect.origin.x, 0.0);
        assert_eq!(cell11_box.rect.origin.y, 0.0);
        assert_eq!(cell11_box.rect.size.width, 50.0);
        assert_eq!(cell11_box.rect.size.height, 50.0);

        let cell12_box = &r1.children[1];
        assert_eq!(cell12_box.rect.origin.x, 50.0);
        assert_eq!(cell12_box.rect.origin.y, 0.0);
        assert_eq!(cell12_box.rect.size.width, 80.0);
        assert_eq!(cell12_box.rect.size.height, 30.0);

        // Row 2
        let r2 = &table_box.children[1];
        assert_eq!(r2.children.len(), 1);
        let cell21_box = &r2.children[0];
        assert_eq!(cell21_box.rect.origin.x, 50.0); // pushed to column 1!
        assert_eq!(cell21_box.rect.origin.y, 30.0);
        assert_eq!(cell21_box.rect.size.width, 80.0);
        assert_eq!(cell21_box.rect.size.height, 20.0);
    }

    #[test]
    fn test_invalid_spans_clamping() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        let row_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row_node);

        // Cell 1: colspan="0"
        let cell1_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![("colspan".to_string(), "0".to_string())],
        });
        dom.append_child(row_node, cell1_node);

        // Cell 2: colspan="-5"
        let cell2_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![("colspan".to_string(), "-5".to_string())],
        });
        dom.append_child(row_node, cell2_node);

        // Cell 3: colspan="abc"
        let cell3_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![("colspan".to_string(), "abc".to_string())],
        });
        dom.append_child(row_node, cell3_node);

        // Cell 4: missing colspan
        let cell4_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell4_node);

        let mut styles = HashMap::new();
        styles.insert(table_node, style_with_display("table"));
        styles.insert(row_node, style_with_display("table-row"));

        for &c in &[cell1_node, cell2_node, cell3_node, cell4_node] {
            let mut style = style_with_display("table-cell");
            style.insert("width".to_string(), CssValue::Length(25.0, LengthUnit::Px));
            style.insert("height".to_string(), CssValue::Length(10.0, LengthUnit::Px));
            styles.insert(c, style);
        }

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // There should be 4 columns of width 25.0 each because all spans clamp to 1.
        assert_eq!(table_box.rect.size.width, 100.0);
        assert_eq!(table_box.children.len(), 1);
        let r = &table_box.children[0];
        assert_eq!(r.children.len(), 4);
        for (i, cell) in r.children.iter().enumerate() {
            assert_eq!(cell.rect.origin.x, (i as f32) * 25.0);
            assert_eq!(cell.rect.size.width, 25.0);
        }
    }
}
