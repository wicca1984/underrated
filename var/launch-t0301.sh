#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0301
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0301 — implement the **Document.getElementsByName(name)** DOM binding in the script module.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/script/ (touch ONLY src/script/mod.rs; do NOT touch other modules, lib.rs, src/dom, src/selector, or other worktrees).

WHY: We already bind sibling collection getters like \`document.getElementsByClassName(className)\` and
\`document.getElementsByTagName(tag)\`, but \`document.getElementsByName(name)\` is MISSING. Per HTML spec,
getElementsByName returns all elements whose \`name\` attribute equals the given string (a live-ish NodeList; a
static array snapshot is acceptable for our MVP, exactly like getElementsByClassName returns here). This is part of
MS-MVP-JS (DOM querying for real pages/forms).

CURRENT STATE (verify before coding, do NOT re-implement):
  - Study \`fn bridge_get_elements_by_class_name(...)\` in src/script/mod.rs. It builds a CSS selector string and
    delegates to the existing helper \`execute_dom_query_to_js_array(&selector, context)\`, returning a JS array of nodes.
  - The selector engine (src/selector) ALREADY supports ATTRIBUTE selectors of the form \`[name=\"value\"]\`
    (see Component::Attribute). So getElementsByName can be implemented purely in src/script/mod.rs by delegating an
    attribute selector to execute_dom_query_to_js_array — do NOT touch src/selector or src/dom.
  - Note the registration pattern: there is a NativeFunction registration block (search for
    \`bridge_get_elements_by_class_name\` near the \`JsString::from(\\\"getElementsByClassName\\\")\` line) AND a JS bootstrap
    line \`document.getElementsByClassName = function(className) { ... }\`. You must add BOTH the native bridge fn and
    the document.getElementsByName JS wiring, mirroring getElementsByClassName exactly.

SCOPE (in src/script/mod.rs ONLY):
  1. Add \`fn bridge_get_elements_by_name(_this, args, context)\` mirroring bridge_get_elements_by_class_name:
     read the first arg as a string \`name\`; if it is empty, return an empty JS array (delegate an unparseable/empty
     selector exactly as the className bridge does for the empty-token case).
  2. Build the attribute selector \`[name=\"<escaped>\"]\` and delegate to \`execute_dom_query_to_js_array(&selector, context)\`.
     ESCAPE any embedded double-quote and backslash in \`name\` (replace \\ with \\\\ and \" with \\\") so the selector string
     stays well-formed; if the selector fails to parse, the helper already returns an empty array safely.
  3. Register the native fn (NativeFunction::from_fn_ptr(bridge_get_elements_by_name), JsString::from(\"getElementsByName\"))
     next to the getElementsByClassName registration.
  4. Wire \`document.getElementsByName = function(name) { ... }\` in the JS bootstrap, mirroring the
     getElementsByClassName function (call bridge.getElementsByName(String(name)) and map the returned keys to nodes
     the SAME way getElementsByClassName does).

TESTS (in src/script/mod.rs, #[cfg(test)], mirror \`test_eval_with_dom_get_elements_by_class_name\`):
  - Build a small DOM with several elements carrying name attributes (e.g. two inputs name='q', one input name='btn').
  - Assert \`document.getElementsByName('q').length\` === 2 and \`document.getElementsByName('btn').length\` === 1.
  - Assert a non-existent name returns length 0, and that the returned items are the right elements
    (e.g. check getElementsByName('q')[0].getAttribute('name') === 'q').
  - Assert empty-string argument returns length 0 (no panic).

CONSTRAINTS (AGENTS.md): no unwrap/expect in non-test code (I-6) — follow the existing \`.to_std_string().unwrap_or_default()\`
style already used by the className bridge; no test skip/ignore (I-4); 1 task = 1 module (I-5); do not edit the main tree,
src/dom, src/selector, lib.rs, or other worktrees (I-3). Do not add new crates (I-1).

WORKFLOW:
  1. Implement in src/script/mod.rs only.
  2. Run: cargo fmt; cargo clippy --all-targets -- -D warnings; cargo test -p underrated script (or the crate's full test).
  3. \`git add -A && git commit\` with message: \`feat(script): implement Document.getElementsByName() DOM binding (t0301)\`
     ending with the Co-Authored-By trailer required by AGENTS.md.
  4. Confirm \`git -C /workspaces/wt/t0301 status\` is clean and \`git diff --name-only origin/main..HEAD\` lists ONLY
     src/script/mod.rs. COMMIT before you finish — do not leave work uncommitted.
" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta 2>&1
