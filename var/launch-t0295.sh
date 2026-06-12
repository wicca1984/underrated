#!/usr/bin/env bash
# Launcher for Gemini worker t0295 — box-shadow paint support (MS-NewTargets polish, paint-only).
# Target: src/paint/mod.rs ONLY. Dispatched via setsid (memory: worker-dispatch-must-setsid).
# Disjoint module from t0294 (script/mod.rs) and t0296 (layout/inline.rs): no merge collision.
set -euo pipefail

WT=/workspaces/wt/t0295
LOG=/workspaces/toy-browser/var/worker-logs/t0295.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0295 — implement CSS `box-shadow` painting (outer drop shadow only). It is currently NOT implemented
(grep confirms 0 functional hits for `box-shadow`; the word "shadow" in src/paint/mod.rs refers ONLY to 3D
button border shading, which is unrelated — do NOT touch it).

Target file: src/paint/mod.rs ONLY (production code + its in-file `#[cfg(test)] mod tests`). Do NOT touch any
other file (no other src/ module, no css parser, no Cargo.toml, no other worktree). 1 task = 1 module (I-5).
Production Rust code must NOT use `unwrap`/`expect` (I-6); test code may.

This is a non-inherited property read directly from the generic computed-style map, EXACTLY like `opacity`
(src/paint/mod.rs around line 292: `match style.get("opacity")`) and `outline-offset` (around line 60). You do
NOT need any style/cascade registration — just read `style.get("box-shadow")` in the paint path and emit a
shadow rect. VERIFY these line references against the ACTUAL code before editing; trust the code over this prompt.

Read first and mirror the EXISTING rect-emission style:
- Find where each box's background `SolidRect` DisplayItem is emitted in the display-list builder, and where the
  border SolidRects are emitted (the border code lives around lines 450-535: it builds Rect geometry and pushes
  `DisplayItem::SolidRect`). Mirror that exact construction (Rect fields, Color type, how items are pushed).
- Find the existing color parser used for `outline-color`/border colors (e.g. `style.get("outline-color")` near
  line 534) and REUSE it to parse the shadow color. Do not write a new color parser.

Implementation (box-shadow v1 — outer shadow, blur ignored):
  Parse `box-shadow` value of the form `<offset-x> <offset-y> [<blur-radius>] [<color>]` where offsets/blur are
  CSS lengths in px (e.g. `2px 2px 4px #888` or `4px 4px black` or `2px 2px`). Default color = current text color
  or black if absent. Parse leniently: split on whitespace, the first two length tokens are offset-x/offset-y,
  an optional third length token is blur (PARSE but IGNORE it in v1), and any non-length token is the color.
  If the value is `none`, empty, `inset`, or contains a comma (multiple shadows), emit NOTHING and add a
  TODO(spec) note (inset shadows, blur rendering, spread radius, and comma-separated multiple shadows are out of
  scope). Emit ONE `DisplayItem::SolidRect` the SAME size as the box's border-box, translated by (offset-x,
  offset-y), pushed BEFORE the box's own background rect so the shadow sits behind it. Add the comment:
  `// TODO(spec): box-shadow v1 — single outer shadow, offset-only (blur/spread ignored, inset & multiple shadows out of scope).`

Tests (add to the in-file tests module — do NOT delete, weaken, `#[ignore]`, or alter ANY existing test). Build
inputs the SAME way the existing paint tests do (look at the opacity test and the border-shadow test near line
2055 `test ... border ... shadows` for the harness: how they build a styled box, run the display-list builder,
and filter `DisplayItem::SolidRect`s). Add ONE test `fn test_box_shadow_emits_offset_rect()` that:
  1) Styles a box with `box-shadow: 5px 5px #ff0000` (a distinct color), builds the display list, and asserts a
     SolidRect of that color exists whose origin is offset by (5,5) relative to the box's border-box origin and
     whose size equals the border-box size, and that it is ordered BEFORE the box's background rect.
  2) Asserts `box-shadow: none` (or absent) emits NO such extra shadow rect (regression guard).
Reuse existing test helpers/patterns; do not invent a new harness.

If any field/type/helper name does not match this description, TRUST THE CODE, not this prompt, and adapt — but
keep the intent: outer offset shadow rect behind the background, blur ignored.

Done when (run from the worktree root /workspaces/wt/t0295):
  `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.

Commit (you MUST `git add -A && git commit` before finishing — uncommitted work is lost; the worktree may be
force-removed after you exit):
  `feat(paint): implement box-shadow outer drop shadow painting (t0295)`

End with a short summary: where you inserted the shadow emission (file:line), the exact parse order you used for
offset-x/offset-y/blur/color, and the SolidRect origin/size/order your test observed.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
