#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0277
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0277 — Apply the CSS `word-spacing` property to add extra horizontal advance after each inter-word space in inline text layout. This is a deliberately small, single-module, additive layout feature, closely modeled on the just-merged `text-indent` work (t0276). `word-spacing` is already registered AND fully resolved to a px length in the style system (src/style/mod.rs, section G near line 446, with em resolution), but it is currently NOT applied anywhere in layout, so it has no visible effect today.

Scope: touch ONLY files under `src/layout/` (specifically `src/layout/mod.rs` and `src/layout/inline.rs`, including their inline `#[cfg(test)] mod tests`). `git diff --name-only` MUST show ONLY files under src/layout/. Do NOT modify style, css/values, paint, font, or any other module/worktree. No style-module change is needed: `word-spacing` flows through the generic property store and you read it with the existing `get_px(...)` helper, exactly like `text-indent` is read today.

Reuse / facts (verified — read these before writing):
- `get_px(style, prop, default)` (src/layout/mod.rs near line 630) resolves a `CssValue::Length` property to px. The block layout code reads `let text_indent = get_px(style, "text-indent", 0.0);` at src/layout/mod.rs line 195. Read word-spacing the SAME way: `let word_spacing = get_px(style, "word-spacing", 0.0);`.
- `text_indent` is already threaded as the LAST positional parameter into BOTH inline entry points: `layout_inline(...)` (call site at src/layout/mod.rs line 214) and `layout_inline_run(...)` (call site at src/layout/mod.rs line 299). Add `word_spacing` as a new LAST parameter right AFTER `text_indent`, mirroring exactly how `text_indent` is threaded.
- In `src/layout/inline.rs`, `layout_inline` (line 362) is a thin wrapper delegating to `layout_inline_run` (line 107). The text branch splits a segment into words via `let words = segment.split_inclusive(" ");` (line 194), so each `word` INCLUDES its own trailing space when present (e.g. "Hello "). Each word becomes a fragment of width `word_width = font.measure(word)` and the cursor advances with `cursor_x += word_width;` at line 246.

Implement (CSS word-spacing semantics — extra advance per inter-word space, layout-only):
1. In `src/layout/mod.rs`, right after `let text_indent = get_px(style, "text-indent", 0.0);` (line 195), add `let word_spacing = get_px(style, "word-spacing", 0.0);`.
2. Thread a new `word_spacing: f32` parameter into `layout_inline` and `layout_inline_run` (add it to BOTH signatures in src/layout/inline.rs as the LAST parameter, AFTER `text_indent: f32`), have `layout_inline` forward it to `layout_inline_run`, and pass `word_spacing` at BOTH call sites in src/layout/mod.rs (right after the `text_indent` argument).
3. In `layout_inline_run`, immediately after `cursor_x += word_width;` (line 246), add the extra inter-word advance ONLY when the word carries a trailing space: `if word.ends_with(" ") { cursor_x += word_spacing; }`. This shifts subsequent words right by the configured amount; the painted word fragment keeps `width = word_width` (the visible text is unchanged — word-spacing only widens the gap between words). Because subsequent words read the advanced `cursor_x`, line-break decisions naturally account for the added spacing. Do NOT alter the existing leading-whitespace skip (the `if collapse && cursor_x == 0.0 && word == " "` branch) or any line-break reset.
4. Update EVERY existing call site of `layout_inline_run` in the in-file unit tests (there are several near lines 519..931 currently ending in `, 0.0)` or `, 40.0)` for the text_indent arg) to ALSO pass the new `word_spacing` param as `0.0` (append `, 0.0` so they end e.g. `, "left", 0.0, 0.0)`), so existing tests keep compiling and asserting unchanged behavior. Do NOT weaken or remove any existing test.
5. Edge cases / spec scope (keep v1 tight): apply word-spacing as a simple per-trailing-space cursor advance in normal left-to-right flow. Do NOT attempt justification interaction, percentage word-spacing, or per-Unicode-space-separator nuance. Leave exactly one `// TODO(spec): word-spacing v1 adds a fixed advance after each word that carries a trailing space; interaction with text-align justify, percentage values, and full Unicode space-separator handling are out of scope.` near your change in `layout_inline_run`.

Do NOT use unwrap/expect/panic/unsafe in non-test code (I-6). No new dependencies (I-1). Do NOT change iterative code back to recursive. Keep public interfaces stable except for the additive `word_spacing` parameter described above.

Acceptance — add inline unit tests in `src/layout/inline.rs` (reuse the existing harness; tests build a DOM + styles and call `layout_inline_run(&dom, &styles, children, <width>, <ox>, <oy>, 0, "left", <indent>, <word_spacing>)`; mirror an existing multi-word test such as the ones around lines 519..931):
- With a positive `word_spacing` (e.g. 10.0) on multi-word text on a single line, the SECOND word fragment starts further right than with `word_spacing = 0.0` — assert the second fragment x for word_spacing=10.0 is exactly 10.0 greater than the second fragment x for word_spacing=0.0 (same content/width/offset otherwise).
- A single word with no trailing space produces the SAME layout for word_spacing=0.0 and word_spacing=10.0 (no spurious advance when there is no inter-word space).
- A regression guard: `word_spacing = 0.0` reproduces the exact current behavior of an existing multi-word test.
Do NOT weaken or remove any existing test.

Done when ALL of these pass in this worktree:
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Comments and identifiers in English.
Commit (you MUST commit before finishing, BEFORE the worktree can be removed): `git add -A && git commit -m "feat(layout): apply word-spacing as inter-word advance in inline layout (t0277)"`.
End with a short English summary of exactly what changed in src/layout/mod.rs and src/layout/inline.rs and the `// TODO(spec):` you left, and confirm you committed.'
