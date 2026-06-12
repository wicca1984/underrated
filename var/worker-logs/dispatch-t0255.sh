#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0255
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0255 — Implement the CSS `border-spacing` property for tables in the SEPARATED borders model (the default). `border-spacing` inserts space between adjacent table cells (and between cells and the table edge). This is common in real table-heavy pages and improves table fidelity.

Target module: src/layout/table.rs (touch ONLY src/layout/table.rs and its inline tests). Do NOT modify src/style, src/css, src/dom or any other module — REUSE the existing ComputedStyle/CssValue API. Read those modules as needed.

Background / reuse (already implemented, do NOT reimplement):
- The style cascade already stores arbitrary length properties generically. `border-spacing: 4px` is available at layout time as `style.get("border-spacing") -> Some(CssValue::Length(4.0, LengthUnit::Px))` on the TABLE element''s ComputedStyle. (Verified: src/style/mod.rs default property arm inserts unknown props verbatim.)
- `layout_table_container(...)` in src/layout/table.rs already computes `col_widths`, per-column x offsets (`col_offset_x` = sum of preceding col widths), `cell_x`/`cell_y`, `row_y_offsets`, and the total table content width/height. You will thread a horizontal and vertical spacing value through these.

Spec (separated borders model — https://www.w3.org/TR/CSS22/tables.html#separated-borders):
- Read the spacing from the TABLE element''s ComputedStyle: `border-spacing: <length>` (a single length) applies to BOTH horizontal and vertical spacing. Default 0 when the property is absent.
- Horizontal spacing `sh` is inserted between adjacent columns AND once at the left edge (before col 0) and once at the right edge (after the last col). For N columns the total horizontal spacing added to table content width is `(N + 1) * sh`. Each cell''s x offset gains `(col_idx + 1) * sh` relative to today (left edge + one gap per preceding column boundary).
- Vertical spacing `sv` is inserted between adjacent rows AND once at the top edge and once at the bottom edge. For M rows the total vertical spacing added to table content height is `(M + 1) * sv`. Each row''s y offset gains `(row_idx + 1) * sv`.
- Spacing affects the table''s own content box size (it grows the table), it does NOT shrink cells.
- The TWO-VALUE form `border-spacing: <h> <v>` (separate horizontal/vertical) — only handle it if `parse_value` already preserves both lengths; if the stored value is a single Length (only the first value survives), implement the single-value (uniform) case and leave a `// TODO(spec): two-value border-spacing (horizontal vertical) once CssValue carries both` and do NOT add CSS-parsing logic here.
- `border-collapse: collapse` is OUT OF SCOPE (separated model only). Do not implement it.

Approach (test-first / TDD):
1. Add a small helper to read the uniform spacing px from the table''s ComputedStyle (Length in Px; treat non-px or absent as 0.0; do not panic).
2. Thread `sh`/`sv` into: column x offsets, cell_x, row_y_offsets/cell_y, and the table content width/height totals so the table box grows correctly.
3. Keep all existing tests green and add new ones.

Acceptance (must all be green) — add inline unit tests in src/layout/table.rs mirroring `test_basic_table_layout`:
  - a 2x2 table with `border-spacing: 10px`: assert cell x/y positions are shifted by the expected (col+1)*10 / (row+1)*10, and the table content width/height grew by (cols+1)*10 / (rows+1)*10 vs the zero-spacing case.
  - a table with NO border-spacing renders identically to before (regression guard: positions unchanged).
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Done when all three pass. No unwrap/expect in non-test code (I-6). No unsafe (forbidden). No test skip/ignore (I-4). Keep the diff limited to src/layout/table.rs — `git diff --name-only` must show ONLY src/layout/table.rs.
Commit on this branch with: `feat(layout): implement separated-model border-spacing for tables (t0255)`. Comments and identifiers in English.
IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the names of the tests you added.
If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
