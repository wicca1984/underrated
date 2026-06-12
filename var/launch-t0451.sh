#!/usr/bin/env bash
# t0451 — fix missing repaint after input events (request_redraw) in src/shell/mod.rs. Base: origin/main e7052f2.
set -euo pipefail
cd /workspaces/wt/t0451

read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the code, run the checks, fix until green, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything is in local files. Network/web search is forbidden (cargo may fetch crates from crates.io — that is allowed and is NOT web search).

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. Read AGENTS.md (passed via --include-directories) and obey I-1..I-7. NEVER use unwrap()/expect() in non-test code (I-6). DO NOT delete or skip tests to fake green.

You are the ONLY worker on this worktree: /workspaces/wt/t0451, branch agent/t0451-input-redraw, base origin/main (commit e7052f2). Touch ONLY this file: src/shell/mod.rs. DO NOT touch any other module or file.

CONTEXT — THE BUG (real browser event-loop integration bug; human-flagged as the core MVP-Live gap):
In src/shell/mod.rs, the winit `ApplicationHandler::window_event` (around line 644-714) does this:
  - line 654: `if let Some(input_event) = map_window_event(&event, self.cursor_pos) { ... (self.on_event)(adjusted_event); }`
  - `map_window_event` returns Some ONLY for a Left mouse press (InputEvent::Click) and a key press (InputEvent::Key). These are exactly the events that MUTATE session state: focus changes, caret movement, and typed text in the focused <input>, plus Enter that triggers form submit + navigation.
  - PROBLEM: after `(self.on_event)(adjusted_event);` there is NO `request_redraw()` call. So when the user clicks an input or types a character or presses Enter to submit, the session state changes in memory but the window is NEVER repainted. The new caret, typed text, focus highlight, or post-navigation page only appear if the OS independently sends a RedrawRequested event (which may never come). This is "mutation without invalidation".
  - Contrast: the scroll branches DO call `request_redraw()` (see the MouseWheel branch ~line 678 and the arrow/page/space key scroll branch ~line 710, both guarded by `if let Some(state) = &self.state { state.window.request_redraw(); }`).

TASK — invalidate (request a repaint) immediately after an input event is dispatched.
1. In `window_event`, right AFTER the `(self.on_event)(adjusted_event);` line (inside the same `if let Some(input_event) = ...` block, after the callback runs), add a redraw request guarded by the same pattern used elsewhere:
     if let Some(state) = &self.state {
         state.window.request_redraw();
     }
   This ensures that every click and key press that reaches `on_event` triggers a repaint of the updated session. Because `map_window_event` returns None for CursorMoved/mouse-move and non-press events, this does NOT repaint on every mouse move — only on real clicks and key presses. Do NOT change `map_window_event`'s mapping. Do NOT touch the existing scroll redraw branches.

2. ADD unit tests (in the existing `#[cfg(test)]` test module of src/shell/mod.rs; if the test module lives inside the `winit_adapter` mod or at file scope, match the existing location/style) that LOCK the invariant "input events are recognized as redraw-triggering":
   - Construct or otherwise drive `map_window_event` for a Left `MouseButton` `Pressed` `MouseInput` event and assert it returns `Some(InputEvent::Click { .. })` (the click path that now triggers redraw).
   - For a `KeyboardInput` with `ElementState::Pressed`, assert it returns `Some(InputEvent::Key { .. })`.
   - For a `KeyboardInput` with `ElementState::Released`, assert it returns `None` (no redraw).
   If synthesizing a full winit `WindowEvent::KeyboardInput`/`MouseInput` in a test is impractical (winit's `KeyEvent`/device-id fields may be non-constructible outside winit), then DO NOT force it — instead check whether such `map_window_event` tests already exist; if they do, leave them. As a guaranteed-constructible alternative, add a tiny pure helper `fn input_event_triggers_redraw(ev: &InputEvent) -> bool` returning `true` for `InputEvent::Click{..}` and `InputEvent::Key{..}`, call it at the redraw site as the guard condition (so the production redraw decision goes through the helper), and unit-test the helper directly for both variants. Prefer whichever approach is genuinely testable without fragile winit type construction. Do NOT weaken or delete existing tests.

3. Keep the change minimal and within src/shell/mod.rs only.

PROCEDURE (iterate until all green):
  - cargo build
  - cargo fmt
  - cargo clippy --all-targets -- -D warnings   (fix every warning)
  - cargo test                                   (all pass)
  - git add -A && git commit -m "fix(shell): request repaint after click/key input events (t0451)"
  COMMIT before finishing (commit partial progress too). Report the final cargo test summary line and the exact lines you added.
EOF

exec gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
