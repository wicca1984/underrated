#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0271
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0271 — Honor `border-collapse: collapse` on tables by collapsing inter-cell spacing to zero. Today the table layout always uses the separated borders model: `border-spacing` (and its default gap) is applied between cells regardless of `border-collapse`. Wikipedia and most content tables use `border-collapse: collapse`, where the gap between cells must vanish. This task implements ONLY the spacing-collapse part of the collapsing borders model; the full per-edge border conflict resolution stays a documented TODO(spec).

Target module: src/layout/table.rs (touch ONLY this file — both the layout code and the inline `#[cfg(test)] mod tests`). Do NOT modify any other file. `git diff --name-only` must show ONLY: src/layout/table.rs.

Reuse / facts (verified — do NOT reinvent):
- `layout_table_container` reads spacing at line ~186: `let (spacing_h, spacing_v) = get_border_spacing(style);` immediately under the comment `// TODO(spec): border-collapse: collapse is not implemented. Defaulting to separate.` (line ~185). `style` is the `&ComputedStyle` for the `<table>`. `spacing_h`/`spacing_v` flow into ALL column-width, cell-position and total-width math below — so zeroing them at this single point correctly collapses the gaps everywhere.
- `get_border_spacing(style: &ComputedStyle) -> (f32, f32)` is defined at line ~427 and shows the exact idiom for reading a value off `style`: `match style.get("..") { Some(CssValue::Length(v, _)) => .., _ => .. }`. `CssValue` is `crate::css::values::CssValue` (already in scope). Keyword values are `CssValue::Keyword(String)` — match with `Some(CssValue::Keyword(kw)) if kw == "collapse" => ..` (compare with `kw.as_str()` or `kw ==`). Confirm the exact `CssValue::Keyword` shape by reading how other code in this file matches keywords before writing yours.

Implement:
1. Add a small helper `fn is_border_collapse(style: &ComputedStyle) -> bool` near `get_border_spacing`, returning true iff `style.get("border-collapse")` is the keyword `collapse`. Default (absent, `separate`, or any other value) => false. Do NOT use unwrap/expect/panic/unsafe.
2. At the spacing read site (~line 186), after `let (spacing_h, spacing_v) = get_border_spacing(style);`, override to zero when collapsing: if `is_border_collapse(style)` then set both to `0.0`. Per CSS 2.2 §17.6.2, `border-spacing` is ignored in the collapsing model — so collapse must force spacing to 0 even if `border-spacing` is also set. Keep it simple (e.g. `let (spacing_h, spacing_v) = if is_border_collapse(style) { (0.0, 0.0) } else { get_border_spacing(style) };`).
3. Update the line-185 comment to reflect reality: spacing now collapses to zero under `border-collapse: collapse`, but the full per-edge border conflict resolution (adjacent borders share one edge; wider border wins; precedence cell>row>row-group>col>col-group>table) is NOT yet implemented. Leave it as a `// TODO(spec): border-collapse: full border conflict resolution (shared edges, width/precedence) not implemented; here we only collapse inter-cell spacing to zero.` Do NOT attempt the conflict-resolution algorithm in this task — that is a separate follow-up. Do not invent border-merging behavior.

Acceptance — extend the inline unit tests. Mirror the existing `fn test_table_border_spacing_separated_with_spacing()` (around line 1312), which builds a 200px-wide table with `border-spacing: 10px` and asserts cells are separated by the gap. Add a NEW test `fn test_table_border_collapse_zeroes_spacing()` that builds the SAME table structure but adds `border-collapse: collapse;` to the table style ALONGSIDE `border-spacing: 10px;`, then asserts that the inter-cell spacing is zero: e.g. adjacent cells in a row abut (cell[1].rect.origin.x == cell[0].rect.origin.x + cell[0].rect.size.width, within a tiny epsilon) and that the first cell starts flush at the table content origin (no leading 10px gap). Reuse the EXACT DOM/stylesheet construction idioms (`dom.create_node`, `append_child`, `parse_stylesheet`, `compute_styles`/`compute_styles`-equivalent, `layout_*`) from `test_table_border_spacing_separated_with_spacing` so the only behavioral difference is the collapse keyword. Do NOT weaken or remove any existing test.

Done when ALL of these pass in this worktree:
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
No `unwrap`/`expect`/`panic!` in non-test code (I-6). No new dependencies (I-1). Comments and identifiers in English.
Commit (you MUST commit before finishing): `feat(layout): collapse inter-cell spacing under border-collapse:collapse (t0271)`.
End with a short English summary of exactly what changed and the `// TODO(spec):` you left.'
