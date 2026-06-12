#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0280
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0280 — Implement the DOM `Node.contains(otherNode)` JavaScript binding. This method returns `true` if `otherNode` is an INCLUSIVE descendant of the node it is called on (i.e. `node.contains(node)` is true, and `node.contains(child-or-deeper-descendant)` is true), and `false` otherwise. Per the DOM spec, `node.contains(null)` returns `false`. It is a deliberately small, single-module, additive JS binding, closely modeled on the existing `parentNode` binding. `Node.contains` is currently NOT implemented anywhere (verified: there is no `bridge_contains` and no `"contains"` Node method registration — do NOT confuse it with the already-implemented `classList.contains(token)`, which is unrelated).

Scope: touch ONLY `src/script/mod.rs` (including its inline `#[cfg(test)] mod tests`). `git diff --name-only` MUST show ONLY `src/script/mod.rs`. Do NOT modify dom, layout, style, or any other module/file, do NOT add to lib.rs, and do NOT touch any other worktree.

Reuse / facts (verified — read these in src/script/mod.rs before writing):
- The closest mirror is the `parentNode` binding. The Rust bridge fn `bridge_parent_node` is at ~line 2957. It is registered on the Node prototype/object via `NativeFunction::from_fn_ptr(bridge_parent_node)` + `JsString::from("parentNode")` at ~lines 256-257 (inside the ObjectInitializer block ~lines 140-359). The JS wrapper exposes it through `Object.defineProperty(node, ''parentNode'', { get() { return getOrCreateNode(bridge.parentNode(this.__key__)); } })` at ~line 1128, and there is a parallel definition on the `document` object at ~line 1364.
- DOM access from a bridge fn goes through the `with_dom(|dom| { ... })` helper at ~line 2127 (returns `Result<R, JsError>`). Read how `bridge_parent_node` extracts the caller node key from its JS args and maps it to a `NodeId` (via the key_to_node map) — mirror that EXACTLY for the first argument.
- The DOM (src/dom/mod.rs — read-only reference, do NOT edit it) provides `pub fn parent(&self, node: NodeId) -> Option<NodeId>` (line 118) and `pub fn descendants(&self, node: NodeId) -> Vec<NodeId>` (line 149). Prefer walking UP from `otherNode` via `dom.parent(...)` in a loop (O(depth)) over materializing all descendants.

Implement:
1. Add a Rust bridge fn `bridge_contains` next to `bridge_parent_node`. It takes TWO JS args: the caller node key (first, exactly like `bridge_parent_node`) and the `otherNode` key (second). Resolve both to `NodeId` inside `with_dom`. Semantics: return JS boolean `true` if `other == this_id`, or if walking `dom.parent(other)` upward eventually reaches `this_id`; otherwise `false`. If the second argument is null/undefined or its key does not resolve to a node, return `false` (per spec, `contains(null)` is `false` — do NOT throw for a null argument). Return a `JsValue::Boolean`.
2. Register it on the same Node object/prototype as `parentNode` (mirror lines 256-257) with the method name `contains` and arg count 1 (the JS-visible arity; the caller key is supplied by the wrapper, the single JS-visible param is `otherNode`).
3. In the JS wrapper, add a `contains(otherNode)` method to the node object created in `getOrCreateNode` (near the `parentNode` defineProperty ~line 1128) AND to the `document` object (near ~line 1364), shaped like: `contains(otherNode) { return bridge.contains(this.__key__, (otherNode && otherNode.__key__) || null); }`. Passing `null` through must make the bridge return `false` (do NOT throw on null).
4. Do NOT alter, weaken, or remove any existing binding, test, or assertion (deleting/altering foreign tests to force green is a hard violation, I-4). Keep all public interfaces stable except for the additive `contains` binding.
5. Spec scope (keep v1 tight): implement inclusive-descendant containment only. Do NOT attempt cross-document / shadow-tree / DocumentFragment host semantics. Leave exactly one `// TODO(spec): Node.contains v1 handles inclusive-descendant containment within a single document tree; cross-document, shadow-root, and DocumentFragment host edge cases are out of scope.` comment near `bridge_contains`.

Do NOT use unwrap/expect/panic/unsafe in non-test code (I-6). No new dependencies (I-1). Do NOT change iterative code back to recursive.

Acceptance — add inline unit/integration tests in `src/script/mod.rs` (reuse the existing JS-evaluation test harness used by the other DOM-binding tests in this file; mirror an existing test that builds a small DOM and runs a JS snippet asserting a boolean result):
- A parent element `contains` its direct child: `parent.contains(child) === true`.
- A node `contains` itself (inclusive): `el.contains(el) === true`.
- A node does NOT contain an unrelated sibling / ancestor: `child.contains(parent) === false` and two siblings do not contain each other.
- A node `contains` a deeper (grand-child) descendant: `true`.
- `node.contains(null) === false` (no throw).
Do NOT weaken or remove any existing test.

Done when ALL of these pass in this worktree:
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Comments and identifiers in English.
Commit (you MUST git add + git commit BEFORE finishing — uncommitted work is lost when the worktree is removed): `git add -A && git commit -m "feat(script): implement Node.contains(otherNode) DOM binding (t0280)"`.
End with a short English summary of exactly what changed in src/script/mod.rs (the bridge fn, the registration, the JS wrapper methods), the `// TODO(spec):` you left, the test names you added, and confirm you committed.'
