#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0258
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0258 — Bind the Element-only DOM traversal accessors that real-world page scripts (e.g. Google) use to walk the element tree while skipping text/comment nodes. Add these read-only accessors to the script bindings: `firstElementChild`, `lastElementChild`, `nextElementSibling`, `previousElementSibling`, `children` (element-only child list), `childElementCount`, and `parentElement`. This advances the MS-MVP-JS milestone (DOM read breadth for JS-driven result pages).

Target module: src/script/mod.rs (touch ONLY src/script/mod.rs and its inline tests). Do NOT modify src/dom, src/html, src/engine, src/event or any other module — REUSE the existing DOM API and node bridging helpers. Read those modules read-only as needed.

Reuse / facts (verified — do NOT reinvent):
- Accessors are registered as native functions in the bindings builder around src/script/mod.rs:231-300 (e.g. `.function(NativeFunction::from_fn_ptr(bridge_first_child), JsString::from("firstChild"), 1)`). Add the new accessors there, mirroring the existing entries exactly.
- Existing node-bridge implementations to mirror: `bridge_first_child` (src/script/mod.rs:2134), `bridge_next_sibling` (src/script/mod.rs:2164), `bridge_parent_node` (src/script/mod.rs:2059), and `bridge_child_nodes`. Copy their structure (how they resolve the node id from `this`, walk the DOM via the existing `dom` handle, and convert a resulting node id back into the JS node object). Do NOT change those existing fns.
- Element detection: a node is an element iff `matches!(dom.data(id), Some(NodeData::Element { .. }))` (this exact pattern is used throughout, e.g. src/script/mod.rs:2587). Use it to skip text/comment nodes.

Semantics (DOM standard):
- `firstElementChild`: first child that is an Element, else null.
- `lastElementChild`: last child that is an Element, else null.
- `nextElementSibling` / `previousElementSibling`: nearest following/preceding sibling that is an Element, else null.
- `children`: an array (use the same JS array/collection shape that `childNodes` returns) of the child nodes that are Elements, in order.
- `childElementCount`: the number of Element children (integer).
- `parentElement`: the parent node if it is an Element, else null (note: a parent that is the document/non-element returns null).

Approach (test-first / TDD):
1. Add new `bridge_*` fns mirroring the existing ones; reuse all existing helpers (node-id resolution + node->JS conversion + the `children`/array builder used by `bridge_child_nodes`). No unwrap/expect, no panic, no unsafe.
2. Register each new accessor in the bindings builder block next to the existing `firstChild`/`nextSibling` entries.
3. Keep ALL existing tests green and add new inline tests.

Acceptance (must all be green) — add inline unit tests in src/script/mod.rs mirroring the existing DOM-accessor tests (build a small DOM with mixed element + text children and assert via boa eval or the existing test harness):
  - For a parent `<div>` whose children are: text, `<span id=a>`, text, `<b id=b>`, text — `firstElementChild` is the span, `lastElementChild` is the `<b>`, `childElementCount` is 2, and `children` has length 2 in order [span, b].
  - From the span: `nextElementSibling` is the `<b>` (skipping the text node) and `previousElementSibling` is null.
  - `parentElement` of the span is the div; `parentElement` of an element whose parent is the document node is null.
  - Regression: the existing `firstChild`/`childNodes`/`nextSibling` tests still pass unchanged.
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Done when all three pass. No unwrap/expect in non-test code (I-6). No unsafe (forbidden). No test skip/ignore (I-4). Keep the diff limited to src/script/mod.rs — `git diff --name-only` must show ONLY src/script/mod.rs.
Commit on this branch with: `feat(script): bind Element traversal accessors (firstElementChild/children/etc) (t0258)`. Comments and identifiers in English.
IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the names of the tests you added.
If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
