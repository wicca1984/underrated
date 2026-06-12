#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0305
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0305 — implement the **Node.isEqualNode(other)** and **Node.isSameNode(other)** DOM bindings in the script module.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/script/ (touch ONLY src/script/mod.rs; do NOT touch other modules, lib.rs, src/dom, src/layout, or other worktrees).

WHY: These are core, widely-used DOM Node methods (part of MS-MVP-JS). We already expose nodeType, nodeName, childNodes,
attributes accessors, etc., but \`grep -rn 'isEqualNode\|isSameNode' src\` → no matches. This is a pure-JS binding that
REUSES the DOM properties we already expose on each node object — no Rust DOM changes needed.

CURRENT STATE (verify before coding, REUSE — do NOT add new modules/crates):
  - In src/script/mod.rs the per-node binding is installed via a sequence of \`Object.defineProperty(node, '<name>', { ... })\`
    calls (search for \`defineProperty(node, 'contains'\`, \`defineProperty(node, 'cloneNode'\`, \`defineProperty(node, 'childNodes'\`).
    Each is a JS snippet executed when a node object is created. ADD the two new methods the SAME way, next to the other
    Node methods (e.g. right after 'contains' / 'cloneNode').
  - Properties already available on a node object to REUSE in your JS implementation: \`nodeType\`, \`nodeName\`,
    \`childNodes\` (with \`.length\` and indexed access), and for elements the attribute accessors
    (\`getAttributeNames()\`, \`getAttribute(name)\`). Confirm the exact names by reading the surrounding defineProperty blocks
    and the test helpers before writing — use what THIS file actually exposes.

SCOPE (in src/script/mod.rs ONLY), implement per the DOM spec:
  1. **isSameNode(other)**: return \`this === other\` (true only if it is literally the same node object; \`isSameNode(null)\`
     returns false). One-liner.
  2. **isEqualNode(other)**: structural deep equality per the WHATWG DOM 'equals' algorithm:
       - If \`other\` is null/undefined → false.
       - Both must have the same \`nodeType\` and same \`nodeName\`.
       - For ELEMENT nodes (nodeType === 1): same set of attributes — equal number of attributes AND for every attribute
         name on this, the same value on other (compare via getAttributeNames()/getAttribute()). Attribute ORDER is NOT
         significant; the SET of (name,value) pairs must match.
       - For TEXT (nodeType === 3) and COMMENT (nodeType === 8) nodes: their text data must be equal. Use whatever text
         accessor this file exposes for character data (e.g. nodeValue/textContent/data — verify which one exists for these
         node objects and use it; if none is directly exposed, leave a precise \`// TODO(spec):\` and compare nodeName only,
         do NOT invent a Rust accessor).
       - Then the children must be equal: \`this.childNodes.length === other.childNodes.length\` AND each
         \`this.childNodes[i].isEqualNode(other.childNodes[i])\` is true, in order (recursive).
     Keep it a self-contained recursive JS function defined inside the method (or a small helper) — no Rust recursion needed.
  3. Keep the change surgical and localized to where the other Node methods are installed. Do NOT add new dependencies,
     new Rust functions, or new display/DOM variants.

TESTS (in src/script/mod.rs, #[cfg(test)], mirror an existing script DOM test such as the cloneNode / contains test —
find one that builds a DOM via createElement/appendChild and runs a JS snippet, and copy its structure exactly):
  - Two separately-created \`<div class='x'>\` elements with identical attributes and identical text children →
    \`a.isEqualNode(b) === true\`.
  - Same two but with a differing attribute value (or an extra attribute) → \`isEqualNode === false\`.
  - Different nodeName (\`<div>\` vs \`<span>\`) → false.
  - Different child count, or a differing text child → false.
  - \`a.isSameNode(a) === true\`, \`a.isSameNode(b) === false\`, \`a.isEqualNode(null) === false\`,
    \`a.isSameNode(null) === false\`.
  Inspect how existing script tests assert on a returned JS value and mirror that exactly.

CONSTRAINTS (AGENTS.md): no unwrap/expect in non-test code (I-6) — follow existing \`.to_std_string().unwrap_or_default()\`
style if you touch any Rust glue; no test skip/ignore (I-4); 1 task = 1 module (I-5); do not edit the main tree, src/dom,
src/layout, lib.rs, or other worktrees (I-3). Do not add new crates (I-1). If a needed character-data accessor genuinely
does not exist, leave a \`// TODO(spec):\` rather than reaching into another module.

WORKFLOW:
  1. Implement in src/script/mod.rs only.
  2. Run: cargo fmt; cargo clippy --all-targets -- -D warnings; cargo test (or at least the script tests).
  3. \`git add -A && git commit\` with message:
     \`feat(script): implement Node.isEqualNode/isSameNode DOM bindings (t0305)\`
     ending with the Co-Authored-By trailer required by AGENTS.md.
  4. Confirm \`git -C /workspaces/wt/t0305 status\` is clean and \`git diff --name-only origin/main..HEAD\` lists ONLY
     src/script/mod.rs. COMMIT before you finish — do not leave work uncommitted." \
  -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
