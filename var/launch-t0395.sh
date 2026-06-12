#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0395
LOG=/workspaces/toy-browser/var/log/t0395.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0395 — expose a typed boolean `hidden` accessor on DOM nodes. Touch ONLY src/dom/mod.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in mod.rs if something truly needs another module.

Background (read before coding):
- Read src/dom/mod.rs. MIRROR EXACTLY the style of the recently added typed `tabindex` accessors:
  * `NodeData::tabindex(&self) -> Option<i32>` (~line 90): pattern-matches `NodeData::Element { attrs, .. }`, scans `attrs` for the key, parses, returns; non-elements return None.
  * `Dom::tabindex(&self, node) -> Option<i32>` (~line 226): the Dom-level wrapper, reading the attribute via the same mechanism `Dom::role`/`Dom::aria`/`get_attribute` use.
- HTML `hidden` is a BOOLEAN attribute: its mere PRESENCE means true, regardless of value (`hidden`, `hidden=""`, `hidden="hidden"`, even `hidden="false"` are all true per the spec — boolean attributes are true iff present). Absence means false. Reference: https://html.spec.whatwg.org/multipage/interaction.html#the-hidden-attribute

Scope for THIS task (single file, typed accessor only):
1. Add `NodeData::hidden(&self) -> bool`: for `NodeData::Element { attrs, .. }`, return true iff an attribute named `hidden` (compared ASCII-case-insensitively if the surrounding code lowercases attr names; otherwise match `"hidden"`) is present. Non-elements return false. Mirror the exact style of `NodeData::tabindex` but return a plain `bool` (presence test, no value parsing).
2. Add `Dom::hidden(&self, node: NodeId) -> bool`: check presence of the `hidden` attribute via the same mechanism the existing `Dom::tabindex`/`Dom::role` use (e.g. `self.get_attribute(node, "hidden").is_some()`). Mirror `Dom::tabindex`.
3. Panic-free: no unwrap/expect/panicking indexing in non-test code.
4. Do NOT wire this into layout/display:none/visibility or any other module — accessor only.

Tests — add to the existing `#[cfg(test)] mod tests` in src/dom/mod.rs (do NOT modify/delete existing tests; mirror `test_tabindex_attribute_accessor` ~line 657 for setup: parse a small HTML snippet, locate the element, assert via BOTH the Dom accessor and the NodeData accessor):
- `<div hidden>x</div>` -> `true` via both `dom.hidden(id)` and the node's `NodeData::hidden()`.
- `<div hidden="">x</div>` -> `true`.
- `<div hidden="false">x</div>` -> `true` (boolean attribute: presence wins, value irrelevant).
- `<div>x</div>` (no attribute) -> `false`.
Use whatever existing HTML-parse + node-lookup helpers the surrounding tests use; do not invent new infrastructure.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(dom): expose typed boolean hidden attribute accessor (t0395)"
Then print "T0395 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
