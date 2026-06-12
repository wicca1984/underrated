#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0434
LOG=/workspaces/toy-browser/var/log/t0434.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic).

Task t0434 — Add a STATIC CSS PROPERTY METADATA table as a NEW, PURELY ADDITIVE module. This is foundation work for milestone MS-CSS-Generic (generalizing the CSS engine). You create new code that NOTHING consumes yet; you must NOT rewire or refactor any existing consumer. Keep the blast radius to one module.

EXACTLY these file changes are allowed:
  1. Create ONE new file: `src/css/property.rs`.
  2. Add ONE line `pub mod property;` to `src/css/mod.rs` (next to the existing `pub mod values;` etc. at the top). That single mod-registration line is the ONLY edit permitted outside the new file.
Do NOT touch values.rs, resolve.rs, parser.rs, style computation, layout, or any struct definition. Do NOT change `ComputedStyle`. This task is additive only.

WHAT TO BUILD in `src/css/property.rs`:
A static, queryable metadata dictionary describing common CSS properties, so future generic cascade/inheritance code can ask "is this property inherited?" and "what is its initial value?" without per-property hardcoding scattered around the codebase.

  - Define `pub struct PropertyMetadata` with at least: `name: &'static str`, `inherited: bool`, `initial: &'static str` (the CSS initial value as a canonical string, e.g. "0" for margin-top, "normal" for font-weight, "transparent" for background-color, "currentcolor" for border-top-color). Keep fields `&'static str`/`bool` so the whole table is `const`/static with no allocation.
  - Define a static table (e.g. `static PROPERTY_METADATA: &[PropertyMetadata] = &[ ... ];`) covering a solid, representative set of well-known longhand properties. Include at minimum these (get inherited/initial right per the CSS spec):
      INHERITED: color (initial "canvastext"/"black" — use "black"), font-family ("serif"), font-size ("medium"), font-style ("normal"), font-weight ("normal"), line-height ("normal"), text-align ("start"), letter-spacing ("normal"), word-spacing ("normal"), white-space ("normal"), visibility ("visible"), list-style-type ("disc"), direction ("ltr"), text-transform ("none"), cursor ("auto").
      NON-INHERITED: display ("inline"), width ("auto"), height ("auto"), margin-top ("0"), margin-right ("0"), margin-bottom ("0"), margin-left ("0"), padding-top ("0"), padding-right ("0"), padding-bottom ("0"), padding-left ("0"), border-top-width ("medium"), border-top-style ("none"), border-top-color ("currentcolor"), background-color ("transparent"), position ("static"), top ("auto"), right ("auto"), bottom ("auto"), left ("auto"), float ("none"), clear ("none"), overflow ("visible"), z-index ("auto"), box-sizing ("content-box"), opacity ("1").
    (If you are unsure of one value, pick the spec-correct CSS initial value; do not invent.)
  - Provide a lookup function: `pub fn lookup(name: &str) -> Option<&'static PropertyMetadata>` that finds the entry by case-insensitive ASCII match on `name` (CSS property names are ASCII case-insensitive). Returning `Option` (not panicking) keeps it I-6 clean.
  - Provide two convenience helpers built on lookup:
      `pub fn is_inherited(name: &str) -> bool` (false if unknown),
      `pub fn initial_value(name: &str) -> Option<&'static str>`.
  - Module-level `//!` doc comment explaining this is the generic property-metadata foundation for MS-CSS-Generic, and that consumers will be wired in later tasks.
  - Add `// TODO(spec):` noting the table is an initial representative subset to be expanded, and that shorthand-expansion metadata is intentionally out of scope for this task (a later task).

NO unwrap/expect/panic in the module code (I-6). Use iterator `.find(...)` returning Option.

TESTS: add a `#[cfg(test)] mod tests` inside `src/css/property.rs` with unit tests asserting e.g.:
  - `is_inherited("color")` is true, `is_inherited("margin-top")` is false, `is_inherited("Color")` is true (case-insensitive), `is_inherited("not-a-real-prop")` is false.
  - `initial_value("display")` == Some("inline"), `initial_value("width")` == Some("auto"), `initial_value("border-top-color")` == Some("currentcolor").
  - `lookup("FONT-SIZE")` is Some and its `.inherited` is true.
  - the table has no duplicate `name` entries (assert by collecting names into a set and comparing lengths).

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test css::property` then full `cargo test` to confirm nothing broke. If all green:
  git add -A && git commit -m "feat(css): add static property-metadata table (inherited/initial) for MS-CSS-Generic (t0434)"
Then print "T0434 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
