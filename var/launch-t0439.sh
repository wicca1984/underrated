#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0439
LOG=/workspaces/toy-browser/var/log/t0439.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic).

Task t0439 — ADD a static CSS *shorthand-expansion* metadata table to the property module. This is PURELY ADDITIVE foundation/data work for milestone MS-CSS-Generic, menu item 2 ("shorthand expansion rules metadata"). You ONLY ADD a new struct + static table + lookup function + tests. You do NOT change any existing struct, the existing PROPERTY_METADATA table, the existing lookup logic, or any consumer. The actual value-distribution logic (turning `margin: 1px 2px` into four values) lives elsewhere and is OUT OF SCOPE — you provide only the DATA mapping each shorthand to its ordered longhand names.

Edit ONLY `src/css/property.rs`. Touch NO other file (no mod.rs, no values.rs, no style, no layout). The module is already registered.

The file `src/css/property.rs` ALREADY EXISTS and contains `pub struct PropertyMetadata`, `static PROPERTY_METADATA`, `pub fn lookup/is_inherited/initial_value`, and a `#[cfg(test)] mod tests`. READ it first.

EXACTLY these changes are allowed (all NEW, additive):
  1. Add a new public struct:
       /// Maps a CSS shorthand property to the ordered list of longhand properties it expands into.
       #[derive(Debug, Clone, Copy, PartialEq, Eq)]
       pub struct ShorthandExpansion {
           /// The canonical lowercase name of the shorthand property.
           pub name: &'static str,
           /// The ordered longhand property names this shorthand sets.
           pub longhands: &'static [&'static str],
       }
  2. Add a new static table `static SHORTHAND_EXPANSIONS: &[ShorthandExpansion] = &[ ... ];` containing SPEC-CORRECT ordered longhand lists for these common shorthands (use the longhand names already used elsewhere in the codebase / standard CSS longhand names):
       - "margin"  -> ["margin-top", "margin-right", "margin-bottom", "margin-left"]
       - "padding" -> ["padding-top", "padding-right", "padding-bottom", "padding-left"]
       - "border-width" -> ["border-top-width", "border-right-width", "border-bottom-width", "border-left-width"]
       - "border-style" -> ["border-top-style", "border-right-style", "border-bottom-style", "border-left-style"]
       - "border-color" -> ["border-top-color", "border-right-color", "border-bottom-color", "border-left-color"]
       - "border-radius" -> ["border-top-left-radius", "border-top-right-radius", "border-bottom-right-radius", "border-bottom-left-radius"]
       - "overflow" -> ["overflow-x", "overflow-y"]
       - "gap" -> ["row-gap", "column-gap"]
       - "inset" -> ["top", "right", "bottom", "left"]
       - "place-items" -> ["align-items", "justify-items"]
       - "place-content" -> ["align-content", "justify-content"]
       - "place-self" -> ["align-self", "justify-self"]
     Do NOT include value-laden or complex parsing-order shorthands like `font`, `background`, `border` (the single all-edges `border`), `flex`, `transition`, `list-style` here — those have special parsing rules and are out of scope for this pure positional-longhand data table.
  3. Add a public lookup function:
       /// Returns the ordered longhand property names for a shorthand, if `name` is a known shorthand.
       /// The lookup is ASCII-case-insensitive, matching `lookup`.
       pub fn shorthand_longhands(name: &str) -> Option<&'static [&'static str]> { ... }
     Implement it by scanning SHORTHAND_EXPANSIONS with `eq_ignore_ascii_case` on `name` (mirror however the existing `lookup` does case handling). NO unwrap/expect/panic in this (non-test) code (I-6).

Do NOT modify the existing PropertyMetadata struct, PROPERTY_METADATA table, or existing functions.

TESTS — ADD a few assertions to the existing `mod tests` (do not delete or modify existing tests):
  - `shorthand_longhands("margin")` == Some(&["margin-top", "margin-right", "margin-bottom", "margin-left"][..])  (compare the returned slice equals the expected slice).
  - `shorthand_longhands("OVERFLOW")` is Some and has length 2 (case-insensitive).
  - `shorthand_longhands("color")` is None (a longhand is not a shorthand).
  - `shorthand_longhands("border-radius")` is Some with 4 entries, first == "border-top-left-radius".
  - Add a test asserting SHORTHAND_EXPANSIONS has no duplicate `name`s.

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test css::property` then full `cargo test` to confirm nothing broke. If all green:
  git add -A && git commit -m "feat(css): add static shorthand-expansion metadata table for MS-CSS-Generic (t0439)"
Then print "T0439 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
