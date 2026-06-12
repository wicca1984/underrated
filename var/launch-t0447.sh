#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0447
LOG=/workspaces/toy-browser/var/log/t0447.log
mkdir -p /workspaces/toy-browser/var/log
# restore GEMINI_API_KEY from canonical .env (bashrc may be wiped on rebuild)
if [ -f /workspaces/underrated-meta/var/.env ]; then set -a; . /workspaces/underrated-meta/var/.env; set +a; fi
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the code, run the checks, fix until green, then commit. This is a LARGE multi-file task — keep going across many edit/build cycles; do not stop until `cargo build` and `cargo test` pass or you are truly blocked.

CRITICAL: Do NOT use web search or any web tool. Everything is in local files, the ADR/SPEC, and this prompt. Network/web search is forbidden.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read AGENTS.md (via --include-directories) and obey I-1..I-7. NEVER use unwrap()/expect() in non-test code (I-6). NEVER add an external crate — std only.

Task t0447 — MS-CSS-Architecture-Apply: BIG-BANG migration to the categorized Arc-shared style type. This is the sanctioned EXCEPTION to I-5 (one module): you MUST edit many files across modules in ONE branch/PR. Temporary build breakage is fine; ONLY the final state must compile and pass tests.

REQUIRED READING FIRST (in order):
  1. /workspaces/underrated-meta/docs/spec/S-CSS-ARCH-APPLY.md   <- your authoritative contract.
  2. /workspaces/underrated-meta/docs/spec/0001-computed-style-layout.md  (ADR 0001, esp. 2.1 property->category table, 2.2 Arc COW).
  3. src/style/categorized.rs   (the scaffold: 11 category structs + CategorizedComputedStyle + initial()/inherit_from() + a few setters). You will EXTEND this.
  4. src/style/mod.rs           (the legacy ComputedStyle = HashMap<String,CssValue>; the cascade compute_styles / compute_styles_with_viewport you must rewrite).

GOAL: make `CategorizedComputedStyle` the ONLY style type. Delete the legacy HashMap-based `ComputedStyle` and its `get(&str)/insert(String,..)` API. No string-key style lookups survive anywhere outside src/style/.

STEP 1 — Extend src/style/categorized.rs:
  (a) Ensure every CSS property the cascade can produce maps to a typed field in some category (use the ADR 2.1 table). Field-type rules are in the SPEC.
  (b) Add COW setters (via `let s = Arc::make_mut(&mut self.<category>); s.<field> = v;`) for EVERY property that layout/paint currently WRITES back. Find them: `grep -rn 'style.insert(' src/layout src/paint`. At minimum: set_width(i32), set_height(i32), set_z_index(i32), and setters for box-shadow / text-shadow (String) — match whatever the grep shows.
  (c) Add a constructor that BUILDS a CategorizedComputedStyle from cascade input: take the parent's style (for inheritance) + the node's matched declarations (Vec/iter of (property_name:&str, CssValue)) and populate typed fields. Inherited categories start from `inherit_from(parent)`; reset categories start from initial(). For each declaration, match on the property name and write the parsed value into the right typed field (parse px/number -> i32/u32/f32; keyword/color/url -> String). Reuse any existing value-parsing helpers in src/css.

STEP 2 — Rewrite the cascade in src/style/mod.rs:
  - compute_styles(...) and compute_styles_with_viewport(...) now return HashMap<NodeId, CategorizedComputedStyle>.
  - Replace the per-node `properties: HashMap` build with the STEP 1(c) constructor.
  - Keep selector matching / specificity / cascade ordering logic intact; only the OUTPUT type changes.
  - DELETE struct ComputedStyle and its get/insert impl once nothing references them.

STEP 3 — Migrate ALL consumers to typed access (NO `.get("..")`, NO `.insert("..")`):
  Files: src/layout/{mod,inline,flex,table,float,position}.rs, src/paint/{mod,stacking}.rs, src/engine/{mod,flush}.rs, src/script/mod.rs.
  - Read: `style.get("display")` -> `style.reset_box.display` (a String; compare with == "block" etc.). `style.get("color")` -> `style.inherited_text.color`. `style.get("margin-top")` -> `style.reset_surround.margin_top` (i32). Map each old string key to its category.field per ADR 2.1. A value previously parsed from CssValue is now the already-typed field — drop the re-parse.
  - Write: `style.insert("width".to_string(), CssValue::..)` -> `style.set_width(px_i32)`. Same for height, z-index, shadows.
  - src/script/mod.rs getComputedStyle: serialize the typed fields back to CSS strings for the JS API (e.g. `format!("{}px", w)` for lengths, the String directly for keywords).
  - Helpers like `get_float_value` / `get_text_align` that took `&CssValue`: refactor to read the typed field, or delete if now trivial.

STEP 4 — Build & verify, iterate until green:
  - `cargo build` first; fix every error. Then `cargo fmt`, `cargo clippy --all-targets -- -D warnings` (fix warnings), `cargo test` (fix or update tests that asserted on the old string API — DO NOT delete tests to fake green; update them to the typed API; if a test checks rendering it must still pass).
  - When everything is green, `git add -A && git commit -m "feat(css): big-bang migrate cascade + all consumers to CategorizedComputedStyle, delete legacy HashMap ComputedStyle (t0447)"`.
  - You are on branch agent/t0447-css-bigbang-migration (base feature/css-arch). COMMIT before finishing. Report the final `cargo test` summary line.

Work methodically: build, read the first errors, fix, repeat. Do not give up after one cycle.
EOF
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null >> "$LOG" 2>&1
