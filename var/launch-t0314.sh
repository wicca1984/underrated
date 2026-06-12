#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0314
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
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7). One task = one module.

Task: t0314 — implement script-side capture of \`location\` assignment as a PENDING NAVIGATION request, inside the script module ONLY.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/script/mod.rs (touch ONLY this file; do NOT touch other modules, lib.rs, mod.rs of other dirs, src/engine, src/forms, or other worktrees).

CONTEXT: Currently \`window.location.href = url\`, \`location.assign(url)\`, and \`location.replace(url)\` are empty TODO stubs (search the JS bootstrap string for 'wire location assignment to navigation pipeline'). The real navigation driver lives in src/engine (engine::navigate) and is OUT OF SCOPE for this task. This task only makes the script host RECORD the requested navigation so a future engine-wiring task can drain it. Do NOT touch src/engine or src/forms.

SCOPE (in src/script/mod.rs ONLY):
  1. Add a thread_local capture slot next to the existing CURRENT_DOM/CURRENT_STYLES thread_local block (around line 96), e.g.:
       static PENDING_NAVIGATION: RefCell<Option<String>> = const { RefCell::new(None) };
     storing the most recently requested navigation URL string.
  2. Register a native host function (mirror how other native bridge functions are registered on the Boa context/global, e.g. the DOM bridge fns) named e.g. \`__request_navigation__(url)\` that writes the string into PENDING_NAVIGATION. Then change the JS location stub so that:
       set href(val) { __request_navigation__(String(val)); }
       assign(url)   { __request_navigation__(String(url)); }
       replace(url)  { __request_navigation__(String(url)); }
       reload()      { __request_navigation__(window.__document_location__.href); }
     Keep the existing getters and toString() byte-for-byte unchanged.
  3. Add a PUBLIC accessor method on BoaHost (the ScriptHost impl) so the engine can later drain it:
       pub fn take_pending_navigation(&mut self) -> Option<String>
     which returns and CLEARS the PENDING_NAVIGATION slot. Document it with a doc-comment that the engine will consume it post-eval to drive navigation. Do NOT add it to the ScriptHost trait (keep the trait minimal); make it an inherent method on BoaHost.
  4. Leave a \`// TODO(spec):\` noting the engine-side wiring (engine::navigate) is a follow-up task and that relative URLs are resolved engine-side, not here.

TESTS (in src/script/mod.rs, #[cfg(test)], mirror the existing test_location_initialized test style):
  - After \`host.eval(\"window.location.href = 'https://example.com/next'\")\`, \`host.take_pending_navigation()\` returns Some(\"https://example.com/next\").
  - A second call to take_pending_navigation() returns None (slot is cleared after taking).
  - \`location.assign('/foo')\` records Some(\"/foo\"); \`location.replace('/bar')\` records Some(\"/bar\").
  - With NO assignment, take_pending_navigation() returns None.
  - location getters still work (href/pathname unchanged) — do not regress existing test_location_initialized.

DELIVERABLE / DEFINITION OF DONE:
  - Run \`cargo fmt\`, \`cargo clippy --all-targets -- -D warnings\`, and \`cargo test\` — ALL must pass (green).
  - NO unwrap()/expect() in non-test code (I-6). NO skipped/ignored tests (I-4). Do NOT delete or weaken any existing test.
  - git add -A and COMMIT on this branch with message:
      feat(script): capture location assignment as pending navigation (t0314)
  - Print the final \`git log --oneline -1\` and \`git status\` so completion can be verified. Commit BEFORE finishing." \
  -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
