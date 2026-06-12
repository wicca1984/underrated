#!/usr/bin/env bash
# Launcher for Gemini worker t0279 — table cell vertical-align (layout/table.rs).
# Dispatched via setsid so it survives the orchestrator tick (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0279
LOG=/workspaces/toy-browser/var/worker-logs/t0279.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0279 — Apply `vertical-align` to table cell content (top / middle / bottom) in table layout.
Read: docs/ARCHITECTURE.md and docs/SPEC.md (table layout) under /workspaces/underrated-meta/.
Target module: src/layout/table.rs ONLY. Do NOT touch any other module, file outside src/layout/table.rs (lib.rs additions are forbidden here), or any other worktree.

Background (already true in the code — verify before changing):
- `layout_table_container` in src/layout/table.rs computes per-row `row_heights`, then re-lays out each cell at `cell_y = row_y_offsets[r]` and force-sets `cell_box.rect.size.height = cell_height` (the full row height, which may exceed the cell's natural content height when a sibling cell in the row is taller).
- Today the cell's CHILDREN are always positioned starting at the top of the cell — i.e. the engine effectively renders `vertical-align: top` for every cell. There is no vertical centering/bottom alignment.

Goal: honor the cell's resolved `vertical-align` style value:
- `top`            -> no change (children flush to cell top). This is the default in our v1.
- `middle`         -> shift the cell's children DOWN by `(cell_height - natural_content_height) / 2`.
- `bottom`         -> shift the cell's children DOWN by `(cell_height - natural_content_height)`.
- `baseline` (CSS default for table-cell) and any other value -> treat as `top` for now and leave a single `// TODO(spec): table-cell vertical-align baseline/sub/super/text-top/text-bottom not implemented; treated as top.` comment.
Only shift when the delta is positive (cell taller than its natural content). Never shift up.

Implementation notes:
- The "natural content height" of a cell is the height produced by `layout_node` BEFORE you overwrite `cell_box.rect.size.height = cell_height`. Capture it (e.g. `let natural_h = cell_box.rect.size.height;`) right after laying the cell out and before the override.
- Resolve the cell's `vertical-align` from `styles.get(&cell_node)` (it is a CssValue::Keyword like "top"/"middle"/"bottom"; the HTML `valign` attribute is already mapped to this property in src/style/mod.rs, so you do NOT need to read attributes).
- To apply the shift, translate every descendant LayoutBox of the cell box by the y delta. Look for an existing offset/translate helper in src/layout (e.g. a function that recursively adds to `rect.origin.y` / `rect.pos.y`); if none exists, add a small private recursive helper INSIDE src/layout/table.rs that walks `LayoutBox.children` and adds `dy` to each box's rect y. Shift the children, NOT the outer cell box (the cell box stays at `cell_y` with height `cell_height`).
- Inspect the actual `Rect` field names in the codebase (do not assume) before writing the shift.

Approach: test-first (TDD).
Acceptance (must be green):
- Add unit test(s) in src/layout/table.rs (in the existing `#[cfg(test)]` module if present, else add one) that build a 1-row table where one cell is intentionally tall (e.g. multi-line content) and a shorter sibling cell carries `vertical-align: middle` (and another `bottom`). Assert that the shorter cell's CHILD content y-origin is offset by the expected amount relative to the `top` baseline. Cover `top` (no shift), `middle` (half), `bottom` (full).
- Keep ALL existing table tests green (do not weaken or delete any existing assertion or test — deleting/altering foreign tests to force green is a hard violation).

Done when: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.
Commit (you MUST git add + git commit before finishing — uncommitted work is lost):
  `feat(layout): apply vertical-align (top/middle/bottom) to table cell content (t0279)`
Comments and identifiers in English. No `unwrap()`/`expect()` in non-test code (I-6).
If the spec is ambiguous or conflicts with real browser behavior, do NOT guess — leave a `// TODO(spec):` and report it.
End with a short summary of exactly what changed and the test names you added.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
