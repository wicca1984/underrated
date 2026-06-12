#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0413
LOG=/workspaces/toy-browser/var/log/t0413.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0413 — expose typed `href` and `src` URL-attribute accessors on the DOM. Touch ONLY src/dom/mutate.rs. Do NOT edit any other file/module. If something truly needs another module, leave a `// TODO(spec): ...` and report instead.

Context (read before coding) — this mirrors EXACTLY the existing typed accessors in src/dom/mutate.rs such as `pub fn get_input_value(&self, node: NodeId) -> Option<String>` and `pub fn get_attribute(&self, node: NodeId, name: &str) -> Option<&str>`. Use `self.arena.get(node)?` and `if let NodeData::Element { name, attrs } = &n.data` and `attrs.iter().find(|(k, _)| k.eq_ignore_ascii_case("..."))`, exactly like `get_input_value` does. Return borrowed `Option<&str>` (like `get_attribute`), not owned Strings, to avoid needless allocation.

What to implement (two methods on `impl Dom`, placed near `get_input_value`):
1. `pub fn get_href(&self, node: NodeId) -> Option<&str>`:
   - Returns the value of the `href` content attribute, but ONLY for elements where `href` is a defined attribute: `<a>`, `<area>`, `<link>`, `<base>` (case-insensitive tag match).
   - Returns `None` if the node is invalid, not an element, not one of those tags, or has no `href` attribute.
2. `pub fn get_src(&self, node: NodeId) -> Option<&str>`:
   - Returns the value of the `src` content attribute, but ONLY for elements where `src` is a defined attribute: `<img>`, `<script>`, `<iframe>`, `<source>`, `<audio>`, `<video>`, `<embed>`, `<track>`, `<input>` (case-insensitive tag match).
   - Returns `None` if the node is invalid, not an element, not one of those tags, or has no `src` attribute.
Each method must return the raw attribute string (no URL resolution). Leave a `// TODO(spec):` noting that resolving these against the document base URL is out of scope and belongs to a higher layer.

Write small private helpers if useful (e.g. a tag-membership check), but keep everything in src/dom/mutate.rs. Document each public method with a `///` doc comment in the same style as `get_input_value`.

Panic-free: no unwrap/expect/panicking indexing in non-test code.

Tests — add to the existing `#[cfg(test)] mod tests` in src/dom/mutate.rs (do NOT modify or delete any existing test; mirror the setup style of the existing accessor tests that create Element nodes):
- `<a href="/foo">` => `get_href` returns `Some("/foo")`; `get_src` returns `None`.
- `<img src="a.png">` => `get_src` returns `Some("a.png")`; `get_href` returns `None`.
- `<link href="style.css">` and `<base href="https://x/">` => `get_href` returns the value.
- `<div href="x">` => `get_href` returns `None` (href not defined on div).
- An element of the right tag but missing the attribute => `None`.
- A non-element node (e.g. a Text node) => both return `None`.
- Case-insensitive tag name (e.g. `A` / `IMG`) is honored.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(dom): expose typed href and src URL-attribute accessors (t0413)"
Then print "T0413 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
