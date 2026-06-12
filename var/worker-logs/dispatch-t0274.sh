#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0274
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0274 — Implement the CSS min/max sizing constraints `min-width`, `max-width`, `min-height`, `max-height` for block boxes. These clamp the already-resolved width/height of a block. This is a deliberately small, single-module, additive layout feature.

Scope: touch ONLY `src/layout/mod.rs` (including its inline `#[cfg(test)] mod tests`). `git diff --name-only` MUST show ONLY that one file. Do NOT modify style, css/values, paint, or any other module/worktree. Properties flow through generically (the style system already stores arbitrary keyword/length properties; you read them with `style.get(...)` / `get_px(...)` — no style-module change is needed, exactly like `width`/`height` are read today).

Reuse / facts (verified — read these before writing):
- `get_px(style, prop, default)` is at src/layout/mod.rs line 629 and resolves a CssValue::Length property to px (the SAME helper used for `width`, `height`, padding, margins). Use it to read min/max props.
- WIDTH is resolved inside `resolve_margins_and_width(...)` (line 665). The content width is assigned to the local `content_width` in TWO branches: the auto-width branch (`content_width = (containing_width - ... ).max(0.0)`) and the definite-width branch (`content_width = get_px(style, "width", 0.0)`), after which `total_non_margin_width = content_width + border_left + border_right + padding_left + padding_right` is computed and margins are resolved from it.
- HEIGHT is resolved at line 382: `let border_box_height = get_px(style, "height", content_height) ...`. `content_height` (line 378) is the auto content height from children.

Implement (CSS clamping semantics):
1. WIDTH clamp — inside `resolve_margins_and_width`, AFTER `content_width` is assigned in BOTH branches and BEFORE `total_non_margin_width` is computed, clamp `content_width` to the min/max-width constraints:
   - Read `max_width = style.get("max-width")` as px via get_px ONLY if it is a definite Length (the keyword `none`, or absent, means no maximum — skip). If a definite max-width is present and `content_width > max_width`, set `content_width = max_width`.
   - Read `min_width` as px via get_px if it is a definite Length (absent / `auto` means 0 → no effective minimum). If present and `content_width < min_width`, set `content_width = min_width`.
   - Apply max FIRST then min, so that min wins when min > max (per CSS 2.1 §10.4). Clamp the result with `.max(0.0)`. Factor this into a tiny local helper if it keeps both call sites DRY, but a small inline block at each site is acceptable — match the surrounding code style. Recompute `total_non_margin_width` from the clamped `content_width` (i.e. just keep the existing line that follows; do not duplicate it).
2. HEIGHT clamp — at line 382, after computing `border_box_height` from `get_px(style, "height", content_height)`, clamp it by `min-height`/`max-height` with the same semantics (max first, then min; `none`/absent max = no limit; absent/`auto` min = 0). Note: per spec min/max-height apply to the box''s height the same way `height` does here; keep parity with how `height` is already treated in this file (do NOT change the existing content-box vs border-box treatment of `height` — only clamp the same quantity `height` produces). Leave a `// TODO(spec): min/max-height clamp box-sizing interaction follows the existing height treatment; percentage min/max sizes are not resolved.` if you notice any ambiguity.
3. Percentages: if min/max props are given as percentages and the existing `get_px`/length machinery does not resolve them, treat them as not-applied and cover with the single `// TODO(spec):` above — do NOT invent percentage resolution beyond what `width`/`height` already do.

Do NOT use unwrap/expect/panic/unsafe in non-test code (I-6). No new dependencies (I-1).

Acceptance — add inline unit tests in src/layout/mod.rs (reuse the existing layout-test harness in this file — find a test that builds a DOM + stylesheet via `parse_stylesheet`, runs the layout entry point, and asserts on a box rect; mirror its construction):
- max-width clamps: a block with `width: 1000px; max-width: 200px;` inside a wide viewport produces a border-box/content width reflecting 200px (not 1000px).
- min-width clamps: a block with `width: 10px; min-width: 300px;` produces a width reflecting 300px.
- min beats max: `width: 500px; max-width: 100px; min-width: 300px;` resolves to 300px.
- max-height clamps: a tall block with an explicit `height: 1000px; max-height: 50px;` produces border-box height 50px.
- min-height clamps: a short block with `min-height: 400px;` produces border-box height >= 400px.
- A block with NONE of these props is unchanged from current behavior (regression guard — assert an existing width/height path still holds).
Do NOT weaken or remove any existing test.

Done when ALL of these pass in this worktree:
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Comments and identifiers in English.
Commit (you MUST commit before finishing, BEFORE the worktree can be removed): `feat(layout): clamp block width/height with min/max-width and min/max-height (t0274)`.
End with a short English summary of exactly what changed in src/layout/mod.rs and any `// TODO(spec):` you left.'
