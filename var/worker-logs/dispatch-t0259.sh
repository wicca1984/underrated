#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0259
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0259 — Honor the CSS `visibility: hidden` (and `collapse`) property in the paint stage so invisible elements and their inherited text are not drawn. Real result pages (e.g. Google) use `visibility:hidden` heavily to hide UI without removing layout. This advances render fidelity for MS-MVP/MS-NewTargets.

Target module: src/paint/mod.rs (touch ONLY src/paint/mod.rs and its inline tests). Do NOT modify src/style, src/layout, src/dom or any other module — `visibility` is ALREADY fully resolved upstream. Read other modules read-only as needed.

Reuse / facts (verified — do NOT reinvent):
- `visibility` is already resolved per-node in style and stored on every ComputedStyle as a keyword. `style.get("visibility")` returns `Some(CssValue::Keyword(k))` where `k` is one of `"visible"`, `"hidden"`, `"collapse"`. Inheritance and `inherit`/`initial` are ALREADY handled in src/style/mod.rs (around lines 464-479) — every node, INCLUDING text nodes, has an authoritative computed `visibility`. Do NOT re-resolve inheritance; just read the value.
- The display-list builder is `build_display_list` (src/paint/mod.rs:253). It does an iterative pre-order traversal. The per-node block starts at src/paint/mod.rs:265 where it binds `(node_id, style)` via `layout_box.node.and_then(|id| styles.get(&id).map(|s| (id, s)))`.
- Within that block the node emits its OWN paint items: button/submit label text (src/paint/mod.rs:322-349), `background-color` SolidRect (src/paint/mod.rs:357-363), borders (the block starting near src/paint/mod.rs:365), and text-node glyphs `if let Some(NodeData::Text(text)) = dom.data(node_id)` emitting `DisplayItem::Text` (src/paint/mod.rs:442-465).
- Text nodes go through the SAME per-node block and have their own ComputedStyle entry, so their inherited `visibility` is authoritative — checking the node-level value covers BOTH element boxes and text in one place.

Semantics (CSS standard) — implement EXACTLY:
- An element whose computed `visibility` is `hidden` or `collapse` must NOT paint its own box decorations (background-color, borders) NOR its own text (button/submit label, or text-node glyphs). It is invisible but STILL occupies layout space — do NOT change layout, do NOT skip its subtree.
- Children are still traversed normally. A descendant that re-asserts `visibility: visible` MUST paint (its own ComputedStyle already carries `visible`, so no special handling is needed — just do not suppress the whole subtree).
- Treat `collapse` the same as `hidden` for this task (correct enough for non-table content). Leave a `// TODO(spec):` noting table-row `collapse` differs.

Approach:
1. Near the top of the per-node block (right after `(node_id, style)` is bound at src/paint/mod.rs:265), compute a boolean, e.g. `let node_hidden = matches!(style.get("visibility"), Some(CssValue::Keyword(k)) if k == "hidden" || k == "collapse");`.
2. Guard this node`s OWN item emission with `if !node_hidden { ... }`: the button/submit label push, the background SolidRect, the border SolidRects, and the text-node `DisplayItem::Text` push. Do NOT guard the children-onto-stack push (subtree must still be traversed). Do NOT set `skip_children` because of visibility.
3. No unwrap/expect/panic/unsafe in non-test code (I-6). Keep the existing iterative traversal.

Acceptance (must all be green) — add inline unit tests in src/paint/mod.rs mirroring the existing paint tests (build a DOM + styles map + layout box and call `build_display_list`, then inspect the returned items):
  - A `<div>` with `visibility:hidden` and `background-color` + a border + a child text node emits NO SolidRect and NO DisplayItem::Text for that subtree (when the text inherits hidden).
  - A hidden `<div>` containing a child element with `visibility:visible` that has a background-color: the parent emits nothing, but the visible child STILL emits its SolidRect (proves subtree is traversed, not skipped).
  - Regression: a `visibility:visible` (or unset) element paints its background/border/text exactly as before.
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Done when all three pass. No unwrap/expect in non-test code (I-6). No unsafe (forbidden). No test skip/ignore (I-4). Keep the diff limited to src/paint/mod.rs — `git diff --name-only` must show ONLY src/paint/mod.rs. Commit on this branch with: `feat(paint): honor visibility:hidden/collapse in display list (t0259)`. Comments and identifiers in English. IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the names of the tests you added. If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
