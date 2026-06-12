#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0409
LOG=/workspaces/toy-browser/var/log/t0409.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0409 — add `toggle` and `replace` DOMTokenList methods to classList. Touch ONLY src/dom/classlist.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` if something truly needs another module.

Background (read before coding):
- src/dom/classlist.rs already implements `class_list`, `has_class`, `add_class`, `remove_class` on `impl Dom`. Read `add_class` and `remove_class` (around lines 46-80) and mirror their exact style: early-return on empty `name`, gate on `Some(NodeData::Element { .. }) = self.data(node)`, compute via `self.class_list(node)`, then `self.set_attribute(node, "class", &new_value)` with a space-joined value.

Scope for THIS task (single file, src/dom/classlist.rs):
1. `pub fn toggle_class(&mut self, node: NodeId, name: &str) -> bool` — DOMTokenList.toggle without a force argument:
   - no-op returning `false` if `name` is empty or node is not an Element.
   - if the class is present, remove all its occurrences and return `false`.
   - if absent, add it (append) and return `true`.
   - normalize the resulting `class` attribute to space-separated, exactly like add_class/remove_class.
   - spec comment: // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-toggle
2. `pub fn replace_class(&mut self, node: NodeId, old: &str, new: &str) -> bool` — DOMTokenList.replace:
   - no-op returning `false` if `old` or `new` is empty, or node is not an Element, or `old` is not currently present.
   - otherwise replace every occurrence of `old` with `new` (dedupe: if `new` already present, just drop `old`), set the normalized space-joined value, and return `true`.
   - spec comment: // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-replace
3. Add `///` rustdoc comments in the same terse style as the neighboring add_class/remove_class (rustdoc warnings are denied in CI, so every new pub fn needs a doc line).

Panic-free: no unwrap/expect/panicking indexing in non-test code.

Tests — add to the existing `#[cfg(test)] mod tests` in src/dom/classlist.rs (do NOT modify/delete existing tests; reuse the existing `elem` helper and class_list assertions):
- toggle on an element without the class adds it and returns true; toggling again removes it and returns false.
- toggle on a non-element returns false and is a no-op.
- replace_class("foo","bar") on `class="foo baz"` yields `["bar","baz"]` and returns true.
- replace_class for an absent `old` returns false and leaves the list unchanged.
- replace_class with empty `old` or `new` returns false (no-op).

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(dom): add classList toggle and replace token-list methods (t0409)"
Then print "T0409 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
