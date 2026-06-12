#!/usr/bin/env bash
# Launcher for Gemini worker t0298 — text-decoration-style (MS-NewTargets polish, paint-only).
# Target: src/paint/mod.rs ONLY. Dispatched via setsid (memory: worker-dispatch-must-setsid).
# Disjoint module from t0296 (layout/inline.rs) and t0297 (script/mod.rs): no merge collision.
set -euo pipefail

WT=/workspaces/wt/t0298
LOG=/workspaces/toy-browser/var/worker-logs/t0298.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0298 — implement `text-decoration-style` (solid | double | dotted | dashed) for the decoration lines
(underline / overline / line-through) that are ALREADY painted today. `text-decoration-line` (underline/overline/
line-through) and `text-decoration-color` are already resolved and painted in src/paint/mod.rs (see the
`Decoration` struct ~line 123 with `underline`/`overline` bool flags, the resolution loop ~line 142, and the
color handling ~line 172). The decoration LINES are currently always drawn as a single solid filled rectangle.
The gap to fill: resolve `text-decoration-style` and render the line accordingly.

Target file: src/paint/mod.rs ONLY (production code + its in-file `#[cfg(test)] mod tests`). Do NOT touch any
other file (no other src/ module, no css parser, no Cargo.toml, no other worktree). 1 task = 1 module (I-5).
Production Rust code must NOT use `unwrap`/`expect` (I-6); test code may.

Read first and VERIFY every claim against the ACTUAL code before editing (do not trust this prompt blindly):
- How the `Decoration` struct is populated and how each enabled line (underline/overline/line-through) is turned
  into the display item(s) that draw it. Identify the exact place where the solid line rectangle is pushed, and
  the line's geometry (x range, y, thickness/color).
- How `text-decoration-color` resolves a single color (you will reuse that same color for the patterned line).

Implementation (text-decoration-style v1):
  Add a `style` field to the decoration state resolved from the `text-decoration-style` property (also accept the
  value when present in the `text-decoration` shorthand if that is already tokenized; if the shorthand path is
  non-trivial, resolve ONLY the longhand `text-decoration-style` and leave a TODO note). Values:
    - `solid` (default): unchanged — one solid line rectangle (current behavior).
    - `double`: draw TWO thin parallel line rectangles separated by a small gap (total height ~= 3x a thin line),
      using the existing decoration color.
    - `dotted`: draw the line as a series of short square dots (length ~= line thickness) with equal gaps along
      the line's x range.
    - `dashed`: draw the line as a series of short dashes (dash length a few px, equal gaps) along the x range.
  Render dots/dashes as multiple small SolidRect display items spanning the SAME x-range and y as the solid line
  would occupy. Keep the same color and the same thickness the solid line uses. Apply to whichever of
  underline/overline/line-through are enabled. Add the comment:
  `// TODO(spec): text-decoration-style v1 — solid/double/dotted/dashed via repeated SolidRects; wavy is approximated/out-of-scope, dash/dot metrics are fixed heuristics, and shorthand-embedded style keyword parsing is best-effort.`
  If you cannot cleanly parse the style keyword out of the shorthand, support the LONGHAND `text-decoration-style`
  only and say so honestly in your summary.

Tests (add to the in-file tests module — do NOT delete, weaken, `#[ignore]`, or alter ANY existing test). Build
inputs the SAME way the existing text-decoration tests do (find the test(s) that assert underline/line-through
SolidRects are emitted and how they read the display list back). Add tests that:
  1) `text-decoration: underline; text-decoration-style: dotted` (or dashed) emits MULTIPLE small SolidRects for
     the single underline (count > 1), all at the underline's y, spanning within the text x-range, in the
     decoration color — versus exactly ONE SolidRect for the default `solid` case (regression-guard the solid
     path still emits a single line).
  2) `text-decoration: underline; text-decoration-style: double` emits TWO parallel line rectangles at distinct
     y offsets.
Reuse existing test helpers/patterns; do not invent a new harness.

If any field/helper/enum name does not match this description, TRUST THE CODE, not this prompt, and adapt — but
keep the intent: solid (1 line), double (2 lines), dotted/dashed (repeated small rects), same color & thickness.

Done when (run from the worktree root /workspaces/wt/t0298):
  `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.

Commit (you MUST `git add -A && git commit` before finishing — uncommitted work is lost; the worktree may be
force-removed after you exit):
  `feat(paint): implement text-decoration-style solid/double/dotted/dashed (t0298)`

End with a short summary: where you added the style resolution + per-style rendering (file:line), exactly which
style values you support (and whether shorthand-embedded style is parsed), and the SolidRect counts your tests
observed for solid vs dotted/dashed vs double.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
