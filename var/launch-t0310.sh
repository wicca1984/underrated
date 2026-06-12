#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0310
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0310 — implement CSS \`word-break: break-all\` in inline layout so that an over-long single word (e.g. a long unbroken string) is broken at character boundaries to fit the line, instead of overflowing.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Primary module: src/layout/inline.rs. You may ALSO add exactly ONE line to src/style/mod.rs (the inheritance allowlist) as described below — that is the ONLY allowed edit outside src/layout/inline.rs. Do NOT touch src/css, src/paint, src/dom, src/script, lib.rs, or any other file.

SPEC (CSS Text): \`word-break: break-all\` (inherited property). Default is \`normal\` (break only at allowed break opportunities / spaces — current behavior). With \`break-all\`, a word that does not fit on the current line may be broken between any two characters (per grapheme/char for v1) so that as many characters as fit are placed, then the remainder wraps to the next line. Keep breaking until the whole word is placed. This applies regardless of whether \`white-space\` allows wrapping is already true; \`break-all\` only takes effect when wrapping is allowed (i.e. not \`white-space: nowrap/pre\`). Out of scope (leave as-is): \`keep-all\`, \`break-word\`, \`overflow-wrap\`, hyphenation, CJK-specific rules, bidi.

WHERE / TEMPLATE (verify by reading the code yourself before coding):
  - src/layout/inline.rs reads the text node's white-space at ~line 160-175 via \`styles.get(&node)\` -> \`style.get(\"white-space\")\`, producing \`(collapse, preserve_newlines, allow_wrap)\`.
  - The per-word wrap loop is at ~line 211-270: it iterates \`segment.split_inclusive(' ')\`, measures each word with \`font.measure(word) as f32\`, and when \`allow_wrap && cursor_x + word_width > containing_width && cursor_x > 0.0\` it flushes the current line. Then it pushes a \`LayoutBox\` for the word and advances \`cursor_x\`.
  - src/style/mod.rs has an inheritance allowlist function (the \`matches!(property, ...)\` around lines 1279-1300 listing \"white-space\", \"word-spacing\", etc). \`word-break\` is an inherited property and MUST be added there or the cascade will not propagate it to text nodes.

SCOPE:
  1. In src/style/mod.rs: add \`| \"word-break\"\` to the inherited-property \`matches!\` list (alphabetically near \"word-spacing\"). This is the ONLY edit outside inline.rs. Do not change anything else in that file.
  2. In src/layout/inline.rs: read the text node's \`word-break\` value the same way white-space is read (\`style.get(\"word-break\")\`); treat keyword \`break-all\` as enabling character breaking, anything else (including absent) as \`normal\`. Bind it as e.g. \`let break_all: bool = ...;\` next to where \`(collapse, preserve_newlines, allow_wrap)\` is computed.
  3. In the per-word loop: when \`allow_wrap && break_all\` and a single word is wider than the available space, break the word at character boundaries. Concretely: if the word does not fit, greedily place as many leading chars as fit in the remaining \`containing_width - cursor_x\` (using \`font.measure\` on the growing prefix; ensure at least 1 char per line to avoid infinite loops when even one char overflows), push that prefix as its own LayoutBox, flush the line, reset \`cursor_x = 0.0\`, advance \`cursor_y += current_line_height\`, and continue with the remainder of the word until it is fully placed. Preserve existing behavior for \`break_all == false\` (do not change the normal-word path).
     - Use char boundaries (iterate over \`word.chars()\` / \`char_indices()\`), never split inside a UTF-8 codepoint.
     - Do NOT use unwrap/expect in non-test code (I-6).
  4. Keep the diff localized and minimal. Do not refactor unrelated code, do not change function signatures, do not alter the normal/nowrap/pre paths' semantics.

VALIDATION before committing:
  - \`cargo fmt --check\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo test\` — all green.
  - Add ONE focused unit test in src/layout/inline.rs (mirror \`test_white_space_nowrap\` at ~line 715 for the harness/setup style). It must prove that a single long unbroken word under \`word-break: break-all\` in a narrow container is split across MULTIPLE line boxes (assert that the resulting line boxes / fragments span more than one line, i.e. more than one distinct y / line box), AND a regression check that the SAME long word under default \`word-break: normal\` stays on a single line (overflowing). Assert precisely; do not weaken assertions to force green. Do not delete or weaken any other test.

When done: \`git add -A && git commit -m 'feat(layout): implement CSS word-break: break-all (t0310)'\`. Commit BEFORE finishing. Do not push or open a PR; the orchestrator handles that." \
  -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta \
  >> /workspaces/underrated-meta/var/worker-logs/t0310.log 2>&1
