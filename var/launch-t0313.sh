#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0313
# Auth: prefer canonical var/.env, fall back to bashrc export.
if grep -q '^export GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env 2>/dev/null; then
  eval "$(grep -m1 '^export GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env)"
else
  eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
fi
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7). One task = one module.

Task: t0313 — implement CSS **overflow-wrap: break-word** (a.k.a. word-wrap: break-word) in the inline layout module.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/layout/inline.rs (touch ONLY this file; do NOT touch other modules, lib.rs, mod.rs, or other worktrees).

WHY: We already implement \`word-break: break-all\` entirely inside \`layout_inline_run\` in src/layout/inline.rs
(search for the string \"word-break\" and the local \`break_all\` variable). That code reads the per-element computed
style directly inside the inline run and, when break-all is set, breaks a long word at the line edge.
\`overflow-wrap: break-word\` is the SAME family but with DIFFERENT semantics and must be added alongside it.

SEMANTICS (get this right — it differs from break-all):
  - \`word-break: break-all\` breaks a word at ANY character as soon as it reaches the line edge (already done).
  - \`overflow-wrap: break-word\` (and its legacy alias \`word-wrap: break-word\`) breaks a long word ONLY as a last
    resort: a word is kept whole if it fits on its OWN line; it is broken between characters ONLY when the word is
    too long to fit on a line by itself (i.e. word_width > containing_width). Normal-width words still wrap at
    soft-wrap opportunities as today.
  - Default value \`normal\` => no character breaking (current overflow behaviour unchanged).
  - Accept BOTH property names: \`overflow-wrap\` and the legacy \`word-wrap\`. If both present, \`overflow-wrap\` wins.

SCOPE (in src/layout/inline.rs ONLY, inside layout_inline_run, mirroring how break-all reads style):
  1. Read the per-element \`overflow-wrap\` (fallback \`word-wrap\`) computed style the SAME way break-all reads
     \`word-break\` (style.get(...) with the Keyword value). Compute a local bool e.g. \`break_word\`.
  2. When \`break_word\` is true AND a single word does not fit on a line by itself (word_width > containing_width),
     break it character-by-character to fill lines (reuse the existing per-character measuring/break logic that
     break-all already uses; factor a small shared helper IF clean, otherwise mirror it — do NOT duplicate wholesale).
  3. When the word DOES fit on its own line, do NOT break it (this is the key difference from break-all): let it move
     to the next line whole, exactly as \`normal\` does today.
  4. If BOTH break-all (word-break) and break-word are active, break-all wins (break anywhere) — keep that precedence.
  5. Keep \`normal\` behaviour byte-for-byte unchanged (regression-safe). Stay on the shipping path: layout_inline_run
     is the real inline driver used by src/layout/mod.rs, so no extra plumbing is needed — verify by tracing the call.

TODO(spec): leave a \`// TODO(spec):\` for \`anywhere\` (overflow-wrap: anywhere affects min-content sizing) being out
of scope; only \`break-word\` and \`normal\` are implemented here.

TESTS (in src/layout/inline.rs, #[cfg(test)], mirror the existing \`test_word_break_break_all\` test):
  - A long unbreakable word (e.g. 60 chars) in a narrow container with \`overflow-wrap: break-word\` must produce
    MORE THAN ONE line box (the word is split across lines).
  - The SAME long word with \`overflow-wrap: normal\` must stay on a SINGLE line box (overflow, no split) — proving the
    property actually gates the behaviour.
  - A SHORT word that fits on its own line, with \`overflow-wrap: break-word\`, must NOT be split (it wraps whole) —
    this proves break-word differs from break-all.
  - Add a test using the legacy alias \`word-wrap: break-word\` to prove the alias is honoured.

DELIVERABLE / DEFINITION OF DONE:
  - Run \`cargo fmt\`, \`cargo clippy --all-targets -- -D warnings\`, and \`cargo test\` — ALL must pass (green).
  - NO unwrap()/expect() in non-test code (I-6). NO skipped/ignored tests (I-4). Do NOT delete or weaken any existing test.
  - git add -A and COMMIT on this branch with message:
      feat(layout): implement CSS overflow-wrap: break-word (t0313)
  - Print the final \`git log --oneline -1\` and \`git status\` so completion can be verified. Commit BEFORE finishing." \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null
