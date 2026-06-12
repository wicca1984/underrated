#!/usr/bin/env bash
# Launcher for Gemini worker t0296 — text-align: justify (MS-NewTargets polish, layout-only).
# Target: src/layout/inline.rs ONLY. Dispatched via setsid (memory: worker-dispatch-must-setsid).
# Disjoint module from t0294 (script/mod.rs) and t0295 (paint/mod.rs): no merge collision.
set -euo pipefail

WT=/workspaces/wt/t0296
LOG=/workspaces/toy-browser/var/worker-logs/t0296.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0296 — implement `text-align: justify` for inline text. The `text-align` property is ALREADY resolved and
threaded into inline layout as a `text_align: &str` parameter; only the `center` and `right` values are handled
today (src/layout/inline.rs around lines 73-76: `match text_align { "center" => ..., "right" => ... }`). The
`justify` value currently falls through to the default (left) — that is the gap to fill.

Target file: src/layout/inline.rs ONLY (production code + its in-file `#[cfg(test)] mod tests`). Do NOT touch any
other file (no other src/ module, no src/style, no css parser, no Cargo.toml, no other worktree). 1 task = 1
module (I-5). Production Rust code must NOT use `unwrap`/`expect` (I-6); test code may.

Read first and VERIFY every claim against the ACTUAL code before editing (do not trust this prompt blindly):
- The line-alignment delta is computed in the `match text_align { ... }` block (~line 74). Understand how the
  laid-out line `width` and the `containing_width` relate, and how per-word fragment X positions are assigned.
- word-spacing (already implemented, ~line 248) ALREADY adds a per-word advance after each word carrying a
  trailing space — study how words/fragments on a line are iterated and positioned, because justify reuses that
  same per-word positioning to distribute extra space into the inter-word gaps.

Implementation (text-align: justify v1):
  When `text_align == "justify"` AND the line is NOT the last line of its block AND the line has 2+ word
  fragments (i.e. at least one inter-word gap): distribute the slack `(containing_width - width)` EQUALLY across
  the N-1 inter-word gaps, shifting each subsequent word right by a cumulative amount so the last word's right
  edge meets `containing_width`. The LAST line of a block (and any single-word line) must remain left-aligned
  (no stretching) — this matches CSS. If slack <= 0, do nothing. Do NOT stretch glyphs or add intra-word space;
  only inter-word gaps grow. Add the comment:
  `// TODO(spec): text-align justify v1 — distributes slack across inter-word gaps on non-last lines only; last-line/forced-break detection is simple word-count based; RTL, percentage widths, hyphenation, and justify-by-character are out of scope.`
  If detecting "last line" precisely is hard, use a defensible heuristic (e.g. a line that does not wrap / is the
  final produced line of the run is left-aligned) and document it in the TODO(spec) note above. Be honest in the
  summary about exactly which heuristic you used.

Tests (add to the in-file tests module — do NOT delete, weaken, `#[ignore]`, or alter ANY existing test). Build
inputs the SAME way the existing alignment / white-space tests do (look at the `test_white_space_*` tests and any
center/right alignment test for the harness: how they build styled inline content with a known containing width,
run inline layout, and read back per-word fragment X positions/widths). Add ONE test
`fn test_text_align_justify_distributes_gaps()` that:
  1) Lays out a multi-word line that does NOT fill the container with `text-align: justify`, and asserts the
     FIRST word starts at the left edge (x≈0) and the LAST word's right edge reaches the containing width
     (within ~1px), with inter-word gaps widened versus the un-justified layout.
  2) Asserts a single-word line (or the last line) with `justify` is NOT stretched (stays left-aligned) — a
     regression guard that justify never moves the only/last word.
Reuse existing test helpers/patterns; do not invent a new harness.

If any parameter/field/helper name does not match this description, TRUST THE CODE, not this prompt, and adapt —
but keep the intent: inter-word slack distribution on non-last multi-word lines only.

Done when (run from the worktree root /workspaces/wt/t0296):
  `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.

Commit (you MUST `git add -A && git commit` before finishing — uncommitted work is lost; the worktree may be
force-removed after you exit):
  `feat(layout): implement text-align justify for inline text (t0296)`

End with a short summary: where you added the justify branch (file:line), the exact "last line" heuristic you
used, and the first-word x / last-word right-edge / gap widths your test observed.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
