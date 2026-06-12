#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0437
LOG=/workspaces/toy-browser/var/log/t0437.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic).

Task t0437 — EXPAND the existing static CSS property-metadata table. This is PURELY ADDITIVE foundation work for milestone MS-CSS-Generic. You only ADD rows (and matching tests) to the existing table; you do NOT change the struct shape, the lookup logic, or any consumer.

The file `src/css/property.rs` ALREADY EXISTS and contains:
  - `pub struct PropertyMetadata { name, inherited, initial }` (all `&'static str` / `bool`).
  - `static PROPERTY_METADATA: &[PropertyMetadata] = &[ ... ];` with a representative subset.
  - `pub fn lookup`, `pub fn is_inherited`, `pub fn initial_value`.
  - a `#[cfg(test)] mod tests`.

EXACTLY these changes are allowed:
  - Edit ONLY `src/css/property.rs`. Touch NO other file (no mod.rs, no values.rs, no style, no layout). The module is already registered.
  - Do NOT change the `PropertyMetadata` struct, the `static` declaration name, or the function signatures. Only ADD new `PropertyMetadata { ... }` rows to the table and ADD new test assertions.

WHAT TO ADD — append rows for these additional common longhand properties with SPEC-CORRECT CSS initial values and inheritance. First READ the existing rows so you do not duplicate any already present (the table must stay free of duplicate `name`s — there is already a test enforcing this). Add ONLY ones not already in the table:
  INHERITED: font-variant ("normal"), font-stretch ("normal"), text-indent ("0"), word-break ("normal"), overflow-wrap ("normal"), text-align-last ("auto"), caption-side ("top"), empty-cells ("show"), border-collapse ("separate"), border-spacing ("0"), list-style-position ("outside"), list-style-image ("none"), quotes ("auto"), tab-size ("8"), hyphens ("manual").
  NON-INHERITED: margin-block-start ("0"), margin-block-end ("0"), padding-block-start ("0"), padding-block-end ("0"), border-right-width ("medium"), border-bottom-width ("medium"), border-left-width ("medium"), border-right-style ("none"), border-bottom-style ("none"), border-left-style ("none"), border-right-color ("currentcolor"), border-bottom-color ("currentcolor"), border-left-color ("currentcolor"), background-image ("none"), background-repeat ("repeat"), background-position ("0% 0%"), background-size ("auto"), background-attachment ("scroll"), border-top-left-radius ("0"), border-top-right-radius ("0"), border-bottom-right-radius ("0"), border-bottom-left-radius ("0"), outline-width ("medium"), outline-style ("none"), outline-color ("invert"), min-width ("0"), min-height ("0"), max-width ("none"), max-height ("none"), flex-grow ("0"), flex-shrink ("1"), flex-basis ("auto"), flex-direction ("row"), flex-wrap ("nowrap"), justify-content ("normal"), align-items ("normal"), align-self ("auto"), order ("0"), table-layout ("auto"), vertical-align ("baseline"), text-decoration-line ("none"), text-decoration-color ("currentcolor"), text-decoration-style ("solid"), text-overflow ("clip"), object-fit ("fill"), pointer-events ("auto"), transition-duration ("0s"), transition-property ("all").
  (If unsure of a value, use the spec-correct CSS initial value; do not invent. If any name above ALREADY exists in the table, SKIP it — never create a duplicate.)

NO unwrap/expect/panic in module (non-test) code (I-6).

TESTS — ADD assertions to the existing `mod tests` (do not delete existing tests):
  - `is_inherited("text-indent")` is true; `is_inherited("max-width")` is false.
  - `initial_value("flex-shrink")` == Some("1"); `initial_value("border-collapse")` == Some("separate"); `initial_value("background-repeat")` == Some("repeat").
  - `lookup("BORDER-BOTTOM-COLOR")` is Some and its `.inherited` is false (case-insensitive lookup).
  - the existing no-duplicate-names test must still pass (so be careful not to add a name twice).

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test css::property` then full `cargo test` to confirm nothing broke. If all green:
  git add -A && git commit -m "feat(css): expand property-metadata table with more longhands (t0437)"
Then print "T0437 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
