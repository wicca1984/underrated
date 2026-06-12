#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0268
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0268 — Render distinct `<ul>` list-item markers per `list-style-type` (`disc`, `circle`, `square`) using proper Unicode bullet glyphs instead of the current ASCII `*` fallback. Today every unordered-list bullet renders the literal `*` regardless of `list-style-type`, so nested lists (which alternate disc/circle/square) and styled lists look wrong. This advances MS-NewTargets (Wiki, where nested bulleted lists are pervasive).

Target module: src/layout/mod.rs (touch ONLY this file — both the layout code and the inline `#[cfg(test)] mod tests`). Do NOT modify any other file. `git diff --name-only` must show ONLY: src/layout/mod.rs.

Reuse / facts (verified — do NOT reinvent):
- The `<ul>` marker is emitted in the `if list_name == "ul"` branch of the list-marker code (around line 422). It currently does: `let text_node = find_first_text_node(dom, node); let marker_text = text_node.map(|_| String::from("*"));` then pushes a `LayoutBox { node: text_node.or(Some(node)), rect: Rect::new(marker_x, marker_y, side, side), children: Vec::new(), text: marker_text };`.
- `list_style_type` is already computed just above as `style.get("list-style-type")` and the `none` case is already handled by `suppress_marker` (the `ul` branch only runs when the marker is NOT suppressed). `style` is the `&ComputedStyle` for the `<li>`.
- `CssValue` is `crate::css::values::CssValue` (already in scope in this file); keyword values are `CssValue::Keyword(String)`. Match it the same way the existing `ol` branch does (`Some(CssValue::Keyword(kw)) => match kw.as_str() { ... }`).
- The marker glyph is rendered as TEXT via the bitmap font stack, which falls back to a system face for non-ASCII (the same path that already renders CJK). So Unicode bullet code points are an acceptable, in-style choice here.

Semantics — choose the marker glyph from the `<li>`s computed `list-style-type` (https://drafts.csswg.org/css-lists/#typedef-counter-style):
- `circle` => "\u{25E6}" (◦ WHITE BULLET)
- `square` => "\u{25AA}" (▪ BLACK SMALL SQUARE)
- `disc` OR any other/unrecognized keyword OR absent (None) => "\u{2022}" (• BULLET)  ← `disc` is the default for `<ul>`.
Keep the marker `rect` (position `marker_x`/`marker_y` and `side` size) EXACTLY as it is today — do NOT change marker geometry/positioning (that is a separate `// TODO(spec)` about baseline/metrics; leave those TODO lines intact). Only change which string `marker_text` is set to. Preserve the existing `text_node`/`node` wiring: still compute `let text_node = find_first_text_node(dom, node);` and set the marker box `text` to `Some(<glyph>.to_string())` only when `text_node.is_some()` (keep the current `text_node.map(|_| ...)` shape so an empty `<li>` with no text node behaves exactly as before). Use `char`/`&str` literals like `"\u{25E6}"`; do NOT use unwrap/expect/panic/unsafe.
- Update the comment on the line above (currently `// As a fallback to actually paint a visible bullet, we render an ASCII "*" character using the first text node.`) to reflect that we now select a Unicode bullet glyph per list-style-type. Keep the other `// TODO(spec):` lines (paint-side fill primitive, baseline/metrics, list-style-position: inside, list-style-image) intact — this task does NOT address those.

Acceptance — extend the inline unit tests. There is an existing test `fn test_list_item_markers()` (around line 1028) that builds a `<ul>` and asserts the disc markers geometry (center_x ~20.0, size 6.4) but does NOT assert marker text. Do BOTH of these:
  1. In `test_list_item_markers`, after the existing `li_a_marker`/`li_b_marker` geometry assertions, add `assert_eq!(li_a_marker.text.as_deref(), Some("\u{2022}"));` (the default-`disc` bullet) to lock in the glyph. Do not weaken or remove any existing assertion.
  2. Add a NEW test `fn test_unordered_list_marker_styles()` that mirrors the helper pattern of `test_list_item_markers` (create `body`, then three `<ul>` elements with inline `style` attrs `list-style-type: disc;`, `list-style-type: circle;`, `list-style-type: square;`, each containing one `<li>` with a text child; build via `dom.create_node(NodeData::Element{ name, attrs: vec![("style".into(), "...".into())] })`, `dom.append_child`, `parse_stylesheet`, `compute_styles`, `layout_document`). For each list, read the `<li>` boxs marker child (the LAST child of the li box, index 1 — same as the existing test) and assert `marker.text.as_deref()` equals `Some("\u{2022}")`, `Some("\u{25E6}")`, `Some("\u{25AA}")` respectively. Reuse the EXACT stylesheet/structure idioms from `test_list_item_markers` (e.g. `ul, ol { display: block; padding-left: 40px; }`, `li { display: block; }`).

Done when ALL of these pass:
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
No unwrap/expect/panic/unsafe in non-test code (I-6). No `unsafe` anywhere. No test skip/ignore (I-4). Keep the diff limited to src/layout/mod.rs — `git diff --name-only` must show ONLY that file. Commit on this branch with: `feat(layout): render disc/circle/square ul markers with bullet glyphs (t0268)`. Comments and identifiers in English. IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the names of the tests you added/changed. If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
