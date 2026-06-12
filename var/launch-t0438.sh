#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0438
LOG=/workspaces/toy-browser/var/log/t0438.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic).

Task t0438 — PERFORMANCE: hoist the per-node, per-rule selector serialization + parsing out of the style-matching hot path. This is a behavior-PRESERVING optimization for milestone MS-NewTargets-Perf. Edit ONLY `src/style/mod.rs`. Touch NO other file.

THE PROBLEM (already verified):
`compute_styles_with_viewport` traverses every DOM node (N nodes) and for each node calls `compute_node_style` -> `collect_matched_rules(dom, node, &stylesheet.rules, ...)`. Inside `collect_matched_rules`, for EACH qualified rule it does:
  - `let selector_str = serialize_component_values(&qualified_rule.prelude);`
  - `crate::selector::parse_selector_list(&selector_str)`
Both of these depend ONLY on the rule, NOT on the node. So for N nodes x R rules the SAME selector string is serialized and re-parsed N times. On a large page (thousands of nodes x hundreds of rules) this dominates style computation. The goal is to serialize+parse each rule's selector list EXACTLY ONCE per call to `compute_styles_with_viewport`, then in the per-node loop only run `matches_complex` + `specificity`.

WHAT TO DO (keep it tight and behavior-preserving):
1. In `compute_styles_with_viewport`, BEFORE the DOM traversal loop, build a pre-parsed view of the stylesheet's rules once. Suggested shape: a `Vec` of an internal struct (e.g. `struct PreparsedRule<'a> { selector_list: crate::selector::SelectorList, declarations: &'a [...], source_order: usize }`) for each `Rule::Qualified` whose selector successfully parses. Preserve the ORIGINAL `source_order` = the rule's index within `stylesheet.rules` (exactly as `rule_index` is used today in `collect_matched_rules`), so cascade ordering and the existing `ua_rules_count` source_order bump in `compute_node_style` stay identical.
2. Thread this pre-parsed structure down through `compute_node_style` and into `collect_matched_rules` (change their signatures as needed — these are private `fn`s in this module, so that is fine) so the per-node code no longer calls `serialize_component_values`/`parse_selector_list` for qualified rules; it only iterates the pre-parsed rules and calls `matches_complex(sel, dom, node)` + `specificity(sel)`.
3. IMPORTANT — `@media` at-rules: today `collect_matched_rules` recursively re-parses inner `@media` stylesheets per node too. Handle this WITHOUT changing behavior. Simplest correct approach: when building the pre-parsed list, also expand matching `@media` rules (those where `evaluate_media_query(prelude, viewport_width)` is true) by recursively parsing their inner stylesheet ONCE and appending their qualified rules in source order, mirroring exactly what the recursive `collect_matched_rules` does today (same media gating, same ordering). If fully flattening media is awkward, you MAY instead keep the media branch as-is (still re-parsed per node) and only hoist the top-level qualified-rule serialize+parse — but the top-level qualified rules MUST be hoisted. Do NOT change which declarations match or their relative order.
4. Do NOT change any public function signature (`compute_styles`, `compute_styles_with_viewport` keep their exact signatures). Do NOT change `ComputedStyle`, cascade logic, specificity, or inline-style handling. The set of matched declarations and their `(specificity, source_order)` for every node MUST be identical to before.

NO unwrap/expect/panic in module (non-test) code (I-6). If `parse_selector_list` returns `Err`, skip that rule exactly as the current code does (it is inside `if let Ok(...)`).

TESTS — the existing `cargo test` suite (style/selector/cascade tests) MUST stay green unchanged; that is your correctness oracle for behavior preservation. ADD one focused unit test in the existing `#[cfg(test)] mod tests` (or create one if absent) asserting that a small DOM with a stylesheet containing a tag rule, a class rule, and an `#id` rule produces the SAME `ComputedStyle` property values as expected (e.g. a node matching multiple rules gets the highest-specificity / latest-source-order winner). Keep it small and deterministic.

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test` (FULL suite — this proves behavior preservation). If all green:
  git add -A && git commit -m "perf(style): hoist per-node selector serialize+parse out of style matching hot path (t0438)"
Then print "T0438 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
