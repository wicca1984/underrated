#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0392
LOG=/workspaces/toy-browser/var/log/t0392.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0392 — expose a typed `tabindex` accessor on DOM nodes. Touch ONLY src/dom/mod.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in mod.rs if something truly needs another module.

Background (read before coding):
- Read src/dom/mod.rs, in particular the existing typed attribute accessors that you must MIRROR exactly in style:
  * `NodeData::role(&self) -> Option<&str>` (~line 35) and `NodeData::aria(&self, name) -> Option<&str>` (~line 46): they pattern-match `NodeData::Element { attrs, .. }`, scan `attrs` for the key, and return the value; non-elements return None.
  * `Dom::role(&self, node) -> Option<&str>` (~line 168) and `Dom::aria(&self, node, name)` (~line 173): the Dom-level wrappers, typically delegating to `get_attribute`.
- HTML `tabindex` is a "valid integer" attribute (may be negative, e.g. -1). Parse it to `i32`. If the attribute is absent, not an element, or the value is not a valid integer per Rust's `i32::from_str` on the trimmed string, return `None`. Reference: https://html.spec.whatwg.org/multipage/interaction.html#the-tabindex-attribute

Scope for THIS task (single file, typed accessor only):
1. Add `NodeData::tabindex(&self) -> Option<i32>`: for `NodeData::Element { attrs, .. }`, find the `tabindex` attribute, trim it, and parse with `str::parse::<i32>()` (or `i32::from_str`); return `Some(n)` on success, `None` otherwise. Non-elements return `None`. Mirror the exact style of `role`/`aria`.
2. Add `Dom::tabindex(&self, node: NodeId) -> Option<i32>`: read the `tabindex` attribute via the same mechanism the existing `Dom::aria`/`Dom::role` use (e.g. `self.get_attribute(node, "tabindex")`), then trim+parse to `i32`. Mirror `Dom::role`/`Dom::aria`.
3. Panic-free: no unwrap/expect/panicking indexing in non-test code; use `?`/Option combinators and `.ok()`.
4. Do NOT wire this into focus/tab-order logic or any other module — accessor only.

Tests — add to the existing `#[cfg(test)] mod tests` in src/dom/mod.rs (do NOT modify/delete existing tests; mirror `test_aria_attribute_retained` ~line 467 for setup: parse a small HTML snippet, locate the element id, assert via BOTH the Dom accessor and the NodeData accessor):
- `<a tabindex="3">x</a>` -> `Some(3)` via both `dom.tabindex(id)` and the node's `NodeData::tabindex()`.
- `<a tabindex="-1">x</a>` -> `Some(-1)`.
- `<a tabindex="  5 ">x</a>` (surrounding whitespace) -> `Some(5)`.
- `<a tabindex="abc">x</a>` -> `None` (invalid integer).
- `<a>x</a>` (no attribute) -> `None`.
Use whatever existing HTML-parse + node-lookup helpers the surrounding tests use (e.g. parsing into a `Dom`, then querying by tag) — do not invent new infrastructure.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(dom): expose typed tabindex (i32) attribute accessor (t0392)"
Then print "T0392 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
