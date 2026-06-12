#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0217
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0217 — HTML Presentational Hints: map presentational HTML attributes to CSS properties (focus: <img width/height>).
Why: a high-res logo (Google) currently renders at full intrinsic size because the HTML width/height attributes are ignored; they must become computed CSS width/height.

Target module: src/style/ (touch ONLY src/style/mod.rs plus its tests; do NOT edit other modules such as src/engine, src/layout, src/paint, src/css, src/dom — read them as needed but do not modify).

Approach (test-first / TDD):
1. Add a function `collect_presentational_hints(dom, node, &mut matched_declarations)` invoked inside `compute_node_style()` in src/style/mod.rs, AFTER `collect_matched_rules()` and BEFORE the cascade sort.
2. Precedence: presentational hints must sit ABOVE the UA stylesheet but BELOW author CSS and inline styles. Use specificity (0,0,0,0) (same as UA element selectors) but give them a source_order that places them after UA rules and before author rules. Inspect how MatchedDeclaration { declaration, specificity, source_order } is sorted and pick values so author CSS and inline style always win. Add a test proving an author `img { width: 10px }` rule overrides a `width="200"` attribute.
3. Map these attributes -> CSS (only when the corresponding CSS property is not otherwise set by the hint path):
   - <img>, <table>, <td>, <th>, <col>, <colgroup>: `width="N"` -> width:Npx, `height="N"` -> height:Npx. A bare integer means px; a value ending in `%` means percent. Ignore malformed values (leave a `// TODO(spec):` only if genuinely ambiguous).
   Keep scope to width/height for this task; do not implement align/bgcolor/border etc.
4. Use the existing DOM attribute accessor (e.g. dom.get_attribute(node, "width")) and the existing CssValue::Length / LengthUnit and Declaration/CssValue types — do not invent new value types.

Acceptance (must all be green):
  - cargo test (add unit tests in src/style/mod.rs: (a) img width="200" -> computed width is 200px; (b) percentage attr maps to percent length; (c) author CSS rule beats the attribute; (d) inline style beats the attribute).
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Done when all three pass. No unwrap/expect in non-test code (I-6). No test skip/ignore (I-4).
Commit on this branch with: `feat(style): map img/table presentational width/height attributes to CSS (t0217)`. Comments and identifiers in English.
IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary of what changed and the test names you added.
If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
