#!/usr/bin/env bash
# Launcher for Gemini worker t0282 — caption-side: bottom in table layout (src/layout/table.rs).
# Dispatched via setsid so it survives the orchestrator tick (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0282
LOG=/workspaces/toy-browser/var/worker-logs/t0282.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0282 — Implement CSS `caption-side: bottom` for table layout.
Read: docs/ARCHITECTURE.md and docs/SPEC.md (layout / tables) under /workspaces/underrated-meta/.
Target module: src/layout/table.rs ONLY. Do NOT touch any other module or any file outside src/layout/table.rs (no lib.rs additions), or any other worktree. The CSS value parsing for keywords is already generic (style.get("caption-side") returns a CssValue::Keyword); do NOT modify the css parser.

Background (verify before changing — there is an explicit `// TODO(spec): caption-side: bottom is not implemented` marker in src/layout/table.rs):
- Today the table caption is always laid out at the TOP: the code finds `first_caption`, lays it out via `layout_node`, stores `caption_height`, and offsets the first row's `curr_y` by `... + caption_height`. The caption box is then pushed into `table_children` and the rows follow it.
- The caption box's vertical position currently coincides with the top of the table content (caption_height reserves space above the rows).

Goal — implement WHATWG/CSS2 `caption-side` semantics:
- `caption-side: top` (default, also when the property is absent or any unknown value) — current behavior: caption sits ABOVE the rows; rows are pushed down by the caption height.
- `caption-side: bottom` — caption sits BELOW all the rows. The rows must start at the normal top position (NOT offset by caption_height), and the caption box must be repositioned so its top y is just below the last row (after the table's row content / bottom border-box flow, consistent with how the table's total height is computed). The table's overall content height must still include the caption height (so the caption is not clipped and following content does not overlap it).

Implementation notes:
- Read `style.get("caption-side")` for the TABLE element's style (the same style map already in scope used elsewhere in this function). Treat `Some(CssValue::Keyword(kw))` where `kw == "bottom"` as bottom; everything else (including None) as top. Match the exact CssValue import/pattern style already used in this file.
- For `top`: keep the existing math (rows offset by caption_height; caption box y stays at the top).
- For `bottom`: do NOT offset the rows by caption_height; instead, after the rows' `curr_y` reaches the bottom of the table content, set the caption box's rect origin y to that bottom position before pushing it into `table_children`, and ensure the returned table box height includes the caption height. Mutate the caption box's `rect.origin.y` (the LayoutBox you got from `layout_node`) — find the exact field path used elsewhere in this file for a box's position (e.g. `cap_box.rect.origin.y = ...`).
- Keep horizontal placement of the caption unchanged.
- No `unwrap()`/`expect()` in non-test code (I-6). Mirror neighboring error/Option handling.

Approach: test-first (TDD).
Acceptance (must be green) — add a unit test in the existing `#[cfg(test)]` module of src/layout/table.rs, copying the style of the existing caption test (search for the test that exercises a `<caption>`):
- Build a table with a `<caption>` and at least one row with one cell, where the table element has `style="caption-side: bottom"`.
- Lay it out and assert that the caption box's top y is GREATER than the first data row's/cell's top y (i.e. caption is below the rows), and that for the default/`top` case the caption box's top y is LESS than the first row's top y.
- Add (or keep) a top-side assertion to prove the default path is unchanged.
- Keep ALL existing table/layout tests green (do not weaken or delete any existing assertion or test — deleting/altering foreign tests to force green is a hard violation).

Done when: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.
Commit (you MUST git add + git commit before finishing — uncommitted work is lost):
  `feat(layout): implement caption-side: bottom for table layout (t0282)`
Comments and identifiers in English.
If the spec is ambiguous or conflicts with real browser behavior, do NOT guess — leave a `// TODO(spec):` and report it.
End with a short summary of exactly what changed and the test name you added.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
