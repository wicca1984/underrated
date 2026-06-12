#!/usr/bin/env bash
# Launcher for Gemini worker t0284 — paint CSS `outline-offset` (src/paint/mod.rs).
# Dispatched via setsid so it survives the orchestrator tick (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0284
LOG=/workspaces/toy-browser/var/worker-logs/t0284.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0284 — Paint CSS `outline-offset`: draw the outline at a gap (offset) outside the border edge.
Read: docs/SPEC.md (paint / css-ui outline) and docs/ARCHITECTURE.md under /workspaces/underrated-meta/.
Target module: src/paint/mod.rs ONLY. Do NOT touch any other module or file (no lib.rs/style/css changes), and do NOT touch any other worktree.

Background (verify before changing — read the code first):
- The outline is painted in the `if has_outline { ... }` block in src/paint/mod.rs (search `has_outline`, around line 496). It currently draws four solid strips (top/bottom/left/right) of thickness `ow` (from `get_outline_width(style)`) flush against the border edge — e.g. the top strip is `Rect::new(x - ow, y - ow, w + 2.0 * ow, ow)`.
- There is a `// TODO(spec):` comment right there noting `outline-offset != 0` is not implemented.
- `outline-offset` is a standalone longhand (NOT part of the `outline` shorthand). The style layer stores declared properties generically, so `style.get("outline-offset")` returns the parsed `CssValue` when the author declared it. Confirm this by checking how `style.get("outline-width")` is read in `get_outline_width`.

Goal (css-ui §outline-offset):
- Add a helper `fn get_outline_offset(style: &ComputedStyle) -> f32` mirroring the style of `get_outline_width`: read `style.get("outline-offset")`, return px for `CssValue::Length(v, LengthUnit::Px)` and `CssValue::Number(v)`, and default to `0.0` for anything else / absent. `outline-offset` MAY be negative — do NOT clamp it to >= 0.
- The outline is drawn `offset` pixels OUTSIDE the border edge. So the outer rectangle of the outline expands by `(offset + ow)` instead of `ow`. Concretely, define `let d = offset + ow;` and replace the per-strip positioning so each strip sits at distance `offset` outside the box with thickness `ow`:
  - top:    `Rect::new(x - d, y - d, w + 2.0 * d, ow)`
  - bottom: `Rect::new(x - d, y + h + offset, w + 2.0 * d, ow)`
  - left:   `Rect::new(x - d, y - d, ow, h + 2.0 * d)`
  - right:  `Rect::new(x + w + offset, y - d, ow, h + 2.0 * d)`
  Verify the EXACT current strip rects in the code and transform them consistently (the current code is the `offset = 0` special case of the above; when `offset == 0`, `d == ow` and your rects MUST reduce to exactly today's rects). Keep the existing `if rect.size.width > 0.0 && rect.size.height > 0.0` guards and the existing color/opacity resolution unchanged.
- Leave the rest of the existing `// TODO(spec):` (non-solid styles, color:invert) in place but remove `outline-offset != 0` from that TODO text since you are implementing it.
- No `unwrap()`/`expect()` in non-test code (I-6). Mirror neighboring Option handling.

Approach: test-first (TDD). Add unit tests in the existing `#[cfg(test)]` module of src/paint/mod.rs (search for an existing outline paint test or a `DisplayItem::SolidRect` assertion to copy the style):
- A box with a known rect + `outline: 2px solid black; outline-offset: 4px` produces a top strip whose outer edge is 6px (offset 4 + width 2) above the box top — assert the emitted SolidRect coordinates.
- A box with `outline-offset: 0` (or absent) reproduces today's flush coordinates (regression guard).
- Optionally a negative offset case.
- Keep ALL existing paint tests green (do not weaken or delete any existing assertion or test — deleting/altering foreign tests to force green is a hard violation).

Done when: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.
Commit (you MUST git add + git commit before finishing — uncommitted work is lost):
  `feat(paint): apply outline-offset when painting outline (t0284)`
Comments and identifiers in English.
If the spec is ambiguous or conflicts with real browser behavior, do NOT guess — leave a `// TODO(spec):` and report it.
End with a short summary of exactly what changed and the test names you added.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
