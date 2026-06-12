#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0442
LOG=/workspaces/toy-browser/var/log/t0442.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic).

Task t0442 — GENERALIZE inheritance resolution in `src/style/mod.rs` by wiring it to the static property-metadata table in `src/css/property.rs`. This advances MS-CSS-Generic item 3 (generic inheritance): instead of a hand-maintained hardcoded list, inheritance should also be driven by the metadata table, in an ADDITIVE, no-regression way.

Edit ONLY `src/style/mod.rs`. Touch NO other file. (`src/css/property.rs` already exposes the public API you need; do not modify it.)

READ both files first. Key existing items:
  - `src/css/property.rs` exposes `pub fn is_inherited(name: &str) -> bool` — case-insensitive; returns true iff the property is present in the metadata table AND marked inherited. Returns false for properties not in the table.
  - `src/style/mod.rs` has `fn is_inherited_property(property: &str) -> bool` — a hardcoded `matches!(property, "color" | "font-family" | ...)` list. This is the function that drives parent->child inheritance in `compute_styles`.

IMPLEMENT (single, minimal, additive change):
  1. Modify `is_inherited_property` so it returns true if EITHER the existing hardcoded `matches!` list matches OR `crate::css::property::is_inherited(property)` returns true. Keep the existing hardcoded `matches!` block intact (rename it into a local `matches!` expression bound to a `let` if helpful, or just `||` the two). The OR ordering means: every property currently treated as inherited stays inherited (no regression), AND any property the metadata table marks inherited is now also covered generically.
     Example shape:
        fn is_inherited_property(property: &str) -> bool {
            // spec: basic inherited properties (hardcoded fast-path, retained for no-regression)
            let hardcoded = matches!(property, "color" | "font-family" | /* ...unchanged... */ "empty-cells");
            // Generic: also honor the static property-metadata table (MS-CSS-Generic).
            hardcoded || crate::css::property::is_inherited(property)
        }
  2. Do NOT remove any entry from the hardcoded list. Do NOT change any other logic in the file.

ADD unit tests in the existing `#[cfg(test)] mod tests` of `src/style/mod.rs` (or create one if absent, following the file's existing test style):
  - A property in the hardcoded list still returns true (e.g. `assert!(is_inherited_property("color"))`).
  - A clearly NON-inherited property still returns false (e.g. `assert!(!is_inherited_property("width"))` and `assert!(!is_inherited_property("background-color"))`). If any of these are unexpectedly true, STOP and report — do not silence by editing the table.
  - A property that is inherited per the metadata table but NOT in the hardcoded list now returns true. To pick one: read `src/css/property.rs`, find a property whose entry has `inherited: true` and which is NOT in the `is_inherited_property` hardcoded `matches!` list, and assert it returns true. (If every table-inherited property is already in the hardcoded list, instead assert that all hardcoded-inherited names also satisfy the combined function, and note this in the commit body.)

VERIFY before committing:
  - `cargo fmt --all`
  - `cargo clippy --all-targets -- -D warnings` (zero warnings)
  - `cargo test` (all green)
  - `git diff --name-only` shows ONLY `src/style/mod.rs`.

Then commit ON THIS BRANCH with:
  git add -A && git commit -m "feat(style): drive inheritance generically via property-metadata table (t0442)"

Then run `git status` to confirm a clean tree and that the commit landed.
Then print "T0442 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
