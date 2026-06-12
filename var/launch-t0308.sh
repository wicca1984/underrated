#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0308
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0308 — implement the read-only \`Node.isConnected\` DOM property so scripts can test whether a node is attached to the document tree. Mirror the existing \`parentNode\` getter + \`bridge_parent_node\` native fn exactly.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/script/ (touch ONLY src/script/mod.rs; do NOT touch src/dom, src/layout, lib.rs, or any other module/worktree).

SPEC: \`node.isConnected\` returns the boolean true if the node is connected to its relevant document — i.e. walking up via parents reaches the document root. Otherwise false. A freshly \`document.createElement(...)\`'d node (not yet appended) is NOT connected; after \`document.appendChild(el)\` (or appended into a connected ancestor) it IS connected.

WHERE / TEMPLATE (verify by reading the code yourself before coding):
  - \`bridge_parent_node\` (around line 3497 in src/script/mod.rs) is the exact native template. It uses
    \`with_dom(|dom, key_to_node| { ... dom.parent(n_id) ... })\` and \`dom.document()\` is available as the root node id.
  - The JS-side \`parentNode\` getter wiring is around lines 1204 and 1588 (\`return getOrCreateNode(bridge.parentNode(this.__key__));\`).
    Find how a getter property is defined via \`Object.defineProperty(node, 'parentNode', { get: function() {...} })\` and mirror it for an \`isConnected\` getter that returns a boolean (NOT a node).

SCOPE (in src/script/mod.rs ONLY):
  1. Add a native bridge fn \`bridge_is_connected\` modeled on \`bridge_parent_node\`. Resolve the node key to a node id;
     then walk upward with \`dom.parent(...)\` until it returns None to find the root; return
     \`Ok(JsValue::from(root_id == dom.document()))\`. If the key does not resolve, return \`Ok(JsValue::from(false))\`.
     Do NOT use unwrap/expect in non-test code (I-6); use the same \`if let\`/\`unwrap_or_default\` idioms as the template.
  2. Register it in the bridge object builder next to parentNode: \`JsString::from(\"isConnected\")\`, arity 1.
  3. Add the JS getter next to the \`parentNode\` getter so \`node.isConnected\` reads \`bridge.isConnected(this.__key__)\`
     and returns the boolean directly (do NOT pass through getOrCreateNode).
  4. Keep the diff tiny and localized. Do NOT change the NodeData enum, DOM serialization, or any other binding.

VALIDATION before committing:
  - \`cargo fmt --check\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo test\` — all green.
  - Add ONE focused unit test (mirror an existing src/script/mod.rs test that uses eval_with_dom) proving:
      * a detached node from \`document.createElement('div')\` has \`isConnected === false\`, and
      * after \`document.appendChild(div)\` the same node has \`isConnected === true\`.
    Assert precisely (exact booleans); do not weaken the assertion to force green. Do not delete or weaken any other test.

When done: \`git add -A && git commit -m 'feat(script): implement Node.isConnected DOM property (t0308)'\`. Commit BEFORE finishing. Do not push or open a PR; the orchestrator handles that." \
  -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta \
  >> /workspaces/underrated-meta/var/worker-logs/t0308.log 2>&1
