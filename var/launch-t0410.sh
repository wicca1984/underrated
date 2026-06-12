#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0410
LOG=/workspaces/toy-browser/var/log/t0410.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0410 — implement the CSS `table-layout: fixed` column-width algorithm in layout. Touch ONLY src/layout/table.rs. Do NOT edit any other file/module. If something truly needs another module, leave a `// TODO(spec): ...` and report instead.

Context (read before coding):
- The `table-layout` property (keywords `auto` | `fixed`) is already parsed and stored in computed style (merged in t0406). You consume it here in layout, just like `caption-side` is already consumed: `layout_table_container` does `let style = styles.get(&node)?;` and reads `style.get("caption-side")`. Read `style.get("table-layout")` the same way (a `CssValue::Keyword`).
- In `src/layout/table.rs`, `layout_table_container` computes per-column widths in the block around lines ~200-280: it builds `col_widths: Vec<f32>` of length `num_cols`, then for each cell calls `get_cell_preferred_width(...)` (content measurement = the AUTO algorithm) and finally redistributes/scales to fit `avail_content_width` / `avail_col_width`.

What "fixed" means (CSS 2.1 §17.5.2.1, scoped to the common case):
- In fixed table layout, column widths are determined ONLY by the table's first row, NOT by cell content. Content does not affect column widths.
- For each cell in the FIRST row (respecting its colspan and column index), if the cell has an explicit `width` in px (read it with the existing `get_px` helper on the cell's computed style, treating absent/0 as "auto"), assign that width to its column(s). Columns with no explicit first-row width are "auto".
- Distribute the table's available content width (`avail_content_width`, already computed from the table's own width) so that: explicitly-sized columns keep their width, and the leftover `(avail_content_width - sum_of_explicit_col_widths - spacing)` is split EQUALLY among the auto columns. If leftover is negative, clamp auto columns to 0.0 and keep explicit widths as-is (do not measure content).

Scope for THIS task (single file, src/layout/table.rs):
1. Near the top of the column-width computation, detect fixed mode once:
   `let table_layout_fixed = matches!(style.get("table-layout"), Some(CssValue::Keyword(kw)) if kw == "fixed");`
2. When `table_layout_fixed` is true, compute `col_widths` from the FIRST row's cells using their explicit `get_px(cell_style, "width", 0.0)` (0.0 => auto), then distribute the remaining available content width equally among the auto columns. Do NOT call `get_cell_preferred_width` in this path (content must be ignored). Reuse the existing `num_cols`, spacing, and `avail_content_width` values already computed above.
3. When `table_layout_fixed` is false (auto or absent), keep the EXISTING auto algorithm completely unchanged.
4. Keep the rest of `layout_table_container` (row layout, colspan/rowspan placement, caption, box model) untouched — only the column-WIDTH determination branches on the new flag.
5. Leave `// TODO(spec):` markers for the out-of-scope cases: percentage column widths, `<colgroup>`/`<col>` width sources, and `width: auto` table (treat table width as already resolved by the existing code).

Panic-free: no unwrap/expect/panicking indexing in non-test code (guard column indexing with bounds checks or iterators, as the surrounding code does).

Tests — add to the existing `#[cfg(test)] mod tests` in src/layout/table.rs (do NOT modify or delete any existing test; mirror the setup style of `test_basic_table_layout` / `test_colspan_table_layout`):
- A `table { table-layout: fixed; width: 300px }` with a first row of 3 cells where one cell has `width: 150px` and the other two are auto: assert the explicit column is 150px and the two auto columns each get an equal share of the remainder (≈ (300 - 150 - spacing)/2), and that a long text string in a LATER row does NOT widen its column (i.e., column widths are independent of content).
- A control test confirming that with `table-layout: auto` (or omitted) the existing content-based widths are unchanged for the same markup (guard against regressions).

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(layout): implement fixed table-layout column-width algorithm (t0410)"
Then print "T0410 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
