#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0309
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0309 — implement the \`Node.hasChildNodes()\` DOM method so scripts can test whether a node has any child nodes. Mirror the existing \`bridge_first_child\` native fn + the \`contains\` method wiring exactly.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/script/ (touch ONLY src/script/mod.rs; do NOT touch src/dom, src/layout, lib.rs, or any other module/worktree).

SPEC: \`node.hasChildNodes()\` returns the boolean true if the node has any child nodes, otherwise false. A freshly \`document.createElement('div')\` with no children returns false; after \`div.appendChild(child)\` it returns true. It is a METHOD (function call with ()), NOT a getter property.

WHERE / TEMPLATE (verify by reading the code yourself before coding):
  - \`fn bridge_first_child\` (around line 3678 in src/script/mod.rs) is the exact native template. It resolves the node key via \`with_dom(|dom, key_to_node| { ... key_to_node.get(&node_key) ... dom.children(n_id) ... })\`.
  - The native bridge registration block is around lines 285-298 (\`.function(NativeFunction::from_fn_ptr(bridge_contains), JsString::from(\"contains\"), 1)\`).
  - The JS-side method wiring for \`contains\` is at ~line 1223 (\`node.contains = function(...) { return bridge.contains(this.__key__, ...); };\`) AND mirrored on \`document\` at ~line 1615 (\`document.contains = function(...) {...};\`).

SCOPE (in src/script/mod.rs ONLY):
  1. Add a native bridge fn \`bridge_has_child_nodes\` modeled on \`bridge_first_child\`. Resolve the node key to a node id;
     return \`Ok(JsValue::from(!dom.children(n_id).is_empty()))\`. If the key does not resolve (or no first arg), return \`Ok(JsValue::from(false))\`.
     Do NOT use unwrap/expect in non-test code (I-6); use the same \`if let\`/\`unwrap_or_default\` idioms as the template.
  2. Register it in the bridge object builder next to \`contains\`: \`NativeFunction::from_fn_ptr(bridge_has_child_nodes)\`, \`JsString::from(\"hasChildNodes\")\`, arity 1.
  3. Add the JS method next to the \`node.contains\` method: \`node.hasChildNodes = function() { return bridge.hasChildNodes(this.__key__); };\` and mirror it on \`document\` next to \`document.contains\`: \`document.hasChildNodes = function() { return bridge.hasChildNodes(this.__key__); };\`.
  4. Keep the diff tiny and localized. Do NOT change the NodeData enum, DOM serialization, or any other binding.

VALIDATION before committing:
  - \`cargo fmt --check\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo test\` — all green.
  - Add ONE focused unit test (mirror an existing src/script/mod.rs test that uses eval_with_dom) proving:
      * a node from \`document.createElement('div')\` with no children has \`hasChildNodes() === false\`, and
      * after \`div.appendChild(document.createElement('span'))\` the same node has \`hasChildNodes() === true\`.
    Assert precisely (exact booleans); do not weaken the assertion to force green. Do not delete or weaken any other test.

When done: \`git add -A && git commit -m 'feat(script): implement Node.hasChildNodes() DOM method (t0309)'\`. Commit BEFORE finishing. Do not push or open a PR; the orchestrator handles that." \
  -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta \
  >> /workspaces/underrated-meta/var/worker-logs/t0309.log 2>&1
