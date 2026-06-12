#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0381
LOG=/workspaces/toy-browser/var/log/t0381.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0381 — make inline layout honor the CSS `line-height` property for line-box height.

Target file: src/layout/inline.rs ONLY. Touch ONLY src/layout/inline.rs. Do NOT edit style/, paint/, css/, engine/, main.rs, layout/mod.rs, or any other file/module. Use ONLY existing public APIs. If something genuinely requires another module, leave a `// TODO(spec): ...` comment in src/layout/inline.rs and stop — do not edit other files.

Context (current behavior, all in src/layout/inline.rs):
- The line-box height is driven by a local `let line_height = font.line_height() as f32;` (the FONT's default metric) and a mutable `current_line_height` that starts from it. The CSS `line-height` property is currently IGNORED for line-box height.
- The resolved CSS value is ALREADY available: `styles.get(&node)` returns the node's computed style map, and `style.get("line-height")` yields the resolved value if present. Style resolution already normalizes `line-height` (see how other tasks read e.g. `style.get("white-space")`, `style.get("text-transform")` in this same file — copy that access pattern exactly).
- The resolved `line-height` CssValue may be a `CssValue::Number(n)` (unitless multiplier of font-size — already resolved upstream, but verify), a `CssValue::Length{ value, unit }` (px etc.), or absent. Inspect the `CssValue` enum (crate::css::values::CssValue) and the existing `style.get(...)` consumers in THIS file to learn the exact variants and how lengths are read. Do NOT invent new parsing — read the already-resolved value.

Goal: when a node has a CSS `line-height`, the line box that lays out that node's text uses THAT height instead of the font default. Absent → keep `font.line_height()` as today (no regression).

Implement (ALL in src/layout/inline.rs):
1. At the point where `current_line_height` / per-node line height is determined for a text node (near where `style.get("white-space")` is read), also read `style.get("line-height")`.
2. If present and resolvable to a concrete pixel height, use it for that node's contribution to `current_line_height` (take the MAX of the font default and the CSS value if mixing is simplest, OR replace — match CSS: an explicit `line-height` SETS the used value; but never shrink below the font's ascent+descent if that would clip — keep it simple and correct: if CSS line-height present use it, else font default). Resolve `CssValue::Length` px directly; resolve `CssValue::Number(n)` as `n * font_size` ONLY if upstream did not already resolve it (check: if style resolution already converts number→px, just read it). When in doubt about whether numbers are pre-resolved, leave a `// TODO(spec): confirm line-height number resolution locus` and use the value as-is if it already looks like px.
3. Do NOT change vertical-align handling, caret math, or fragment x-advance. ONLY the line-box height source.

Add/extend unit tests in the existing `#[cfg(test)] mod tests` block in src/layout/inline.rs (reuse the existing test harness/helpers in this file to build styled inline content — do NOT invent a new layout entry point; copy an existing inline-layout test and adapt):
- `test_line_height_px_sets_line_box`: a single line of text on a node with `line-height: 40px` (font default smaller) produces a line box of height 40 (or block content height == 40 for one line). Assert the line/box height equals 40.0.
- `test_line_height_absent_uses_font_default`: same content with NO line-height yields the prior font-default height (regression guard — assert it equals `font.line_height()`-derived value, i.e. unchanged from before).
If a `Number` (unitless) multiplier is straightforward given how style resolves it, also add `test_line_height_number_multiplier` with `line-height: 2` over a known font-size; otherwise leave a `// TODO(spec):` and skip that third test.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(layout): honor CSS line-height for inline line-box height (t0381)"
Then print "T0381 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
