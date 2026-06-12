#!/usr/bin/env bash
# Launcher for Gemini worker t0293 — B-4 stray-box fragment fix (zero-area background SolidRect guard).
# Target: src/paint/mod.rs ONLY. Dispatched via setsid (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0293
LOG=/workspaces/toy-browser/var/worker-logs/t0293.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0293 — MS-Regression-Google B-4 (stray "lost little box" fragments). Symptom: tiny empty boxes / dot
rows are painted near the buttons and to the left of the search box on the Google top page. Root cause to fix:
the background-color SolidRect for a box is emitted UNCONDITIONALLY, even when the box has zero (or negative)
width or height. Empty / zero-size elements that carry a background-color therefore emit a 0-area (or
degenerate) SolidRect — a stray fragment. Border edges are ALREADY guarded against empty/negative rects in
this file; the background rect is NOT. Add the same guard, and lock it down with a test.

Target file: src/paint/mod.rs ONLY (production code + its in-file `#[cfg(test)] mod tests`). Do NOT touch any
other file (no other src/ module, no other test file, no fixtures, no Cargo.toml, no other worktree).
1 task = 1 module (I-5). Production code must NOT use `unwrap`/`expect` (I-6); test code may.

Read first and VERIFY every claim against the ACTUAL code before editing (do not trust this prompt blindly):
- In `pub fn build_display_list(...)`, find the block guarded by
  `if let Some(CssValue::Color(color)) = style.get("background-color") {` (around line 430). It does
  `items.push(DisplayItem::SolidRect { rect: layout_box.rect, color: scale_color_alpha(color, effective_opacity) });`
  with NO size check.
- A few lines below, the border-edge emission wraps each `items.push(DisplayItem::SolidRect { ... })` in a
  guard like `if top_rect.size.width > 0.0 && top_rect.size.height > 0.0 { ... }`. Match that exact style.
- `layout_box.rect` is a `Rect` whose `.size.width` / `.size.height` are `f32`. Confirm the field path against
  the real `Rect` type used in this file before relying on it.

Change (production):
  Wrap the background SolidRect push so it ONLY emits when
  `layout_box.rect.size.width > 0.0 && layout_box.rect.size.height > 0.0`.
  Keep behavior otherwise identical (same color, same scale_color_alpha, same rect). Do not change border,
  image, or text emission. Add a short comment: `// B-4: do not paint background for zero/negative-area boxes`.

Test (add to the in-file tests module — do NOT delete, weaken, `#[ignore]`, or alter ANY existing test):
  Add ONE test `fn test_zero_area_box_emits_no_background_rect()` that builds a layout where an element has a
  background-color but a zero-area rect (width == 0 OR height == 0), runs `build_display_list`, and asserts
  that NO `DisplayItem::SolidRect` is produced for that degenerate box. Construct the inputs the SAME way the
  existing tests in this module do (look at how they build a `LayoutBox`, `Dom`, and the
  `HashMap<NodeId, ComputedStyle>` and call `build_display_list`). ALSO assert (positive control) that an
  otherwise-identical element with a NON-zero area DOES emit exactly one background SolidRect — so the test is
  not vacuously passing. Reuse the existing test helpers/patterns; do not invent a new harness.

If any field path / type / helper name does not match this description, TRUST THE CODE, not this prompt, and
adapt — but keep the intent: zero/negative-area boxes must not emit a background SolidRect, non-empty boxes
still must.

Done when (run from the worktree root /workspaces/wt/t0293):
  `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.

Commit (you MUST `git add -A && git commit` before finishing — uncommitted work is lost; the worktree may be
force-removed after you exit):
  `fix(paint): suppress background SolidRect for zero-area boxes (t0293)`

End with a short summary: the exact guard condition you added, the Rect field path you confirmed, and the
SolidRect counts your new test observed for the zero-area vs non-zero-area cases.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
