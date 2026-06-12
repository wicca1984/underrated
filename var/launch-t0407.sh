#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0407
LOG=/workspaces/toy-browser/var/log/t0407.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0407 — add typed `checked()` and `selected()` boolean attribute accessors to the DOM. Touch ONLY src/dom/mod.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` if something truly needs another module.

Background (read before coding):
- src/dom/mod.rs already has boolean presence accessors `hidden()` and `disabled()`, each defined TWICE: once as a method on `NodeData` (around lines 101-115) and once as a method on `Dom` taking a `NodeId` (around lines 281-290). Read those to mirror their exact style — `checked` and `selected` are HTML boolean attributes, identical in shape to `disabled` (presence-only).

Scope for THIS task (single file, src/dom/mod.rs):
1. On `impl NodeData`, add (mirror `disabled()` on NodeData exactly):
   - `pub fn checked(&self) -> bool` — true iff this is an `Element` whose attrs contain a `checked` attribute (presence only).
   - `pub fn selected(&self) -> bool` — true iff this is an `Element` whose attrs contain a `selected` attribute (presence only).
2. On `impl Dom`, add (mirror `disabled(&self, node)` exactly):
   - `pub fn checked(&self, node: NodeId) -> bool` — `self.get_attribute(node, "checked").is_some()`.
   - `pub fn selected(&self, node: NodeId) -> bool` — `self.get_attribute(node, "selected").is_some()`.
3. Add `///` doc comments in the same terse style as the neighboring `disabled()`/`hidden()` accessors (rustdoc warnings are denied in CI, so every new pub fn needs a doc line).
4. Panic-free: no unwrap/expect/panicking indexing in non-test code.

Tests — add to the existing `#[cfg(test)] mod tests` in src/dom/mod.rs (do NOT modify/delete existing tests; mirror the existing disabled/value accessor test for setup style, using the same document-parsing + `query_selector` helpers the neighboring tests use):
- `<input checked>` → `dom.checked(id)` is true and `data.checked()` is true; `<input>` → both false.
- `<option selected>` → `dom.selected(id)` is true and `data.selected()` is true; `<option>` → both false.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(dom): expose typed checked and selected boolean attribute accessors (t0407)"
Then print "T0407 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
