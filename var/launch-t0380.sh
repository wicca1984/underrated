#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0380
LOG=/workspaces/toy-browser/var/log/t0380.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0380 — add a session history stack (back/forward primitive) to BrowsingContext.

Target file: src/engine/mod.rs ONLY. Touch ONLY src/engine/mod.rs. Do NOT edit main.rs, loader/, dom/, layout/, paint/, forms/, src/engine/flush.rs, or any other file/module/worktree. Use ONLY existing public APIs. If something genuinely requires another module, leave a `// TODO(spec): ...` comment in src/engine/mod.rs and stop — do not edit other files.

Context (already in src/engine/mod.rs):
- `pub struct BrowsingContext { pub page: Page, pub focus_node: Option<NodeId>, pub caret_index: usize, pub scroll_y: f32, pub content_height: f32 }`.
- `impl BrowsingContext { pub fn new(page: Page) -> Self {...}; pub fn navigate(&mut self, page: Page) {...}; ... }`.
- `use crate::url::Url;` is already imported at the top of the file. `Url` is the engine's URL type; `Url::parse(&str) -> Result<Url, _>` exists (see main.rs usage). Treat `Url` as an owned, `Clone`-able value.

Goal: maintain a back/forward URL history INSIDE BrowsingContext as a pure data structure. This is the engine-side primitive ONLY; the main.rs window wiring (Alt+Left etc.) is a SEPARATE follow-up task — do NOT touch main.rs. Leave a `// TODO(spec): wire Alt+Left/back navigation in main.rs to drive go_back()` comment near the new methods.

Implement (ALL in src/engine/mod.rs):
1. Add two private fields to `BrowsingContext`:
   - `history: Vec<Url>` — the visited-URL stack (oldest first).
   - `history_index: usize` — index into `history` of the CURRENTLY displayed entry. When `history` is empty, treat the context as having no current entry.
   Keep these fields PRIVATE (no `pub`) so the invariant is owned by the methods below. (Existing pub fields stay pub.)
2. Update `BrowsingContext::new(page)` to initialize `history: Vec::new()` and `history_index: 0`. Do NOT change its signature.
3. Add these public methods on `impl BrowsingContext`:
   - `pub fn push_history(&mut self, url: Url)` — records a NEW forward navigation. Truncate any forward entries (drop everything after `history_index` when history is non-empty), push `url`, and set `history_index` to the index of the just-pushed entry (`history.len() - 1`). On the very first push (empty history), index becomes 0.
   - `pub fn can_go_back(&self) -> bool` — true iff there is a previous entry (`!history.is_empty() && history_index > 0`).
   - `pub fn can_go_forward(&self) -> bool` — true iff there is a next entry (`!history.is_empty() && history_index + 1 < history.len()`).
   - `pub fn go_back(&mut self) -> Option<Url>` — if `can_go_back()`, decrement `history_index` and return `Some(history[history_index].clone())`; else `None`. (Caller re-fetches/renders that URL and calls `navigate(new_page)`.)
   - `pub fn go_forward(&mut self) -> Option<Url>` — symmetric: if `can_go_forward()`, increment `history_index` and return `Some(history[history_index].clone())`; else `None`.
   - `pub fn current_url(&self) -> Option<&Url>` — `history.get(history_index)` (None when history empty).
   Do NOT modify the existing `navigate(&mut self, page: Page)` method body (it must remain history-agnostic so a back/forward-driven `navigate` does NOT itself push history). Place a `// TODO(spec): wire Alt+Left/back navigation in main.rs to drive go_back()` comment by the new methods.
4. Keep everything panic-free (AGENTS.md I-6): NO `unwrap()`/`expect()`/indexing that can panic in non-test code. Use `.get()`, `if`, `match`. (Indexing `history[history_index]` is acceptable ONLY where a preceding `can_go_*` guard proves it in-bounds; prefer `.get(...).cloned()` to be safe.)

Add unit tests in a `#[cfg(test)] mod tests` block in src/engine/mod.rs (create the block if absent; if one exists, add to it). To build a `BrowsingContext` you need a `Page`; construct a minimal one. Look at how existing tests/code build a `Page` (it is `Page { dom, styles, layout }`). The SIMPLEST route: use `render_page` with empty HTML and a stub loader the same way other engine tests do — search this file and tests/ for an existing pattern (e.g. a `NullLoader`/`StubLoader` implementing `ResourceLoader`, or `render_page(\"\", &base, &loader, 800.0)`). Reuse whatever pattern already compiles; do NOT invent a new loader if one exists. If building a Page is genuinely heavy, you may build it once via a small local helper inside the test module.
Tests to add (use `Url::parse(\"https://a.test/\").expect(...)` etc. — expect() is fine IN TESTS):
- `test_history_push_and_current`: new context has `current_url() == None` and `!can_go_back()`. After `push_history(a)`, `current_url() == Some(&a)`, `!can_go_back()`, `!can_go_forward()`. After `push_history(b)`, `current_url()==Some(&b)`, `can_go_back()`, `!can_go_forward()`.
- `test_go_back_and_forward`: push a, b, c. `go_back()` returns `Some(b)` and current becomes b; `go_back()` returns `Some(a)`; `go_back()` returns `None` (at oldest). Then `go_forward()` returns `Some(b)`, `go_forward()` returns `Some(c)`, `go_forward()` returns `None`.
- `test_push_truncates_forward`: push a, b, c; go_back to a (two go_back calls); then `push_history(d)`. Now `current_url()==Some(&d)`, `can_go_back()` true, `can_go_forward()` false, and a further `go_back()` returns `Some(a)` (the forward entries b,c were dropped).

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(engine): add back/forward history stack to BrowsingContext (t0380)"
Then print "T0380 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
