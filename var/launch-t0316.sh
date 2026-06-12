#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0316
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

Task: t0316 — implement the DOM method \`Node.prototype.normalize()\` inside the script module ONLY.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/script/mod.rs (touch ONLY this file; do NOT touch other modules, lib.rs, mod.rs of other dirs, src/engine, src/forms, src/dom, or other worktrees).

CONTEXT: src/script/mod.rs hosts a Boa JS context whose DOM is built up in a large JS bootstrap string. Many Node/Element methods are already defined there per-node via Object.defineProperty (e.g. \`contains\`, \`before\`, \`after\`, \`append\`, \`compareDocumentPosition\`, the \`childNodes\`/\`parentNode\`/\`nodeType\`/\`nodeValue\`/\`textContent\` accessors, \`removeChild\`, \`createTextNode\`, and \`setTextContent\`). \`normalize\` is NOT yet implemented (verified by grep: 0 hits for 'normalize'). Implement it ENTIRELY in the JS bootstrap using the accessors/bridges that already exist (childNodes, nodeType, nodeValue/textContent, removeChild, setTextContent). Do NOT add a new Rust native bridge unless strictly necessary — prefer a pure-JS implementation mirroring how \`compareDocumentPosition\`/\`contains\` are defined per-node with Object.defineProperty, and expose it on the document object too (mirror how compareDocumentPosition is added to both per-node and document).

SPEC (per WHATWG DOM \`normalize()\`): runs on the node and ALL of its descendant nodes (deep, recursive). For the tree rooted at the context node:
  - Text node = a node whose nodeType === 3 (use the existing nodeType accessor / Node.TEXT_NODE === 3).
  - Remove every empty exclusive Text node (a Text node whose data/nodeValue length is 0).
  - For each run of two or more contiguous (adjacent sibling) Text nodes, concatenate their data into the FIRST text node of the run (set its textContent/nodeValue to the concatenation) and remove the rest from their parent (removeChild).
  - The context node itself is processed for its children; then recurse into each remaining element child.
Algorithm sketch (pure JS):
  function normalizeHelper(node) {
    // snapshot children first (live childNodes will change as we remove)
    const kids = Array.prototype.slice.call(node.childNodes);
    let i = 0;
    while (i < kids.length) {
      const child = kids[i];
      if (child.nodeType === 3) {
        // gather the contiguous run of text nodes
        let text = String(child.nodeValue || '');
        let j = i + 1;
        while (j < kids.length && kids[j].nodeType === 3) {
          text += String(kids[j].nodeValue || '');
          j++;
        }
        // remove following text nodes in the run
        for (let k = i + 1; k < j; k++) { node.removeChild(kids[k]); }
        if (text.length === 0) { node.removeChild(child); }
        else { child.textContent = text; }  // or set nodeValue, whichever the engine honors
        i = j;
      } else {
        normalizeHelper(child);  // recurse into element/other children
        i++;
      }
    }
  }
  (Verify whether setting textContent on a Text node updates its data without re-parsing children; if textContent is element-only here, use nodeValue / setTextContent on the text node's key instead. Pick whichever the existing bridges actually honor for text nodes and confirm with a test.)

SCOPE (in src/script/mod.rs ONLY):
  1. Define \`normalize()\` as a per-node method (Object.defineProperty, no-arg) calling normalizeHelper(this). Add it both per-node and on the document object, mirroring \`compareDocumentPosition\`.
  2. Guard against nodes with no children (no-op). Do not throw.
  3. Leave a \`// TODO(spec):\` for any edge case you intentionally simplified (e.g. CDATA sections, which this engine does not expose).

TDD (write tests FIRST, then implement until green) — add a \`#[test]\` in the existing tests module of src/script/mod.rs that, using \`eval_with_dom\`, builds a tree with adjacent text nodes (e.g. via createTextNode + appendChild) and asserts:
  - After normalize(), adjacent text nodes are merged into one (childNodes length shrinks; the merged text equals the concatenation).
  - Empty text nodes are removed.
  - Recursion reaches nested elements (a nested element with adjacent text children is also merged).
Follow the existing test style (assert on \`eval_with_dom(...)\` Ok(String) results, e.g. read childNodes.length and textContent).

When done: run \`cargo test\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo fmt --check\`, \`cargo doc --no-deps\` — ALL must be green. Then \`git add -A && git commit\` with message exactly:
  feat(script): implement Node.normalize() DOM method (t0316)
Then print the final \`git log -1 --oneline\` and confirm the working tree is clean. Do NOT push or open a PR (the orchestrator handles that)." \
  -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta \
  > /workspaces/underrated-meta/var/worker-logs/t0316.log 2>&1
