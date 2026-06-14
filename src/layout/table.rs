use crate::css::values::{CssValue, LengthUnit};
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
    let is_collapsed = is_border_collapse(style);
    let padding_left = if is_collapsed {
        0.0
    } else {
        get_px(style, "padding-left", 0.0)
    };
    let padding_right = if is_collapsed {
        0.0
    } else {
        get_px(style, "padding-right", 0.0)
    };
    let padding_top = if is_collapsed {
        0.0
    } else {
        get_px(style, "padding-top", 0.0)
    };
    let padding_bottom = if is_collapsed {
        0.0
    } else {
        get_px(style, "padding-bottom", 0.0)
    };

    let mut border_left = get_px(style, "border-left-width", 0.0);
    let mut border_right = get_px(style, "border-right-width", 0.0);
    let mut border_top = get_px(style, "border-top-width", 0.0);
    let mut border_bottom = get_px(style, "border-bottom-width", 0.0);

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

    // Find all captions among layoutable children
    let layoutable_children = get_layoutable_children(dom, styles, node);
    let mut captions = Vec::new();
    for &child in &layoutable_children {
        if let Some(NodeData::Element { name, .. }) = dom.data(child)
            && name == "caption"
        {
            captions.push(child);
        }
    }

    // Gather table rows
    let rows = gather_table_rows(dom, styles, node);
    if rows.is_empty() {
        let table_width = content_width + padding_left + padding_right + border_left + border_right;
        if captions.is_empty() {
            return Some(LayoutBox {
                node: Some(node),
                rect: Rect::new(
                    border_box_x,
                    border_box_y,
                    table_width,
                    padding_top + padding_bottom + border_top + border_bottom,
                ),
                children: Vec::new(),
                text: None,
            });
        } else {
            let mut top_caption_boxes = Vec::new();
            let mut bottom_caption_boxes = Vec::new();

            for &caption_node in &captions {
                let is_bottom = if let Some(style) = styles.get(&caption_node) {
                    style.inherited_table.caption_side == "bottom" || caption_side_bottom
                } else {
                    caption_side_bottom
                };

                if let Some(cap_box) = layout_node(
                    dom,
                    styles,
                    caption_node,
                    table_width,
                    border_box_x,
                    border_box_y,
                    depth + 1,
                ) {
                    if is_bottom {
                        bottom_caption_boxes.push(cap_box);
                    } else {
                        top_caption_boxes.push(cap_box);
                    }
                }
            }

            let mut curr_y = border_box_y;
            let mut wrapper_children = Vec::new();

            // Position top captions (stacking top to bottom)
            for mut cap_box in top_caption_boxes {
                let dy = curr_y - cap_box.rect.origin.y;
                translate_y(&mut cap_box, dy);
                curr_y += cap_box.rect.size.height;
                wrapper_children.push(cap_box);
            }

            let grid_box_y = curr_y;
            let grid_box_height = padding_top + padding_bottom + border_top + border_bottom;
            curr_y += grid_box_height;

            let empty_table_grid_box = LayoutBox {
                node: Some(node),
                rect: Rect::new(border_box_x, grid_box_y, table_width, grid_box_height),
                children: Vec::new(),
                text: None,
            };
            wrapper_children.push(empty_table_grid_box);

            // Position bottom captions (stacking top to bottom)
            for mut cap_box in bottom_caption_boxes {
                let dy = curr_y - cap_box.rect.origin.y;
                translate_y(&mut cap_box, dy);
                curr_y += cap_box.rect.size.height;
                wrapper_children.push(cap_box);
            }

            return Some(LayoutBox {
                node: None, // Table Wrapper Box is anonymous
                rect: Rect::new(
                    border_box_x,
                    border_box_y,
                    table_width,
                    curr_y - border_box_y,
                ),
                children: wrapper_children,
                text: None,
            });
        }
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
            let mut rowspan = parse_span_attribute(dom, cell_node, "rowspan");
            if rowspan == 0 {
                if let Some(current_row_node) = row_info.node
                    && let Some(parent_id) = dom.parent(current_row_node)
                    && parent_id != node
                {
                    let mut count = 0;
                    for future_row in &rows[r..] {
                        if let Some(fr_node) = future_row.node {
                            if dom.parent(fr_node) == Some(parent_id) {
                                count += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    rowspan = count.max(1);
                } else {
                    rowspan = (rows.len() - r).max(1);
                }
            } else {
                if let Some(current_row_node) = row_info.node
                    && let Some(parent_id) = dom.parent(current_row_node)
                    && parent_id != node
                {
                    let mut count = 0;
                    for future_row in &rows[r..] {
                        if let Some(fr_node) = future_row.node {
                            if dom.parent(fr_node) == Some(parent_id) {
                                count += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    rowspan = rowspan.clamp(1, count.max(1));
                } else {
                    let remaining_rows = rows.len() - r;
                    rowspan = rowspan.clamp(1, remaining_rows);
                }
            }

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
        // Empty table, but might have captions
        let table_width = content_width + padding_left + padding_right + border_left + border_right;
        let mut table_children = Vec::new();
        let mut top_caption_boxes = Vec::new();
        let mut bottom_caption_boxes = Vec::new();

        // Collect and layout captions
        for &caption_node in &captions {
            let is_bottom = if let Some(style) = styles.get(&caption_node) {
                style.inherited_table.caption_side == "bottom" || caption_side_bottom
            } else {
                caption_side_bottom
            };

            if let Some(cap_box) = layout_node(
                dom,
                styles,
                caption_node,
                table_width,
                border_box_x,
                border_box_y,
                depth + 1,
            ) {
                if is_bottom {
                    bottom_caption_boxes.push(cap_box);
                } else {
                    top_caption_boxes.push(cap_box);
                }
            }
        }

        // Position top captions (stacking top to bottom)
        let mut curr_y = border_box_y;
        for mut cap_box in top_caption_boxes {
            let dy = curr_y - cap_box.rect.origin.y;
            translate_y(&mut cap_box, dy);
            curr_y += cap_box.rect.size.height;
            table_children.push(cap_box);
        }

        // Space for table border box
        curr_y += padding_top + padding_bottom + border_top + border_bottom;

        // Position bottom captions (stacking top to bottom)
        for mut cap_box in bottom_caption_boxes {
            let dy = curr_y - cap_box.rect.origin.y;
            translate_y(&mut cap_box, dy);
            curr_y += cap_box.rect.size.height;
            table_children.push(cap_box);
        }

        return Some(LayoutBox {
            node: Some(node),
            rect: Rect::new(
                border_box_x,
                border_box_y,
                table_width,
                curr_y - border_box_y,
            ),
            children: table_children,
            text: None,
        });
    }

    // Determine which rows are hidden (empty-cells: hide handling in separated borders model)
    let mut row_is_hidden = vec![false; rows.len()];
    if !is_border_collapse(style) {
        for r in 0..rows.len() {
            let mut cell_nodes_in_row = Vec::new();
            for col in 0..num_cols {
                if let Some(placement) = cell_placements.values().find(|p| {
                    r >= p.row_idx
                        && r < p.row_idx + p.rowspan
                        && col >= p.col_idx
                        && col < p.col_idx + p.colspan
                }) && !cell_nodes_in_row.contains(&placement.node)
                {
                    cell_nodes_in_row.push(placement.node);
                }
            }

            if !cell_nodes_in_row.is_empty() {
                let mut all_hidden_empty = true;
                for &cell_node in &cell_nodes_in_row {
                    let cell_style = styles.get(&cell_node);
                    let is_hide =
                        cell_style.is_some_and(|s| s.inherited_effects.empty_cells == "hide");
                    let is_empty = is_cell_empty(dom, cell_node);
                    if !(is_hide && is_empty) {
                        all_hidden_empty = false;
                        break;
                    }
                }
                if all_hidden_empty {
                    row_is_hidden[r] = true;
                }
            } else {
                let table_style = styles.get(&node);
                let row_style = rows[r].node.and_then(|rn| styles.get(&rn));
                let empty_cells_val = row_style
                    .map(|s| s.inherited_effects.empty_cells.as_str())
                    .unwrap_or_else(|| {
                        table_style
                            .map(|s| s.inherited_effects.empty_cells.as_str())
                            .unwrap_or("show")
                    });
                if empty_cells_val == "hide" {
                    row_is_hidden[r] = true;
                }
            }
        }
    }

    // TODO(spec): border-collapse: full border conflict resolution (shared edges, width/precedence) not implemented; here we only collapse inter-cell spacing to zero.
    let (spacing_h, spacing_v) = if is_border_collapse(style) {
        (0.0, 0.0)
    } else {
        get_border_spacing(style)
    };
    let total_spacing_h = (num_cols + 1) as f32 * spacing_h;
    let avail_content_width = (content_width - total_spacing_h).max(0.0);

    // Determine the width of each column
    let mut col_widths = vec![0.0_f32; num_cols];

    let col_element_widths = gather_col_widths(dom, styles, node, avail_content_width);

    let mut col_is_explicit = vec![false; num_cols];
    for (c, item) in col_element_widths.iter().enumerate().take(num_cols) {
        if item.is_some() {
            col_is_explicit[c] = true;
        }
    }
    for placement in cell_placements.values() {
        if placement.colspan == 1
            && let Some(cell_style) = styles.get(&placement.node)
            && has_explicit_width(cell_style)
        {
            col_is_explicit[placement.col_idx] = true;
        }
    }

    // Let's create a local modified styles map if border-collapse is active
    let mut local_styles;
    let styles_ref = if is_collapsed {
        local_styles = styles.clone();

        struct CellBorderInfo {
            top: (f32, String),
            bottom: (f32, String),
            left: (f32, String),
            right: (f32, String),
        }

        let mut cell_borders = HashMap::new();
        for &cell_node in cell_placements.keys() {
            if let Some(cell_style) = styles.get(&cell_node) {
                let bt = get_px(cell_style, "border-top-width", 0.0);
                let bb = get_px(cell_style, "border-bottom-width", 0.0);
                let bl = get_px(cell_style, "border-left-width", 0.0);
                let br = get_px(cell_style, "border-right-width", 0.0);

                let bt_style = cell_style.reset_surround.border_top_style.clone();
                let bb_style = cell_style.reset_surround.border_bottom_style.clone();
                let bl_style = cell_style.reset_surround.border_left_style.clone();
                let br_style = cell_style.reset_surround.border_right_style.clone();

                cell_borders.insert(
                    cell_node,
                    CellBorderInfo {
                        top: (bt, bt_style),
                        bottom: (bb, bb_style),
                        left: (bl, bl_style),
                        right: (br, br_style),
                    },
                );
            }
        }

        // Table's own borders and styles
        let table_style = styles.get(&node)?;
        let t_bt = get_px(table_style, "border-top-width", 0.0);
        let t_bb = get_px(table_style, "border-bottom-width", 0.0);
        let t_bl = get_px(table_style, "border-left-width", 0.0);
        let t_br = get_px(table_style, "border-right-width", 0.0);

        let t_bt_style = table_style.reset_surround.border_top_style.clone();
        let t_bb_style = table_style.reset_surround.border_bottom_style.clone();
        let t_bl_style = table_style.reset_surround.border_left_style.clone();
        let t_br_style = table_style.reset_surround.border_right_style.clone();

        let (col_elements, colgroup_elements) = gather_cols_and_colgroups(dom, node, num_cols);

        let is_row_group_element = |dom: &Dom, node: NodeId| -> bool {
            if let Some(NodeData::Element { name, .. }) = dom.data(node) {
                name == "tbody" || name == "thead" || name == "tfoot"
            } else {
                false
            }
        };

        let get_row_group = |r: usize| -> Option<NodeId> {
            if r < rows.len()
                && let Some(row_node) = rows[r].node
                && let Some(parent) = dom.parent(row_node)
                && is_row_group_element(dom, parent)
            {
                Some(parent)
            } else {
                None
            }
        };

        let row_is_empty_or_hidden = |r: usize| -> bool {
            if r >= rows.len() {
                return true;
            }
            if r < row_is_hidden.len() && row_is_hidden[r] {
                return true;
            }
            let mut has_cell = false;
            for col in 0..num_cols {
                if occupied.get(&(r, col)).copied().unwrap_or(false) {
                    has_cell = true;
                    break;
                }
            }
            !has_cell
        };

        let col_is_empty = |c: usize| -> bool {
            if c >= num_cols {
                return true;
            }
            let mut has_cell = false;
            for r in 0..rows.len() {
                if occupied.get(&(r, c)).copied().unwrap_or(false) {
                    has_cell = true;
                    break;
                }
            }
            !has_cell
        };

        // Resolve borders for each cell
        for (&cell_node, placement) in &cell_placements {
            let info = if let Some(i) = cell_borders.get(&cell_node) {
                i
            } else {
                continue;
            };

            let row_idx = placement.row_idx;
            let col_idx = placement.col_idx;
            let colspan = placement.colspan;
            let rowspan = placement.rowspan;

            // Top Border Resolution
            let mut top_conflicts = vec![BorderConflict {
                width: info.top.0,
                style: info.top.1.clone(),
                element_priority: 5,
            }];
            if let Some(row_node) = rows[row_idx].node {
                let (row_bt, row_bt_style) = get_element_borders(styles, row_node).0;
                top_conflicts.push(BorderConflict {
                    width: row_bt,
                    style: row_bt_style,
                    element_priority: 4,
                });
            }
            if let Some(rg_node) = get_row_group(row_idx) {
                let is_first_row_of_rg =
                    row_idx == 0 || get_row_group(row_idx - 1) != Some(rg_node);
                if is_first_row_of_rg {
                    let (rg_bt, rg_bt_style) = get_element_borders(styles, rg_node).0;
                    top_conflicts.push(BorderConflict {
                        width: rg_bt,
                        style: rg_bt_style,
                        element_priority: 3,
                    });
                }
            }
            let is_outer_top = (0..row_idx).all(&row_is_empty_or_hidden);
            if is_outer_top {
                top_conflicts.push(BorderConflict {
                    width: t_bt,
                    style: t_bt_style.clone(),
                    element_priority: 0,
                });
            } else {
                let mut prev_visible_row = None;
                for r in (0..row_idx).rev() {
                    if !row_is_empty_or_hidden(r) {
                        prev_visible_row = Some(r);
                        break;
                    }
                }
                if let Some(pr) = prev_visible_row {
                    for (&other_node, other_p) in &cell_placements {
                        if other_p.row_idx + other_p.rowspan == pr + 1 {
                            let overlap = other_p.col_idx < col_idx + colspan
                                && col_idx < other_p.col_idx + other_p.colspan;
                            if overlap && let Some(other_info) = cell_borders.get(&other_node) {
                                top_conflicts.push(BorderConflict {
                                    width: other_info.bottom.0,
                                    style: other_info.bottom.1.clone(),
                                    element_priority: 5,
                                });
                            }
                        }
                    }
                    if let Some(prev_row_node) = rows[pr].node {
                        let (row_bb, row_bb_style) = get_element_borders(styles, prev_row_node).1;
                        top_conflicts.push(BorderConflict {
                            width: row_bb,
                            style: row_bb_style,
                            element_priority: 4,
                        });
                    }
                    if let Some(prev_rg_node) = get_row_group(pr) {
                        let is_last_row_of_prev_rg = get_row_group(row_idx) != Some(prev_rg_node);
                        if is_last_row_of_prev_rg {
                            let (rg_bb, rg_bb_style) = get_element_borders(styles, prev_rg_node).1;
                            top_conflicts.push(BorderConflict {
                                width: rg_bb,
                                style: rg_bb_style,
                                element_priority: 3,
                            });
                        }
                    }
                }
            }
            let (resolved_top_w, resolved_top_s) = resolve_collapsed_borders_new(&top_conflicts);

            // Bottom Border Resolution
            let mut bottom_conflicts = vec![BorderConflict {
                width: info.bottom.0,
                style: info.bottom.1.clone(),
                element_priority: 5,
            }];
            if let Some(row_node) = rows[row_idx + rowspan - 1].node {
                let (row_bb, row_bb_style) = get_element_borders(styles, row_node).1;
                bottom_conflicts.push(BorderConflict {
                    width: row_bb,
                    style: row_bb_style,
                    element_priority: 4,
                });
            }
            if let Some(rg_node) = get_row_group(row_idx + rowspan - 1) {
                let is_last_row_of_rg = (row_idx + rowspan) == rows.len()
                    || get_row_group(row_idx + rowspan) != Some(rg_node);
                if is_last_row_of_rg {
                    let (rg_bb, rg_bb_style) = get_element_borders(styles, rg_node).1;
                    bottom_conflicts.push(BorderConflict {
                        width: rg_bb,
                        style: rg_bb_style,
                        element_priority: 3,
                    });
                }
            }
            let is_outer_bottom = ((row_idx + rowspan)..rows.len()).all(&row_is_empty_or_hidden);
            if is_outer_bottom {
                bottom_conflicts.push(BorderConflict {
                    width: t_bb,
                    style: t_bb_style.clone(),
                    element_priority: 0,
                });
            } else {
                let mut next_visible_row = None;
                for r in (row_idx + rowspan)..rows.len() {
                    if !row_is_empty_or_hidden(r) {
                        next_visible_row = Some(r);
                        break;
                    }
                }
                if let Some(nr) = next_visible_row {
                    for (&other_node, other_p) in &cell_placements {
                        if other_p.row_idx == nr {
                            let overlap = other_p.col_idx < col_idx + colspan
                                && col_idx < other_p.col_idx + other_p.colspan;
                            if overlap && let Some(other_info) = cell_borders.get(&other_node) {
                                bottom_conflicts.push(BorderConflict {
                                    width: other_info.top.0,
                                    style: other_info.top.1.clone(),
                                    element_priority: 5,
                                });
                            }
                        }
                    }
                    if let Some(next_row_node) = rows[nr].node {
                        let (row_bt, row_bt_style) = get_element_borders(styles, next_row_node).0;
                        bottom_conflicts.push(BorderConflict {
                            width: row_bt,
                            style: row_bt_style,
                            element_priority: 4,
                        });
                    }
                    if let Some(next_rg_node) = get_row_group(nr) {
                        let is_first_row_of_next_rg =
                            get_row_group(row_idx + rowspan - 1) != Some(next_rg_node);
                        if is_first_row_of_next_rg {
                            let (rg_bt, rg_bt_style) = get_element_borders(styles, next_rg_node).0;
                            bottom_conflicts.push(BorderConflict {
                                width: rg_bt,
                                style: rg_bt_style,
                                element_priority: 3,
                            });
                        }
                    }
                }
            }
            let (resolved_bottom_w, resolved_bottom_s) =
                resolve_collapsed_borders_new(&bottom_conflicts);

            // Left Border Resolution
            let mut left_conflicts = vec![BorderConflict {
                width: info.left.0,
                style: info.left.1.clone(),
                element_priority: 5,
            }];
            if col_idx < num_cols
                && let Some(col_node) = col_elements[col_idx]
            {
                let (col_bl, col_bl_style) = get_element_borders(styles, col_node).2;
                left_conflicts.push(BorderConflict {
                    width: col_bl,
                    style: col_bl_style,
                    element_priority: 2,
                });
            }
            if col_idx < num_cols
                && let Some(cg_node) = colgroup_elements[col_idx]
            {
                let is_first_col_of_cg =
                    col_idx == 0 || colgroup_elements[col_idx - 1] != Some(cg_node);
                if is_first_col_of_cg {
                    let (cg_bl, cg_bl_style) = get_element_borders(styles, cg_node).2;
                    left_conflicts.push(BorderConflict {
                        width: cg_bl,
                        style: cg_bl_style,
                        element_priority: 1,
                    });
                }
            }
            let is_outer_left = (0..col_idx).all(&col_is_empty);
            if is_outer_left {
                left_conflicts.push(BorderConflict {
                    width: t_bl,
                    style: t_bl_style.clone(),
                    element_priority: 0,
                });
            } else {
                let mut prev_visible_col = None;
                for c in (0..col_idx).rev() {
                    if !col_is_empty(c) {
                        prev_visible_col = Some(c);
                        break;
                    }
                }
                if let Some(pc) = prev_visible_col {
                    for (&other_node, other_p) in &cell_placements {
                        if other_p.col_idx + other_p.colspan == pc + 1 {
                            let overlap = other_p.row_idx < row_idx + rowspan
                                && row_idx < other_p.row_idx + other_p.rowspan;
                            if overlap && let Some(other_info) = cell_borders.get(&other_node) {
                                left_conflicts.push(BorderConflict {
                                    width: other_info.right.0,
                                    style: other_info.right.1.clone(),
                                    element_priority: 5,
                                });
                            }
                        }
                    }
                    if let Some(prev_col_node) = col_elements[pc] {
                        let (col_br, col_br_style) = get_element_borders(styles, prev_col_node).3;
                        left_conflicts.push(BorderConflict {
                            width: col_br,
                            style: col_br_style,
                            element_priority: 2,
                        });
                    }
                    if let Some(prev_cg_node) = colgroup_elements[pc] {
                        let is_last_col_of_prev_cg =
                            col_idx == num_cols || colgroup_elements[col_idx] != Some(prev_cg_node);
                        if is_last_col_of_prev_cg {
                            let (cg_br, cg_br_style) = get_element_borders(styles, prev_cg_node).3;
                            left_conflicts.push(BorderConflict {
                                width: cg_br,
                                style: cg_br_style,
                                element_priority: 1,
                            });
                        }
                    }
                }
            }
            let (resolved_left_w, resolved_left_s) = resolve_collapsed_borders_new(&left_conflicts);

            // Right Border Resolution
            let mut right_conflicts = vec![BorderConflict {
                width: info.right.0,
                style: info.right.1.clone(),
                element_priority: 5,
            }];
            if col_idx + colspan - 1 < num_cols
                && let Some(col_node) = col_elements[col_idx + colspan - 1]
            {
                let (col_br, col_br_style) = get_element_borders(styles, col_node).3;
                right_conflicts.push(BorderConflict {
                    width: col_br,
                    style: col_br_style,
                    element_priority: 2,
                });
            }
            if col_idx + colspan - 1 < num_cols
                && let Some(cg_node) = colgroup_elements[col_idx + colspan - 1]
            {
                let is_last_col_of_cg = (col_idx + colspan) == num_cols
                    || colgroup_elements[col_idx + colspan] != Some(cg_node);
                if is_last_col_of_cg {
                    let (cg_br, cg_br_style) = get_element_borders(styles, cg_node).3;
                    right_conflicts.push(BorderConflict {
                        width: cg_br,
                        style: cg_br_style,
                        element_priority: 1,
                    });
                }
            }
            let is_outer_right = ((col_idx + colspan)..num_cols).all(&col_is_empty);
            if is_outer_right {
                right_conflicts.push(BorderConflict {
                    width: t_br,
                    style: t_br_style.clone(),
                    element_priority: 0,
                });
            } else {
                let mut next_visible_col = None;
                for c in (col_idx + colspan)..num_cols {
                    if !col_is_empty(c) {
                        next_visible_col = Some(c);
                        break;
                    }
                }
                if let Some(nc) = next_visible_col {
                    for (&other_node, other_p) in &cell_placements {
                        if other_p.col_idx == nc {
                            let overlap = other_p.row_idx < row_idx + rowspan
                                && row_idx < other_p.row_idx + other_p.rowspan;
                            if overlap && let Some(other_info) = cell_borders.get(&other_node) {
                                right_conflicts.push(BorderConflict {
                                    width: other_info.left.0,
                                    style: other_info.left.1.clone(),
                                    element_priority: 5,
                                });
                            }
                        }
                    }
                    if let Some(next_col_node) = col_elements[nc] {
                        let (col_bl, col_bl_style) = get_element_borders(styles, next_col_node).2;
                        right_conflicts.push(BorderConflict {
                            width: col_bl,
                            style: col_bl_style,
                            element_priority: 2,
                        });
                    }
                    if let Some(next_cg_node) = colgroup_elements[nc] {
                        let is_first_col_of_next_cg =
                            colgroup_elements[col_idx + colspan - 1] != Some(next_cg_node);
                        if is_first_col_of_next_cg {
                            let (cg_bl, cg_bl_style) = get_element_borders(styles, next_cg_node).2;
                            right_conflicts.push(BorderConflict {
                                width: cg_bl,
                                style: cg_bl_style,
                                element_priority: 1,
                            });
                        }
                    }
                }
            }
            let (resolved_right_w, resolved_right_s) =
                resolve_collapsed_borders_new(&right_conflicts);

            // Mutate in local_styles
            if let Some(cell_style) = local_styles.get_mut(&cell_node) {
                let surround = std::sync::Arc::make_mut(&mut cell_style.reset_surround);
                surround.border_top_width = resolved_top_w.round() as i32;
                surround.border_bottom_width = resolved_bottom_w.round() as i32;
                surround.border_left_width = resolved_left_w.round() as i32;
                surround.border_right_width = resolved_right_w.round() as i32;

                surround.border_top_style = resolved_top_s;
                surround.border_bottom_style = resolved_bottom_s;
                surround.border_left_style = resolved_left_s;
                surround.border_right_style = resolved_right_s;
            }
        }

        // Update table's outer borders by taking max of resolved outer borders of outer cells
        for (&cell_node, placement) in &cell_placements {
            if let Some(cell_style) = local_styles.get(&cell_node) {
                let is_outer_top = (0..placement.row_idx).all(&row_is_empty_or_hidden);
                if is_outer_top {
                    border_top = border_top.max(cell_style.reset_surround.border_top_width as f32);
                }
                let is_outer_bottom = ((placement.row_idx + placement.rowspan)..rows.len())
                    .all(&row_is_empty_or_hidden);
                if is_outer_bottom {
                    border_bottom =
                        border_bottom.max(cell_style.reset_surround.border_bottom_width as f32);
                }
                let is_outer_left = (0..placement.col_idx).all(&col_is_empty);
                if is_outer_left {
                    border_left =
                        border_left.max(cell_style.reset_surround.border_left_width as f32);
                }
                let is_outer_right =
                    ((placement.col_idx + placement.colspan)..num_cols).all(&col_is_empty);
                if is_outer_right {
                    border_right =
                        border_right.max(cell_style.reset_surround.border_right_width as f32);
                }
            }
        }

        &local_styles
    } else {
        styles
    };

    let styles = styles_ref;

    let table_layout_fixed = style.reset_table.table_layout == "fixed";

    let final_content_width = if table_layout_fixed {
        let mut col_has_width = vec![false; num_cols];
        // 1. Gather column element widths
        for c in 0..num_cols {
            if let Some(Some(col_w)) = col_element_widths.get(c) {
                col_widths[c] = *col_w;
                col_has_width[c] = true;
            }
        }
        // 2. Gather first row cell widths (only if column doesn't already have width from col elements)
        let mut sorted_first_row_placements: Vec<&CellPlacement> = cell_placements
            .values()
            .filter(|p| p.row_idx == 0)
            .collect();
        sorted_first_row_placements.sort_by_key(|p| p.col_idx);
        for placement in sorted_first_row_placements {
            if let Some(cell_style) = styles.get(&placement.node) {
                let w_px = get_resolved_width(cell_style, avail_content_width).unwrap_or(0.0);
                if w_px > 0.0 {
                    let mut already_resolved_width = 0.0_f32;
                    let mut unresolved_cols = Vec::new();
                    for c in placement.col_idx..(placement.col_idx + placement.colspan) {
                        if c < num_cols {
                            if col_has_width[c] {
                                already_resolved_width += col_widths[c];
                            } else {
                                unresolved_cols.push(c);
                            }
                        }
                    }
                    if !unresolved_cols.is_empty() {
                        let leftover = (w_px - already_resolved_width).max(0.0);
                        let share = leftover / unresolved_cols.len() as f32;
                        for &c in &unresolved_cols {
                            col_widths[c] = share;
                            col_has_width[c] = true;
                        }
                    }
                }
            }
        }

        let has_definite_width = style.reset_box.width != -1;
        if has_definite_width {
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
            content_width
        } else {
            // For width: auto table in fixed layout, any auto-columns (col_has_width == false)
            // use the preferred width of their first-row cell, or default to 0.0.
            for placement in cell_placements.values().filter(|p| p.row_idx == 0) {
                for c in placement.col_idx..(placement.col_idx + placement.colspan) {
                    if c < num_cols && !col_has_width[c] {
                        let cell_pref_w = get_cell_preferred_width(
                            dom,
                            styles,
                            placement.node,
                            avail_content_width,
                            depth + 1,
                        );
                        let share = cell_pref_w / placement.colspan as f32;
                        col_widths[c] = col_widths[c].max(share);
                    }
                }
            }
            let sum_col_widths: f32 = col_widths.iter().sum();
            (sum_col_widths + total_spacing_h).min(content_width)
        }
    } else {
        // Initialize col_widths with column element widths as baseline
        for (c, item) in col_element_widths.iter().enumerate().take(num_cols) {
            if let Some(col_w) = item
                && *col_w > col_widths[c]
            {
                col_widths[c] = *col_w;
            }
        }

        // First, handle all cells with colspan == 1 to establish baseline column widths
        let mut sorted_colspan_1_placements: Vec<&CellPlacement> = cell_placements
            .values()
            .filter(|p| p.colspan == 1)
            .collect();
        sorted_colspan_1_placements.sort_by_key(|p| (p.row_idx, p.col_idx));
        for placement in sorted_colspan_1_placements {
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

        // Next, handle all cells with colspan > 1 (sorted by colspan ascending, then row_idx, then col_idx)
        let mut colspanning_cells: Vec<&CellPlacement> =
            cell_placements.values().filter(|p| p.colspan > 1).collect();
        colspanning_cells.sort_by(|a, b| {
            a.colspan
                .cmp(&b.colspan)
                .then_with(|| a.row_idx.cmp(&b.row_idx))
                .then_with(|| a.col_idx.cmp(&b.col_idx))
        });

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
                let spanned_range = placement.col_idx..(placement.col_idx + placement.colspan);
                let auto_cols_count = col_is_explicit[spanned_range.clone()]
                    .iter()
                    .filter(|&&exp| !exp)
                    .count();

                if auto_cols_count > 0 {
                    let share = deficit / auto_cols_count as f32;
                    for c in spanned_range {
                        if !col_is_explicit[c] {
                            col_widths[c] += share;
                        }
                    }
                } else {
                    let current_individual_sum: f32 =
                        col_widths[spanned_range.clone()].iter().sum();
                    if current_individual_sum > 0.0 {
                        for c in spanned_range {
                            col_widths[c] += (col_widths[c] / current_individual_sum) * deficit;
                        }
                    } else {
                        let share = deficit / placement.colspan as f32;
                        for c in spanned_range {
                            col_widths[c] += share;
                        }
                    }
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
                // Distribute remaining space first to flexible (non-explicit) columns to preserve explicit widths
                let remaining = avail_col_width - sum_col_widths;
                let auto_cols_count = col_is_explicit.iter().filter(|&&exp| !exp).count();
                if auto_cols_count > 0 {
                    let share = remaining / auto_cols_count as f32;
                    for c in 0..num_cols {
                        if !col_is_explicit[c] {
                            col_widths[c] += share;
                        }
                    }
                } else {
                    // All columns are explicit, distribute remaining space equally
                    let share = remaining / num_cols as f32;
                    for w in &mut col_widths {
                        *w += share;
                    }
                }
            } else if avail_col_width < sum_col_widths {
                // Scale down columns proportionally
                let scale = avail_col_width / sum_col_widths;
                for w in &mut col_widths {
                    *w *= scale;
                }
            }
        } else if num_cols > 0 && avail_col_width > 0.0 {
            let share = avail_col_width / num_cols as f32;
            col_widths.fill(share);
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

    let mut row_is_explicit = vec![false; rows.len()];
    for (r, row_info) in rows.iter().enumerate() {
        if let Some(row_node) = row_info.node
            && let Some(row_style) = styles.get(&row_node)
        {
            let row_h = get_px(row_style, "height", 0.0);
            if row_h > 0.0 {
                row_heights[r] = row_h;
                row_is_explicit[r] = true;
            }
        }
    }

    let mut sorted_rowspan_1_placements: Vec<&CellPlacement> = cell_placements
        .values()
        .filter(|p| p.rowspan == 1)
        .collect();
    sorted_rowspan_1_placements.sort_by_key(|p| (p.row_idx, p.col_idx));

    for placement in &sorted_rowspan_1_placements {
        if let Some(cell_style) = styles.get(&placement.node)
            && cell_style.reset_box.height != -1
        {
            row_is_explicit[placement.row_idx] = true;
        }
    }

    // First, establish baseline row heights using rowspan == 1 cells
    for placement in &sorted_rowspan_1_placements {
        if let Some(cell_box) = cell_boxes.get(&placement.node) {
            let h = cell_box.rect.size.height;
            if h > row_heights[placement.row_idx] {
                row_heights[placement.row_idx] = h;
            }
        }
    }

    // Next, resolve cells with rowspan > 1 (sorted by rowspan ascending, then row_idx, then col_idx)
    let mut rowspanning_cells: Vec<&CellPlacement> =
        cell_placements.values().filter(|p| p.rowspan > 1).collect();
    rowspanning_cells.sort_by(|a, b| {
        a.rowspan
            .cmp(&b.rowspan)
            .then_with(|| a.row_idx.cmp(&b.row_idx))
            .then_with(|| a.col_idx.cmp(&b.col_idx))
    });

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
                let spanned_range = placement.row_idx..(placement.row_idx + placement.rowspan);
                let auto_rows_count = row_is_explicit[spanned_range.clone()]
                    .iter()
                    .filter(|&&exp| !exp)
                    .count();

                if auto_rows_count > 0 {
                    let share = deficit / auto_rows_count as f32;
                    for r in spanned_range {
                        if !row_is_explicit[r] {
                            row_heights[r] += share;
                        }
                    }
                } else {
                    let current_individual_sum: f32 =
                        row_heights[spanned_range.clone()].iter().sum();
                    if current_individual_sum > 0.0 {
                        for r in spanned_range {
                            row_heights[r] += (row_heights[r] / current_individual_sum) * deficit;
                        }
                    } else {
                        let share = deficit / placement.rowspan as f32;
                        for r in spanned_range {
                            row_heights[r] += share;
                        }
                    }
                }
            }
        }
    }

    // Force hidden rows to 0.0 height
    for r in 0..rows.len() {
        if row_is_hidden[r] {
            row_heights[r] = 0.0;
        }
    }

    // Distribute remaining height to rows if table has a definite height
    let has_definite_height = style.reset_box.height != -1;
    if has_definite_height {
        let table_height = get_px(style, "height", 0.0);
        let visible_rows: Vec<usize> = (0..rows.len()).filter(|&ri| !row_is_hidden[ri]).collect();
        let total_spacing_v = (visible_rows.len() + 1) as f32 * spacing_v;
        let sum_row_heights: f32 = row_heights.iter().sum();
        let current_table_content_height = sum_row_heights
            + total_spacing_v
            + padding_top
            + padding_bottom
            + border_top
            + border_bottom;

        if table_height > current_table_content_height {
            let extra_height = table_height - current_table_content_height;
            let auto_rows_count = row_is_explicit
                .iter()
                .enumerate()
                .filter(|&(r, &exp)| !exp && !row_is_hidden[r])
                .count();

            if auto_rows_count > 0 {
                let share = extra_height / auto_rows_count as f32;
                for r in 0..rows.len() {
                    if !row_is_hidden[r] && !row_is_explicit[r] {
                        row_heights[r] += share;
                    }
                }
            } else {
                let visible_count = visible_rows.len();
                if visible_count > 0 {
                    let share = extra_height / visible_count as f32;
                    for r in 0..rows.len() {
                        if !row_is_hidden[r] {
                            row_heights[r] += share;
                        }
                    }
                }
            }
        }
    }

    let table_width =
        final_content_width + padding_left + padding_right + border_left + border_right;

    let mut top_caption_boxes = Vec::new();
    let mut bottom_caption_boxes = Vec::new();

    for &caption_node in &captions {
        let is_bottom = if let Some(style) = styles.get(&caption_node) {
            style.inherited_table.caption_side == "bottom" || caption_side_bottom
        } else {
            caption_side_bottom
        };

        if let Some(cap_box) = layout_node(
            dom,
            styles,
            caption_node,
            table_width,
            border_box_x,
            border_box_y,
            depth + 1,
        ) {
            if is_bottom {
                bottom_caption_boxes.push(cap_box);
            } else {
                top_caption_boxes.push(cap_box);
            }
        }
    }

    let mut table_children = Vec::new();

    // 1. Position top captions (stacking top to bottom)
    let mut curr_y = border_box_y;
    for cap_box in &mut top_caption_boxes {
        let dy = curr_y - cap_box.rect.origin.y;
        translate_y(cap_box, dy);
        curr_y += cap_box.rect.size.height;
    }

    let grid_box_y = curr_y;

    // 2. Table row content begins here
    let mut row_y_offsets = vec![0.0_f32; rows.len()];
    let row_start_y = curr_y + border_top + padding_top;
    curr_y = row_start_y;

    let visible_rows: Vec<usize> = (0..rows.len()).filter(|&ri| !row_is_hidden[ri]).collect();
    if !visible_rows.is_empty() {
        for &vr in &visible_rows {
            curr_y += spacing_v;
            row_y_offsets[vr] = curr_y;
            curr_y += row_heights[vr];
        }
        curr_y += spacing_v;
    }
    for r in 0..rows.len() {
        if row_is_hidden[r] {
            row_y_offsets[r] = row_start_y;
        }
    }

    // 3. Table border box bottom edge is here
    let table_bottom_y = curr_y + padding_bottom + border_bottom;

    // 4. Position bottom captions (stacking top to bottom), but don't push them to table_children yet
    let mut positioned_bottom_captions = Vec::new();
    let mut bottom_y = table_bottom_y;
    for mut cap_box in bottom_caption_boxes {
        let dy = bottom_y - cap_box.rect.origin.y;
        translate_y(&mut cap_box, dy);
        bottom_y += cap_box.rect.size.height;
        positioned_bottom_captions.push(cap_box);
    }
    curr_y = bottom_y;

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

                let spanned_rows: Vec<usize> = (r..(r + placement.rowspan))
                    .filter(|&ri| !row_is_hidden[ri])
                    .collect();

                let (cell_y, cell_height) = if spanned_rows.is_empty() {
                    (row_start_y, 0.0)
                } else {
                    let first_vr = spanned_rows[0];
                    let last_vr = spanned_rows[spanned_rows.len() - 1];
                    let top = row_y_offsets[first_vr];
                    let bottom = row_y_offsets[last_vr] + row_heights[last_vr];
                    (top, bottom - top)
                };

                let cell_width: f32 = col_widths[col_idx..(col_idx + placement.colspan)]
                    .iter()
                    .sum::<f32>()
                    + (placement.colspan - 1) as f32 * spacing_h;

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

    if captions.is_empty() {
        Some(LayoutBox {
            node: Some(node),
            rect: Rect::new(
                border_box_x,
                border_box_y,
                table_width,
                curr_y - border_box_y,
            ),
            children: table_children,
            text: None,
        })
    } else {
        let table_grid_box = LayoutBox {
            node: Some(node),
            rect: Rect::new(
                border_box_x,
                grid_box_y,
                table_width,
                table_bottom_y - grid_box_y,
            ),
            children: table_children,
            text: None,
        };

        let mut wrapper_children = Vec::new();
        wrapper_children.extend(top_caption_boxes);
        wrapper_children.push(table_grid_box);
        wrapper_children.extend(positioned_bottom_captions);

        Some(LayoutBox {
            node: None, // Table Wrapper Box is anonymous
            rect: Rect::new(
                border_box_x,
                border_box_y,
                table_width,
                curr_y - border_box_y,
            ),
            children: wrapper_children,
            text: None,
        })
    }
}

fn has_explicit_width(style: &CategorizedComputedStyle) -> bool {
    if style.reset_box.width != -1 {
        return true;
    }
    if let Some(ref extra) = style.extra_values
        && extra.contains_key("width")
    {
        return true;
    }
    false
}

pub(crate) fn is_border_collapse(style: &CategorizedComputedStyle) -> bool {
    style.inherited_table.border_collapse == "collapse"
}

fn get_border_spacing(style: &CategorizedComputedStyle) -> (f32, f32) {
    if let Some(ref extra) = style.extra_values
        && let Some(val) = extra.get("border-spacing")
    {
        match val {
            CssValue::Length(v, _) | CssValue::Number(v) => {
                return (*v, *v);
            }
            CssValue::Multiple(vec) => {
                let resolved: Vec<f32> = vec
                    .iter()
                    .filter_map(|item| match item {
                        CssValue::Length(v, _) | CssValue::Number(v) => Some(*v),
                        _ => None,
                    })
                    .collect();
                if resolved.len() == 2 {
                    return (resolved[0], resolved[1]);
                } else if resolved.len() == 1 {
                    return (resolved[0], resolved[0]);
                }
            }
            _ => {}
        }
    }
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

fn parse_html_non_negative_integer(s: &str) -> Option<i32> {
    let mut chars = s.chars().peekable();
    // Skip leading ASCII whitespace (space, tab, LF, FF, CR)
    while let Some(&c) = chars.peek() {
        if c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\x0C' {
            chars.next();
        } else {
            break;
        }
    }
    // Optional leading plus sign
    if let Some(&c) = chars.peek()
        && c == '+'
    {
        chars.next();
    }
    let mut digits = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            digits.push(c);
            chars.next();
        } else {
            break;
        }
    }
    if digits.is_empty() {
        None
    } else {
        digits.parse::<i32>().ok()
    }
}

fn parse_span_attribute(dom: &Dom, node: NodeId, name: &str) -> usize {
    if let Some(s) = dom.get_attribute(node, name) {
        if let Some(val) = parse_html_non_negative_integer(s) {
            if val < 0 {
                1
            } else if val == 0 {
                if name == "rowspan" { 0 } else { 1 }
            } else {
                val as usize
            }
        } else {
            1
        }
    } else {
        1
    }
}

fn get_resolved_width(style: &CategorizedComputedStyle, avail_content_width: f32) -> Option<f32> {
    if let Some(ref extra) = style.extra_values {
        if let Some(CssValue::Length(v, LengthUnit::Percent)) = extra.get("width") {
            return Some((*v / 100.0) * avail_content_width);
        }
        if let Some(CssValue::Length(v, LengthUnit::Percent)) = extra.get("min-width") {
            return Some((*v / 100.0) * avail_content_width);
        }
    }
    if style.reset_box.min_width >= crate::style::categorized::WIDTH_PERCENT_BAND {
        let pct =
            (style.reset_box.min_width - crate::style::categorized::WIDTH_PERCENT_BAND) as f32;
        return Some((pct / 100.0) * avail_content_width);
    }
    if style.reset_box.width != -1 {
        let w = get_px(style, "width", 0.0);
        if w > 0.0 {
            return Some(w);
        }
    }
    if style.reset_box.min_width != -1 {
        let w = get_px(style, "min-width", 0.0);
        if w > 0.0 {
            return Some(w);
        }
    }
    None
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
        && let Some(w) = get_resolved_width(cs, content_width)
    {
        width = w;
    }
    if width == 0.0
        && let Some(cell_box) = layout_node(dom, styles, cell_node, content_width, 0.0, 0.0, depth)
    {
        width = cell_box.rect.size.width;
    }
    width
}

fn gather_col_widths(
    dom: &Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    table_node: NodeId,
    avail_content_width: f32,
) -> Vec<Option<f32>> {
    let mut col_widths = Vec::new();
    for &child in dom.children(table_node) {
        if let Some(NodeData::Element { name, .. }) = dom.data(child) {
            if name == "colgroup" {
                // If it's a colgroup, check if it has nested col elements
                let colgroup_children = dom.children(child);
                let has_cols = colgroup_children.iter().any(|&c| {
                    if let Some(NodeData::Element {
                        name: child_name, ..
                    }) = dom.data(c)
                    {
                        child_name == "col"
                    } else {
                        false
                    }
                });

                if has_cols {
                    for &colgroup_child in colgroup_children {
                        if let Some(NodeData::Element {
                            name: child_name, ..
                        }) = dom.data(colgroup_child)
                            && child_name == "col"
                        {
                            let span =
                                parse_span_attribute(dom, colgroup_child, "span").clamp(1, 1000);
                            let width_opt = if let Some(style) = styles.get(&colgroup_child) {
                                get_resolved_width(style, avail_content_width)
                            } else {
                                None
                            };
                            for _ in 0..span {
                                col_widths.push(width_opt);
                            }
                        }
                    }
                } else {
                    // colgroup without cols: check its span and its own style/width
                    let span = parse_span_attribute(dom, child, "span").clamp(1, 1000);
                    let width_opt = if let Some(style) = styles.get(&child) {
                        get_resolved_width(style, avail_content_width)
                    } else {
                        None
                    };
                    for _ in 0..span {
                        col_widths.push(width_opt);
                    }
                }
            } else if name == "col" {
                let span = parse_span_attribute(dom, child, "span").clamp(1, 1000);
                let width_opt = if let Some(style) = styles.get(&child) {
                    get_resolved_width(style, avail_content_width)
                } else {
                    None
                };
                for _ in 0..span {
                    col_widths.push(width_opt);
                }
            }
        }
    }
    col_widths
}

fn cell_has_rendered_content(dom: &Dom, node_id: NodeId) -> bool {
    for &child in dom.children(node_id) {
        if let Some(data) = dom.data(child) {
            match data {
                NodeData::Element { .. } => {
                    return true;
                }
                NodeData::Text(s) if !s.is_empty() => {
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

fn gather_cols_and_colgroups(
    dom: &Dom,
    table_node: NodeId,
    num_cols: usize,
) -> (Vec<Option<NodeId>>, Vec<Option<NodeId>>) {
    let mut col_elements = vec![None; num_cols];
    let mut colgroup_elements = vec![None; num_cols];

    let mut col_idx = 0;
    for &child in dom.children(table_node) {
        if col_idx >= num_cols {
            break;
        }
        if let Some(NodeData::Element { name, .. }) = dom.data(child) {
            if name == "colgroup" {
                let colgroup_children = dom.children(child);
                let has_cols = colgroup_children.iter().any(|&c| {
                    if let Some(NodeData::Element {
                        name: child_name, ..
                    }) = dom.data(c)
                    {
                        child_name == "col"
                    } else {
                        false
                    }
                });

                if has_cols {
                    for &colgroup_child in colgroup_children {
                        if col_idx >= num_cols {
                            break;
                        }
                        if let Some(NodeData::Element {
                            name: child_name, ..
                        }) = dom.data(colgroup_child)
                            && child_name == "col"
                        {
                            let span =
                                parse_span_attribute(dom, colgroup_child, "span").clamp(1, 1000);
                            for _ in 0..span {
                                if col_idx < num_cols {
                                    col_elements[col_idx] = Some(colgroup_child);
                                    colgroup_elements[col_idx] = Some(child);
                                    col_idx += 1;
                                }
                            }
                        }
                    }
                } else {
                    let span = parse_span_attribute(dom, child, "span").clamp(1, 1000);
                    for _ in 0..span {
                        if col_idx < num_cols {
                            colgroup_elements[col_idx] = Some(child);
                            col_idx += 1;
                        }
                    }
                }
            } else if name == "col" {
                let span = parse_span_attribute(dom, child, "span").clamp(1, 1000);
                for _ in 0..span {
                    if col_idx < num_cols {
                        col_elements[col_idx] = Some(child);
                        col_idx += 1;
                    }
                }
            }
        }
    }
    (col_elements, colgroup_elements)
}

fn style_priority(style: &str) -> i32 {
    match style {
        "hidden" => 9,
        "double" => 8,
        "solid" => 7,
        "dashed" => 6,
        "dotted" => 5,
        "ridge" => 4,
        "outset" => 3,
        "groove" => 2,
        "inset" => 1,
        _ => 0, // "none" or any other value
    }
}

#[derive(Clone)]
struct BorderConflict {
    width: f32,
    style: String,
    element_priority: i32,
}

fn resolve_collapsed_borders_new(conflicts: &[BorderConflict]) -> (f32, String) {
    if conflicts.is_empty() {
        return (0.0, "none".to_string());
    }

    // Coerce "none" style to "solid" if width is > 0.0, to match original codebase behavior
    let coerced_conflicts: Vec<BorderConflict> = conflicts
        .iter()
        .map(|c| {
            let mut style = c.style.clone();
            if c.width > 0.0 && style == "none" {
                style = "solid".to_string();
            }
            BorderConflict {
                width: c.width,
                style,
                element_priority: c.element_priority,
            }
        })
        .collect();

    // 1. Check for "hidden"
    for conflict in &coerced_conflicts {
        if conflict.style == "hidden" {
            return (0.0, "hidden".to_string());
        }
    }

    // 2. Filter out "none"
    let mut active: Vec<&BorderConflict> = coerced_conflicts
        .iter()
        .filter(|c| c.style != "none")
        .collect();

    if active.is_empty() {
        return (0.0, "none".to_string());
    }

    // 3. Find max width
    let mut max_w = -1.0_f32;
    for c in &active {
        if c.width > max_w {
            max_w = c.width;
        }
    }

    // Keep those with max width (with a small float tolerance)
    active.retain(|c| (c.width - max_w).abs() < 1e-4);

    if active.len() == 1 {
        return (active[0].width, active[0].style.clone());
    }

    // 4. Find max style priority
    let mut max_style_pri = -1;
    for c in &active {
        let pri = style_priority(&c.style);
        if pri > max_style_pri {
            max_style_pri = pri;
        }
    }

    active.retain(|c| style_priority(&c.style) == max_style_pri);

    if active.len() == 1 {
        return (active[0].width, active[0].style.clone());
    }

    // 5. Find max element priority
    let mut max_elem_pri = -1;
    for c in &active {
        if c.element_priority > max_elem_pri {
            max_elem_pri = c.element_priority;
        }
    }

    active.retain(|c| c.element_priority == max_elem_pri);

    (active[0].width, active[0].style.clone())
}

#[allow(clippy::type_complexity)]
fn get_element_borders(
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    node: NodeId,
) -> ((f32, String), (f32, String), (f32, String), (f32, String)) {
    if let Some(style) = styles.get(&node) {
        let bt = get_px(style, "border-top-width", 0.0);
        let bb = get_px(style, "border-bottom-width", 0.0);
        let bl = get_px(style, "border-left-width", 0.0);
        let br = get_px(style, "border-right-width", 0.0);

        let bt_style = style.reset_surround.border_top_style.clone();
        let bb_style = style.reset_surround.border_bottom_style.clone();
        let bl_style = style.reset_surround.border_left_style.clone();
        let br_style = style.reset_surround.border_right_style.clone();

        (
            (bt, bt_style),
            (bb, bb_style),
            (bl, bl_style),
            (br, br_style),
        )
    } else {
        (
            (0.0, "none".to_string()),
            (0.0, "none".to_string()),
            (0.0, "none".to_string()),
            (0.0, "none".to_string()),
        )
    }
}

fn is_cell_empty(dom: &Dom, cell_node: NodeId) -> bool {
    !cell_has_rendered_content(dom, cell_node)
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
    fn test_relaxed_spans_parsing() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Row 1: sets the column widths
        let row1_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row1_node);

        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell11_node);

        let cell12_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell12_node);

        let cell13_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell13_node);

        let cell14_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell14_node);

        // Row 2: contains relaxed spans
        let row2_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row2_node);

        let cell21_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![("colspan".to_string(), "2px".to_string())],
        });
        dom.append_child(row2_node, cell21_node);

        let cell22_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![("colspan".to_string(), "2abc".to_string())],
        });
        dom.append_child(row2_node, cell22_node);

        let mut styles = HashMap::new();
        styles.insert(table_node, style_with_display("table"));
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        // Setup row 1 cell widths (10, 10, 15, 15)
        let mut s11 = style_with_display("table-cell");
        s11.set_width(10);
        s11.set_height(10);
        styles.insert(cell11_node, s11);

        let mut s12 = style_with_display("table-cell");
        s12.set_width(10);
        s12.set_height(10);
        styles.insert(cell12_node, s12);

        let mut s13 = style_with_display("table-cell");
        s13.set_width(15);
        s13.set_height(10);
        styles.insert(cell13_node, s13);

        let mut s14 = style_with_display("table-cell");
        s14.set_width(15);
        s14.set_height(10);
        styles.insert(cell14_node, s14);

        // Row 2 cell styles with explicit widths (but larger colspan/spanning than 1)
        let mut s21 = style_with_display("table-cell");
        s21.set_width(20);
        s21.set_height(10);
        styles.insert(cell21_node, s21);

        let mut s22 = style_with_display("table-cell");
        s22.set_width(30);
        s22.set_height(10);
        styles.insert(cell22_node, s22);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // Table width should be 10 + 10 + 15 + 15 = 50.
        assert_eq!(table_box.rect.size.width, 50.0);
        assert_eq!(table_box.children.len(), 2);

        let r1 = &table_box.children[0];
        assert_eq!(r1.children.len(), 4);
        assert_eq!(r1.children[0].rect.size.width, 10.0);
        assert_eq!(r1.children[1].rect.size.width, 10.0);
        assert_eq!(r1.children[2].rect.size.width, 15.0);
        assert_eq!(r1.children[3].rect.size.width, 15.0);

        let r2 = &table_box.children[1];
        assert_eq!(r2.children.len(), 2);
        // Cell 2.1 spans columns 0 and 1 -> width is 10 + 10 = 20
        assert_eq!(r2.children[0].rect.origin.x, 0.0);
        assert_eq!(r2.children[0].rect.size.width, 20.0);
        // Cell 2.2 spans columns 2 and 3 -> width is 15 + 15 = 30
        assert_eq!(r2.children[1].rect.origin.x, 20.0);
        assert_eq!(r2.children[1].rect.size.width, 30.0);
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

        // 3. Children of table should be: caption and then the table grid box
        assert_eq!(table_box.children.len(), 2);

        let cap_box = &table_box.children[0];
        assert_eq!(cap_box.node, Some(caption_node));
        assert_eq!(cap_box.rect.origin.x, 10.0); // starts at table's border_box_x
        assert_eq!(cap_box.rect.origin.y, 20.0); // starts at table's border_box_y
        assert_eq!(cap_box.rect.size.width, 200.0); // spans full table width
        assert_eq!(cap_box.rect.size.height, 40.0);

        let grid_box = &table_box.children[1];
        assert_eq!(grid_box.node, Some(table_node));
        assert_eq!(grid_box.children.len(), 1);

        let row_box = &grid_box.children[0];
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

        // 3. Children of table should be: table grid box and then the caption
        assert_eq!(table_box.children.len(), 2);

        let grid_box = &table_box.children[0];
        assert_eq!(grid_box.node, Some(table_node));
        assert_eq!(grid_box.children.len(), 1);

        let row_box = &grid_box.children[0];
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
    fn test_table_multiple_captions() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Create table element
        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Create top caption 1
        let caption_top1 = dom.create_node(NodeData::Element {
            name: "caption".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, caption_top1);

        // Create top caption 2
        let caption_top2 = dom.create_node(NodeData::Element {
            name: "caption".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, caption_top2);

        // Create bottom caption 1
        let caption_bottom1 = dom.create_node(NodeData::Element {
            name: "caption".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, caption_bottom1);

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

        // Top caption 1 style: height 20px
        let mut cap_top1_style = style_with_display("block");
        cap_top1_style.insert("height".to_string(), CssValue::Length(20.0, LengthUnit::Px));
        styles.insert(caption_top1, cap_top1_style);

        // Top caption 2 style: height 15px
        let mut cap_top2_style = style_with_display("block");
        cap_top2_style.insert("height".to_string(), CssValue::Length(15.0, LengthUnit::Px));
        styles.insert(caption_top2, cap_top2_style);

        // Bottom caption 1 style: height 25px, caption-side: bottom
        let mut cap_bottom1_style = style_with_display("block");
        cap_bottom1_style.insert("height".to_string(), CssValue::Length(25.0, LengthUnit::Px));
        cap_bottom1_style.insert(
            "caption-side".to_string(),
            CssValue::Keyword("bottom".to_string()),
        );
        styles.insert(caption_bottom1, cap_bottom1_style);

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
        // 2. Table total height should include:
        //    top captions (20px + 15px) + row (30px) + bottom caption (25px) = 90px
        assert_eq!(table_box.rect.size.height, 90.0);

        // 3. Children of table should be: top captions, then the table grid box, then bottom caption
        assert_eq!(table_box.children.len(), 4);

        let cap1_box = &table_box.children[0];
        assert_eq!(cap1_box.node, Some(caption_top1));
        assert_eq!(cap1_box.rect.origin.y, 20.0);
        assert_eq!(cap1_box.rect.size.height, 20.0);

        let cap2_box = &table_box.children[1];
        assert_eq!(cap2_box.node, Some(caption_top2));
        assert_eq!(cap2_box.rect.origin.y, 20.0 + 20.0);
        assert_eq!(cap2_box.rect.size.height, 15.0);

        let grid_box = &table_box.children[2];
        assert_eq!(grid_box.node, Some(table_node));
        assert_eq!(grid_box.children.len(), 1);

        let row_box = &grid_box.children[0];
        assert_eq!(row_box.node, Some(row_node));
        assert_eq!(row_box.rect.origin.y, 40.0 + 15.0);
        assert_eq!(row_box.rect.size.height, 30.0);

        let cap3_box = &table_box.children[3];
        assert_eq!(cap3_box.node, Some(caption_bottom1));
        assert_eq!(cap3_box.rect.origin.y, 55.0 + 30.0);
        assert_eq!(cap3_box.rect.size.height, 25.0);
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

    #[test]
    fn test_colgroup_and_col_widths() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Create <colgroup> with a <col> child
        let colgroup_node = dom.create_node(NodeData::Element {
            name: "colgroup".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, colgroup_node);

        let col1_node = dom.create_node(NodeData::Element {
            name: "col".to_string(),
            attrs: vec![("span".to_string(), "1".to_string())],
        });
        dom.append_child(colgroup_node, col1_node);

        // Create direct <col> child
        let col2_node = dom.create_node(NodeData::Element {
            name: "col".to_string(),
            attrs: vec![("span".to_string(), "1".to_string())],
        });
        dom.append_child(table_node, col2_node);

        // Rows and cells
        let row_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row_node);

        let cell1_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell1_node);

        let cell2_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell2_node);

        let mut styles = HashMap::new();

        // Table style: width 300px, fixed table layout
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(300.0, LengthUnit::Px));
        table_style.insert(
            "table-layout".to_string(),
            CssValue::Keyword("fixed".to_string()),
        );
        styles.insert(table_node, table_style);

        styles.insert(row_node, style_with_display("table-row"));

        // Column 1 width: 120px from <col>
        let mut col1_style = style_with_display("table-column");
        col1_style.insert("width".to_string(), CssValue::Length(120.0, LengthUnit::Px));
        styles.insert(col1_node, col1_style);

        // Column 2 width: 80px from <col>
        let mut col2_style = style_with_display("table-column");
        col2_style.insert("width".to_string(), CssValue::Length(80.0, LengthUnit::Px));
        styles.insert(col2_node, col2_style);

        // Cells
        styles.insert(cell1_node, style_with_display("table-cell"));
        styles.insert(cell2_node, style_with_display("table-cell"));

        // Fixed layout:
        // Col 1 is 120.0, Col 2 is 80.0.
        // Leftover = 300 - 200 = 100.
        // Because both columns are explicit, they don't divide the remaining space,
        // and table width is 300.0 content_width.
        // Actually, the leftover distribution only distributes to non-explicit (auto) columns in fixed layout.
        // So they should be exactly 120.0 and 80.0.
        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        assert_eq!(table_box.children.len(), 1);
        let row_box = &table_box.children[0];
        assert_eq!(row_box.children.len(), 2);
        assert_eq!(row_box.children[0].rect.size.width, 120.0);
        assert_eq!(row_box.children[1].rect.size.width, 80.0);
    }

    #[test]
    fn test_tr_styled_height() {
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

        let cell_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell_node);

        let mut styles = HashMap::new();
        styles.insert(table_node, style_with_display("table"));

        // Row styled height: 85px
        let mut row_style = style_with_display("table-row");
        row_style.insert("height".to_string(), CssValue::Length(85.0, LengthUnit::Px));
        styles.insert(row_node, row_style);

        // Cell is height 30px
        let mut cell_style = style_with_display("table-cell");
        cell_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell_node, cell_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // The row should be 85px height!
        let r_box = &table_box.children[0];
        assert_eq!(r_box.rect.size.height, 85.0);
        assert_eq!(r_box.children[0].rect.size.height, 85.0);
    }

    #[test]
    fn test_colspan_and_rowspan_deficit_distribution_preferring_auto() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Row 1: spans 2 columns
        let row1_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row1_node);

        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![("colspan".to_string(), "2".to_string())],
        });
        dom.append_child(row1_node, cell11_node);

        // Row 2: establishing 2 columns
        let row2_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row2_node);

        let cell21_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell21_node);

        let cell22_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell22_node);

        let mut styles = HashMap::new();
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(120.0, LengthUnit::Px));
        styles.insert(table_node, table_style);
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        // Cell 1.1: colspan=2, preferred width 120px
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(120.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        // Cell 2.1: Column 1 explicit width 50px
        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        styles.insert(cell21_node, cell21_style);

        // Cell 2.2: Column 2 auto (has max-width: 10px, so it's auto but constrained)
        let mut cell22_style = style_with_display("table-cell");
        cell22_style.insert(
            "max-width".to_string(),
            CssValue::Length(10.0, LengthUnit::Px),
        );
        styles.insert(cell22_node, cell22_style);

        // Under auto table layout with table width 120px:
        // Col 1 is explicit (50px). Col 2 is auto with max-width (10px).
        // Spanned cell is preferred 120px. Deficit = 120 - 50 - 10 = 60px.
        // Since Col 1 is explicit and Col 2 is auto, Col 2 should receive the entire 60px deficit.
        // Final column widths: Col 1 = 50.0, Col 2 = 10.0 + 60.0 = 70.0.
        // Total column width = 120.0, matching table width exactly so no scaling occurs.
        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        let r2 = &table_box.children[1];
        assert_eq!(r2.children[0].rect.size.width, 50.0);
        assert_eq!(r2.children[1].rect.size.width, 70.0);
    }

    #[test]
    fn test_collapsed_padding_border_collapse() {
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

        let cell_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell_node);

        let mut styles = HashMap::new();

        // Table style: width 200px, padding 30px, border-collapse: collapse
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        table_style.insert(
            "padding-left".to_string(),
            CssValue::Length(30.0, LengthUnit::Px),
        );
        table_style.insert(
            "padding-right".to_string(),
            CssValue::Length(30.0, LengthUnit::Px),
        );
        table_style.insert(
            "padding-top".to_string(),
            CssValue::Length(30.0, LengthUnit::Px),
        );
        table_style.insert(
            "padding-bottom".to_string(),
            CssValue::Length(30.0, LengthUnit::Px),
        );
        table_style.insert(
            "border-collapse".to_string(),
            CssValue::Keyword("collapse".to_string()),
        );
        styles.insert(table_node, table_style);

        styles.insert(row_node, style_with_display("table-row"));

        let mut cell_style = style_with_display("table-cell");
        cell_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        styles.insert(cell_node, cell_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // Table padding should be completely ignored.
        // Table content width is exactly 200.0, so table box size.width should be exactly 200.0
        // (no padding added on sides!).
        assert_eq!(table_box.rect.size.width, 200.0);
    }

    #[test]
    fn test_rowspan_zero_with_row_groups() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Group 1 (tbody)
        let tbody_node = dom.create_node(NodeData::Element {
            name: "tbody".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, tbody_node);

        // Row 1
        let row1_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(tbody_node, row1_node);

        // Cell 1.1: rowspan="0"
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![("rowspan".to_string(), "0".to_string())],
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
        dom.append_child(tbody_node, row2_node);

        // Cell 2.1
        let cell21_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell21_node);

        // Group 2 (tfoot)
        let tfoot_node = dom.create_node(NodeData::Element {
            name: "tfoot".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, tfoot_node);

        // Row 3
        let row3_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(tfoot_node, row3_node);

        // Cell 3.1
        let cell31_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row3_node, cell31_node);

        let mut styles = HashMap::new();
        styles.insert(tbody_node, style_with_display("block"));
        styles.insert(tfoot_node, style_with_display("block"));
        styles.insert(table_node, style_with_display("table"));
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));
        styles.insert(row3_node, style_with_display("table-row"));

        // Set dimensions
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(20.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        let mut cell12_style = style_with_display("table-cell");
        cell12_style.insert("width".to_string(), CssValue::Length(80.0, LengthUnit::Px));
        cell12_style.insert("height".to_string(), CssValue::Length(15.0, LengthUnit::Px));
        styles.insert(cell12_node, cell12_style);

        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(80.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(25.0, LengthUnit::Px));
        styles.insert(cell21_node, cell21_style);

        let mut cell31_style = style_with_display("table-cell");
        cell31_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        cell31_style.insert("height".to_string(), CssValue::Length(10.0, LengthUnit::Px));
        styles.insert(cell31_node, cell31_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // Cell 1.1 has rowspan="0". It is in Row 1, which belongs to Group 1 (tbody).
        // Group 1 has Row 1 and Row 2. So Cell 1.1 spans 2 rows (Row 1 and Row 2), not Row 3.
        // Row 1 height is max(cell11_height=20, cell12_height=15) = 20px (initially, but actually rowspan cell is ignored for row baseline if rowspan > 1, so row 1 baseline is cell 1.2 height = 15px. Row 2 height is cell 2.1 height = 25px. The combined height of row 1 and row 2 is 15 + 25 = 40px. Since cell 1.1's preferred height is 20, and the combined height of the rows is 40, cell 1.1 spans both rows with height 40).
        // Let's verify cell positions and sizes.
        assert_eq!(table_box.children.len(), 3); // 3 rows

        let r1 = &table_box.children[0];
        let cell11_box = &r1.children[0];
        assert_eq!(cell11_box.node, Some(cell11_node));
        assert_eq!(cell11_box.rect.size.width, 50.0);
        assert_eq!(cell11_box.rect.size.height, 40.0); // Spans row 1 (15px) + row 2 (25px) = 40px!

        let cell12_box = &r1.children[1];
        assert_eq!(cell12_box.node, Some(cell12_node));
        assert_eq!(cell12_box.rect.size.height, 15.0);

        let r2 = &table_box.children[1];
        let cell21_box = &r2.children[0];
        assert_eq!(cell21_box.node, Some(cell21_node));
        assert_eq!(cell21_box.rect.origin.x, 50.0); // pushed right by the rowspan cell

        let r3 = &table_box.children[2];
        let cell31_box = &r3.children[0];
        assert_eq!(cell31_box.node, Some(cell31_node));
        assert_eq!(cell31_box.rect.origin.x, 0.0); // starts at column 0 because the rowspan cell does NOT leak into row 3!
    }

    #[test]
    fn test_rowspan_zero_without_row_groups() {
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

        // Cell 1.1: rowspan="0"
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![("rowspan".to_string(), "0".to_string())],
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

        // Set dimensions
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(20.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        let mut cell12_style = style_with_display("table-cell");
        cell12_style.insert("width".to_string(), CssValue::Length(80.0, LengthUnit::Px));
        cell12_style.insert("height".to_string(), CssValue::Length(15.0, LengthUnit::Px));
        styles.insert(cell12_node, cell12_style);

        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(80.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(25.0, LengthUnit::Px));
        styles.insert(cell21_node, cell21_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // Cell 1.1 has rowspan="0". It is directly in table (no row group).
        // It spans all remaining rows of the table (Row 1 and Row 2).
        assert_eq!(table_box.children.len(), 2);

        let r1 = &table_box.children[0];
        let cell11_box = &r1.children[0];
        assert_eq!(cell11_box.node, Some(cell11_node));
        assert_eq!(cell11_box.rect.size.height, 40.0); // Spans row 1 (15px) + row 2 (25px) = 40px

        let r2 = &table_box.children[1];
        let cell21_box = &r2.children[0];
        assert_eq!(cell21_box.node, Some(cell21_node));
        assert_eq!(cell21_box.rect.origin.x, 50.0); // pushed right
    }

    #[test]
    fn test_border_collapse_outer_borders_resolution() {
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

        let cell_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell_node);

        let mut styles = HashMap::new();

        // Table style: width 200px, border-left: 2px, border-right: 2px, border-top: 2px, border-bottom: 2px, border-collapse: collapse
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        table_style.insert(
            "border-left-width".to_string(),
            CssValue::Length(2.0, LengthUnit::Px),
        );
        table_style.insert(
            "border-right-width".to_string(),
            CssValue::Length(2.0, LengthUnit::Px),
        );
        table_style.insert(
            "border-top-width".to_string(),
            CssValue::Length(2.0, LengthUnit::Px),
        );
        table_style.insert(
            "border-bottom-width".to_string(),
            CssValue::Length(2.0, LengthUnit::Px),
        );
        table_style.insert(
            "border-collapse".to_string(),
            CssValue::Keyword("collapse".to_string()),
        );
        styles.insert(table_node, table_style);

        styles.insert(row_node, style_with_display("table-row"));

        // Cell style: border-left: 5px (should win over table's 2px!), width: 100px, height: 30px
        let mut cell_style = style_with_display("table-cell");
        cell_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        cell_style.insert(
            "border-left-width".to_string(),
            CssValue::Length(5.0, LengthUnit::Px),
        );
        cell_style.insert(
            "border-top-width".to_string(),
            CssValue::Length(4.0, LengthUnit::Px),
        ); // wins over table's 2px
        cell_style.insert(
            "border-bottom-width".to_string(),
            CssValue::Length(1.0, LengthUnit::Px),
        ); // table's 2px should win over cell's 1px
        styles.insert(cell_node, cell_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 10.0, 20.0, 0)
            .expect("should layout table");

        // The collapsed table borders should be:
        // border-left = max(table 2px, cell 5px) = 5px
        // border-top = max(table 2px, cell 4px) = 4px
        // border-bottom = max(table 2px, cell 1px) = 2px
        // border-right = max(table 2px, cell 0px) = 2px

        // Cell positioning should start at border_box_x (10.0) + border_left (5px) = 15.0
        let r = &table_box.children[0];
        let c = &r.children[0];
        assert_eq!(c.rect.origin.x, 15.0);
        // Cell y-offset should be row_start_y, which is border_box_y (20.0) + border_top (4px) = 24.0
        assert_eq!(c.rect.origin.y, 24.0);

        // Table width should be content width (200px) + border_left (5px) + border_right (2px) = 207.0
        assert_eq!(table_box.rect.size.width, 207.0);

        // Table height should be row_heights sum (36px, which includes cell borders 4px and resolved 2px) + border_top (4px) + border_bottom (2px) = 42.0
        assert_eq!(table_box.rect.size.height, 42.0);
    }

    #[test]
    fn test_percentage_column_widths_auto_layout() {
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

        let cell1_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell1_node);

        let cell2_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell2_node);

        let mut styles = HashMap::new();

        // Table style: width 400px
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(400.0, LengthUnit::Px));
        styles.insert(table_node, table_style);

        styles.insert(row_node, style_with_display("table-row"));

        // Cell 1 style: width: 25% (resolved against 400px = 100px)
        let mut cell1_style = style_with_display("table-cell");
        let mut extra = HashMap::new();
        extra.insert(
            "width".to_string(),
            CssValue::Length(25.0, LengthUnit::Percent),
        );
        cell1_style.extra_values = Some(std::sync::Arc::new(extra));
        styles.insert(cell1_node, cell1_style);

        // Cell 2 style: min-width: 75% (resolved against 400px = 300px)
        let mut cell2_style = style_with_display("table-cell");
        cell2_style.set_property("min-width", &CssValue::Length(75.0, LengthUnit::Percent));
        styles.insert(cell2_node, cell2_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // Table width should be 400.0
        assert_eq!(table_box.rect.size.width, 400.0);

        let r = &table_box.children[0];
        assert_eq!(r.children.len(), 2);

        let c1 = &r.children[0];
        let c2 = &r.children[1];

        // Column widths should be exactly 100px and 300px
        assert_eq!(c1.rect.size.width, 100.0);
        assert_eq!(c2.rect.size.width, 300.0);
    }

    #[test]
    fn test_percentage_column_widths_fixed_layout() {
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

        let cell1_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell1_node);

        let cell2_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell2_node);

        let cell3_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell3_node);

        let mut styles = HashMap::new();

        // Table style: width 500px, table-layout: fixed
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(500.0, LengthUnit::Px));
        table_style.insert(
            "table-layout".to_string(),
            CssValue::Keyword("fixed".to_string()),
        );
        styles.insert(table_node, table_style);

        styles.insert(row_node, style_with_display("table-row"));

        // Cell 1 style: width: 40% (resolved against 500px = 200px)
        let mut cell1_style = style_with_display("table-cell");
        let mut extra = HashMap::new();
        extra.insert(
            "width".to_string(),
            CssValue::Length(40.0, LengthUnit::Percent),
        );
        cell1_style.extra_values = Some(std::sync::Arc::new(extra));
        styles.insert(cell1_node, cell1_style);

        // Cell 2 style: width: 20% (resolved against 500px = 100px)
        let mut cell2_style = style_with_display("table-cell");
        let mut extra2 = HashMap::new();
        extra2.insert(
            "width".to_string(),
            CssValue::Length(20.0, LengthUnit::Percent),
        );
        cell2_style.extra_values = Some(std::sync::Arc::new(extra2));
        styles.insert(cell2_node, cell2_style);

        // Cell 3 style: auto width (should get remaining 200px)
        styles.insert(cell3_node, style_with_display("table-cell"));

        let table_box = layout_table_container(&dom, &styles, table_node, 600.0, 0.0, 0.0, 0)
            .expect("should layout table");

        assert_eq!(table_box.rect.size.width, 500.0);

        let r = &table_box.children[0];
        assert_eq!(r.children.len(), 3);

        let c1 = &r.children[0];
        let c2 = &r.children[1];
        let c3 = &r.children[2];

        // Column widths should be exactly 200px, 100px, and 200px
        assert_eq!(c1.rect.size.width, 200.0);
        assert_eq!(c2.rect.size.width, 100.0);
        assert_eq!(c3.rect.size.width, 200.0);
    }

    #[test]
    fn test_empty_cells_layout_hiding() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Create table element
        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Row 1: has one empty cell (empty-cells: show)
        let row1_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row1_node);

        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell11_node);

        // Row 2: has one empty cell (empty-cells: hide) -> should be completely hidden/collapsed!
        let row2_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row2_node);

        let cell21_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell21_node);

        // Row 3: has one non-empty cell (empty-cells: hide, but not empty) -> should remain visible!
        let row3_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row3_node);

        let cell31_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row3_node, cell31_node);
        let cell31_text = dom.create_node(NodeData::Text("Hello".to_string()));
        dom.append_child(cell31_node, cell31_text);

        // Setup styles
        let mut styles = HashMap::new();

        // Table style: width 200px, border-spacing: 10px
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        table_style.insert(
            "border-spacing".to_string(),
            CssValue::Length(10.0, LengthUnit::Px),
        );
        styles.insert(table_node, table_style);

        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));
        styles.insert(row3_node, style_with_display("table-row"));

        // Cell 1.1: empty, empty-cells: show (default). Height 30px.
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        // Cell 2.1: empty, empty-cells: hide. Height 40px.
        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        cell21_style.inherited_effects =
            std::sync::Arc::new(crate::style::categorized::InheritedEffects {
                visibility: "visible".to_string(),
                empty_cells: "hide".to_string(),
            });
        styles.insert(cell21_node, cell21_style);

        // Cell 3.1: non-empty, empty-cells: hide. Height 25px.
        let mut cell31_style = style_with_display("table-cell");
        cell31_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell31_style.insert("height".to_string(), CssValue::Length(25.0, LengthUnit::Px));
        cell31_style.inherited_effects =
            std::sync::Arc::new(crate::style::categorized::InheritedEffects {
                visibility: "visible".to_string(),
                empty_cells: "hide".to_string(),
            });
        styles.insert(cell31_node, cell31_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 10.0, 20.0, 0)
            .expect("should layout table");

        // Verify row 2 is hidden:
        // Visible rows should be: Row 1 (index 0) and Row 3 (index 2).
        // Total heights: Row 1 (30.0) + Row 3 (25.0) = 55.0.
        // Spacings: Spacing is only between visible rows and outer edges:
        // Spacing before Row 1 (10.0), spacing between Row 1 and Row 3 (10.0), spacing after Row 3 (10.0).
        // Total spacing height: 3 * 10.0 = 30.0.
        // Total table height should be: 55.0 + 30.0 = 85.0!
        assert_eq!(table_box.rect.size.height, 85.0);

        // Let's check row positions and heights:
        assert_eq!(table_box.children.len(), 3);

        // Row 1 (visible)
        let r1 = &table_box.children[0];
        assert_eq!(r1.rect.origin.y, 30.0); // 20.0 (border_box_y) + 10.0 (spacing)
        assert_eq!(r1.rect.size.height, 30.0);

        // Row 2 (hidden)
        let r2 = &table_box.children[1];
        assert_eq!(r2.rect.size.height, 0.0);

        // Row 3 (visible)
        let r3 = &table_box.children[2];
        assert_eq!(r3.rect.origin.y, 70.0); // 30.0 (Row 1 top) + 30.0 (Row 1 height) + 10.0 (spacing between r1 and r3)
        assert_eq!(r3.rect.size.height, 25.0);
    }

    #[test]
    fn test_auto_layout_flexible_column_distribution() {
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

        let cell1_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell1_node);

        let cell2_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell2_node);

        let mut styles = HashMap::new();

        // Table style: width 300px
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(300.0, LengthUnit::Px));
        styles.insert(table_node, table_style);

        styles.insert(row_node, style_with_display("table-row"));

        // Cell 1: explicit width: 100px
        let mut cell1_style = style_with_display("table-cell");
        cell1_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        styles.insert(cell1_node, cell1_style);

        // Cell 2: auto width, explicit min-width: 50px
        let mut cell2_style = style_with_display("table-cell");
        cell2_style.insert(
            "min-width".to_string(),
            CssValue::Length(50.0, LengthUnit::Px),
        );
        styles.insert(cell2_node, cell2_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 400.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // Table width should be 300.0
        assert_eq!(table_box.rect.size.width, 300.0);

        let r = &table_box.children[0];
        assert_eq!(r.children.len(), 2);

        let c1 = &r.children[0];
        let c2 = &r.children[1];

        // Column 1 is explicit (100px), Column 2 is auto (50px preferred).
        // Total sum_col_widths = 150px. Remaining table content width = 300px.
        // Extra space (150px) should be distributed ONLY to the flexible column (Column 2).
        // So Column 1 should remain exactly 100px, and Column 2 should stretch to 50px + 150px = 200px.
        assert_eq!(c1.rect.size.width, 100.0);
        assert_eq!(c2.rect.size.width, 200.0);
    }

    #[test]
    fn test_table_explicit_height_distribution() {
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

        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell11_node);

        // Row 2
        let row2_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row2_node);

        let cell21_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell21_node);

        let mut styles = HashMap::new();

        // Table style: width 200px, height 150px
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        table_style.insert(
            "height".to_string(),
            CssValue::Length(150.0, LengthUnit::Px),
        );
        styles.insert(table_node, table_style);

        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));

        // Cell 1.1: explicit height: 40px
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(40.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        // Cell 2.1: auto height (natural height 30px)
        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        std::sync::Arc::make_mut(&mut cell21_style.reset_box).height = -1; // make it auto height
        styles.insert(cell21_node, cell21_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 400.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // Table border box height should match configured height (150px)
        assert_eq!(table_box.rect.size.height, 150.0);

        // Row 1 should have its original explicit height (40px)
        // Row 2 is auto, so it should receive all extra height.
        // sum_row_heights (40 + 30) = 70px. Table height = 150px.
        // Remaining height to distribute = 80px.
        // Flexible Row 2 should get 30 + 80 = 110px.
        assert_eq!(table_box.children.len(), 2);
        let r1 = &table_box.children[0];
        let r2 = &table_box.children[1];

        assert_eq!(r1.rect.size.height, 40.0);
        assert_eq!(r2.rect.size.height, 110.0);
    }

    #[test]
    fn test_rowspan_positive_clamped_to_row_group() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let table_node = dom.create_node(NodeData::Element {
            name: "table".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(doc, table_node);

        // Group 1 (tbody)
        let tbody_node = dom.create_node(NodeData::Element {
            name: "tbody".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, tbody_node);

        // Row 1 in tbody
        let row1_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(tbody_node, row1_node);

        // Cell 1.1: rowspan="3" (but there are only 2 rows in tbody, so should be clamped to 2)
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![("rowspan".to_string(), "3".to_string())],
        });
        dom.append_child(row1_node, cell11_node);

        // Cell 1.2
        let cell12_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell12_node);

        // Row 2 in tbody
        let row2_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(tbody_node, row2_node);

        // Cell 2.1
        let cell21_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row2_node, cell21_node);

        // Group 2 (tfoot)
        let tfoot_node = dom.create_node(NodeData::Element {
            name: "tfoot".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, tfoot_node);

        // Row 3 (in tfoot)
        let row3_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(tfoot_node, row3_node);

        // Cell 3.1
        let cell31_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row3_node, cell31_node);

        let mut styles = HashMap::new();
        styles.insert(tbody_node, style_with_display("block"));
        styles.insert(tfoot_node, style_with_display("block"));
        styles.insert(table_node, style_with_display("table"));
        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));
        styles.insert(row3_node, style_with_display("table-row"));

        // Set dimensions
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        cell11_style.insert("height".to_string(), CssValue::Length(20.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        let mut cell12_style = style_with_display("table-cell");
        cell12_style.insert("width".to_string(), CssValue::Length(80.0, LengthUnit::Px));
        cell12_style.insert("height".to_string(), CssValue::Length(15.0, LengthUnit::Px));
        styles.insert(cell12_node, cell12_style);

        let mut cell21_style = style_with_display("table-cell");
        cell21_style.insert("width".to_string(), CssValue::Length(80.0, LengthUnit::Px));
        cell21_style.insert("height".to_string(), CssValue::Length(25.0, LengthUnit::Px));
        styles.insert(cell21_node, cell21_style);

        let mut cell31_style = style_with_display("table-cell");
        cell31_style.insert("width".to_string(), CssValue::Length(50.0, LengthUnit::Px));
        cell31_style.insert("height".to_string(), CssValue::Length(10.0, LengthUnit::Px));
        styles.insert(cell31_node, cell31_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // Checks:
        // Cell 1.1 has rowspan="3" but belongs to tbody with only 2 rows. It should be clamped to span 2 rows, not Row 3 (in tfoot).
        assert_eq!(table_box.children.len(), 3); // 3 rows

        let r1 = &table_box.children[0];
        let cell11_box = &r1.children[0];
        assert_eq!(cell11_box.node, Some(cell11_node));
        assert_eq!(cell11_box.rect.size.width, 50.0);
        assert_eq!(cell11_box.rect.size.height, 40.0); // Spans row 1 (15px) + row 2 (25px) = 40px

        let cell12_box = &r1.children[1];
        assert_eq!(cell12_box.node, Some(cell12_node));
        assert_eq!(cell12_box.rect.size.height, 15.0);

        let r2 = &table_box.children[1];
        let cell21_box = &r2.children[0];
        assert_eq!(cell21_box.node, Some(cell21_node));
        assert_eq!(cell21_box.rect.origin.x, 50.0); // pushed right by the rowspan cell

        let r3 = &table_box.children[2];
        let cell31_box = &r3.children[0];
        assert_eq!(cell31_box.node, Some(cell31_node));
        assert_eq!(cell31_box.rect.origin.x, 0.0); // starts at column 0 because the rowspan cell did NOT leak into row 3!
    }

    #[test]
    fn test_table_border_spacing_dual_values() {
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

        // Table style: width 200px, border-spacing: 15px 5px;
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        table_style.insert(
            "border-spacing".to_string(),
            CssValue::Multiple(vec![
                CssValue::Length(15.0, LengthUnit::Px),
                CssValue::Length(5.0, LengthUnit::Px),
            ]),
        );
        styles.insert(table_node, table_style);

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

        // Table box checks:
        // Table total height = row 1 (30px) + row 2 (40px) + 3 vertical spacings (3 * 5px = 15px) = 85px
        assert_eq!(table_box.rect.size.height, 85.0);

        // Row 1 checks:
        let r1 = &table_box.children[0];
        assert_eq!(r1.rect.origin.y, 25.0); // 20.0 + 5.0 (vertical spacing)

        // Cell 1.1 checks:
        let c11 = &r1.children[0];
        assert_eq!(c11.rect.origin.x, 25.0); // 10.0 + 15.0 (horizontal spacing)
    }

    #[test]
    fn test_table_layout_fixed_width_auto() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Create table element (width: auto)
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

        // Cell 1.1 (width: 120px)
        let cell11_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell11_node);

        // Cell 1.2 (width: 80px)
        let cell12_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell12_node);

        let mut styles = HashMap::new();

        // Table style: width: auto, table-layout: fixed, border-collapse: collapse
        let mut table_style = style_with_display("table");
        table_style.insert(
            "table-layout".to_string(),
            CssValue::Keyword("fixed".to_string()),
        );
        table_style.insert(
            "border-collapse".to_string(),
            CssValue::Keyword("collapse".to_string()),
        );
        styles.insert(table_node, table_style);

        styles.insert(row1_node, style_with_display("table-row"));

        // Row 1 Cell widths
        let mut cell11_style = style_with_display("table-cell");
        cell11_style.insert("width".to_string(), CssValue::Length(120.0, LengthUnit::Px));
        styles.insert(cell11_node, cell11_style);

        let mut cell12_style = style_with_display("table-cell");
        cell12_style.insert("width".to_string(), CssValue::Length(80.0, LengthUnit::Px));
        styles.insert(cell12_node, cell12_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // Width auto table under fixed table layout should resolve its width to the sum of its column widths: 120.0 + 80.0 = 200.0
        assert_eq!(table_box.rect.size.width, 200.0);

        let r1 = &table_box.children[0];
        let c11_box = &r1.children[0];
        let c12_box = &r1.children[1];

        assert_eq!(c11_box.rect.size.width, 120.0);
        assert_eq!(c12_box.rect.size.width, 80.0);
    }

    #[test]
    fn test_col_width_distribution_empty_table_fixed_width() {
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

        let cell1_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell1_node);

        let cell2_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row_node, cell2_node);

        let mut styles = HashMap::new();

        // Table style: width 200px, border-collapse: collapse
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        table_style.insert(
            "border-collapse".to_string(),
            CssValue::Keyword("collapse".to_string()),
        );
        styles.insert(table_node, table_style);

        styles.insert(row_node, style_with_display("table-row"));

        // Both cells are empty and have no width. Column widths are 0 initially.
        let cell1_style = style_with_display("table-cell");
        styles.insert(cell1_node, cell1_style);

        let cell2_style = style_with_display("table-cell");
        styles.insert(cell2_node, cell2_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // The columns should be expanded to fill the table width (200px / 2 = 100px each)
        assert_eq!(table_box.rect.size.width, 200.0);
        let r = &table_box.children[0];
        assert_eq!(r.children[0].rect.size.width, 100.0);
        assert_eq!(r.children[1].rect.size.width, 100.0);
    }

    #[test]
    fn test_border_collapse_equal_width_style_priority() {
        let c1 = BorderConflict {
            width: 3.0,
            style: "solid".to_string(),
            element_priority: 5,
        };
        let c2 = BorderConflict {
            width: 3.0,
            style: "double".to_string(),
            element_priority: 0,
        };
        let (resolved_w, resolved_s) = resolve_collapsed_borders_new(&[c1, c2]);
        assert_eq!(resolved_w, 3.0);
        assert_eq!(resolved_s, "double");
    }

    #[test]
    fn test_border_collapse_equal_width_and_style_element_priority() {
        let c1 = BorderConflict {
            width: 2.0,
            style: "solid".to_string(),
            element_priority: 5, // cell
        };
        let c2 = BorderConflict {
            width: 2.0,
            style: "solid".to_string(),
            element_priority: 4, // row
        };
        let c3 = BorderConflict {
            width: 2.0,
            style: "solid".to_string(),
            element_priority: 3, // row group
        };
        let (resolved_w, resolved_s) = resolve_collapsed_borders_new(&[c1, c2, c3]);
        assert_eq!(resolved_w, 2.0);
        assert_eq!(resolved_s, "solid");
    }

    #[test]
    fn test_border_collapse_empty_hidden_rows_skipping() {
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

        // Cell 1
        let cell1_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row1_node, cell1_node);

        // Row 2 (completely empty - has no cells at all)
        let row2_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row2_node);

        // Row 3
        let row3_node = dom.create_node(NodeData::Element {
            name: "tr".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(table_node, row3_node);

        // Cell 3
        let cell3_node = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: Vec::new(),
        });
        dom.append_child(row3_node, cell3_node);

        let mut styles = HashMap::new();

        // Table style: border-collapse: collapse
        let mut table_style = style_with_display("table");
        table_style.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        table_style.insert(
            "border-collapse".to_string(),
            CssValue::Keyword("collapse".to_string()),
        );
        styles.insert(table_node, table_style);

        styles.insert(row1_node, style_with_display("table-row"));
        styles.insert(row2_node, style_with_display("table-row"));
        styles.insert(row3_node, style_with_display("table-row"));

        // Cell 1 style: border-bottom 5px solid
        let mut cell1_style = style_with_display("table-cell");
        cell1_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell1_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        cell1_style.insert(
            "border-bottom-width".to_string(),
            CssValue::Length(5.0, LengthUnit::Px),
        );
        styles.insert(cell1_node, cell1_style);

        // Cell 3 style: border-top 10px solid
        let mut cell3_style = style_with_display("table-cell");
        cell3_style.insert("width".to_string(), CssValue::Length(100.0, LengthUnit::Px));
        cell3_style.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        cell3_style.insert(
            "border-top-width".to_string(),
            CssValue::Length(10.0, LengthUnit::Px),
        );
        styles.insert(cell3_node, cell3_style);

        let table_box = layout_table_container(&dom, &styles, table_node, 500.0, 0.0, 0.0, 0)
            .expect("should layout table");

        // The layout should resolve and find that Row 2 is empty, so Row 3 and Row 1 are directly adjacent for border conflict resolution.
        // Therefore, Cell 1's border-bottom should resolve against Cell 3's border-top: max(5, 10) = 10px.
        assert_eq!(table_box.children.len(), 3);
        let r1 = &table_box.children[0];
        let r3 = &table_box.children[2]; // Index 2 is Row 3, as Row 2 is at Index 1

        assert_eq!(r1.children.len(), 1);
        assert_eq!(r3.children.len(), 1);
    }
}
