use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;
use crate::layout::{
    LayoutBox, get_layoutable_children, get_px, layout_node, resolve_margins_and_width,
};
use crate::style::CategorizedComputedStyle;
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

fn translate_y(layout_box: &mut LayoutBox, dy: f32) {
    layout_box.rect.origin.y += dy;
    for child in &mut layout_box.children {
        translate_y(child, dy);
    }
}

pub fn layout_table_container(
    dom: &Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    node: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    depth: usize,
) -> Option<LayoutBox> {
    let style = styles.get(&node)?;
    let caption_side_bottom = style.inherited_table.caption_side == "bottom";

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

    // Find first caption among layoutable children
    let layoutable_children = get_layoutable_children(dom, styles, node);
    let mut first_caption = None;
    let mut found_first = false;
    for &child in &layoutable_children {
        if let Some(NodeData::Element { name, .. }) = dom.data(child)
            && name == "caption"
        {
            if !found_first {
                first_caption = Some(child);
                found_first = true;
            } else {
                // TODO(spec): Multiple captions are not supported. Ignore any later captions.
            }
        }
    }

    // Gather table rows
    let rows = gather_table_rows(dom, styles, node);
    if rows.is_empty() {
        // Empty table, but might have a caption
        let table_width = content_width + padding_left + padding_right + border_left + border_right;
        let mut table_children = Vec::new();
        let mut caption_height = 0.0_f32;
        if let Some(caption_node) = first_caption
            && let Some(mut cap_box) = layout_node(
                dom,
                styles,
                caption_node,
                table_width,
                border_box_x,
                border_box_y,
                depth + 1,
            )
        {
            caption_height = cap_box.rect.size.height;
            if caption_side_bottom {
                let dy = padding_top + padding_bottom + border_top + border_bottom;
                translate_y(&mut cap_box, dy);
            }
            table_children.push(cap_box);
        }
        let border_box_height =
            padding_top + padding_bottom + border_top + border_bottom + caption_height;
        return Some(LayoutBox {
            node: Some(node),
            rect: Rect::new(border_box_x, border_box_y, table_width, border_box_height),
            children: table_children,
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
        // Empty table, but might have a caption
        let table_width = content_width + padding_left + padding_right + border_left + border_right;
        let mut table_children = Vec::new();
        let mut caption_height = 0.0_f32;
        if let Some(caption_node) = first_caption
            && let Some(mut cap_box) = layout_node(
                dom,
                styles,
                caption_node,
                table_width,
                border_box_x,
                border_box_y,
                depth + 1,
            )
        {
            caption_height = cap_box.rect.size.height;
            if caption_side_bottom {
                let dy = padding_top + padding_bottom + border_top + border_bottom;
                translate_y(&mut cap_box, dy);
            }
            table_children.push(cap_box);
        }
        let border_box_height =
            padding_top + padding_bottom + border_top + border_bottom + caption_height;
        return Some(LayoutBox {
            node: Some(node),
            rect: Rect::new(border_box_x, border_box_y, table_width, border_box_height),
            children: table_children,
            text: None,
        });
    }

    // Determine the width of each column
    let mut col_widths = vec![0.0_f32; num_cols];

    // TODO(spec): border-collapse: full border conflict resolution (shared edges, width/precedence) not implemented; here we only collapse inter-cell spacing to zero.
    let (spacing_h, spacing_v) = if is_border_collapse(style) {
        (0.0, 0.0)
    } else {
        get_border_spacing(style)
    };
    let total_spacing_h = (num_cols + 1) as f32 * spacing_h;
    let avail_content_width = (content_width - total_spacing_h).max(0.0);

    let table_layout_fixed = style.reset_table.table_layout == "fixed";

    let final_content_width = if table_layout_fixed {
        let mut col_has_width = vec![false; num_cols];
        // Gather first row cell widths
        for placement in cell_placements.values() {
            if placement.row_idx == 0
                && let Some(cell_style) = styles.get(&placement.node)
            {
                let w_px = get_px(cell_style, "width", 0.0);
                if w_px > 0.0 {
                    let share = w_px / placement.colspan as f32;
                    for c in placement.col_idx..(placement.col_idx + placement.colspan) {
                        if c < num_cols {
                            col_widths[c] = share;
                            col_has_width[c] = true;
                        }
                    }
                }
            }
        }

        let sum_explicit_widths: f32 = col_widths.iter().sum();
        let leftover = avail_content_width - sum_explicit_widths;
        let auto_count = col_has_width.iter().filter(|&&has| !has).count();
        if auto_count > 0 {
            let share = if leftover > 0.0 {
                leftover / auto_count as f32
            } else {
                0.0
            };
            for c in 0..num_cols {
                if !col_has_width[c] {
                    col_widths[c] = share;
                }
            }
        }
        // TODO(spec): percentage column widths, <colgroup>/<col> width sources, and width: auto table (treat table width as already resolved by the existing code)
        content_width
    } else {
        // First, handle all cells with colspan == 1 to establish baseline column widths
        for placement in cell_placements.values() {
            if placement.colspan == 1 {
                let cell_pref_w = get_cell_preferred_width(
                    dom,
                    styles,
                    placement.node,
                    avail_content_width,
                    depth + 1,
                );
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
            let cell_pref_w = get_cell_preferred_width(
                dom,
                styles,
                placement.node,
                avail_content_width,
                depth + 1,
            );
            let current_combined: f32 = col_widths
                [placement.col_idx..(placement.col_idx + placement.colspan)]
                .iter()
                .sum::<f32>()
                + (placement.colspan - 1) as f32 * spacing_h;
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
        let has_definite_width = style.reset_box.width != -1;

        let final_content_width = if has_definite_width {
            content_width
        } else {
            (sum_col_widths + total_spacing_h).min(content_width)
        };

        let avail_col_width = (final_content_width - total_spacing_h).max(0.0);

        if sum_col_widths > 0.0 {
            if avail_col_width > sum_col_widths {
                // Distribute remaining space equally to match table's final width
                let remaining = avail_col_width - sum_col_widths;
                let share = remaining / num_cols as f32;
                for w in &mut col_widths {
                    *w += share;
                }
            } else if avail_col_width < sum_col_widths {
                // Scale down columns proportionally
                let scale = avail_col_width / sum_col_widths;
                for w in &mut col_widths {
                    *w *= scale;
                }
            }
        }
        final_content_width
    };

    // Pre-layout every cell to determine height based on final column widths
    let mut cell_boxes = HashMap::new();
    for (&cell_node, placement) in &cell_placements {
        let cell_width: f32 = col_widths
            [placement.col_idx..(placement.col_idx + placement.colspan)]
            .iter()
            .sum::<f32>()
            + (placement.colspan - 1) as f32 * spacing_h;
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
                .sum::<f32>()
                + (placement.rowspan - 1) as f32 * spacing_v;
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

    let table_width =
        final_content_width + padding_left + padding_right + border_left + border_right;

    let mut caption_box = None;
    let mut caption_height = 0.0_f32;
    if let Some(caption_node) = first_caption
        && let Some(cap_box) = layout_node(
            dom,
            styles,
            caption_node,
            table_width,
            border_box_x,
            border_box_y,
            depth + 1,
        )
    {
        caption_height = cap_box.rect.size.height;
        caption_box = Some(cap_box);
    }

    let mut row_y_offsets = vec![0.0_f32; rows.len()];
    let mut curr_y = border_box_y + border_top + padding_top;
    if !caption_side_bottom {
        curr_y += caption_height;
    }
    let row_start_y = curr_y;
    for r in 0..rows.len() {
        curr_y += spacing_v;
        row_y_offsets[r] = curr_y;
        curr_y += row_heights[r];
    }
    curr_y += spacing_v;

    let mut table_children = Vec::new();
    let mut bottom_caption_box = None;
    if let Some(mut cap_box) = caption_box {
        if caption_side_bottom {
            let target_y = curr_y + padding_bottom + border_bottom;
            let dy = target_y - cap_box.rect.origin.y;
            translate_y(&mut cap_box, dy);
            bottom_caption_box = Some(cap_box);
        } else {
            table_children.push(cap_box);
        }
    }

    // Lay out rows and place cells
    for (r, row_info) in rows.into_iter().enumerate() {
        let mut row_cells_boxes = Vec::new();

        for &cell_node in &row_info.cells {
            if let Some(placement) = cell_placements.get(&cell_node) {
                let col_idx = placement.col_idx;
                let col_offset_x: f32 = col_widths[0..col_idx].iter().sum::<f32>();
                let cell_x = border_box_x
                    + border_left
                    + padding_left
                    + (col_idx + 1) as f32 * spacing_h
                    + col_offset_x;
                let cell_y = row_y_offsets[r];

                let cell_width: f32 = col_widths[col_idx..(col_idx + placement.colspan)]
                    .iter()
                    .sum::<f32>()
                    + (placement.colspan - 1) as f32 * spacing_h;
                let cell_height: f32 = row_heights[r..(r + placement.rowspan)].iter().sum::<f32>()
                    + (placement.rowspan - 1) as f32 * spacing_v;

                if let Some(mut cell_box) = layout_node(
                    dom,
                    styles,
                    cell_node,
                    cell_width,
                    cell_x,
                    cell_y,
                    depth + 2,
                ) {
                    let natural_height = cell_box.rect.size.height;
                    cell_box.rect.size.width = cell_width;
                    cell_box.rect.size.height = cell_height;

                    if cell_height > natural_height {
                        let mut dy = 0.0;
                        if let Some(cs) = styles.get(&cell_node) {
                            match cs.reset_box.vertical_align {
                                -6 => {
                                    // middle
                                    dy = (cell_height - natural_height) / 2.0;
                                }
                                -5 => {
                                    // bottom
                                    dy = cell_height - natural_height;
                                }
                                _ => {} // top and others
                            }
                        }
                        if dy > 0.0 {
                            for child in &mut cell_box.children {
                                translate_y(child, dy);
                            }
                        }
                    }

                    row_cells_boxes.push(cell_box);
                }
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

    if let Some(cap_box) = bottom_caption_box {
        table_children.push(cap_box);
    }

    let final_content_height = curr_y - row_start_y;
    let border_box_height = final_content_height
        + padding_top
        + padding_bottom
        + border_top
        + border_bottom
        + caption_height;

    Some(LayoutBox {
        node: Some(node),
        rect: Rect::new(border_box_x, border_box_y, table_width, border_box_height),
        children: table_children,
        text: None,
    })
}

pub(crate) fn is_border_collapse(style: &CategorizedComputedStyle) -> bool {
    style.inherited_table.border_collapse == "collapse"
}

fn get_border_spacing(style: &CategorizedComputedStyle) -> (f32, f32) {
    let bs = style.inherited_table.border_spacing as f32;
    (bs, bs)
}

fn gather_table_rows(
    dom: &Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    node: NodeId,
) -> Vec<TableRowInfo> {
    let mut rows = Vec::new();
    let mut implicit_row_cells = Vec::new();

    fn traverse(
        dom: &Dom,
        styles: &HashMap<NodeId, CategorizedComputedStyle>,
        node: NodeId,
        rows: &mut Vec<TableRowInfo>,
        implicit_row_cells: &mut Vec<NodeId>,
    ) {
        let layoutable_children = get_layoutable_children(dom, styles, node);
        for &child in &layoutable_children {
            if let Some(NodeData::Element { name, .. }) = dom.data(child)
                && name == "caption"
            {
                continue;
            }
            let style = styles.get(&child);
            let is_row = style.is_some_and(|s| {
                s.reset_box.display == "table-row" || s.reset_box.display == "table-row-group"
            }) || is_table_row_element(dom, child);
            let is_cell = style.is_some_and(|s| s.reset_box.display == "table-cell")
                || is_table_cell_element(dom, child);

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
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    row_node: NodeId,
    cells: &mut Vec<NodeId>,
) {
    let layoutable_children = get_layoutable_children(dom, styles, row_node);
    for &child in &layoutable_children {
        let style = styles.get(&child);
        let is_cell = style.is_some_and(|s| s.reset_box.display == "table-cell")
            || is_table_cell_element(dom, child);
        if is_cell {
            cells.push(child);
        } else {
            gather_row_cells(dom, styles, child, cells);
        }
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
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    cell_node: NodeId,
    content_width: f32,
    depth: usize,
) -> f32 {
    let mut width = 0.0_f32;
    if let Some(cs) = styles.get(&cell_node)
        && cs.reset_box.width != -1
    {
        width = cs.reset_box.width as f32;
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
    use crate::style::CategorizedComputedStyle;
    use std::collections::HashMap;

    fn style_with_display(display: &str) -> CategorizedComputedStyle {
        let mut s = CategorizedComputedStyle::default();
        let val = match display {
            "table" => CssValue::Display(DisplayValue::Table),
            "table-row" => CssValue::Display(DisplayValue::TableRow),
            "table-cell" => CssValue::Display(DisplayValue::TableCell),
            _ => CssValue::Keyword(display.to_string()),
        };
        s.set_property("display", &val);
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

        // Add a block child to cell 1.1 to verify child positioning
        let cell11_child_node = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(cell11_node, cell11_child_node);

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

        // Add a block child to cell 2.1 to verify child positioning
        let cell21_child_node = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(cell21_node, cell21_child_node);

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

        // Cell 1.1 child: width 20px, height 10px
        let mut cell11_child_style = style_with_display("block");
        cell11_child_style.insert("width".to_string(), CssValue::Length(20.0, LengthUnit::Px));
        cell11_child_style.insert("height".to_string(), CssValue::Length(10.0, LengthUnit::Px));
        styles.insert(cell11_child_node, cell11_child_style);

        // Cell 2.1 child: width 20px, height 10px
        let mut cell21_child_style = style_with_display("block");
        cell21_child_style.insert("width".to_string(), CssValue::Length(20.0, LengthUnit::Px));
        cell21_child_style.insert("height".to_string(), CssValue::Length(10.0, LengthUnit::Px));
        styles.insert(cell21_child_node, cell21_child_style);

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

        // Verify child positioning of Cell 1.1: child should be translated with the cell
        assert_eq!(cell11_box.children.len(), 1);
        let cell11_child_box = &cell11_box.children[0];
        assert_eq!(cell11_child_box.node, Some(cell11_child_node));
        assert_eq!(cell11_child_box.rect.origin.x, 10.0);
        assert_eq!(cell11_child_box.rect.origin.y, 20.0);

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

        // Verify child positioning of Cell 2.1: child should be translated with the cell
        assert_eq!(cell21_box.children.len(), 1);
        let cell21_child_box = &cell21_box.children[0];
        assert_eq!(cell21_child_box.node, Some(cell21_child_node));
        assert_eq!(cell21_child_box.rect.origin.x, 10.0);
        assert_eq!(cell21_child_box.rect.origin.y, 60.0);
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
            style.set_width(25);
            style.set_height(10);
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

    #[test]
    fn test_table_caption_layout() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Create table element
        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Create caption element (first child)
        let caption_node = dom.create_node(NodeData::Element {
            name: "caption".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, caption_node);

        // Create row
        let row_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row_node);

        // Create cell
        let cell_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell_node);

        let mut styles = HashMap::new();

        // Table style: width 200px
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        styles.insert(table_node, table_style);

        // Caption style: height 40px
        let mut caption_style = style_with_display("block");
        caption_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(caption_node, caption_style);

        // Row style
        styles.insert(row_node, style_with_display("table-row"));

        // Cell style: width 200px, height 30px
        let mut cell_style = style_with_display("table-cell");
        cell_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        cell_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell_node, cell_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 10.0, 20.0, 0)
            .expect("should layout table");

        // Checks:
        // 1. Table width is 200px
        assert_eq!(table_box.rect.size.width, 200.0);
        // 2. Table total height should include caption (40px) + cell/row (30px) = 70px
        assert_eq!(table_box.rect.size.height, 70.0);

        // 3. Children of table should be: caption and then the row
        assert_eq!(table_box.children.len(), 2);

        let cap_box = &table_box.children[0];
        assert_eq!(cap_box.node, Some(caption_node));
        assert_eq!(cap_box.rect.origin.x, 10.0); // starts at table's border_box_x
        assert_eq!(cap_box.rect.origin.y, 20.0); // starts at table's border_box_y
        assert_eq!(cap_box.rect.size.width, 200.0); // spans full table width
        assert_eq!(cap_box.rect.size.height, 40.0);

        let row_box = &table_box.children[1];
        assert_eq!(row_box.node, Some(row_node));
        assert_eq!(row_box.rect.origin.x, 10.0);
        assert_eq!(row_box.rect.origin.y, 20.0 + 40.0); // offset downward by caption height
        assert_eq!(row_box.rect.size.height, 30.0);

        let cell_box = &row_box.children[0];
        assert_eq!(cell_box.node, Some(cell_node));
        assert_eq!(cell_box.rect.origin.x, 10.0);
        assert_eq!(cell_box.rect.origin.y, 20.0 + 40.0); // cell is also offset downward
        assert_eq!(cell_box.rect.size.height, 30.0);
    }

    #[test]
    fn test_table_caption_side_bottom() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Create table element
        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Create caption element
        let caption_node = dom.create_node(NodeData::Element {
            name: "caption".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, caption_node);

        // Create row
        let row_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row_node);

        // Create cell
        let cell_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell_node);

        let mut styles = HashMap::new();

        // Table style: width 200px, caption-side: bottom
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        table_style.insert(
            "caption-side".to_string(),
            CssValue::Keyword("bottom".to_string()),
        );
        styles.insert(table_node, table_style);

        // Caption style: height 40px
        let mut caption_style = style_with_display("block");
        caption_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(caption_node, caption_style);

        // Row style
        styles.insert(row_node, style_with_display("table-row"));

        // Cell style: width 200px, height 30px
        let mut cell_style = style_with_display("table-cell");
        cell_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        cell_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell_node, cell_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 10.0, 20.0, 0)
            .expect("should layout table");

        // Checks:
        // 1. Table width is 200px
        assert_eq!(table_box.rect.size.width, 200.0);
        // 2. Table total height should include caption (40px) + cell/row (30px) = 70px
        assert_eq!(table_box.rect.size.height, 70.0);

        // 3. Children of table should be: row and then the caption
        assert_eq!(table_box.children.len(), 2);

        let row_box = &table_box.children[0];
        assert_eq!(row_box.node, Some(row_node));
        assert_eq!(row_box.rect.origin.x, 10.0);
        assert_eq!(row_box.rect.origin.y, 20.0); // starts at normal top position (not offset)
        assert_eq!(row_box.rect.size.height, 30.0);

        let cell_box = &row_box.children[0];
        assert_eq!(cell_box.node, Some(cell_node));
        assert_eq!(cell_box.rect.origin.y, 20.0); // cell also starts at normal top position

        let cap_box = &table_box.children[1];
        assert_eq!(cap_box.node, Some(caption_node));
        assert_eq!(cap_box.rect.origin.x, 10.0);
        assert_eq!(cap_box.rect.origin.y, 20.0 + 30.0); // positioned below the row (row top 20.0 + row height 30.0 = 50.0)
        assert_eq!(cap_box.rect.size.width, 200.0);
        assert_eq!(cap_box.rect.size.height, 40.0);

        // Assert that caption top y (50.0) is greater than row top y (20.0)
        assert!(cap_box.rect.origin.y > row_box.rect.origin.y);
    }

    #[test]
    fn test_table_border_spacing_separated_control() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // 2x2 table
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

        // Cell 1.1
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
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

        // Cell 2.2
        let cell22_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell22_node);

        let mut styles = HashMap::new();

        // Table style: width 200px
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        // No border-spacing set -> default 0.0
        styles.insert(table_node, table_style);

        // Rows
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        // Cell 1.1: width 60px, height 30px
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(60.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        // Cell 1.2: width 40px, height 30px
        let mut cell12_style = style_with_display("table-cell");
        cell12_style.insert("width".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell12_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell12_node, cell12_style);

        // Cell 2.1: width 60px, height 40px
        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(60.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell21_node, cell21_style);

        // Cell 2.2: width 40px, height 40px
        let mut cell22_style = style_with_display("table-cell");
        cell22_style.insert("width".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell22_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell22_node, cell22_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 10.0, 20.0, 0)
            .expect("should layout table");

        // Table box checks
        assert_eq!(table_box.rect.origin.x, 10.0);
        assert_eq!(table_box.rect.origin.y, 20.0);
        assert_eq!(table_box.rect.size.width, 200.0);
        assert_eq!(table_box.rect.size.height, 70.0); // 30 + 40

        assert_eq!(table_box.children.len(), 2);

        // Row 1 checks
        let r1 = &table_box.children[0];
        assert_eq!(r1.rect.origin.y, 20.0);
        assert_eq!(r1.rect.size.height, 30.0);
        assert_eq!(r1.children.len(), 2);

        // Cell 1.1 (Row 1 Col 0) checks
        let c11 = &r1.children[0];
        assert_eq!(c11.rect.origin.x, 10.0);
        assert_eq!(c11.rect.origin.y, 20.0);
        assert_eq!(c11.rect.size.width, 110.0); // 60 + 50
        assert_eq!(c11.rect.size.height, 30.0);

        // Cell 1.2 (Row 1 Col 1) checks
        let c12 = &r1.children[1];
        assert_eq!(c12.rect.origin.x, 10.0 + 110.0);
        assert_eq!(c12.rect.origin.y, 20.0);
        assert_eq!(c12.rect.size.width, 90.0); // 40 + 50
        assert_eq!(c12.rect.size.height, 30.0);

        // Row 2 checks
        let r2 = &table_box.children[1];
        assert_eq!(r2.rect.origin.y, 20.0 + 30.0);
        assert_eq!(r2.rect.size.height, 40.0);
        assert_eq!(r2.children.len(), 2);

        // Cell 2.1 (Row 2 Col 0) checks
        let c21 = &r2.children[0];
        assert_eq!(c21.rect.origin.x, 10.0);
        assert_eq!(c21.rect.origin.y, 50.0);
        assert_eq!(c21.rect.size.width, 110.0);
        assert_eq!(c21.rect.size.height, 40.0);

        // Cell 2.2 (Row 2 Col 1) checks
        let c22 = &r2.children[1];
        assert_eq!(c22.rect.origin.x, 10.0 + 110.0);
        assert_eq!(c22.rect.origin.y, 50.0);
        assert_eq!(c22.rect.size.width, 90.0);
        assert_eq!(c22.rect.size.height, 40.0);
    }

    #[test]
    fn test_table_border_spacing_separated_with_spacing() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // 2x2 table
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

        // Cell 1.1
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
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

        // Cell 2.2
        let cell22_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell22_node);

        let mut styles = HashMap::new();

        // Table style: width 200px, border-spacing: 10px
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        table_style.insert(
            "border-spacing".to_string(),
            CssValue::Length(10.0, LengthUnit::Px),
        );
        styles.insert(table_node, table_style);

        // Rows
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        // Cell 1.1: width 60px, height 30px
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(60.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        // Cell 1.2: width 40px, height 30px
        let mut cell12_style = style_with_display("table-cell");
        cell12_style.insert("width".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell12_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell12_node, cell12_style);

        // Cell 2.1: width 60px, height 40px
        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(60.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell21_node, cell21_style);

        // Cell 2.2: width 40px, height 40px
        let mut cell22_style = style_with_display("table-cell");
        cell22_style.insert("width".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell22_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell22_node, cell22_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 10.0, 20.0, 0)
            .expect("should layout table");

        // Table box checks
        assert_eq!(table_box.rect.origin.x, 10.0);
        assert_eq!(table_box.rect.origin.y, 20.0);
        assert_eq!(table_box.rect.size.width, 200.0);
        // Table total height = row 1 (30px) + row 2 (40px) + 3 spacings (30px) = 100px
        assert_eq!(table_box.rect.size.height, 100.0);

        assert_eq!(table_box.children.len(), 2);

        // Row 1 checks
        let r1 = &table_box.children[0];
        assert_eq!(r1.rect.origin.y, 30.0); // 20.0 + 10.0 (spacing)
        assert_eq!(r1.rect.size.height, 30.0);
        assert_eq!(r1.children.len(), 2);

        // Cell 1.1 checks
        let c11 = &r1.children[0];
        assert_eq!(c11.rect.origin.x, 20.0); // 10.0 + 10.0 (spacing)
        assert_eq!(c11.rect.origin.y, 30.0);
        assert_eq!(c11.rect.size.width, 95.0); // Col 0: 60 + 35
        assert_eq!(c11.rect.size.height, 30.0);

        // Cell 1.2 checks
        let c12 = &r1.children[1];
        assert_eq!(c12.rect.origin.x, 125.0); // 10.0 + 20.0 (spacing) + 95.0 (Col 0)
        assert_eq!(c12.rect.origin.y, 30.0);
        assert_eq!(c12.rect.size.width, 75.0); // Col 1: 40 + 35
        assert_eq!(c12.rect.size.height, 30.0);

        // Row 2 checks
        let r2 = &table_box.children[1];
        assert_eq!(r2.rect.origin.y, 70.0); // 30.0 + 30.0 (Row 1) + 10.0 (spacing)
        assert_eq!(r2.rect.size.height, 40.0);
        assert_eq!(r2.children.len(), 2);

        // Cell 2.1 checks
        let c21 = &r2.children[0];
        assert_eq!(c21.rect.origin.x, 20.0);
        assert_eq!(c21.rect.origin.y, 70.0);
        assert_eq!(c21.rect.size.width, 95.0);
        assert_eq!(c21.rect.size.height, 40.0);

        // Cell 2.2 checks
        let c22 = &r2.children[1];
        assert_eq!(c22.rect.origin.x, 125.0);
        assert_eq!(c22.rect.origin.y, 70.0);
        assert_eq!(c22.rect.size.width, 75.0);
        assert_eq!(c22.rect.size.height, 40.0);
    }

    #[test]
    fn test_table_border_collapse_zeroes_spacing() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // 2x2 table
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

        // Cell 1.1
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
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

        // Cell 2.2
        let cell22_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell22_node);

        let mut styles = HashMap::new();

        // Table style: width 200px, border-spacing: 10px, border-collapse: collapse
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        table_style.insert(
            "border-spacing".to_string(),
            CssValue::Length(10.0, LengthUnit::Px),
        );
        table_style.insert(
            "border-collapse".to_string(),
            CssValue::Keyword("collapse".to_string()),
        );
        styles.insert(table_node, table_style);

        // Rows
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        // Cell 1.1: width 60px, height 30px
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(60.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        // Cell 1.2: width 40px, height 30px
        let mut cell12_style = style_with_display("table-cell");
        cell12_style.insert("width".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell12_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell12_node, cell12_style);

        // Cell 2.1: width 60px, height 40px
        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(60.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell21_node, cell21_style);

        // Cell 2.2: width 40px, height 40px
        let mut cell22_style = style_with_display("table-cell");
        cell22_style.insert("width".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell22_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell22_node, cell22_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 10.0, 20.0, 0)
            .expect("should layout table");

        // Table box checks
        assert_eq!(table_box.rect.origin.x, 10.0);
        assert_eq!(table_box.rect.origin.y, 20.0);
        assert_eq!(table_box.rect.size.width, 200.0);
        // Table total height = row 1 (30px) + row 2 (40px) = 70px (spacing ignored)
        assert_eq!(table_box.rect.size.height, 70.0);

        assert_eq!(table_box.children.len(), 2);

        // Row 1 checks
        let r1 = &table_box.children[0];
        assert_eq!(r1.rect.origin.y, 20.0); // starts immediately at border_box_y (no spacing)
        assert_eq!(r1.rect.size.height, 30.0);
        assert_eq!(r1.children.len(), 2);

        // Cell 1.1 checks
        let c11 = &r1.children[0];
        assert_eq!(c11.rect.origin.x, 10.0); // starts at border_box_x (no spacing)
        assert_eq!(c11.rect.origin.y, 20.0);
        assert_eq!(c11.rect.size.width, 110.0); // Col 0: 60 + 50 (extra width shared equally from 100px remaining)
        assert_eq!(c11.rect.size.height, 30.0);

        // Cell 1.2 checks
        let c12 = &r1.children[1];
        // Adjacent cells abut: c11.rect.origin.x + c11.rect.size.width = 10.0 + 110.0 = 120.0
        assert_eq!(c12.rect.origin.x, 120.0);
        assert_eq!(c12.rect.origin.y, 20.0);
        assert_eq!(c12.rect.size.width, 90.0); // Col 1: 40 + 50
        assert_eq!(c12.rect.size.height, 30.0);

        // Row 2 checks
        let r2 = &table_box.children[1];
        assert_eq!(r2.rect.origin.y, 50.0); // starts immediately after Row 1 (20.0 + 30.0)
        assert_eq!(r2.rect.size.height, 40.0);
        assert_eq!(r2.children.len(), 2);

        // Cell 2.1 checks
        let c21 = &r2.children[0];
        assert_eq!(c21.rect.origin.x, 10.0);
        assert_eq!(c21.rect.origin.y, 50.0);
        assert_eq!(c21.rect.size.width, 110.0);
        assert_eq!(c21.rect.size.height, 40.0);

        // Cell 2.2 checks
        let c22 = &r2.children[1];
        assert_eq!(c22.rect.origin.x, 120.0);
        assert_eq!(c22.rect.origin.y, 50.0);
        assert_eq!(c22.rect.size.width, 90.0);
        assert_eq!(c22.rect.size.height, 40.0);
    }

    #[test]
    fn test_table_border_spacing_auto_width_growth() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // 2x2 table
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

        // Cell 1.1
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
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

        // Cell 2.2
        let cell22_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell22_node);

        let mut styles = HashMap::new();

        // Table style: NO definite width. Spacing: 10px.
        let mut table_style = style_with_display("table");
        table_style.insert(
            "border-spacing".to_string(),
            CssValue::Length(10.0, LengthUnit::Px),
        );
        styles.insert(table_node, table_style);

        // Rows
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        // Cell 1.1: width 60px, height 30px
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(60.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        // Cell 1.2: width 40px, height 30px
        let mut cell12_style = style_with_display("table-cell");
        cell12_style.insert("width".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell12_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell12_node, cell12_style);

        // Cell 2.1: width 60px, height 40px
        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(60.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell21_node, cell21_style);

        // Cell 2.2: width 40px, height 40px
        let mut cell22_style = style_with_display("table-cell");
        cell22_style.insert("width".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell22_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell22_node, cell22_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 10.0, 20.0, 0)
            .expect("should layout table");

        // Table box checks
        assert_eq!(table_box.rect.origin.x, 10.0);
        assert_eq!(table_box.rect.origin.y, 20.0);
        // Table total width = columns preferred (60 + 40 = 100px) + 3 spacings (30px) = 130px
        assert_eq!(table_box.rect.size.width, 130.0);
        // Table total height = row 1 (30px) + row 2 (40px) + 3 spacings (30px) = 100px
        assert_eq!(table_box.rect.size.height, 100.0);

        assert_eq!(table_box.children.len(), 2);

        // Row 1 checks
        let r1 = &table_box.children[0];
        assert_eq!(r1.rect.origin.y, 30.0); // 20.0 + 10.0 (spacing)
        assert_eq!(r1.rect.size.height, 30.0);
        assert_eq!(r1.children.len(), 2);

        // Cell 1.1 checks
        let c11 = &r1.children[0];
        assert_eq!(c11.rect.origin.x, 20.0); // 10.0 + 10.0 (spacing)
        assert_eq!(c11.rect.origin.y, 30.0);
        assert_eq!(c11.rect.size.width, 60.0); // Col 0: exactly preferred width 60px since auto-width table
        assert_eq!(c11.rect.size.height, 30.0);

        // Cell 1.2 checks
        let c12 = &r1.children[1];
        assert_eq!(c12.rect.origin.x, 90.0); // 10.0 + 20.0 (spacing) + 60.0 (Col 0)
        assert_eq!(c12.rect.origin.y, 30.0);
        assert_eq!(c12.rect.size.width, 40.0); // Col 1: exactly preferred width 40px since auto-width table
        assert_eq!(c12.rect.size.height, 30.0);

        // Row 2 checks
        let r2 = &table_box.children[1];
        assert_eq!(r2.rect.origin.y, 70.0); // 30.0 + 30.0 (Row 1) + 10.0 (spacing)
        assert_eq!(r2.rect.size.height, 40.0);
        assert_eq!(r2.children.len(), 2);

        // Cell 2.1 checks
        let c21 = &r2.children[0];
        assert_eq!(c21.rect.origin.x, 20.0);
        assert_eq!(c21.rect.origin.y, 70.0);
        assert_eq!(c21.rect.size.width, 60.0);
        assert_eq!(c21.rect.size.height, 40.0);

        // Cell 2.2 checks
        let c22 = &r2.children[1];
        assert_eq!(c22.rect.origin.x, 90.0);
        assert_eq!(c22.rect.origin.y, 70.0);
        assert_eq!(c22.rect.size.width, 40.0);
        assert_eq!(c22.rect.size.height, 40.0);
    }

    #[test]
    fn test_table_cell_vertical_align() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Create table element
        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Create row 1
        let row_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row_node);

        // Create cell 1 (tall)
        let cell1_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell1_node);
        let cell1_child_node = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(cell1_node, cell1_child_node);

        // Create cell 2 (middle)
        let cell2_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell2_node);
        let cell2_child_node = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(cell2_node, cell2_child_node);

        // Create cell 3 (bottom)
        let cell3_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell3_node);
        let cell3_child_node = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(cell3_node, cell3_child_node);

        // Create cell 4 (top)
        let cell4_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell4_node);
        let cell4_child_node = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(cell4_node, cell4_child_node);

        // Setup styles
        let mut styles = HashMap::new();

        // Table style
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(400.0, LengthUnit::Px));
        styles.insert(table_node, table_style);

        // Row style
        styles.insert(row_node, style_with_display("table-row"));

        // Cell 1: height 100px
        let mut cell1_style = style_with_display("table-cell");
        cell1_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell1_style.insert(
            "height".to_string(),
            CssValue::Length(100.0, LengthUnit::Px),
        );
        styles.insert(cell1_node, cell1_style);

        let mut cell1_child_style = style_with_display("block");
        cell1_child_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        cell1_child_style.insert(
            "height".to_string(),
            CssValue::Length(100.0, LengthUnit::Px),
        );
        styles.insert(cell1_child_node, cell1_child_style);

        // Cell 2: height 40px, middle
        let mut cell2_style = style_with_display("table-cell");
        cell2_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell2_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell2_style.insert(
            "vertical-align".to_string(),
            CssValue::Keyword("middle".to_string()),
        );
        styles.insert(cell2_node, cell2_style);

        let mut cell2_child_style = style_with_display("block");
        cell2_child_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        cell2_child_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell2_child_node, cell2_child_style);

        // Cell 3: height 40px, bottom
        let mut cell3_style = style_with_display("table-cell");
        cell3_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell3_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell3_style.insert(
            "vertical-align".to_string(),
            CssValue::Keyword("bottom".to_string()),
        );
        styles.insert(cell3_node, cell3_style);

        let mut cell3_child_style = style_with_display("block");
        cell3_child_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        cell3_child_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell3_child_node, cell3_child_style);

        // Cell 4: height 40px, top
        let mut cell4_style = style_with_display("table-cell");
        cell4_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell4_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell4_style.insert(
            "vertical-align".to_string(),
            CssValue::Keyword("top".to_string()),
        );
        styles.insert(cell4_node, cell4_style);

        let mut cell4_child_style = style_with_display("block");
        cell4_child_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        cell4_child_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell4_child_node, cell4_child_style);

        // Document layout (offset_x = 0, offset_y = 0)
        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // The single row
        assert_eq!(table_box.children.len(), 1);
        let row_box = &table_box.children[0];
        assert_eq!(row_box.children.len(), 4);

        // Row should start at y = 0 since no border spacing, padding, margins or captions
        let cell_y = row_box.rect.origin.y;

        // Cell 1: tall, height = 100.0
        let c1 = &row_box.children[0];
        assert_eq!(c1.rect.size.height, 100.0);
        let c1_child = &c1.children[0];
        assert_eq!(c1_child.rect.origin.y, cell_y);

        // Cell 2: middle, height = 100.0, dy = (100 - 40) / 2 = 30
        let c2 = &row_box.children[1];
        assert_eq!(c2.rect.size.height, 100.0);
        let c2_child = &c2.children[0];
        assert_eq!(c2_child.rect.origin.y, cell_y + 30.0);

        // Cell 3: bottom, height = 100.0, dy = 100 - 40 = 60
        let c3 = &row_box.children[2];
        assert_eq!(c3.rect.size.height, 100.0);
        let c3_child = &c3.children[0];
        assert_eq!(c3_child.rect.origin.y, cell_y + 60.0);

        // Cell 4: top, height = 100.0, dy = 0
        let c4 = &row_box.children[3];
        assert_eq!(c4.rect.size.height, 100.0);
        let c4_child = &c4.children[0];
        assert_eq!(c4_child.rect.origin.y, cell_y);
    }

    #[test]
    fn test_table_layout_fixed_algorithm() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Create table element
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

        // Cell 1.1 (width: 150px)
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell11_node);

        // Cell 1.2 (auto)
        let cell12_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell12_node);

        // Cell 1.3 (auto)
        let cell13_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell13_node);

        // Row 2
        let row2_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row2_node);

        // Cell 2.1 (has a very wide child, 400px)
        let cell21_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell21_node);
        let cell21_child = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(cell21_node, cell21_child);

        // Cell 2.2 (auto)
        let cell22_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell22_node);

        // Cell 2.3 (auto)
        let cell23_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell23_node);

        // Setup styles
        let mut styles = HashMap::new();

        // Table style: width 300px, table-layout: fixed, border-collapse: collapse
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(300.0, LengthUnit::Px));
        table_style.insert(
            "table-layout".to_string(),
            CssValue::Keyword("fixed".to_string()),
        );
        table_style.insert(
            "border-collapse".to_string(),
            CssValue::Keyword("collapse".to_string()),
        );
        styles.insert(table_node, table_style);

        // Rows
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        // Row 1 Cell widths
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(150.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        styles.insert(cell12_node, style_with_display("table-cell"));
        styles.insert(cell13_node, style_with_display("table-cell"));

        // Row 2 Cell styles and the wide child style
        styles.insert(cell21_node, style_with_display("table-cell"));
        let mut wide_style = style_with_display("block");
        wide_style.insert("width".to_string(), CssValue::Length(400.0, LengthUnit::Px));
        styles.insert(cell21_child, wide_style);

        styles.insert(cell22_node, style_with_display("table-cell"));
        styles.insert(cell23_node, style_with_display("table-cell"));

        // Perform Layout
        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // The first row box
        let r1_box = &table_box.children[0];
        assert_eq!(r1_box.children.len(), 3);

        // Verify Column widths:
        // Column 1 should be exactly 150px.
        // Column 2 and 3 should share the remaining 150px equally (75px each).
        let c1 = &r1_box.children[0];
        let c2 = &r1_box.children[1];
        let c3 = &r1_box.children[2];

        assert_eq!(c1.rect.size.width, 150.0);
        assert_eq!(c2.rect.size.width, 75.0);
        assert_eq!(c3.rect.size.width, 75.0);

        // The second row box
        let r2_box = &table_box.children[1];
        assert_eq!(r2_box.children.len(), 3);
        let c21_box = &r2_box.children[0];
        let c22_box = &r2_box.children[1];
        let c23_box = &r2_box.children[2];

        // Column widths in row 2 must match row 1 exactly (content of row 2 cell did not affect them!)
        assert_eq!(c21_box.rect.size.width, 150.0);
        assert_eq!(c22_box.rect.size.width, 75.0);
        assert_eq!(c23_box.rect.size.width, 75.0);
    }

    #[test]
    fn test_table_layout_auto_control() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Create table element
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

        // Cell 1.1 (width: 150px)
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell11_node);

        // Cell 1.2 (auto)
        let cell12_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell12_node);

        // Cell 1.3 (auto)
        let cell13_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell13_node);

        // Row 2
        let row2_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row2_node);

        // Cell 2.1 (has a very wide child, 400px)
        let cell21_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell21_node);
        let cell21_child = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(cell21_node, cell21_child);

        // Cell 2.2 (auto)
        let cell22_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell22_node);

        // Cell 2.3 (auto)
        let cell23_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell23_node);

        // Setup styles
        let mut styles = HashMap::new();

        // Table style: width 300px, border-collapse: collapse (omitted table-layout means default auto)
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(300.0, LengthUnit::Px));
        table_style.insert(
            "border-collapse".to_string(),
            CssValue::Keyword("collapse".to_string()),
        );
        styles.insert(table_node, table_style);

        // Rows
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        // Row 1 Cell widths
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(150.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        styles.insert(cell12_node, style_with_display("table-cell"));
        styles.insert(cell13_node, style_with_display("table-cell"));

        // Row 2 Cell styles and the wide child style
        styles.insert(cell21_node, style_with_display("table-cell"));
        let mut wide_style = style_with_display("block");
        wide_style.insert("width".to_string(), CssValue::Length(400.0, LengthUnit::Px));
        styles.insert(cell21_child, wide_style);

        styles.insert(cell22_node, style_with_display("table-cell"));
        styles.insert(cell23_node, style_with_display("table-cell"));

        // Perform Layout
        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // The first row box
        let r1_box = &table_box.children[0];
        assert_eq!(r1_box.children.len(), 3);

        // In auto layout, the auto cells in the columns make their preferred width large,
        // and because sum of preferred widths exceeds table width, they are scaled down proportionally.
        // Thus Column 1 width ends up being 100.0 (less than 150.0), whereas in fixed layout it was exactly 150.0.
        let c1 = &r1_box.children[0];
        assert_eq!(c1.rect.size.width, 100.0);
        let c2 = &r1_box.children[1];
        assert_eq!(c2.rect.size.width, 100.0);
        let c3 = &r1_box.children[2];
        assert_eq!(c3.rect.size.width, 100.0);
    }
}
