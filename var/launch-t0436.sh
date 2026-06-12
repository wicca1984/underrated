#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0436
LOG=/workspaces/toy-browser/var/log/t0436.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic).

Task t0436 — CSS PARSER: generic comma support in `parse_value` (milestone MS-CSS-Generic). This is BLESSED by the PdM as Option A: add generic `CssToken::Comma` handling to `parse_value`, mirroring the existing `Delim('/')` handling. Scope is EXACTLY ONE FILE: `src/css/values.rs`. Do NOT touch any other file.

BACKGROUND (confirm by reading the code before editing):
`pub fn parse_value(components: &[ComponentValue]) -> Option<CssValue>` in `src/css/values.rs` splits a declaration's component values into groups separated by whitespace, and already specially handles a `/` delimiter: when it sees `ComponentValue::Token(CssToken::Delim('/'))` it flushes the current group via `parse_single_value` and pushes `CssValue::Keyword("/".to_string())`. Commas currently fall into the `_ => current_group.push(component)` arm, so a multi-value declaration like `box-shadow: 5px 5px red, 10px 10px blue` puts a comma into a group, `parse_single_value` returns None on that group, and the WHOLE declaration is discarded. As a result comma-separated multi-value properties never reach paint/layout.

THE FIX (mirror the `Delim('/')` arm exactly, for commas):
1. In `parse_value`, add a new match arm BEFORE the catch-all `_` arm:

       ComponentValue::Token(CssToken::Comma) => {
           if !current_group.is_empty() {
               if let Some(val) = parse_single_value(&current_group) {
                   values.push(val);
                   current_group.clear();
               } else {
                   return None;
               }
           }
           values.push(CssValue::Keyword(",".to_string()));
       }

   Add a short comment explaining (like the `/` one) that a comma separates values in multi-value properties (box-shadow, transition, font-family, gradients) and is emitted as its own keyword so downstream consumers can split on it.

2. Do NOT change `parse_single_value` or any other function. Do NOT change the `Delim('/')` arm. Do NOT remove the catch-all `_` arm.

WHY THIS IS SAFE (do not second-guess, just preserve behavior elsewhere):
- Previously every comma-containing declaration parsed to `None` (silently dropped). Now it parses to `CssValue::Multiple([.., Keyword(","), ..])`. Existing consumers match on specific value shapes and ignore unknown `Multiple`/keywords, so this turns "dropped" into "harmlessly carried". Box-shadow paint (already merged) consumes the `,` keyword to split multiple shadows.

TESTS (add, do not modify/delete existing — I-4):
- Add `#[cfg(test)]` unit tests in `src/css/values.rs` asserting:
  a. A two-shadow-like input `5px 5px red , 10px 10px blue` (build the `ComponentValue` slice the same way the existing tests in this file do — look at how other `parse_value` tests construct `token(...)` / component values, e.g. around the existing comma tests near lines 1240-1267) parses to a `CssValue::Multiple` whose values contain exactly one `CssValue::Keyword(",")` separating the two groups.
  b. A single comma between two simple keywords yields a `Multiple` with a `Keyword(",")` in the middle.
  c. (Regression) `aspect-ratio` style `16 / 9` still works (the `/` path is unchanged).
- Reuse the existing test helpers (`token`, etc.) already present in this file; do not invent new infrastructure.

CONSTRAINTS:
- ONLY edit `src/css/values.rs`. NO unwrap/expect/panic in non-test code (I-6). Do not change any public signature.
- Behavior for non-comma declarations MUST be byte-for-byte identical.

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test` (FULL suite). If ALL green:
  git add -A && git commit -m "feat(css): generic comma handling in parse_value for multi-value properties (t0436)"
Then print "T0436 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
