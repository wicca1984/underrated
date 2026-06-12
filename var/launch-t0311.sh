#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0311
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0311 — implement CSS \`vertical-align\` for inline-level content within a line box (inline layout).
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/layout/ (touch ONLY src/layout/inline.rs; do NOT touch other modules, lib.rs, table.rs, or other worktrees).

WHY: real pages (sub/superscripts, inline images, form controls, icons next to text) rely on \`vertical-align\`
to position inline-level boxes against the line's baseline. Today inline layout places every inline fragment on a
single baseline and ignores \`vertical-align\` entirely (it is only handled for table cells in table.rs, which is OUT
OF SCOPE here). This adds per-fragment vertical offset within the line box for inline-level content.

BACKGROUND (verify on main before coding — do NOT re-implement what exists):
  - src/layout/inline.rs builds line boxes from inline fragments. Find where fragment y-positions / the line baseline
    are computed (search for baseline / line height / fragment y assignment). \`white-space\`, \`word-break\`, and
    \`text-align\` are already handled here — mirror their style-reading pattern (\`style.get(\"...\")\`).
  - The computed style key is \`vertical-align\` (already parsed/stored as a string in ComputedStyle; confirm by
    grepping style/mod.rs — do NOT add CSS parsing here).

SCOPE (in src/layout/inline.rs ONLY):
  1. Read \`vertical-align\` per inline fragment and apply a vertical shift relative to the line's baseline. Support the
     keyword values:
       - \`baseline\` (default) — no shift (current behavior).
       - \`sub\` — shift the fragment DOWN by a small fraction of the font size (e.g. 0.2em-ish; pick a constant and
         document it).
       - \`super\` — shift UP by a small fraction of the font size.
       - \`text-top\` — align the fragment's top with the top of the line's text content box.
       - \`text-bottom\` — align the fragment's bottom with the bottom of the line's text content box.
       - \`middle\` — align the fragment's vertical center with the baseline plus half the x-height (approximate
         x-height as 0.5em if no font metric is available; document the approximation).
       - \`top\` — align the fragment's top with the top of the line box.
       - \`bottom\` — align the fragment's bottom with the bottom of the line box.
     Any unrecognized value (including percentage/length values) falls back to \`baseline\`; leave a
     \`// TODO(spec):\` noting that \`<percentage>\` and \`<length>\` vertical-align values and precise font-metric-based
     x-height/text-top/text-bottom are out of scope for v1.
  2. Keep the change contained: the shift must NOT alter horizontal advance / wrapping logic — only the per-fragment
     vertical (y) position within its line. Do NOT change line-height computation semantics beyond what is needed to
     express these offsets. Do NOT regress existing baseline placement (default path must be byte-for-byte equivalent).
  3. Do NOT change any public Rust interface, struct field visibility, or function signature used by other modules
     unless strictly necessary; if a signature must change, keep it minimal and within inline.rs's own helpers.

Approach: STRICTLY test-first (TDD). Add #[cfg(test)] tests in src/layout/inline.rs mirroring the existing inline
layout tests (search the existing \`#[cfg(test)]\` block in inline.rs for how a small styled inline run is laid out and
how fragment positions are asserted). Cover at least:
  - Default \`baseline\`: a fragment's y is unchanged vs the existing expectation (guard against regression).
  - \`super\` shifts a fragment's y UP relative to a sibling \`baseline\` fragment on the same line.
  - \`sub\` shifts a fragment's y DOWN relative to a sibling \`baseline\` fragment on the same line.
  - \`middle\` / \`text-top\` (pick at least one more) produce a y distinct from baseline in the documented direction.
Assert on numeric fragment y-offsets (relative comparisons like \`assert!(sup_y < base_y)\` are fine; avoid brittle
exact-pixel asserts where a relative comparison expresses the intent). Do NOT delete or weaken any existing test.
Do NOT change iterative code back to recursive. Keep public interfaces stable.

Done when: \`cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check\` ALL pass.
Commit your work BEFORE finishing (do not leave changes uncommitted): \`git add -A && git commit\` with message
\`feat(layout): implement CSS vertical-align for inline-level content (t0311)\`. Comments and identifiers in English.
Must follow: AGENTS.md I-1..I-7 (no unwrap/expect in non-test code (I-6), no skipping/ignoring tests (I-4),
1 task = 1 module (I-5), no cross-worktree access (I-3)).
If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a
\`// TODO(spec):\` and report it in your summary (§8).
End with a short summary of what changed and confirm you committed." \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null
