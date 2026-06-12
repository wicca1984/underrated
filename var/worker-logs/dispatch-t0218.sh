#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0218
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0218 — Form element UA default styling: make <button>, <input type="submit|button|reset">, and <input type="text|search|email|url|tel|password|number"> look like real buttons/fields (border, padding, background, display:inline-block) driven by the UA stylesheet.
Why: a submit button currently looks like plain inline text ("is that really a button?"). The UA default stylesheet has no rules for form controls.

Target: this is form-rendering. Touch ONLY src/engine/mod.rs (the `UA_DEFAULT_CSS` constant) and src/paint/mod.rs (reconcile the hardcoded button/input painting). Do NOT modify src/layout, src/css, src/style, src/forms, src/dom. Read them as needed.

Approach (test-first / TDD):
1. In src/engine/mod.rs, add UA_DEFAULT_CSS rules for form controls, e.g.:
   button, input[type="submit"], input[type="button"], input[type="reset"] { display: inline-block; padding: 1px 6px; border: 2px outset #c0c0c0; background-color: #e9e9e9; color: #000; }
   input[type="text"], input[type="search"], input[type="email"], input[type="url"], input[type="tel"], input[type="password"], input[type="number"] { display: inline-block; padding: 1px 2px; border: 2px inset #c0c0c0; background-color: #fff; }
   Verify the existing CSS parser + selector engine actually support attribute selectors like input[type="submit"]; if they do NOT, fall back to styling the bare `button` / `input` element selectors and leave a `// TODO(spec):` noting the attribute-selector gap (do not implement selector features — that is another module).
2. In src/paint/mod.rs, the button/submit/text-input boxes are currently painted with HARDCODED background/border colors (search for the button branch ~line 320-440 and text-input branch ~line 441-564, and the generic background-color/border painting ~line 566-635). Adjust so the box background and borders come from the computed style (UA CSS) and are NOT double-painted. Concretely: if the generic style-driven background/border painting now covers these boxes, remove the redundant hardcoded color fills so there is exactly ONE background rect and one set of border strips per control. Keep the button LABEL text rendering (centered label from <button> text or input value, default "Submit"). Do not regress text-input value rendering.

Acceptance (must all be green):
  - cargo test (add/adjust unit tests: (a) a <button> computed style has display:inline-block and a non-empty border-width and a background-color from UA CSS; (b) the paint display list for a submit input contains exactly one background SolidRect and four border strips, plus the centered label Text item — no duplicate fills).
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Done when all three pass. No unwrap/expect in non-test code (I-6). No test skip/ignore (I-4). Keep the diff limited to src/engine/mod.rs, src/paint/mod.rs and their tests (git diff --name-only must show only those).
Commit on this branch with: `feat(engine): add UA default CSS for form controls and reconcile paint (t0218)`. Comments and identifiers in English.
IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the test names you added.
If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
