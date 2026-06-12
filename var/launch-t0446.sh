#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0446
LOG=/workspaces/toy-browser/var/log/t0446.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the code, run the checks, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files, the ADR, and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7. NEVER use unwrap()/expect() in non-test code (I-6). NEVER add a new external crate dependency to Cargo.toml — use only std.

Task t0446 — MS-CSS-Architecture-Apply, item 1 (Core Data Structures). This is a SELF-CONTAINED, ADDITIVE, SINGLE-MODULE task. You create ONE new file `src/style/categorized.rs` and add ONE `mod` line to `src/style/mod.rs`. You do NOT touch or rename the existing `ComputedStyle` (HashMap-based) — it must keep compiling and all existing tests must stay green. We are introducing the new categorized type ALONGSIDE the old one; later tasks migrate the cascade and consumers.

REQUIRED READING (read these first, in this order):
  1. `/workspaces/underrated-meta/docs/spec/0001-computed-style-layout.md` — ADR 0001. Section 2.1 has a TABLE mapping EVERY supported CSS property into exactly one of 11 category structs (5 inherited + 6 reset). This table is your authoritative field list.
  2. `src/css/prototype/mod.rs` — the proven prototype (toy: InheritedText + ResetBox only). It shows EXACTLY the patterns to follow: per-category struct with `Default`, process-wide shared initial `Arc`s via `OnceLock`, a bundle struct holding `Arc<Category>` per category, `initial()`, `inherit_from(parent)`, and `Arc::make_mut`-based COW setters. PORT AND EXPAND this pattern to all 11 categories with production fields.

STEP 1 — Create `src/style/categorized.rs` with the 11 category structs from ADR 2.1, EVERY property in the ADR table represented as one field. Categories (and which fields go in each) are EXACTLY per the ADR table:
  Inherited: `InheritedText`, `InheritedList`, `InheritedTable`, `InheritedUI`, `InheritedEffects`.
  Reset:     `ResetBox`, `ResetSurround`, `ResetBackground`, `ResetFlex`, `ResetTable`, `ResetEffects`.

  FIELD NAMING: snake_case of the CSS property (e.g. `background-color` -> `background_color`, `border-top-left-radius` -> `border_top_left_radius`).

  FIELD TYPE RULES (apply consistently; this is a structural scaffold — the cascade wiring in a LATER task will refine as needed, so do NOT overthink types):
    - Length-valued properties that can be `auto`/unspecified (width, height, min-/max- width/height, all margin-*, all padding-*, all border-*-width, top/right/bottom/left, flex-basis, all border-*-radius, text-indent, letter-spacing, word-spacing, outline-width, vertical-align): `i32`, with sentinel `-1` meaning auto/initial. Default `-1`.
    - Strictly-positive size properties (font-size, line-height, tab-size, border-spacing, transition-duration): `u32`. Sensible defaults: font-size `16`, line-height `20`, tab-size `8`, others `0`.
    - Factor properties (opacity, flex-grow, flex-shrink): `f32`. Defaults: opacity `1.0`, flex-grow `0.0`, flex-shrink `1.0`.
    - Integer-valued (z-index, order): `i32`, default `0`.
    - EVERYTHING ELSE (all keyword enums like display/position/float/clear/overflow/box-sizing/white-space/direction/text-align/text-transform/font-style/font-weight/font-variant/font-stretch/word-break/overflow-wrap/text-align-last/hyphens/list-style-type/list-style-position/caption-side/border-collapse/cursor/visibility/empty-cells/object-fit/pointer-events/flex-direction/flex-wrap/justify-content/align-items/align-self/table-layout/all border-*-style/outline-style/background-repeat/background-attachment/text-decoration-line/text-decoration-style; AND color/url/textual values like color/font-family/all border-*-color/background-color/background-image/background-position/background-size/outline-color/text-decoration-color/text-overflow/transition-property/list-style-image/quotes): `String`, holding the CSS keyword/initial value as text (e.g. color `"black"`, font-family `"sans-serif"`, display `"inline"`, position `"static"`, visibility `"visible"`, background-color `"transparent"`, etc. Use the CSS-spec initial value as the Default).

  Each category struct: `#[derive(Debug, Clone, PartialEq)]` + a hand-written `impl Default` returning the CSS initial values per the rules above.

