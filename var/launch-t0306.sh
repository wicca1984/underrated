#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0306
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0306 — make the CSS value parser RETAIN the slash (\`/\`) token inside a multi-token declaration value, so that \`aspect-ratio: <w> / <h>\` actually reaches layout through the SHIPPING parse path.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/css/ (touch ONLY src/css/values.rs; do NOT touch src/layout, src/style, lib.rs, or any other module/worktree).

WHY (root cause — verify it yourself before coding):
  - Layout ALREADY implements aspect-ratio sizing (in src/layout/mod.rs on another branch) and reads it via
    \`style.get('aspect-ratio')\`, handling three shapes: CssValue::Number, CssValue::Keyword, and
    CssValue::Multiple([Number, Keyword(\"/\"), Number]).
  - BUT the shipping CSS parse path DROPS the whole declaration for the \`<w> / <h>\` form. In src/css/values.rs,
    \`parse_value\` splits the component list on whitespace into single-token groups and calls \`parse_single_value\`
    on each; for the lone \`/\` token (\`ComponentValue::Token(CssToken::Delim('/'))\`) \`parse_single_value\` hits its
    \`_ => None\` arm, which makes \`parse_value\` return None, so the ENTIRE declaration is discarded and never stored
    in ComputedStyle. Confirm this by reading \`parse_value\` and \`parse_single_value\` in src/css/values.rs.
  - The bare-number form (\`aspect-ratio: 4\`) already works; only the slash form is dead. This task fixes the slash.

SCOPE (in src/css/values.rs ONLY):
  1. In \`parse_single_value\`, make a lone \`CssToken::Delim('/')\` parse to \`Some(CssValue::Keyword(\"/\".to_string()))\`
     instead of falling through to \`_ => None\`. This is the SMALLEST change that lets \`parse_value\` keep the slash:
     \`16 / 9\` then becomes \`CssValue::Multiple([Number(16.0), Keyword(\"/\"), Number(9.0)])\`, which the existing layout
     code already understands. Do NOT add aspect-ratio-specific logic here — keep the parser generic.
  2. Do NOT change \`parse_value\`'s grouping logic, the CssValue enum, or any other property. Keep the diff tiny.

REGRESSION GUARD (critical — this token is shared by many properties):
  - Run the FULL existing suite: \`cargo test\` must stay 100% green. The change turns previously-DROPPED declarations
    that contain \`/\` (e.g. font shorthand \`16px/1.5\`) into a stored \`Multiple\` value. If any existing test breaks,
    STOP and report it rather than deleting/altering foreign tests (forbidden by I-4).
  - Add ONE focused unit test in src/css/values.rs proving \`parse_value\` on the tokens of \`16 / 9\` returns
    \`CssValue::Multiple\` containing the two numbers and a \`Keyword(\"/\")\` (assert the structure precisely).

VALIDATION before committing:
  - \`cargo fmt --check\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo test\` — all green.
  - No \`unwrap\`/\`expect\` in non-test code (I-6).

COMMIT (you MUST commit before finishing — uncommitted work is lost):
  - \`git add -A && git commit\` on branch \`agent/t0306-aspect-ratio-slash-parse\` with message:
    \`feat(css): retain slash token in multi-value parse so aspect-ratio w/h reaches layout (t0306)\`
  - Confirm with \`git -C /workspaces/wt/t0306 diff --name-only origin/main...HEAD\` that ONLY src/css/values.rs changed.
" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta 2>&1 | tee /workspaces/underrated-meta/var/worker-logs/t0306.jsonl
