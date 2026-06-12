#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0319
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

Task: t0319 (milestone MS-MVP-JS, parallel/generic fill) — implement the DOM method \`Node.getRootNode()\` plus deterministic unit tests, entirely inside the SCRIPT module.
Target module: src/script/mod.rs ONLY. Do NOT modify src/engine, src/loader, src/layout, src/url, src/net, lib.rs, any other module's mod.rs, or tests/ outside src/script/mod.rs's own #[cfg(test)] module. Do NOT touch other worktrees.

CONTEXT (verified — read these before coding):
- DOM JS methods are attached to node objects inside a Rust raw-string of JavaScript. See the existing pattern around src/script/mod.rs:1362 where \`node.isSameNode\`, \`node.isEqualNode\`, \`node.hasChildNodes\`, \`node.compareDocumentPosition\` are attached as \`node.<name> = function(...) { ... };\`.
- The SAME methods are ALSO attached to the \`document\` object separately around src/script/mod.rs:1773 (e.g. \`document.isSameNode\`, \`document.isEqualNode\`). You MUST attach getRootNode in BOTH places (node objects AND document), mirroring how the neighbouring methods are duplicated, so both \`element.getRootNode()\` and \`document.getRootNode()\` work.
- Existing JS-level navigation: node objects already expose \`parentNode\` (used elsewhere in this file). Implement getRootNode in pure JS using parentNode — do NOT add new Rust bridge methods unless strictly required (and if you think you do, STOP and leave a \`// TODO(spec):\` instead — adding bridge fns risks crossing module/registration seams).
- Unit tests live in the existing \`#[cfg(test)] mod tests\` of src/script/mod.rs as JS-string evals via \`host.eval_with_dom(\"...\", &mut dom)\`. See the isSameNode test block around src/script/mod.rs:5268 for the exact harness/builder style (create_node, append_child, getElementById, assert_eq! on Ok(\"...\".to_string())).

SPEC (WHATWG DOM \`getRootNode(options)\`):
- Returns the root of the context object: the topmost ancestor reachable via parentNode. Walk up parentNode until there is no parent; return that topmost node.
- For a node attached under the document, the root is the document (i.e. \`el.getRootNode() === document\` when el is in the document tree). Verify how parentNode terminates in THIS engine's DOM (whether the document object itself appears as an ancestor) and make the implementation return the correct topmost object so that \`document.body's descendant.getRootNode()\` is the document. If parentNode chains stop at the root element rather than the document, match real-browser semantics: an in-document node's root is the document — adjust accordingly using whatever this engine exposes (e.g. an ownerDocument/document reference) WITHOUT touching other modules. If you cannot determine this cleanly from src/script alone, leave a \`// TODO(spec):\` and implement the plain topmost-parentNode walk.
- \`document.getRootNode()\` returns the document itself.
- The \`options\` argument (\`{composed: ...}\`) can be accepted and ignored (no shadow DOM in this engine).
- A detached node (no parent) returns itself.

TDD — write tests FIRST in the existing #[cfg(test)] mod tests, then implement until green. Required deterministic cases (no network, no files):
  a. An element appended into the document: \`<el>.getRootNode() === document\` (or, if engine semantics make the html/root element the top, assert against whatever the documented root is — but prefer document). Use isSameNode/=== to compare identity.
  b. \`document.getRootNode() === document\`.
  c. A freshly created, NOT-yet-appended node: \`n.getRootNode() === n\` (returns itself).
  d. A nested element (grandchild) returns the same root as its parent.
Keep assertions identity-based (\`===\` or \`.isSameNode(...)\`), returning \"true\"/\"false\" strings to match the harness.

When done: run \`cargo test\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo fmt --check\`, \`cargo doc --no-deps\` — ALL must be green. This is a non-rendering (logic-only) task, so NO PNG is required. Then \`git add -A && git commit\` with message exactly:
  feat(script): implement Node.getRootNode() DOM method (t0319)
Then print \`git log -1 --oneline\`, run \`git status --porcelain\` and confirm the working tree is clean. Do NOT push or open a PR (the orchestrator handles that). If the spec is ambiguous or real browser behavior conflicts, do NOT decide alone — leave \`// TODO(spec):\` and report. Finish with a short English summary of what changed and which files you touched (must be only src/script/mod.rs)." -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
