#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0262
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0262 — Bind the `document.documentElement`, `document.body`, and `document.head` accessor properties in the JavaScript prelude. Real-world JS (including Google search result pages) constantly reads `document.body` / `document.documentElement` to append nodes or read geometry; without them those scripts throw on a null/undefined access. This advances MS-MVP-JS (DOM read/write surface for running real page scripts).

Target module: src/script/mod.rs (touch ONLY this file — both the JS prelude string and the inline `#[cfg(test)] mod tests`). Do NOT modify any other file. `git diff --name-only` must show ONLY: src/script/mod.rs.

Reuse / facts (verified — do NOT reinvent, mirror the EXISTING document accessors):
- The entire JS prelude is a single Rust raw-string literal `let setup_code = r#" ... "#;` spanning src/script/mod.rs lines ~375..1276. All DOM bindings live inside it.
- Inside that prelude the global `document` object already has methods defined on it, e.g. `document.getElementsByTagName = function(tagName) { const keys = bridge.getElementsByTagName(String(tagName)); if (!keys) return []; return keys.map(key => getOrCreateNode(key)); };` (around line 1065). This returns an array of wrapper nodes; each wrapper has an uppercase `.tagName` (e.g. `"HTML"`, `"BODY"`, `"HEAD"`).
- The prelude already defines `document` accessor properties via `Object.defineProperty(document, "<name>", { get() { ... }, enumerable: true, configurable: true });` — see the existing `Object.defineProperty(document, "parentNode", {...})` and `Object.defineProperty(document, "childNodes", {...})` blocks (around lines 1124..1135). MIRROR THIS EXACT SHAPE for the three new accessors and place them immediately after those existing `document` defineProperty blocks (still inside the prelude string).
- `getOrCreateNode` and the local `bridge` are already in scope inside the prelude; you do NOT need them directly here because you will reuse `document.getElementsByTagName(...)`.

Semantics — implement EXACTLY these three read-only getters on `document` (no setters):
- `document.documentElement` → the root `<html>` element: `return document.getElementsByTagName("html")[0] || null;`
- `document.body` → the `<body>` element: `return document.getElementsByTagName("body")[0] || null;`
- `document.head` → the `<head>` element: `return document.getElementsByTagName("head")[0] || null;`
Each as an `Object.defineProperty(document, "...", { get() { ... }, enumerable: true, configurable: true });`. Each returns `null` (NOT undefined, NOT a throw) when the element is absent. Add a `// spec: https://dom.spec.whatwg.org/#dom-document-body` (and the documentElement / head equivalents) comment line above the blocks, matching the commenting style already used in the file.
Note on simplification vs. spec: the WHATWG spec defines `document.body` as the first child of the document element that is a body OR frameset element, and documentElement as the document''s element child. Returning the first matching descendant via getElementsByTagName is the pragmatic approach already used elsewhere in this engine; add a `// TODO(spec):` comment noting this getElementsByTagName-based lookup does not enforce the "must be a child of documentElement" / frameset rules. Do NOT try to implement the full spec algorithm — keep it small and safe.

Acceptance (must all be green) — add ONE inline unit test (mirror the existing `eval_with_dom` tests, e.g. `test_eval_with_dom_basic` near line 3203 and `test_eval_with_dom_tree_navigation`). Build the DOM manually exactly like those tests do:
  let mut dom = Dom::new();
  let document = dom.document();
  let html = dom.create_node(NodeData::Element { name: "html".to_string(), attrs: vec![] });
  let head = dom.create_node(NodeData::Element { name: "head".to_string(), attrs: vec![] });
  let body = dom.create_node(NodeData::Element { name: "body".to_string(), attrs: vec![] });
  dom.append_child(html, head);
  dom.append_child(html, body);
  dom.append_child(document, html);
  let mut host = BoaHost::new();
Then assert (in a single eval returning the string "true"):
  - `document.documentElement.tagName === "HTML"`
  - `document.body.tagName === "BODY"`
  - `document.head.tagName === "HEAD"`
  - identity holds: `document.body === document.getElementsByTagName("body")[0]`
  And add a SECOND assertion path (separate eval, fresh `Dom::new()` with only `document` and no html/body) proving `document.body === null` and `document.documentElement === null` (the no-op/absent case returns null without throwing). Use `eval_with_dom` and compare the returned `Ok("true".to_string())` exactly as the existing tests do.

Done when ALL of these pass:
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
No unwrap/expect/panic/unsafe in non-test code (I-6). No `unsafe` anywhere (forbidden). No test skip/ignore (I-4). Keep the diff limited to src/script/mod.rs — `git diff --name-only` must show ONLY that file. Commit on this branch with: `feat(script): bind document.documentElement/body/head accessors (t0262)`. Comments and identifiers in English. IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the name of the test(s) you added. If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
