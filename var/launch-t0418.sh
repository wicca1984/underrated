#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0418
LOG=/workspaces/toy-browser/var/log/t0418.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code — note: test code MAY use unwrap/panic as the existing tests do).

Task t0418 — implement the DOM `Node.contains(other)` query method. Touch ONLY the file `src/dom/query.rs` (add the method to the existing `impl Dom` block there, plus unit tests in that same file's `#[cfg(test)] mod tests` if one exists; if there is no test module in query.rs, add one). Do NOT edit any other file.

Spec (https://dom.spec.whatwg.org/#dom-node-contains):
`node.contains(other)` returns true if `other` is an INCLUSIVE DESCENDANT of `node`, i.e. `other == node` OR `other` is a descendant of `node`. If `other` is None/invalid in the DOM tree, the result is false. The method must NOT panic.

Implement exactly this signature in `impl Dom` in `src/dom/query.rs`:
```
/// Returns true if `other` is an inclusive descendant of `node`
/// (i.e. `other` is `node` itself, or a descendant of `node`).
// spec: https://dom.spec.whatwg.org/#dom-node-contains
pub fn contains(&self, node: NodeId, other: NodeId) -> bool { ... }
```
Implementation guidance: walk UP from `other` via `self.parent(..)` (an existing method: `pub fn parent(&self, node: NodeId) -> Option<NodeId>`). Start at `other`; if it equals `node` return true; otherwise follow parents until you either reach `node` (return true) or run out of parents (return false). This is O(depth) and cannot infinite-loop on a well-formed tree. Do NOT use unwrap/expect in the method body.

Existing facts you can rely on (already in the codebase, do not redefine):
- `use crate::infra::NodeId;` is already imported at the top of query.rs.
- `Dom::parent(&self, node: NodeId) -> Option<NodeId>` exists (src/dom/mod.rs).
- `Dom::document(&self) -> NodeId`, `Dom::create_node`, `Dom::append_child(parent, child)` exist for building test trees. Look at the existing tests in `src/dom/mod.rs` (e.g. `test_descendants`) for the exact construction pattern (NodeData::Element { name, attrs, .. } / NodeData::Text(..)). Mirror that style.

Add focused unit tests in src/dom/query.rs covering:
1. A node contains itself (`contains(n, n) == true`).
2. A parent contains its direct child.
3. An ancestor contains a deep (grand+) descendant.
4. A node does NOT contain its own ancestor (`contains(child, parent) == false`).
5. Two sibling subtrees do not contain each other.
6. The document root contains every node in the tree.

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(dom): add Node.contains inclusive-descendant query (t0418)"
Then print "T0418 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
