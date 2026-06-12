#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0440
LOG=/workspaces/toy-browser/var/log/t0440.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic).

Task t0440 — FIX a CSS cascade correctness bug in `src/style/mod.rs`: rules nested inside `@media` blocks are assigned a `source_order` that RESETS per nesting level instead of reflecting their true global document order. This breaks the cascade tie-break for rules of EQUAL specificity (a later `@media` rule must win over an earlier top-level rule, but currently can lose).

Edit ONLY `src/style/mod.rs`. Touch NO other file.

READ the file first. The relevant code is the function `fn preparse_rules(rules: &[Rule], viewport_width: f32, preparsed: &mut Vec<PreparsedRule>)` near the top, plus `struct PreparsedRule { selector_list, declarations, source_order }`. Currently it does:

    for (rule_index, rule) in rules.iter().enumerate() {
        match rule {
            Rule::Qualified(qualified_rule) => {
                ... preparsed.push(PreparsedRule { ..., source_order: rule_index }); ...
            }
            Rule::At(at_rule) if media... => {
                ... preparse_rules(&inner_stylesheet.rules, viewport_width, preparsed); // RECURSES
            }
            _ => {}
        }
    }

THE BUG: `source_order: rule_index` uses the per-recursion-level `enumerate()` index. A `@media` block's inner rule starts at index 0 again, so it can compare as "earlier" than a top-level rule. The `preparsed` Vec is itself pushed in correct document order, so the simplest correct fix is to make `source_order` equal the running push position.

THE FIX (exactly this approach — minimal):
  - Replace `source_order: rule_index` with `source_order: preparsed.len()` at the push site. Because pushes happen in document order across all nesting levels, `preparsed.len()` (the index this rule will occupy) is a monotonically increasing, globally-correct source order.
  - You may then drop the now-unused `rule_index` binding: change `for (rule_index, rule) in rules.iter().enumerate()` to `for rule in rules.iter()` to avoid an unused-variable clippy warning. (Verify `rule_index` is not used elsewhere in the function first.)
  - Do NOT change the `PreparsedRule` struct, `collect_matched_rules`, the cascade sort logic, or any other function. The downstream code already sorts/compares by `source_order`; only the value assigned here is wrong.

Confirm (read the code) that nothing else inside `preparse_rules` references `rule_index`.

TESTS — ADD a new `#[test]` to the existing `#[cfg(test)] mod tests` in `src/style/mod.rs` (do NOT delete or modify existing tests). Name it e.g. `test_media_rule_source_order_beats_earlier_rule`. It must FAIL before the fix and PASS after. Construct a small DOM with one element (e.g. a `div`) and a stylesheet where:
  - a top-level rule sets a property (e.g. `div { color: red; }`) EARLY, and
  - a LATER `@media` block (whose query matches the default viewport width 1024px, e.g. `@media (min-width: 100px) { div { color: blue; } }`) sets the SAME property with EQUAL specificity.
Compute styles with `compute_styles` (default 1024px viewport) and assert the element's `color` resolves to the `@media` value (blue), because the `@media` rule comes LATER in source order and has equal specificity. Mirror how existing tests in this module build DOMs, parse stylesheets (`parse_stylesheet`), call `compute_styles`, and read computed values (`.get("color")` returning a `CssValue::Color(...)`). Use the same `Color::Rgba(...)` form existing tests use (look at an existing color assertion to copy the exact enum path and rgba values: red = 255,0,0,255; blue = 0,0,255,255).

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test style::` then full `cargo test` to confirm nothing broke. If all green:
  git add -A && git commit -m "fix(style): correct source_order for @media-nested rules in cascade (t0440)"
Then print "T0440 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
