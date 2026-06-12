#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0336
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

Task: t0336 (milestone MS-MVP-LAYOUT support, hidden form controls). This is an ENGINE-UA-STYLESHEET-ONLY task. When the orchestrator ran the REAL Google home page (~83KB) through the shipping snapshot path (\`underrated::oracle::export_snapshot\`), several \`<input type=\\\"hidden\\\">\` elements were each laid out as a small visible ~6px box (x=132,138,144,150,...; width=6). They SHOULD NOT render at all. Root cause: \`UA_DEFAULT_CSS\` in \`src/engine/mod.rs\` has a generic \`input { display: inline-block; padding: 1px 2px; border: 1px solid ... }\` rule but NO rule giving \`input[type=\\\"hidden\\\"]\` (nor the global HTML \`hidden\` attribute) \`display: none\`. So hidden inputs inherit the visible inline-block box. Per the HTML standard UA stylesheet, \`input[type=hidden i] { display: none !important; }\` and \`[hidden] { display: none; }\`.

CONFIRMED CONTEXT (verified by orchestrator):
- \`UA_DEFAULT_CSS\` is a \`pub const &str\` near the top of \`src/engine/mod.rs\` (starts at line ~19). It already uses attribute selectors that the engine's selector matcher SUPPORTS, e.g. \`input[type=\\\"submit\\\"]\`, \`input[type=\\\"text\\\"]\`, etc. So a \`input[type=\\\"hidden\\\"]\` selector is GUARANTEED to match the same way.
- \`display: none\` is already honored end-to-end: the const includes \`head, style, script, meta, link, title { display: none; }\` and there is a passing fixture test \`05_display_none\`. So setting \`display:none\` on hidden inputs WILL exclude them from layout/snapshot.

WHAT TO DO (ONE MODULE — src/engine/mod.rs ONLY):
1. In \`UA_DEFAULT_CSS\`, ADD a rule \`input[type=\\\"hidden\\\"] { display: none; }\`. Place it AFTER the existing generic \`input { ... }\` block and AFTER the \`input[type=\\\"text\\\"], ...\` block so cascade order makes \`display:none\` win over the generic \`input { display: inline-block }\` (later rule, equal specificity beats earlier; but \`input[type=hidden]\` is also MORE specific than bare \`input\`, so it wins regardless). Follow the EXACT existing string-literal style (each physical line ends with \`\\n\\\` inside the Rust string).
2. ADDITIONALLY add a rule for the global HTML \`hidden\` attribute: \`[hidden] { display: none; }\`. BUT FIRST VERIFY the selector engine supports a BARE attribute selector (no tag). Write a quick unit test (below) that asserts a \`<div hidden>\` gets \`display:none\`. If the matcher does NOT support bare \`[hidden]\` selectors (test fails and you confirm via the selector-parsing code path that bare attribute selectors are unsupported), DO NOT force it: remove the \`[hidden]\` rule, leave a \`// TODO(spec): bare [hidden] attribute selector needs selector-engine support\` comment in the const, and keep ONLY the \`input[type=\\\"hidden\\\"]\` rule. The \`input[type=\\\"hidden\\\"]\` rule is the REQUIRED deliverable; \`[hidden]\` is best-effort.
3. Do NOT touch any other module (no style/, layout/, paint/, css/, oracle/). Do NOT add a DisplayItem. Do NOT add dependencies.

SCOPE — STRICT, ONE MODULE. \`git diff --name-only origin/main...HEAD\` MUST list ONLY \`src/engine/mod.rs\` (plus any \`var/\` html/png you create). Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere. If you think you must touch another file, STOP, leave a \`// TODO(spec):\` note, and report the blocker.

REQUIRED NEW TESTS (in \`src/engine/mod.rs\` \`#[cfg(test)] mod tests\`; if none exists, create one — mirror how other modules set up \`#[cfg(test)] mod tests\`). Drive them through the public snapshot API \`crate::oracle::export_snapshot(html, \\\"\\\", 800, 600)\` and walk the returned \`serde_json::Value\` tree (type/tag/children), the SAME shape \`tests/oracle_snapshot_test.rs\` uses (read that file for the exact JSON field names: \`type\`==\\\"element\\\", \`tag\`, \`attrs\`, \`children\`, \`rect\`):
- \`test_hidden_input_not_rendered\`: snapshot of \`<html><body><input type=\\\"hidden\\\" name=\\\"x\\\"><p>visible</p></body></html>\` MUST contain the \`<p>\` element but MUST NOT contain any \`<input>\` element in the tree (display:none drops it). Assert by recursively searching for an element with tag==\\\"input\\\" and asserting it is absent.
- \`test_text_input_still_rendered\`: snapshot of \`<html><body><input type=\\\"text\\\"></body></html>\` MUST still contain the \`<input>\` element (regression guard — we only hide hidden inputs).
- \`test_hidden_attribute_div\` (for the \`[hidden]\` rule): snapshot of \`<html><body><div hidden>gone</div><div>here</div></body></html>\` — if bare \`[hidden]\` is supported, assert exactly one \`<div>\` remains; if you determined it is unsupported and removed the rule, instead assert BOTH divs are present and add a \`// TODO(spec):\` note in the test explaining bare-attribute-selector limitation. (Pick whichever matches what you actually shipped — the suite must be green.)
Each shipped test MUST be consistent with the final behavior and pass.

GATES (all must pass before commit):
- \`cargo test\` (entire suite) green — including your new tests and ALL existing tests (esp. \`oracle_snapshot_test\` fixtures).
- \`cargo clippy --all-targets -- -D warnings\` clean (NO \`unwrap\`/\`expect\` in non-test production code — I-6).
- \`cargo fmt\` then \`cargo fmt --check\` clean.

VERIFIED-IN-WINDOW (REQUIRED — this changes what renders): after gates pass, create \`var/t0336-hidden-input.html\` (\`mkdir -p var\` first) containing exactly:
\`<html><body><input type=\\\"hidden\\\" name=\\\"a\\\"><input type=\\\"hidden\\\" name=\\\"b\\\"><input type=\\\"text\\\" value=\\\"visible box\\\"></body></html>\`
then render the shipping-path PNG and SAVE it:
\`cargo run --example render_local_png -- /workspaces/wt/t0336/var/t0336-hidden-input.html --width 320 --height 120 --out /workspaces/wt/t0336/var/t0336-hidden-input.png\`
The PNG MUST show ONLY the single text input box (one bordered box) and NO small empty boxes from the two hidden inputs. Report the saved PNG path in your summary. (The orchestrator will independently re-verify before merge.)

COMMIT: after ALL gates pass AND the PNG is saved, \`git add -A\` then \`git commit -m \\\"fix(engine): hide input[type=hidden] (and [hidden]) via UA display:none (t0336)\\\"\`. Then run \`git diff --name-only origin/main...HEAD\` and confirm ONLY \`src/engine/mod.rs\` (plus var/ html+png) is listed. Do NOT push and do NOT open a PR — the orchestrator reviews, gates, and merges. Report a concise summary: the one-module change, whether \`[hidden]\` was shippable, the new tests, gate results, and the saved PNG path." -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
