#!/usr/bin/env bash
# t0448 — restore defensive clamps lost in t0447 big-bang migration. Base: feature/css-arch.
set -euo pipefail
cd /workspaces/wt/t0448

read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the code, run the checks, fix until green, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything is in local files. Network/web search is forbidden.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. Read AGENTS.md (passed via --include-directories) and obey I-1..I-7. NEVER use unwrap()/expect() in non-test code (I-6). NEVER add an external crate (std only). DO NOT delete or skip tests to fake green.

You are the ONLY worker on this worktree: /workspaces/wt/t0448, branch agent/t0448-css-restore-clamps, base feature/css-arch (commit 9edf920).

Task t0448 — restore two defensive value clamps that were accidentally dropped during the t0447 CategorizedComputedStyle migration. These are out-of-range guards required before this branch merges to main. Touch ONLY these two files.

FIX 1 — src/paint/mod.rs, fn get_opacity:
  Currently:
      fn get_opacity(style: &CategorizedComputedStyle) -> f32 {
          style.reset_effects.opacity
      }
  The categorized field `reset_effects.opacity` is no longer clamped at read time. Restore the clamp so out-of-range opacity is constrained to [0,1]:
      fn get_opacity(style: &CategorizedComputedStyle) -> f32 {
          style.reset_effects.opacity.clamp(0.0, 1.0)
      }

FIX 2 — src/layout/flex.rs, where row_gap and col_gap are resolved from style.reset_flex (around the `let row_gap = if style.reset_flex.row_gap == -1 { 0.0 } else { style.reset_flex.row_gap as f32 };` block). The migration dropped the defensive `.max(0.0)` on the non-sentinel branch, so a negative stored gap is no longer floored at 0. Restore it by flooring the resolved value at 0.0 for both row_gap and col_gap (e.g. wrap the non-sentinel result in `.max(0.0)`, keeping the `-1` sentinel meaning "unset -> 0.0"). Do not change the sentinel handling or the flex_direction mapping below it.

ADD a focused regression test for each fix:
  - A unit test asserting get_opacity returns 1.0 for an opacity input of 1.5 and 0.0 for -0.2 (build a CategorizedComputedStyle and set reset_effects.opacity accordingly via the public/crate API used elsewhere in the file's tests).
  - A unit/integration test asserting a negative resolved gap is floored to 0.0 (mirror the style construction used by existing flex tests).
  If constructing the style in a test is awkward, follow the exact pattern already used by neighbouring tests in the same file. Do NOT weaken or delete existing tests.

PROCEDURE (iterate until all green):
  - cargo build
  - cargo fmt
  - cargo clippy --all-targets -- -D warnings   (fix every warning)
  - cargo test                                   (all pass)
  - git add -A && git commit -m "fix(paint,layout): restore opacity[0,1] and flex gap>=0 clamps dropped in t0447 migration (t0448)"
  COMMIT before finishing (commit partial progress too). Report the final cargo test summary line.
EOF

exec gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
