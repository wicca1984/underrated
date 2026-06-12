#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0302
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0302 — implement the **Element.insertAdjacentElement(position, element)** and **Element.insertAdjacentHTML(position, html)** DOM bindings in the script module.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/script/ (touch ONLY src/script/mod.rs; do NOT touch other modules, lib.rs, src/dom, src/html, src/selector, or other worktrees).

WHY: Real pages and frameworks frequently call \`el.insertAdjacentHTML('beforeend', '<li>...')\` and
\`el.insertAdjacentElement('afterend', node)\` to splice content relative to an existing element. We already bind
the harder primitives (\`set_inner_html\` parses an HTML fragment; \`insertBefore\`/\`appendChild\`/\`removeChild\` mutate the
tree) but the insertAdjacent* family is MISSING. This is part of MS-MVP-JS (DOM mutation for real pages).

THE 4 POSITIONS (HTML spec, case-insensitive):
  - 'beforebegin' : insert as a previous sibling of the reference element (into reference.parentNode, before reference).
  - 'afterbegin'  : insert as the FIRST child of the reference element.
  - 'beforeend'   : insert as the LAST child of the reference element (i.e. append).
  - 'afterend'    : insert as a next sibling of the reference element (into reference.parentNode, after reference).
  For 'beforebegin'/'afterend', if the reference element has no parent, the operation is a no-op (do nothing; do not panic).

CURRENT STATE (verify before coding, do NOT re-implement, REUSE these):
  - Study \`fn bridge_set_inner_html(...)\` in src/script/mod.rs (around line 3065). It shows the canonical fragment-parse
    pattern: wrap html in \`<body>...</body>\`, call \`crate::html::parse_document(...)\`, find the temp <body>, then for each
    temp child call \`copy_node_to_dom_recursive(&temp_dom, temp_child_id, dom)\` and attach to the live dom. REUSE this exact
    fragment-parse + copy approach for insertAdjacentHTML.
  - Study \`fn bridge_insert_before(...)\` and \`fn bridge_append_child(...)\` and \`fn bridge_get_elements_by_class_name\` for the
    \`with_dom(|dom, key_to_node| { ... })\` access pattern and how a node key (string) maps to a NodeId, and how a returned
    node is mapped back to a JS handle/key the bootstrap layer understands.
  - Inspect the \`crate::dom\` API already used in this file (do NOT add new dom methods): you should find methods like
    \`dom.children(id)\`, \`dom.append_child(parent, child)\`, \`dom.insert_before(...)\` or equivalent, and a way to get a node's
    parent. Use ONLY methods that already exist and are already called from src/script/mod.rs. If the exact insertion
    primitive you need (e.g. insert-as-first-child, or insert-after-sibling) is not directly available, COMPOSE it from the
    existing children/append/insert_before/parent primitives — do NOT modify src/dom.

SCOPE (in src/script/mod.rs ONLY):
  1. Add \`fn bridge_insert_adjacent_element(_this, args, context)\`: args = (refNodeKey, position, elementNodeKey).
     Resolve both node keys to NodeIds via with_dom; lowercase/trim the position string and match the 4 positions;
     splice the element node into the live tree at the correct spot (compose from existing dom primitives). Return the
     inserted element handle the SAME way sibling bridges return a node (mirror bridge_append_child's return), or null/undefined
     on a no-op / unknown position, consistent with how the existing bridges signal 'nothing inserted'.
  2. Add \`fn bridge_insert_adjacent_html(_this, args, context)\`: args = (refNodeKey, position, htmlString).
     Parse the html fragment EXACTLY like bridge_set_inner_html (wrapped <body>, parse_document, find body,
     copy_node_to_dom_recursive each child), but instead of replacing children, INSERT the resulting top-level nodes at the
     position relative to the reference element, preserving their order. Returns undefined (per spec).
  3. Register BOTH native fns (NativeFunction::from_fn_ptr(...), JsString::from(\"...\")) next to the existing
     insertBefore/appendChild registrations.
  4. Wire the JS bootstrap so element handles expose \`insertAdjacentElement(position, element)\` and
     \`insertAdjacentHTML(position, html)\`, mirroring EXACTLY how existing element methods like \`insertBefore\` / \`appendChild\` /
     \`setInnerHTML\`-backed \`innerHTML\` setter are wired onto element handles (call the native bridge with the element's own key
     plus the args, and map any returned key back to a node the same way the sibling methods do).

TESTS (in src/script/mod.rs, #[cfg(test)], mirror \`test_element_inner_html_getter_setter\` and the insertBefore/appendChild tests):
  - Build a DOM like <ul id='L'><li id='a'>a</li></ul>. Get el = document.getElementById('L') (or 'a').
  - insertAdjacentHTML('beforeend', '<li>b</li>') then assert L has 2 children and the last child text is 'b'.
  - insertAdjacentHTML('afterbegin', '<li>z</li>') then assert the FIRST child text is 'z'.
  - On 'a' (a child): insertAdjacentHTML('beforebegin', '<li>pre</li>') and assert 'pre' is now a's previous sibling;
    insertAdjacentHTML('afterend', '<li>post</li>') and assert 'post' is a's next sibling.
  - insertAdjacentElement: create an element via document.createElement('li'), set its text, insert it at 'beforeend' and
    assert it is the last child and that the SAME node object/handle is returned.
  - Unknown position string (e.g. 'nope') and a beforebegin/afterend on a parentless element must be safe no-ops (length
    unchanged, no panic).

CONSTRAINTS (AGENTS.md): no unwrap/expect in non-test code (I-6) — follow the existing
\`.to_std_string().unwrap_or_default()\` style; no test skip/ignore (I-4); 1 task = 1 module (I-5); do not edit the main
tree, src/dom, src/html, src/selector, lib.rs, or other worktrees (I-3). Do not add new crates (I-1).
If real-browser semantics are ambiguous vs our fragment-parse emulation, leave a \`// TODO(spec):\` and proceed with the
beforeend-style emulation already used by set_inner_html (§8) — do NOT invent new behavior.

WORKFLOW:
  1. Implement in src/script/mod.rs only.
  2. Run: cargo fmt; cargo clippy --all-targets -- -D warnings; cargo test (or at least the script tests).
  3. \`git add -A && git commit\` with message:
     \`feat(script): implement Element.insertAdjacentElement/insertAdjacentHTML (t0302)\`
     ending with the Co-Authored-By trailer required by AGENTS.md.
  4. Confirm \`git -C /workspaces/wt/t0302 status\` is clean and \`git diff --name-only origin/main..HEAD\` lists ONLY
     src/script/mod.rs. COMMIT before you finish — do not leave work uncommitted.
" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta 2>&1
