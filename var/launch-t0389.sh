#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0389
LOG=/workspaces/toy-browser/var/log/t0389.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0389 — implement the two-value `gap` shorthand for flex containers. Touch ONLY src/layout/flex.rs. Do NOT edit any other file/module. If something genuinely requires another module, leave a `// TODO(spec): ...` in src/layout/flex.rs and stop.

Background (read before coding):
- Read src/layout/flex.rs in full. Around lines 111-145 the code resolves `row_gap` and `col_gap`:
  - It reads longhand `row-gap` and `column-gap` first (px only).
  - Then, if either is still None, it reads the `gap` shorthand. Currently the `gap` shorthand only handles a SINGLE `CssValue::Length(px, Px)` value (applies the same px to both row and col), and there is a `// TODO(spec): two-value gap shorthand` and `// TODO(spec): non-px gap units`.
- CSS `gap` shorthand: `gap: <row-gap> <column-gap>`. With ONE value, it applies to both axes. With TWO values, the FIRST is the row gap and the SECOND is the column gap. Reference: https://developer.mozilla.org/docs/Web/CSS/gap

Scope for THIS task (keep it to flex.rs and to what the value model already supports):
1. Handle the two-value `gap` shorthand. Inspect how a multi-value `gap` is represented in the `CssValue` model. Look at `crate::css::values::CssValue` variants already imported/available (e.g. there may be a list/multiple/space-separated representation). If `gap` parses to a two-length value, map value[0] -> row_gap (when row_gap is None) and value[1] -> col_gap (when col_gap is None). Keep the existing single-value behavior (same px to both) intact.
2. Only consume px lengths (`LengthUnit::Px`), consistent with the surrounding code. Leave the existing `// TODO(spec): non-px gap units` markers in place for non-px units — do NOT attempt unit conversion (that needs font/viewport context = another module).
3. If — and only if — the `CssValue` model has NO way to represent two space-separated lengths for `gap` (i.e. the parser collapses it), do NOT edit the css module; instead leave a precise `// TODO(spec): gap shorthand needs a two-length CssValue representation in css::values` and implement what you can, then report that limitation. (First genuinely check the available variants before concluding this.)
4. Panic-free: no unwrap()/expect()/panicking indexing in non-test code (use `.get(0)`/`.get(1)` or pattern matching).

Tests — add to the existing `#[cfg(test)] mod tests` in src/layout/flex.rs (do NOT modify/delete existing tests). Build a small flex container fixture with `display:flex` and assert child positions reflect the resolved gaps. Cover:
- `gap: 10px 20px` on a row flex container -> row gap 10, column gap 20 (children separated horizontally by 20 in a row layout).
- single `gap: 15px` still applies 15 to both axes (regression guard).
- `row-gap`/`column-gap` longhands still take precedence over `gap` (regression guard).
Mirror the existing test helpers/fixtures in this file for layout assertions.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(layout): honor two-value gap shorthand for flex containers (t0389)"
Then print "T0389 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
