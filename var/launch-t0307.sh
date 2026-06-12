#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0307
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0307 — implement the \`Document.createComment(data)\` DOM binding so scripts can create Comment nodes, mirroring the existing \`Document.createTextNode\` binding exactly.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/script/ (touch ONLY src/script/mod.rs; do NOT touch src/dom, src/layout, lib.rs, or any other module/worktree).

WHY / WHERE (verify by reading the code yourself before coding):
  - The DOM already has a \`NodeData::Comment(String)\` variant (src/dom/mod.rs) — DO NOT add a new node kind; just reuse it.
  - \`Document.createTextNode\` is the exact template to copy. It has THREE parts in src/script/mod.rs:
      1. A native bridge fn \`bridge_create_text_node\` (around line 2768) that reads the string arg, calls
         \`with_dom(|dom, key_to_node| { let node_id = dom.create_node(NodeData::Text(data)); ... })\` and returns the node key.
      2. Registration in the bridge object builder: \`.function(NativeFunction::from_fn_ptr(bridge_create_text_node), JsString::from(\"createTextNode\"), 1)\` (around line 146).
      3. JS wiring (around line 1477): \`document.createTextNode = function(data) { const key = bridge.createTextNode(...); return getOrCreateNode(key); };\`

SCOPE (in src/script/mod.rs ONLY):
  1. Add a native bridge fn \`bridge_create_comment\` that is identical to \`bridge_create_text_node\` EXCEPT it creates
     \`NodeData::Comment(data)\` instead of \`NodeData::Text(data)\`. Per the DOM spec, a missing/undefined argument
     coerces to the string \"undefined\" via String(); but to match createTextNode's existing behavior, keep the same
     arg-handling code path (copy it verbatim, only swapping the NodeData variant).
  2. Register it in the bridge builder next to createTextNode: \`JsString::from(\"createComment\")\`, arity 1.
  3. Add the JS wiring next to createTextNode:
       document.createComment = function(data) {
           const key = bridge.createComment(data !== undefined ? String(data) : \"\");
           return getOrCreateNode(key);
       };
  4. Do NOT change any other binding, the NodeData enum, or DOM serialization. Keep the diff tiny and localized.

VALIDATION before committing:
  - \`cargo fmt --check\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo test\` — all green.
  - No \`unwrap\`/\`expect\` in non-test code (I-6).
  - Add ONE focused unit test (mirror an existing createTextNode test in src/script/mod.rs) proving that
    \`document.createComment('hi')\` returns a node whose nodeType is the Comment type (8) — or, if nodeType is not
    exposed, that the created node round-trips (e.g. appendChild into an element then serialize/outerHTML contains
    \`<!--hi-->\`). Assert precisely; do not weaken the assertion to force green.

COMMIT (you MUST commit before finishing — uncommitted work is lost):
  - \`git add -A && git commit\` on branch \`agent/t0307-create-comment\` with message:
    \`feat(script): implement Document.createComment() DOM binding (t0307)\`
  - Confirm with \`git -C /workspaces/wt/t0307 diff --name-only origin/main...HEAD\` that ONLY src/script/mod.rs changed.
" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta 2>&1 | tee /workspaces/underrated-meta/var/worker-logs/t0307.jsonl
