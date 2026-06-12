#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0276
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0276 — Apply the CSS `text-indent` property to indent the FIRST line of a block container by the resolved length. This is a deliberately small, single-module, additive layout feature. `text-indent` is already registered as an inherited property in the style system (src/style/mod.rs line ~1295) but is currently NOT applied anywhere in layout, so it has no visible effect today.

Scope: touch ONLY files under `src/layout/` (specifically `src/layout/mod.rs` and `src/layout/inline.rs`, including their inline `#[cfg(test)] mod tests`). `git diff --name-only` MUST show ONLY files under src/layout/. Do NOT modify style, css/values, paint, font, or any other module/worktree. No style-module change is needed: `text-indent` flows through the generic property store and you read it with the existing `get_px(...)` helper, exactly like `width`/`margin-*` are read today.

Reuse / facts (verified — read these before writing):
- `get_px(style, prop, default)` is at `src/layout/mod.rs` line 630: `pub(crate) fn get_px(style: &ComputedStyle, prop: &str, default: f32) -> f32`. It resolves a `CssValue::Length` property to px (the SAME helper used for width/height/margins). Use `get_px(style, "text-indent", 0.0)` to read the indent of the block whose inline content you are laying out.
- The block layout code in `src/layout/mod.rs` already extracts `let text_align = get_text_align(dom, node, style);` at line 194, then passes `text_align` into BOTH inline entry points: `layout_inline(...)` (the all-inline-children branch, call ~line 207) and `layout_inline_run(...)` (the mixed/anonymous-run branch, call ~line 297). The block `style: &ComputedStyle` is in scope at both call sites.
- In `src/layout/inline.rs`, `layout_inline` (line 360) is a thin wrapper that just delegates to `layout_inline_run` (line 107). `layout_inline_run` initializes `let mut cursor_x = 0.0;` (line 122) and, on every line break, resets `cursor_x = 0.0;` (lines ~187, ~218, ~291). The painted glyph/inline-box start x is derived from `cursor_x`, so shifting the initial `cursor_x` shifts where the first line begins — the indent renders through the existing paint path with no paint change.

Implement (CSS text-indent semantics — FIRST line only):
1. In `src/layout/mod.rs`, right after `let text_align = get_text_align(dom, node, style);` (line 194), add `let text_indent = get_px(style, "text-indent", 0.0);`.
2. Thread a new `text_indent: f32` parameter into `layout_inline` and `layout_inline_run` (add it to BOTH signatures in src/layout/inline.rs, placed AFTER `text_align: &str` to minimize churn), have `layout_inline` forward it to `layout_inline_run`, and pass `text_indent` at BOTH call sites in src/layout/mod.rs.
3. In `layout_inline_run`, change the initial `let mut cursor_x = 0.0;` to `let mut cursor_x = text_indent;` so ONLY the first line is indented. Do NOT touch the existing `cursor_x = 0.0;` line-break resets — subsequent lines must start at 0 (text-indent applies to the first line only).
4. Update EVERY existing call site of `layout_inline_run` in the in-file unit tests (there are several near lines 519..835 calling it with positional args ending in `, "left")`) to pass the new param as `0.0` (e.g. `, "left", 0.0)`), so existing tests keep compiling and asserting unchanged behavior. Do NOT weaken or remove any existing test.
5. Edge cases / spec scope (keep v1 tight): apply text-indent as a simple shift of the first lines starting cursor in normal left-to-right flow. Do NOT attempt to special-case its interaction with `text-align: center/right/justify` or RTL — leave exactly one `// TODO(spec): text-indent interaction with text-align (center/right/justify) and RTL, and percentage text-indent resolution, are out of scope; only length values shift the first-line start.` near your change. Percentages: if `text-indent` is given as a percentage and the existing length machinery does not resolve it, `get_px` returns the default (0.0) — that non-application is acceptable and is covered by the same TODO; do NOT invent percentage resolution.

Do NOT use unwrap/expect/panic/unsafe in non-test code (I-6). No new dependencies (I-1). Do NOT change iterative code back to recursive. Keep public interfaces stable except for the additive `text_indent` parameter described above.

Acceptance — add inline unit tests in `src/layout/inline.rs` (reuse the existing harness in that file: tests build a DOM + styles and call `layout_inline_run(&dom, &styles, children, <width>, 0.0, 0.0, 0, "left", <indent>)`; mirror an existing test such as the ones around lines 519..835 to construct the DOM and assert on the first fragment/line-box x or cursor_x):
- With a positive `text_indent` (e.g. 40.0), the FIRST inline fragment/line-box on the first line starts at x = offset_x + 40.0 (i.e. shifted right by the indent) versus the same content with indent 0.0.
- A SECOND line produced by wrapping starts at the un-indented offset_x (indent applies to first line only) — construct content wide enough to wrap and assert the second line box x is NOT shifted.
- A regression guard: `text_indent = 0.0` reproduces the exact current behavior of an existing test (first line starts at offset_x).
Do NOT weaken or remove any existing test.

Done when ALL of these pass in this worktree:
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Comments and identifiers in English.
Commit (you MUST commit before finishing, BEFORE the worktree can be removed): `git add -A && git commit -m "feat(layout): indent the first line of a block by text-indent (t0276)"`.
End with a short English summary of exactly what changed in src/layout/mod.rs and src/layout/inline.rs and the `// TODO(spec):` you left, and confirm you committed.'
