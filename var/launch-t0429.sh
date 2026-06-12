#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0429
LOG=/workspaces/toy-browser/var/log/t0429.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic as the existing tests do).

Task t0429 — Add CSS recognition (parse + validation + hold) for the `text-overflow` property. Touch ONLY `src/css/values.rs` (plus you MAY add tests in that same file). Do NOT edit any other file under src/.

WHY this is new (do not duplicate): grep confirms `text-overflow` appears NOWHERE in src/. The property `white-space` (including `nowrap`) is already parsed in this same file. Your task is the CSS-LAYER recognition ONLY: make the engine recognize, validate, and retain a `text-overflow` declaration so it survives the style pipeline. The actual layout-time text truncation/clipping (drawing the ellipsis glyph, clipping the inline box) is OUT OF SCOPE for this task — leave a `// TODO(spec):` marker noting that layout-side truncation is a separate future task.

Read these IN FULL before writing anything, and copy the EXACT existing pattern for a similar keyword-valued layout property (use `white-space` as your template — search for `"white-space"`):
- The `is_known_layout_property` list (around line ~300-312, the `matches!(...)` arm listing "white-space", "cursor", "direction", etc.). Add `"text-overflow"` to this list, alphabetically/logically grouped like the others.
- The `is_valid_property_value` function (search `"white-space" => match value`): add a `"text-overflow" => match value { ... }` arm that accepts the standard keyword values `clip` and `ellipsis` (and the CSS-wide keywords `initial` / `inherit` if the neighboring arms accept them — MATCH what `white-space` does). Reject other keywords.
- The value-normalization/parse arm (the SECOND `"white-space" =>` near line ~607, in whatever function holds it): add the analogous `"text-overflow" =>` arm so a valid value is normalized/retained exactly the way `white-space` is. MATCH the surrounding return type and style precisely (e.g. `Some(val)` / wrapping in the same CssValue variant).

Steps:
1. Add `"text-overflow"` to the known-layout-property list.
2. Add validation accepting only `clip` | `ellipsis` (+ `initial`/`inherit` iff neighbors do).
3. Add the parse/normalization arm mirroring `white-space`.
4. Add a `// TODO(spec):` comment stating layout-time truncation/ellipsis rendering is out of scope (a separate future task).
5. Add ONE `#[test]` (mirror the existing `test_cursor_property` / white-space tests in this file): assert `is_known_layout_property("text-overflow")` is true, that `clip` and `ellipsis` validate as valid, and that an unsupported keyword (e.g. `"bogus"`) is rejected. Put a short `//` comment above it naming what it guards.

Keep I-6: no `unwrap`/`expect` in non-test code paths.

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(css): recognize and validate text-overflow property (clip|ellipsis) (t0429)"
Then print "T0429 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
