#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0315
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

Task: t0315 — implement the DOM method \`Node.prototype.compareDocumentPosition(other)\` inside the script module ONLY.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/script/mod.rs (touch ONLY this file; do NOT touch other modules, lib.rs, mod.rs of other dirs, src/engine, src/forms, src/dom, or other worktrees).

CONTEXT: src/script/mod.rs hosts a Boa JS context whose DOM is built up in a large JS bootstrap string. Many Node/Element methods are already defined there via Object.defineProperty on each node (e.g. \`contains\`, \`before\`, \`after\`, \`append\`, the \`childNodes\`/\`parentNode\`/\`nodeType\` accessors, and an \`isEqualNodeHelper\` near line 982). \`compareDocumentPosition\` is NOT yet implemented (verified by grep). Implement it ENTIRELY in the JS bootstrap using the accessors that already exist (parentNode, childNodes, contains, nodeType). Do NOT add a Rust native bridge function unless strictly necessary — prefer a pure-JS implementation mirroring how \`contains\`/\`before\` are defined per-node with Object.defineProperty.

SPEC (per WHATWG DOM \`compareDocumentPosition(other)\`): returns a bitmask of these constants (which must also be exposed as properties, both on Node instances and ideally on a global \`Node\` constructor object if one exists; at minimum the numeric return value must be correct):
  DOCUMENT_POSITION_DISCONNECTED            = 1
  DOCUMENT_POSITION_PRECEDING               = 2
  DOCUMENT_POSITION_FOLLOWING               = 4
  DOCUMENT_POSITION_CONTAINS                = 8
  DOCUMENT_POSITION_CONTAINED_BY            = 16
  DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC = 32
Algorithm:
  - If node === other, return 0.
  - If they are in different trees (no common ancestor / not connected), return DISCONNECTED | IMPLEMENTATION_SPECIFIC | PRECEDING (i.e. 1|32|2 = 35) — use a consistent order; the spec allows implementation-specific tie-break, so a deterministic choice is fine.
  - If \`other\` is an ancestor of node (other.contains(node)): return CONTAINS | PRECEDING (8|2 = 10).
  - If \`other\` is a descendant of node (node.contains(other)): return CONTAINED_BY | FOLLOWING (16|4 = 20).
  - Otherwise, determine document order: if \`other\` precedes node in a pre-order/document traversal, return PRECEDING (2); else return FOLLOWING (4).
  To compute document order, walk up to the common ancestor and compare the index of the two diverging child branches in that ancestor's childNodes.

SCOPE (in src/script/mod.rs ONLY):
  1. Add the six DOCUMENT_POSITION_* numeric constants as properties on each node where the other per-node methods are defined (alongside e.g. \`before\`/\`after\` near line 1384). If a global \`Node\` object/constructor already exists in the bootstrap, also expose the constants there; if not, do NOT invent a new global beyond what's needed (the per-instance constants are sufficient for the tests).
  2. Define \`compareDocumentPosition(other)\` as a per-node method (Object.defineProperty, mirroring the existing \`contains\` definition) implementing the algorithm above using existing accessors. Guard against null/undefined \`other\` (return DISCONNECTED-style value, do not throw).
  3. Leave a \`// TODO(spec):\` for any edge case you intentionally simplified (e.g. attribute nodes, which this engine does not expose).

TESTS (in src/script/mod.rs, #[cfg(test)], mirror the existing DOM test style such as test_element_classlist / the isEqualNode tests — build a small DOM via eval and assert numeric results):
  - A node compared to itself returns 0.
  - Parent.compareDocumentPosition(child) returns 20 (CONTAINED_BY|FOLLOWING).
  - Child.compareDocumentPosition(parent) returns 10 (CONTAINS|PRECEDING).
  - For two sibling elements a (earlier) and b (later) under the same parent: a.compareDocumentPosition(b) has the FOLLOWING bit (4) set and NOT the PRECEDING bit; b.compareDocumentPosition(a) has PRECEDING (2) set.
  - A node compared to a freshly-created, unattached node has the DISCONNECTED bit (1) set.
  - Do NOT regress any existing test (run the full suite).

DELIVERABLE / DEFINITION OF DONE:
  - Run \`cargo fmt\`, \`cargo clippy --all-targets -- -D warnings\`, and \`cargo test\` — ALL must pass (green).
  - NO unwrap()/expect() in non-test code (I-6). NO skipped/ignored tests (I-4). Do NOT delete or weaken any existing test.
  - git add -A and COMMIT on this branch with message:
      feat(script): implement Node.compareDocumentPosition() DOM method (t0315)
  - Print the final \`git log --oneline -1\` and \`git status\` so completion can be verified. Commit BEFORE finishing." \
  -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
