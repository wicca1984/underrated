#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0312
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0312 — implement the DOM method \`Element.insertAdjacentText(position, data)\` (script bindings).
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/script/ (touch ONLY src/script/mod.rs; do NOT touch other modules, lib.rs, or other worktrees).

WHY: \`insertAdjacentText\` is a standard DOM method real pages use to inject text relative to an element without
parsing HTML. The sibling methods \`insertAdjacentElement\` and \`insertAdjacentHTML\` are ALREADY implemented in
src/script/mod.rs; \`insertAdjacentText\` is the missing third member of the family.

BACKGROUND (verify on main before coding — do NOT re-implement what exists):
  - \`insertAdjacentElement(position, element)\` is defined as a JS method (search src/script/mod.rs around the
    \`insertAdjacentElement(position, element) {\` shim) that calls the native bridge
    \`bridge.insertAdjacentElement(this.__key__, position, element.__key__)\` → Rust fn
    \`bridge_insert_adjacent_element\`. Read BOTH the JS shim and the Rust bridge fn to learn the exact pattern.
  - \`document.createTextNode(data)\` ALREADY exists and returns a wrapped text node with a \`__key__\`.
  - Reference test: \`test_element_insert_adjacent_html_and_element\` shows how these are exercised via
    \`eval_with_dom\`. Mirror its structure.

SCOPE (in src/script/mod.rs ONLY):
  1. Implement \`Element.prototype.insertAdjacentText(position, data)\`. The four positions (case-insensitive per
     spec) are: \`beforebegin\`, \`afterbegin\`, \`beforeend\`, \`afterend\`. Semantics: create a Text node from
     String(data) and insert it at the given position relative to the element — identical placement rules to
     \`insertAdjacentElement\`.
  2. Prefer the SIMPLEST contained implementation: a JS method on the element prototype that does
     \`const t = document.createTextNode(String(data));\` and then reuses the SAME placement logic as
     \`insertAdjacentElement\` (either by delegating through the existing element-insert bridge, or by adding a
     thin native \`bridge_insert_adjacent_text\` that mirrors \`bridge_insert_adjacent_element\` but creates/uses a
     text node). Do NOT duplicate placement logic if you can reuse it. Do NOT add HTML parsing.
  3. Match the spec return value: \`insertAdjacentText\` returns \`undefined\` (unlike insertAdjacentElement which
     returns the inserted node). For an unrecognized \`position\` string, throw a DOMException-like error the same
     way the sibling methods do today (mirror their error path; do NOT invent new error machinery).
  4. Do NOT change any public Rust interface or existing function signature used by other modules. Keep any new
     Rust helper private to script/mod.rs. Do NOT touch src/dom or any other module.

Approach: STRICTLY test-first (TDD). Add a #[cfg(test)] test in src/script/mod.rs modeled on
\`test_element_insert_adjacent_html_and_element\`, using \`eval_with_dom\`. Cover at least:
  - \`beforeend\` and \`afterbegin\` insert the text as the last / first child text of the target (assert via
    textContent or child inspection).
  - \`beforebegin\` / \`afterend\` insert the text as a sibling adjacent to the target.
  - An invalid position string is rejected (error / no insertion), matching sibling behavior.
Do NOT delete or weaken any existing test. Do NOT change iterative code back to recursive. Keep public
interfaces stable.

Done when: \`cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check\` ALL pass.
Commit your work BEFORE finishing (do not leave changes uncommitted): \`git add -A && git commit\` with message
\`feat(script): implement Element.insertAdjacentText() DOM method (t0312)\`. Comments and identifiers in English.
Must follow: AGENTS.md I-1..I-7 (no unwrap/expect in non-test code (I-6), no skipping/ignoring tests (I-4),
1 task = 1 module (I-5), no cross-worktree access (I-3)).
If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a
\`// TODO(spec):\` and report it in your summary (§8).
End with a short summary of what changed and confirm you committed." \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null
