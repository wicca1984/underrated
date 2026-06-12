#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0375
LOG=/workspaces/toy-browser/var/log/t0375.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0375 — wire the interactive browsing session loop in src/main.rs.

Target module: src/main.rs (touch ONLY this file; do not touch other modules or other worktrees).

Goal: replace the current one-shot `window.run(move || render_page_to_canvas(...))` call in `fn main()` with an interactive loop using the EXISTING building blocks, so the user can click an <input>, type a query, and press Enter to navigate. This is a pure INTEGRATION task — all parts already exist; do not reimplement them.

Existing parts you MUST reuse (read them first to learn exact signatures):
- `crate::shell::WinitWindow::run_with_input(draw: FnMut() -> Canvas, on_event: FnMut(InputEvent))` in src/shell/mod.rs (~line 392).
- `crate::shell::InputEvent` enum (src/shell/mod.rs ~line 9): variants `Click { x, y }` and `Key { key: String }`.
- `crate::shell::ShellInputManager` (src/shell/mod.rs ~line 58): `new()`, `focus(NodeId)`, `blur()`, `focused_element()`, `handle_click(x, y, hit_test: Option<NodeId>)`, `text_buffer(NodeId)`, `set_text_buffer(NodeId, String)`, `caret_position(NodeId)`, `set_caret_position(NodeId, usize)`.
- `crate::layout::hit_test(root: &LayoutBox, x: f32, y: f32) -> Option<NodeId>` (src/layout/mod.rs ~line 1155).
- `crate::engine::navigate_from_enter(dom, focused: NodeId, values: &FormState, base: &Url, loader: &dyn ResourceLoader, viewport_width: f32) -> Option<Page>` (src/engine/mod.rs ~line 435).
- `crate::engine::BrowsingContext` (src/engine/mod.rs): owns the current `Page` plus `focus_node`, `caret_index`, `scroll_y`, `content_height`; methods `new(page)`, `navigate(page)`, `set_focus(Option<NodeId>)`. Use this to hold the live session state.
- The caret-aware render entry point added in t0372 (`build_display_list_with_caret`) and `render_page_to_canvas` in src/engine — read src/engine to find the right function to turn the current Page + focus/caret into a Canvas.

Implementation outline (adapt to real signatures — do not invent APIs):
1. After the initial fetch + first render, build the initial `Page` and wrap it in a `BrowsingContext`.
2. Create a `ShellInputManager` and a `FormState` to accumulate typed values.
3. Share the mutable session state (BrowsingContext + ShellInputManager + FormState) between the `draw` and `on_event` closures. Because `run_with_input` takes two closures, use interior mutability (`std::rc::Rc<std::cell::RefCell<...>>`) — NOT unsafe, NOT a second thread. Clone the `Rc` into each closure.
4. `draw` closure: borrow the session, render the current Page to a Canvas reflecting the focused input's text buffer + caret position.
5. `on_event` closure:
   - `Click { x, y }`: run `hit_test` against the current layout, call `ShellInputManager::handle_click`, and update `BrowsingContext` focus via `set_focus`.
   - `Key { key }`: if an input is focused, mutate its text buffer (append printable chars, handle Backspace) and advance/retreat the caret; sync the value into `FormState`. If `key` is "Enter"/"Return", call `navigate_from_enter`; on `Some(page)` call `browsing_context.navigate(page)` so the next `draw` shows the result page.

Hard constraints (AGENTS.md I-1..I-7):
- Touch ONLY src/main.rs. Do NOT modify shell, engine, layout, paint, or any other module. If a needed helper is genuinely missing, leave a `// TODO(spec): <what>` comment and adapt rather than editing another module.
- NO `unwrap()`/`expect()` in non-test code (I-6) — use `match`/`if let`/`?` and graceful fallbacks (the existing code returns/logs on error).
- Do NOT skip or delete any test. Keep all existing #[test] in src/main.rs.
- Keep `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` green.
- Note: the winit event loop itself cannot be unit-tested headlessly; rely on `cargo build` + clippy + existing tests passing. The decision-logic regression guard is a separate tests/ task — do not add it here.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(main): wire interactive browsing session loop (click/type/Enter) (t0375)"
Then print "T0375 DONE" as the last line.
EOF
exec gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
