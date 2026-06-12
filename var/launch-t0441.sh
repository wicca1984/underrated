#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0441
LOG=/workspaces/toy-browser/var/log/t0441.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic).

Task t0441 — ADD CSS `hsl()` / `hsla()` color-function support to `src/css/values.rs`. Real sites commonly specify colors as `hsl(210, 50%, 40%)` or `hsla(210, 50%, 40%, 0.5)`, but the parser currently only understands `rgb()/rgba()`. This is purely additive and self-contained: the output is the EXISTING `Color::Rgba(u8,u8,u8,u8)` variant, so NO downstream consumer changes are needed.

Edit ONLY `src/css/values.rs`. Touch NO other file.

READ the file first. Study these existing items to match style exactly:
  - `pub enum Color { Rgba(u8, u8, u8, u8) }`
  - `fn parse_single_value(...)` — its `ComponentValue::Function { name, value }` arm currently does: `if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") { return parse_rgb_function(value).map(CssValue::Color); }`
  - `fn parse_rgb_function(components: &[ComponentValue]) -> Option<Color>` — copy its argument-collection shape (it skips Whitespace/Comma and pushes Number/Percentage args).

IMPLEMENT:
  1. In the `Function { name, value }` arm of `parse_single_value`, ADD (next to the rgb/rgba check):
       `if name.eq_ignore_ascii_case("hsl") || name.eq_ignore_ascii_case("hsla") { return parse_hsl_function(value).map(CssValue::Color); }`
  2. ADD a new free function `fn parse_hsl_function(components: &[ComponentValue]) -> Option<Color>`:
       - Collect args skipping Whitespace and Comma (mirror `parse_rgb_function`).
       - Hue: accept a `CssToken::Number(v)` (degrees). Normalize with `let h = ((v % 360.0) + 360.0) % 360.0;` so negatives wrap.
       - Saturation and Lightness: accept `CssToken::Percentage(v)` as the value in 0..=100; convert to 0.0..=1.0 (divide by 100) and clamp to [0.0, 1.0]. (Do NOT accept bare numbers for S/L.)
       - Optional 4th arg = alpha: accept `CssToken::Number(v)` clamped to [0.0,1.0] -> `(a * 255.0)` as u8. If absent, alpha = 255.
       - Require exactly 3 args (hsl) or 4 args (hsla); otherwise return None.
       - Convert HSL->RGB. Use this standard algorithm (no external crates):
            let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
            let hp = h / 60.0;
            let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
            let (r1, g1, b1) = match hp as i32 {
                0 => (c, x, 0.0),
                1 => (x, c, 0.0),
                2 => (0.0, c, x),
                3 => (0.0, x, c),
                4 => (x, 0.0, c),
                _ => (c, 0.0, x), // covers hp in [5,6)
            };
            let m = l - c / 2.0;
            let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;  // same for g, b
       - Return `Some(Color::Rgba(r, g, b, alpha))`.
  3. I-6: NO `unwrap`/`expect` in this function. Use `?`/match/clamp only.

TESTS — ADD `#[test]` functions to the existing `#[cfg(test)] mod tests` in `src/css/values.rs` (do NOT modify/delete existing tests). Cover:
  - `hsl(0, 100%, 50%)` parses to `CssValue::Color(Color::Rgba(255, 0, 0, 255))` (pure red).
  - `hsl(120, 100%, 50%)` -> green `Rgba(0, 255, 0, 255)`.
  - `hsl(240, 100%, 50%)` -> blue `Rgba(0, 0, 255, 255)`.
  - `hsl(0, 0%, 100%)` -> white `Rgba(255, 255, 255, 255)`; `hsl(0, 0%, 0%)` -> black `Rgba(0,0,0,255)`.
  - `hsla(0, 100%, 50%, 0.5)` -> `Rgba(255, 0, 0, 127)` (accept 127 or 128 — assert alpha is within 1 of 127). Actually assert `(alpha as i32 - 127).abs() <= 1`.
  - case-insensitivity: `HSL(...)` works.
  - Build the input via `parse_value(...)` the same way other tests in the file construct `&[ComponentValue]` (look at how existing rgb tests tokenize/parse, and reuse that exact path — e.g. via the tokenizer/parser helpers the tests already use). If existing tests call `parse_value` on a tokenized string, do the same.
Test code MAY use unwrap.

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test css::values` then full `cargo test` to confirm nothing broke. If all green:
  git add -A && git commit -m "feat(css): add hsl()/hsla() color function parsing (t0441)"
Then print "T0441 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
