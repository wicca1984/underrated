#!/usr/bin/env bash
# Launcher for Gemini worker t0285 — DOM ChildNode.before()/after() (src/script/mod.rs).
# Dispatched via setsid so it survives the orchestrator tick (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0285
LOG=/workspaces/toy-browser/var/worker-logs/t0285.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0285 — Implement the DOM `ChildNode` insertion methods `Element.before(node)` and `Element.after(node)`
(WHATWG DOM ChildNode mixin). These insert `node` into the tree immediately before / after `this` element.
Read: docs/SPEC.md (script / DOM) and docs/ARCHITECTURE.md under /workspaces/underrated-meta/.
Target module: src/script/mod.rs ONLY. Do NOT touch any other module or file, and do NOT touch any other worktree.

Background (READ THE CODE FIRST — verify every claim before changing):
- DOM bindings are set up in src/script/mod.rs. There are TWO layers:
  (1) Rust "bridge" native functions, e.g. `bridge_insert_before` (search it, ~line 2488) which calls
      `dom.insert_before(parent_id, child_id, reference_id)`, and `bridge_parent_node` (~line 3054).
  (2) A JS prelude string that defines prototype properties via `Object.defineProperty(...)`
      (search `Object.defineProperty(node, 'parentNode'`, ~line 1133). This is where `parentNode`,
      `nextSibling`, `insertBefore`, `appendChild` etc. are exposed on element objects.
- CRITICAL: `before`/`after` do NOT need any new Rust bridge. They are pure JS-side convenience methods
  expressible entirely in terms of primitives ALREADY exposed on the element prototype:
  `this.parentNode`, `this.nextSibling`, and `parentNode.insertBefore(newNode, referenceNode)`.
  Confirm those three are already exposed (they are: parentNode and insertBefore are registered; verify
  nextSibling exists too — search `'nextSibling'`. If nextSibling is NOT exposed, STOP and instead leave a
  `// TODO(spec):` explaining the missing primitive rather than adding a new bridge — keep this task to before/after only).

Semantics (WHATWG DOM, the Node-argument subset only):
- `element.before(node)`:
    if `this.parentNode` is null → no-op (return). Otherwise `this.parentNode.insertBefore(node, this)`.
- `element.after(node)`:
    if `this.parentNode` is null → no-op (return). Otherwise `this.parentNode.insertBefore(node, this.nextSibling)`.
    (When `this.nextSibling` is null, insertBefore with a null reference appends to the end — verify
    `bridge_insert_before` treats a null/undefined reference as "append at end"; the spec for insertBefore
    is: null reference => append. Confirm the Rust side handles `reference_id == None` as append.)
- Scope LIMIT (keep the task small and unambiguous): implement ONLY the single-Node-argument form.
  Do NOT implement DOMString arguments (text-node coercion) nor the variadic multi-node form.
  Add a `// TODO(spec): ChildNode.before/after — DOMString args and variadic nodes not yet supported`
  in the JS prelude near the new methods.

Implementation: add the two methods on the SAME element prototype object where `parentNode`/`insertBefore`
are attached, mirroring the existing `Object.defineProperty(node, '...')` style (use a non-enumerable
`value: function(node){ ... }` property, matching how neighboring methods are defined). Keep identifiers/comments English.
No `unwrap()`/`expect()` in non-test Rust (I-6) — but note most of this change is inside the JS prelude string.

Approach: test-first (TDD). Add unit tests in the existing `#[cfg(test)]` module of src/script/mod.rs.
Find an existing test that builds a DOM, runs a JS snippet via the engine, and asserts on the resulting tree
(search for tests around `insertBefore` / `appendChild` / `parentNode`, e.g. near line 4192). Copy that harness style.
Add at least:
- `before`: given `<div id=ref></div>` with a parent, JS creates a node and `ref.before(newNode)`, assert the
  new node is `ref.previousSibling` (or that parent's children order is [..., newNode, ref]).
- `after` with a following sibling: assert order becomes [ref, newNode, oldNext].
- `after` when ref is the LAST child: assert newNode becomes the last child (append path).
- `before`/`after` on a node with null parentNode is a no-op (does not throw).
- Keep ALL existing script tests green (do NOT weaken or delete any existing assertion or test — deleting/altering
  foreign tests to force green is a hard violation).

Done when: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.
Commit (you MUST git add + git commit before finishing — uncommitted work is lost):
  `feat(script): implement DOM ChildNode.before()/after() insertion (t0285)`
If the spec is ambiguous or conflicts with real browser behavior, do NOT guess — leave a `// TODO(spec):` and report it.
End with a short summary of exactly what changed and the test names you added.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
