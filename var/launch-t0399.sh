#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0399
LOG=/workspaces/toy-browser/var/log/t0399.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0399 — expose typed accessors for the `colspan` and `rowspan` table-cell attributes. Touch ONLY src/dom/mod.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in mod.rs if something truly needs another module.

Background (read before coding):
- Read src/dom/mod.rs. There is an EXISTING pattern for typed attribute accessors. On `NodeData` (an enum with an `Element { attrs, .. }` variant), see `pub fn tabindex(&self) -> Option<i32>` (~line 90) and `pub fn hidden(&self) -> bool` (~line 101). tabindex parses an attribute string into `i32` via `.trim().parse::<i32>().ok()`.
- There is ALSO a thin wrapper layer on the Dom struct itself that forwards by NodeId: see `pub fn tabindex(&self, node: NodeId) -> Option<i32>` (~line 234) and `pub fn hidden(&self, node: NodeId) -> bool` (~line 240). MIRROR both layers (the NodeData inherent method AND the Dom-by-NodeId forwarder) exactly for the new accessors.

Scope for THIS task (single file, src/dom/mod.rs):
1. On `NodeData`, add `pub fn colspan(&self) -> Option<u32>` returning the parsed `colspan` attribute value (mirror the tabindex impl but parse `u32` instead of `i32`, since colspan/rowspan are non-negative). Add `pub fn rowspan(&self) -> Option<u32>` likewise for `rowspan`.
2. On the Dom struct, add the by-NodeId forwarders `pub fn colspan(&self, node: NodeId) -> Option<u32>` and `pub fn rowspan(&self, node: NodeId) -> Option<u32>`, mirroring the existing `tabindex(&self, node)` forwarder (look up the node's data and delegate to the NodeData method).
3. Doc-comment each new method in the same one-line `///` style as tabindex/hidden (RUSTDOCFLAGS=-D warnings is enforced; keep docs valid).
4. Panic-free: no unwrap/expect/panicking indexing in non-test code; use Option combinators.

Tests — add to the existing `#[cfg(test)] mod tests` in src/dom/mod.rs (do NOT modify/delete existing tests; mirror the existing tabindex/hidden accessor tests for setup style — build a small DOM with an element carrying the attribute, then assert the accessor return value):
- An element with `colspan="3"` returns `Some(3)` from both the NodeData method and the Dom-by-NodeId forwarder.
- An element with `rowspan="2"` returns `Some(2)`.
- An element WITHOUT the attribute returns `None`.
- An element with a non-numeric / invalid value (e.g. `colspan="abc"`) returns `None`.
Use whatever existing DOM-construction helpers the surrounding tests use; do not invent new infrastructure.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(dom): expose typed colspan/rowspan (u32) attribute accessors (t0399)"
Then print "T0399 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