STEP 2 — Define the bundle type. Name it `CategorizedComputedStyle` (do NOT name it `ComputedStyle` — that name is taken by the existing type; we rename/promote later). Exactly like the ADR 2.1 struct but for all 11 categories:
  ```
  #[derive(Debug, Clone, PartialEq)]
  pub struct CategorizedComputedStyle {
      pub inherited_text: Arc<InheritedText>,
      pub inherited_list: Arc<InheritedList>,
      pub inherited_table: Arc<InheritedTable>,
      pub inherited_ui: Arc<InheritedUI>,
      pub inherited_effects: Arc<InheritedEffects>,
      pub reset_box: Arc<ResetBox>,
      pub reset_surround: Arc<ResetSurround>,
      pub reset_background: Arc<ResetBackground>,
      pub reset_flex: Arc<ResetFlex>,
      pub reset_table: Arc<ResetTable>,
      pub reset_effects: Arc<ResetEffects>,
  }
  ```
  Implement, mirroring the prototype:
    - process-wide `static INITIAL_<CATEGORY>: OnceLock<Arc<Category>>` for ALL 11 categories, each `get_or_init(|| Arc::new(Category::default()))`.
    - `pub fn initial() -> Self` — every field is the shared initial Arc (zero per-node allocation).
    - `pub fn inherit_from(parent: &Self) -> Self` — INHERITED categories are `parent.<cat>.clone()` (Arc pointer copy); RESET categories are the shared initial Arc (fresh reset).
    - At least these representative COW setters using `Arc::make_mut` (prove the pattern across an inherited and a reset category): `set_color(&mut self, String)` on inherited_text.color, `set_font_size(&mut self, u32)` on inherited_text.font_size, `set_width(&mut self, i32)` on reset_box.width, `set_display(&mut self, String)` on reset_box.display. Add a `// TODO(spec): generated/typed setters for the full property set arrive with the cascade-migration task (item 2).`

  Add a module doc comment noting this is the production target type per ADR 0001, introduced additively; the legacy `ComputedStyle` is migrated off in later tasks.

STEP 3 — In `src/style/mod.rs`, add near the TOP (after the existing `use`/at module root, before the `specificity` fn) the single line:
  `pub mod categorized;`
  Do NOT change anything else in mod.rs. Do NOT add `use` glob imports of it elsewhere. Do NOT touch the existing `ComputedStyle`.

STEP 4 — Add `#[cfg(test)] mod tests` INSIDE `src/style/categorized.rs` proving the sharing/COW thesis (mirror the prototype's assertions, scaled down — no 10k benchmark needed):
  - `initial()` then `inherit_from` a child: assert `Arc::ptr_eq` holds for an inherited category (e.g. inherited_text) between parent and child, and that a reset category (reset_box) of an inherited child equals the shared initial (ptr_eq to a fresh `initial()`'s reset_box).
  - COW: clone a style, call `set_color("red")` on the clone; assert the original still has the default color, the clone has "red", and their `inherited_text` Arcs are NO longer ptr_eq (diverged), while an untouched category (e.g. reset_flex) IS still ptr_eq between them.
  - Assert a couple of Default values match CSS initials (color == "black", display == default you chose, opacity == 1.0, width == -1).
  In tests, unwrap()/expect() is allowed.

SCOPE GUARD — `git status` before commit must show ONLY: new file `src/style/categorized.rs` and a one-line change to `src/style/mod.rs`. NOTHING else. If anything else changed, revert it. No new Cargo dependency (std only: `std::sync::{Arc, OnceLock}`). No unwrap()/expect()/panic in non-test code.

VERIFY before committing (ALL must pass):
  - `cargo fmt --all`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test style:: -- --nocapture`   (existing ComputedStyle tests AND your new categorized tests must pass)
  - `cargo build`

COMMIT on branch agent/t0446-categorized-style-types with message:
  feat(style): categorized Arc-shared CategorizedComputedStyle scaffold per ADR 0001 (t0446)
Then STOP. Report: the 11 categories with field counts, which COW setters you added, the test assertions, and confirm git status shows only the 2 expected files changed.
EOF
echo "$PROMPT" | setsid gemini -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta > "$LOG" 2>&1 &
echo "launched t0446 pid=$!"
