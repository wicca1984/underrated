#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0335
# Auth: prefer canonical var/.env, fall back to bashrc export.
if grep -q '^export GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env 2>/dev/null; then
  eval "$(grep -m1 '^export GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env)"
elif grep -q '^GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env 2>/dev/null; then
  export "$(grep -m1 '^GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env)"
else
  eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
fi
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7). One task = one module/concern.

Task: t0335 (milestone MS-NewTargets, CSS \`list-style\` shorthand expansion). This is a STYLE-MODULE-ONLY task. Real Wiki/News pages frequently write the \`list-style\` shorthand (e.g. \`ul { list-style: square }\`, \`list-style: circle inside\`, \`list-style: none\`) instead of the longhands. Today the shorthand is registered as an inherited property in \`src/style/mod.rs\` (in the \`is_inherited\`-style match list around line 1409: \`\"list-style\"\`) but it is NEVER decomposed into its longhands. So a declared \`list-style: square\` never sets \`list-style-type: square\`, and the layout stage (which reads ONLY the longhand \`list-style-type\` at \`src/layout/mod.rs:431\`) silently falls back to the default disc marker. We must expand the shorthand.

CONFIRMED CONTEXT (verified by orchestrator):
- In \`src/style/mod.rs\`, function \`apply_declarations\` has a \`for matched in matched_declarations { ... }\` loop. \`font\` is special-cased first (\`expand_font_shorthand\`, line ~185), then a \`match name { ... }\` block handles \`margin\`, \`padding\`, \`border-width\`, \`border-color\`, \`border-style\`, \`border\` (line ~239), \`background\` (line ~307), etc. — each arm inserts the expanded longhands into \`properties\`. Add a NEW arm for \`list-style\` in EXACTLY this style.
- Longhands to set: \`list-style-type\`, \`list-style-position\`, \`list-style-image\`. All three already exist as recognized inherited properties (same match list at ~1407-1409).
- Layout consumes \`list-style-type\` only (values it acts on: \`none\`, \`disc\`, \`circle\`, \`square\`, plus \`lower-alpha\`/\`upper-roman\` per existing tests). \`list-style-position\`/\`list-style-image\` are not yet consumed by layout — that is fine; still set them for correctness/cascade.

WHAT TO DO (ONE MODULE — src/style/ ONLY):
1. Add a \`\"list-style\" => { ... }\` arm to the \`match name\` block in \`apply_declarations\` (place it near the other shorthand arms, e.g. after \`background\`). Parse the shorthand's component values and classify each token per CSS \`list-style\` grammar (https://www.w3.org/TR/css-lists-3/#propdef-list-style): order-independent, up to three components —
   - \`list-style-type\`: one of the keywords \`disc | circle | square | decimal | lower-alpha | upper-alpha | lower-roman | upper-roman | none\` (accept the full set the existing longhand path already understands; at minimum the four bullet/none keywords plus the alpha/roman ones layout tests use).
   - \`list-style-position\`: \`inside | outside\`.
   - \`list-style-image\`: \`none\` or a \`url(...)\` value. (\`none\` is ambiguous with type \`none\`: per spec a single \`none\` sets BOTH type and image to none — handle this: if exactly one \`none\` appears and no other type/image token, set list-style-type:none AND list-style-image:none.)
   - Any component the shorthand OMITS must be reset to its initial value (type=disc, position=outside, image=none) — shorthands reset omitted longhands. Insert all three longhands unconditionally so a later \`list-style: circle\` fully overrides an earlier \`list-style: square inside url(x)\`.
2. Keep \`list-style\` ALSO present? No — once expanded, do NOT also insert the raw \`list-style\` property (mirror how \`margin\`/\`padding\` arms work: they insert only longhands). (The \`border-color\`/\`border-style\` arms additionally keep the shorthand for a paint bevel heuristic — list-style has no such consumer, so insert longhands ONLY.) Use \`continue\`-equivalent flow consistent with the surrounding arms so the shorthand itself is not stored.
3. Implementation notes: reuse existing helpers where possible. To read tokens, inspect \`matched.declaration.value\` (the same \`&[ComponentValue]\` the other arms parse) — look at how \`expand_font_shorthand\` and the \`background\` arm walk component values to extract keyword idents and \`url()\`. Do NOT add a new DisplayItem, do NOT touch parser/css/layout/paint. Build the three longhand \`CssValue\`s the same way the longhand path would (parse via the same \`parse_value\`/ident path the surrounding code uses).

SCOPE — STRICT, ONE MODULE. \`git diff --name-only origin/main...HEAD\` MUST list ONLY \`src/style/mod.rs\` (and any \`var/\` html/png). Do NOT touch layout/, paint/, css/, parser, engine/, or any other module. Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere. Do NOT add dependencies. If you think you must do any of these, STOP, leave a \`// TODO(spec):\` note, and report the blocker.

REQUIRED NEW TESTS (in \`src/style/mod.rs\` \`#[cfg(test)] mod tests\`, mirror existing style shorthand tests like the border/background expansion tests):
- \`test_list_style_shorthand_expands_type\`: apply a rule with \`list-style: square\` to a \`<ul>\` and assert the computed \`list-style-type\` == \`square\` AND \`list-style-position\` == \`outside\` (initial) AND \`list-style-image\` == \`none\` (initial).
- \`test_list_style_shorthand_type_and_position\`: \`list-style: circle inside\` -> type==circle, position==inside, image==none.
- \`test_list_style_shorthand_single_none_resets_type_and_image\`: \`list-style: none\` -> list-style-type==none AND list-style-image==none.
- \`test_list_style_shorthand_overrides_previous\`: an earlier \`list-style: square inside\` followed by a later \`list-style: circle\` yields type==circle AND position==outside (omitted position reset to initial).
Each test MUST fail before your change and pass after. Use the SAME test harness/assertion style as the existing shorthand tests in this file (find one and copy its setup precisely).

GATES (all must pass before commit):
- \`cargo test\` (entire suite) green — including your 4 new tests and ALL existing tests (esp. the layout list-style-type tests).
- \`cargo clippy --all-targets -- -D warnings\` clean (NO \`unwrap\`/\`expect\` in non-test production code — I-6).
- \`cargo fmt\` then \`cargo fmt --check\` clean.

VERIFIED-IN-WINDOW (REQUIRED — this changes what markers render): after gates pass, create \`var/t0335-list-style.html\` (\`mkdir -p var\` first) containing exactly:
\`<html><head><style>ul.a{list-style:disc}ul.b{list-style:circle}ul.c{list-style:square}</style></head><body><ul class=\\\"a\\\"><li>disc</li></ul><ul class=\\\"b\\\"><li>circle</li></ul><ul class=\\\"c\\\"><li>square</li></ul></body></html>\`
then render the shipping-path PNG and SAVE it:
\`cargo run --example render_local_png -- /workspaces/wt/t0335/var/t0335-list-style.html --width 240 --height 160 --out /workspaces/wt/t0335/var/t0335-list-style.png\`
The PNG MUST show three list items each with a visible marker driven by the SHORTHAND (disc/circle/square) — none should fall back to identical default disc. Report the saved PNG path in your summary. (The orchestrator will independently re-verify before merge.)

COMMIT: after ALL gates pass AND the PNG is saved, \`git add -A\` then \`git commit -m \\\"feat(style): expand list-style shorthand to type/position/image longhands (t0335)\\\"\`. Then run \`git diff --name-only origin/main...HEAD\` and confirm ONLY \`src/style/mod.rs\` (plus var/ html+png) is listed. Do NOT push and do NOT open a PR — the orchestrator reviews, gates, and merges. Report a concise summary: the one-module change, the 4 new tests, gate results, and the saved PNG path." -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
