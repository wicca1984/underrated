#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0378
LOG=/workspaces/toy-browser/var/log/t0378.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0378 — implement the CSS `empty-cells` property for table cell border suppression.

Target files: src/paint/table.rs (primary) and src/style/mod.rs (minimal: parse + default the property). Touch ONLY these two files. Do NOT touch layout/, engine/, main.rs, dom/, or any other module or worktree. If something genuinely requires another module, leave a `// TODO(spec): ...` and stop there.

Background:
- `src/paint/table.rs` renders presentational `<table border>` gridlines. `table_border_items(dom, node_id, name, rect)` returns the 4 edge SolidRect items for a bordered table cell (or empty Vec). It checks the nearest ancestor `<table>`'s `border` attribute via `dom.get_attribute`.
- The CSS property `empty-cells` (inherited; values `show` | `hide`; initial `show`) says: when a cell (`<td>`/`<th>`) has NO content, and `empty-cells: hide` is in effect, the cell's borders (and background) are NOT drawn.

Implement:
1. In `src/style/mod.rs`, parse the `empty-cells` declaration so the computed style for a node exposes `empty-cells` as a CssValue::Keyword ("show" or "hide"). It is an INHERITED property with initial value "show". Follow the existing pattern other inherited keyword properties use in this file (look at how e.g. text-align or an existing inherited keyword prop is parsed/defaulted and mirror it). Keep it small — just plumb the keyword through to the computed style map.
2. In `src/paint/table.rs`, inside `table_border_items` (which is invoked for `<td>`/`<th>` cells), before emitting the 4 border strips for a CELL, determine whether the cell is EMPTY and whether `empty-cells: hide` is in effect for that cell; if both true, return an empty Vec (skip the cell's border items). Definition of "empty" for this toy browser: the cell node has no child nodes that produce rendered content — i.e. no element children and no non-whitespace text in its subtree. Add a small private helper `fn cell_has_rendered_content(dom: &Dom, node_id: NodeId) -> bool` in this file (walk the cell's descendants; return true if any text node has non-whitespace text or any element child exists that would render, e.g. `<img>`). Only apply this suppression to cells (`name` is `td`/`th`), NOT to the `<table>` box itself.
   - To read the computed `empty-cells` value: the cell's style should already be reachable the same way other paint code reads computed styles. If `table_border_items` does not currently receive the style, read the property via the same mechanism the surrounding paint code uses for that node; if there is genuinely no access to computed style here without changing the function signature in a way that touches other modules, instead read it pragmatically (default to "show" when unavailable) and leave a `// TODO(spec): thread computed empty-cells via style` — but PREFER reading the real computed value if a Dom/style accessor already exists in scope.

Constraints (AGENTS.md I-1..I-7):
- Touch ONLY src/paint/table.rs and src/style/mod.rs.
- NO `unwrap()`/`expect()` in non-test code (I-6) — use match/if let.
- Do NOT skip, #[ignore], or delete any existing test. Keep ALL existing #[test].
- Keep `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` green.

Add unit tests in the existing `#[cfg(test)] mod tests` block in src/paint/table.rs:
- A bordered table with a NON-empty `<td>` and `empty-cells: hide` -> that cell STILL produces its border SolidRect items.
- A bordered table with an EMPTY `<td>` and `empty-cells: hide` -> that empty cell produces ZERO border items.
- A bordered table with an EMPTY `<td>` and `empty-cells: show` (default) -> the empty cell STILL produces its border items (regression guard that default behavior is unchanged).

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(table): suppress empty cell borders via CSS empty-cells:hide (t0378)"
Then print "T0378 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
