#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0405
LOG=/workspaces/toy-browser/var/log/t0405.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0405 — add typed `disabled()` (boolean) and `value()` (string) attribute accessors to the DOM. Touch ONLY src/dom/mod.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` if something truly needs another module.

Background (read before coding):
- src/dom/mod.rs already has typed accessors `tabindex()`, `hidden()`, `colspan()`, `rowspan()`, each defined TWICE: once as a method on `NodeData` (around lines 90-128) and once as a method on `Dom` taking a `NodeId` (around lines 256-280). Read those to mirror their exact style.
- Model `hidden()` (a boolean accessor based on attribute presence) and `tabindex()` (a parsed accessor) precisely.

Scope for THIS task (single file, src/dom/mod.rs):
1. On `impl NodeData`, add:
   - `pub fn disabled(&self) -> bool` — true iff this is an `Element` whose attrs contain a `disabled` attribute (presence only; mirror `hidden()` exactly, which uses `get_attribute(...).is_some()` pattern on Dom side, but on NodeData side mirror how `hidden()` is implemented on NodeData — check attrs directly).
   - `pub fn value(&self) -> Option<&str>` — returns the `value` attribute's string value if present on an `Element` (mirror the `attrs.iter().find(|(k,_)| k == "value").map(|(_, v)| v.as_str())` style; return the raw string, do NOT trim/parse).
2. On `impl Dom`, add:
   - `pub fn disabled(&self, node: NodeId) -> bool` — mirror `hidden(&self, node)`: `self.get_attribute(node, "disabled").is_some()`.
   - `pub fn value(&self, node: NodeId) -> Option<&str>` — `self.get_attribute(node, "value")` (return type matching whatever `get_attribute` yields; if `get_attribute` returns `Option<&str>` return it directly, else `.map(|v| v.as_str())`). Verify `get_attribute`'s signature first and match it.
3. Add `///` doc comments in the same terse style as the neighboring accessors (rustdoc warnings are denied in CI, so every new pub fn needs a doc line).
4. Panic-free: no unwrap/expect/panicking indexing in non-test code.

Tests — add to the existing `#[cfg(test)] mod tests` in src/dom/mod.rs (do NOT modify/delete existing tests; mirror `test_colspan_rowspan_attribute_accessors` / the hidden test for setup style, using `parse_document` + `InputStream::from_utf8` and `query_selector`):
- `<input disabled>` → `dom.disabled(id)` is true and `data.disabled()` is true; `<input>` → false.
- `<input value="hi">` → `dom.value(id)` == Some("hi") and `data.value()` == Some("hi"); `<input>` (no value) → None.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(dom): expose typed disabled and value attribute accessors (t0405)"
Then print "T0405 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
