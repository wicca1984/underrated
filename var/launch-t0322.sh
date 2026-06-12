#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0322
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

Task: t0322 (milestone MS-MVP-JS parallel filler) — implement the DOM method \`Element.hasAttributes()\`. It returns a boolean: true if the element has one or more attributes, false otherwise (per the DOM spec, Element.hasAttributes()). ENTIRELY inside the SCRIPT module.
Target file: src/script/mod.rs ONLY. Do NOT modify src/loader, src/engine, src/layout, src/url, src/net, lib.rs, any other module's mod.rs, or any file under tests/. Do NOT touch other worktrees.

WHERE / HOW (precise — mirror the existing sibling method, do not redesign):
- The element prototype methods are defined in the big JS prototype-injection string in src/script/mod.rs. Find the existing \`hasAttribute(name) { ... }\` method (around line 1180) — it lives in the same object-literal of element methods. Add a new sibling method \`hasAttributes()\` right next to the other \`hasAttribute*\`/attribute methods, following the EXACT same indentation and style.
- The element prototype already exposes \`getAttributeNames()\` (returns an array of this element's attribute names). The simplest correct, bridge-free implementation is:
  \`\`\`js
  hasAttributes() {
      return this.getAttributeNames().length > 0;
  }
  \`\`\`
  Use \`getAttributeNames()\` if and only if it is actually available on \`this\` in that prototype scope (grep \`getAttributeNames\` in this file to confirm the exact name/shape). If it is NOT directly callable there, instead route through the existing Rust bridge the same way \`hasAttribute\` does (\`bridge.<method>(this.__key__, ...)\`) — but do NOT add a new bridge method or touch Rust bridge code if the pure-JS form works. Prefer the pure-JS form.
- Do NOT change any function signature, the bridge, or any other method. This is purely additive.

TDD — add a test in the existing \`#[cfg(test)] mod tests\` block of src/script/mod.rs. Study an existing DOM-method test that uses \`eval_with_dom\` (grep \`fn test_node_get_root_node\` or \`fn test_node_normalize\` and the \`host.eval_with_dom(...)\` idiom) and reuse that EXACT pattern (build DOM, eval JS expression, assert the returned JS value). All tests MUST be network-free and deterministic. Required assertions in a new \`#[test] fn test_element_has_attributes()\`:
  a. An element WITH an attribute returns true: e.g. create \`<div id=\"x\" class=\"y\">\`, then \`document.getElementById('x').hasAttributes() === true\`.
  b. An element WITH NO attributes returns false: e.g. \`const n = document.createElement('span'); n.hasAttributes() === false\`.
  c. After adding an attribute to a previously-empty element it returns true: \`const m = document.createElement('p'); const before = m.hasAttributes(); m.setAttribute('data-k','v'); const after = m.hasAttributes(); /* assert before===false && after===true */\`.
Keep assertions concrete (compare the exact boolean). Do NOT weaken or delete any existing test.

When done: run \`cargo test\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo fmt --check\`, \`cargo doc --no-deps\` — ALL must be green. This is a logic-only DOM-API task, so NO PNG is required. Then \`git add -A && git commit\` with message exactly:
  feat(script): implement Element.hasAttributes() DOM method (t0322)
Then print \`git log -1 --oneline\`, run \`git status --porcelain\` and confirm the working tree is clean. Do NOT push or open a PR (the orchestrator handles that). If the spec is genuinely ambiguous, do NOT decide alone — leave a \`// TODO(spec):\` and report. Finish with a short English summary of what changed and which files you touched (must be only src/script/mod.rs)." -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
