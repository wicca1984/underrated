#!/usr/bin/env bash
# t0449 — correct unitless line-height (number) per-element resolution in layout. Base: feature/css-arch.
set -euo pipefail
cd /workspaces/wt/t0449

read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the code, run the checks, fix until green, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything is in local files. Network/web search is forbidden.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. Read AGENTS.md (passed via --include-directories) and obey I-1..I-7. NEVER use unwrap()/expect() in non-test code (I-6). NEVER add an external crate (std only). DO NOT delete or skip tests to fake green.

You are the ONLY worker on this worktree: /workspaces/wt/t0449, branch agent/t0449-line-height-number, base feature/css-arch (commit 9edf920). Touch ONLY layout consumers of line-height (primarily src/layout/inline.rs; also src/layout/block.rs or src/layout/mod.rs ONLY if they likewise resolve line-height). DO NOT touch src/paint/, src/layout/flex.rs, src/layout/table.rs, or src/style/.

BACKGROUND — the bug:
CSS `line-height: <number>` (unitless, e.g. 1.5) is a MULTIPLIER that inherits as the NUMBER; each element must resolve its own line-height = number * (that element's own font-size). The cascade (src/style/categorized.rs) already stores BOTH on inherited_text:
  - `line_height: u32`            -> px resolved against the DECLARING element's font-size (sentinel crate::style::categorized::LINE_HEIGHT_NORMAL means unset/normal)
  - `line_height_number: Option<f32>`  -> Some(multiplier) when the value was unitless
Because inherited_text is shared via Arc inheritance, a child that did not redeclare line-height inherits the parent's `line_height` px (computed from the PARENT's font-size) AND inherits `line_height_number`. Today the layout consumers read ONLY `style.inherited_text.line_height as f32`, so a child with a different font-size than the declaring ancestor gets the wrong (parent-derived) px. This is the "line-height Number representation" defect.

THE FIX (per-element resolution):
In each layout site that currently resolves a node's line-height from `style.inherited_text.line_height`, change the resolution to PREFER the unitless multiplier when present:
  - If `style.inherited_text.line_height_number` is Some(n): line_height = n * (this element's own font-size in px).
  - Else if `style.inherited_text.line_height != LINE_HEIGHT_NORMAL`: line_height = that px value (existing behaviour).
  - Else: fall back to the font's intrinsic line height (existing behaviour).
Obtain "this element's own font-size" from the element's resolved style font-size field used elsewhere in layout (grep for how font-size/font px is already read from CategorizedComputedStyle in the layout module — reuse that exact accessor; do NOT invent a new one and do NOT touch src/style/). If a node's own font-size is not readily available at the resolution site, thread it from where font metrics are already computed in that function. Keep the change minimal and localized.

In src/layout/inline.rs, the relevant spot resolves `node_line_height` for Text nodes (search for `inherited_text.line_height` and `current_line_height`). Apply the preference order there. Apply the same preference order at any other layout site that reads `inherited_text.line_height` to set a line box height.

If anything about per-element font-size resolution is genuinely ambiguous, leave a `// TODO(spec):` and still implement the multiplier-times-font-size path for the common case rather than stopping.

ADD a regression test: a parent with `line-height: 2` (unitless) and font-size 10px, containing a child with font-size 20px; assert the child's resolved line box height reflects 2*20=40 (child's own font-size), not 2*10=20 (parent's). Mirror the construction pattern of existing layout tests in the file. Do NOT weaken or delete existing tests.

PROCEDURE (iterate until all green):
  - cargo build
  - cargo fmt
  - cargo clippy --all-targets -- -D warnings   (fix every warning)
  - cargo test                                   (all pass; update any layout test that asserted the old parent-derived px to the correct per-element value, but do NOT delete tests)
  - git add -A && git commit -m "fix(layout): resolve unitless line-height per element against own font-size (t0449)"
  COMMIT before finishing (commit partial progress too). Report the final cargo test summary line.
EOF

exec gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
