#!/usr/bin/env bash
# Launcher for Gemini worker t0288 — ChildNode.replaceWith() DOM binding.
# Target: src/script/mod.rs ONLY. Dispatched via setsid (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0288
LOG=/workspaces/toy-browser/var/worker-logs/t0288.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0288 — implement the DOM `ChildNode.replaceWith(node)` binding in the script host.
This is the natural sibling of the already-merged `ChildNode.before()` / `ChildNode.after()` (t0285) and
`ChildNode.remove()` (t0287). It replaces the node in its parent's child list with a single given node.

Target module: src/script/mod.rs ONLY. Do NOT touch any other file (no other src/ module, no fixtures, no
other worktree, no Cargo.toml). 1 task = 1 module (I-5). No `unwrap`/`expect` in non-test production paths (I-6).

Read first (verify every claim against the actual code before editing):
- src/script/mod.rs around lines 1269-1308: the existing `Object.defineProperty(node, 'before', {...})`,
  `'after'`, and `'remove'` JS shims installed on each wrapped DOM node inside `BoaHost`. They call host
  bridge methods like `this.parentNode`, `this.parentNode.insertBefore(...)`, `this.parentNode.removeChild(this)`.
  Match this EXACT pattern and placement — add your `'replaceWith'` property right after `'remove'`.
- The existing `insertBefore` / `removeChild` / `parentNode` / `nextSibling` bridges these shims rely on.
  Use ONLY bindings that already exist; verify each one is present before calling it from JS.

What to implement (mirror the before/after/remove scope EXACTLY — single Node arg, v1):
  Install on each node, via Object.defineProperty (enumerable:false, configurable:true, writable:true):
    replaceWith(newNode): {
      // TODO(spec): ChildNode.replaceWith() v1 — single Node arg only; variadic nodes and DOMString
      // arguments are out of scope (same limitation as before()/after()).
      if (!this.parentNode) return;        // no-op when detached (matches remove())
      this.parentNode.insertBefore(newNode, this);
      this.parentNode.removeChild(this);
    }
  Do NOT add variadic or string-to-text-node handling — keep parity with the existing ChildNode shims and
  leave the TODO(spec) marker so scope is explicit.

Add ONE Rust integration test next to the existing `test_dom_childnode_remove` (search for it, ~line 5510):
  `fn test_dom_childnode_replacewith()` that:
   - builds parent with children [a, b, c] (spans with textContent),
   - replaces b with a new span 'x' via `b.replaceWith(x)`,
   - asserts via the Rust DOM side that parent children are exactly [a, x, c] in order,
   - and asserts calling replaceWith on a detached node (no parent) is a silent no-op (does not throw).
  Use `dom.children(...)` and `dom.text_content(...)` exactly like `test_dom_childnode_remove` does.
  Test-side `unwrap`/`assert!` is fine (I-6 forbids unwrap only in non-test production code).

Do NOT delete, weaken, `#[ignore]`, or alter any existing test or assertion to force green (hard violation).

Done when (run from the worktree root /workspaces/wt/t0288):
  `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.

Commit (you MUST `git add -A && git commit` before finishing — uncommitted work is lost):
  `feat(script): implement ChildNode.replaceWith() DOM binding (t0288)`

End with a short summary: the exact JS shim you added, the bridge methods it calls, and confirmation that
the new test asserts [a, x, c] ordering plus the detached no-op case.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
