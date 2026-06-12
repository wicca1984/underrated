#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0297
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0297 — Implement the DOM `Node.contains(other)` JavaScript binding. It returns a boolean: `true` if `other` is `this` node itself or an inclusive descendant of `this` (i.e. `this` is an ancestor-or-self of `other`); otherwise `false`. Per the DOM spec, `node.contains(node)` is `true`, and `node.contains(null)` is `false`. This is NOT currently implemented (verified: there is no `contains` method on the node wrapper object — `grep "''contains''" src/script/mod.rs` returns nothing).

Scope: touch ONLY `src/script/mod.rs` (including its inline `#[cfg(test)] mod tests`). `git diff --name-only` MUST show ONLY `src/script/mod.rs`. Do NOT modify dom, layout, style, paint, or any other module/file, do NOT add to lib.rs, and do NOT touch any other worktree.

Reuse / facts (verified — read these in src/script/mod.rs BEFORE writing):
- Node wrappers are created inside `getOrCreateNode`. Each wrapper already exposes a `parentNode` getter (defined via `Object.defineProperty(node, ''parentNode'', {...})` at ~line 1133) and an internal `__key__` property that uniquely identifies the underlying DOM node (used for node identity comparisons elsewhere, e.g. the existing `replaceChildren` / `before`/`after` bindings compare/insert by `__key__`).
- The CLEANEST implementation is purely in the JS wrapper (NO new Rust bridge fn), mirroring the shape of existing wrapper methods like `before`/`after`/`replaceChildren`: `Object.defineProperty(node, ''contains'', { value: function(other){ ... }, enumerable:false, configurable:true, writable:true })`.
- Implement by walking UP from `other` using the `parentNode` getter: start at `cur = other`; while `cur` is non-null, if `cur.__key__ === this.__key__` return `true`, else `cur = cur.parentNode`; if the loop ends return `false`. Handle `other == null`/`undefined` by returning `false` BEFORE the loop. This makes `contains(self)` true (inclusive) and walks ancestors correctly. Confirm `parentNode` returns null/undefined at the document/root boundary so the loop terminates (read the parentNode getter to verify its top-of-tree return value, and terminate on whatever falsy value it yields).

Implement (JS-wrapper-only):
1. Inside `getOrCreateNode`, add ONE `Object.defineProperty(node, ''contains'', { value: function(other){ ... }, enumerable:false, configurable:true, writable:true })` block, placed alongside the other ChildNode/Node wrapper method bindings (e.g. right after the `replaceChildren` block).
2. Leave exactly one `// TODO(spec): Node.contains() v1 — ancestor-or-self walk via parentNode; cross-document / shadow-tree edge cases out of scope.` comment above the new block.
3. Do NOT alter, weaken, or remove any existing binding, test, or assertion (deleting/altering foreign tests to force green is a hard violation, I-4). Keep all public interfaces stable except for the additive `contains` binding.

Do NOT use unwrap/expect/panic/unsafe in non-test code (I-6). No new dependencies (I-1). Do NOT change iterative code back to recursive.

Acceptance — add inline integration test(s) in `src/script/mod.rs` (reuse the existing JS-evaluation test harness; mirror an existing DOM test such as `test_dom_parentnode_replace_children` which builds a small DOM, runs a JS snippet, then asserts results — for contains, have the JS snippet store boolean results into element textContent or use multiple asserts via separate evals, then assert from the Rust side):
- self: `parent.contains(parent)` is `true`.
- descendant: build parent > child > grandchild; `parent.contains(grandchild)` is `true` and `parent.contains(child)` is `true`.
- non-descendant: a sibling/unrelated node `other`; `parent.contains(other)` is `false`.
- null: `parent.contains(null)` is `false`.
  (You may surface each boolean to Rust by assigning e.g. `parent.textContent = String(parent.contains(grandchild))` on a scratch element, or by building a small results string; pick whatever the existing harness supports and assert via `dom.text_content(...)`.)
Do NOT weaken or remove any existing test.

Done when ALL of these pass in this worktree (run from the worktree root /workspaces/wt/t0297):
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Comments and identifiers in English.
Commit (you MUST git add + git commit BEFORE finishing — uncommitted work is lost when the worktree is removed): `git add -A && git commit -m "feat(script): implement Node.contains() DOM binding (t0297)"`.
End with a short English summary of exactly what changed in src/script/mod.rs (the new `contains` defineProperty block and the `// TODO(spec):` you left), the test name(s) you added, and confirm you committed.'
