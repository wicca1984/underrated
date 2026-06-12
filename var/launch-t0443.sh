#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0443
LOG=/workspaces/toy-browser/var/log/t0443.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the document, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7.

Task t0443 — AUTHOR an Architecture Decision Record (ADR) for the CSS style-engine redesign (milestone MS-CSS-Architecture). This is a DESIGN/DOCUMENTATION task: create ONE new Markdown file. Do NOT change any Rust source code, Cargo.toml, or tests. The ONLY file you create is `docs/architecture/0001-computed-style-layout.md`.

GOAL: Document a "no-going-back" foundation for `ComputedStyle` that scales to large DOMs without memory blowup or cache misses — modeled on Servo (Stylo) / WebKit. The decision direction is ALREADY chosen by the PdM; your job is to flesh it out concretely and faithfully, grounded in the ACTUAL current code (read it first), not to invent a different design.

STEP 1 — READ the current state (cite real names/paths in the ADR):
  - `src/style/mod.rs` — find the current `ComputedStyle` representation (how properties are stored today: is it a HashMap/BTreeMap of String->CssValue? how is inheritance resolved? `is_inherited_property`, `compute_styles`). Note its memory shape per node.
  - `src/css/property.rs` — the static property-metadata table (`is_inherited`, initial values). This is the seed of compile-time static dispatch.
  - `src/css/values.rs`, `src/css/colors.rs` — the `CssValue` enum and parsed value types.
  - Skim `src/style/resolve.rs` and any cascade code.

STEP 2 — WRITE `docs/architecture/0001-computed-style-layout.md` with these sections:
  1. **Title / Status / Date / Context**: Status = "Proposed". Context = why the current per-node dictionary `ComputedStyle` (cite what you actually found) does not scale: memory per node, cache locality, repeated string lookups in the cascade hot path.
  2. **Decision**: Adopt a Servo/Stylo-style **categorized, Arc-shared ComputedStyle**:
     - Split `ComputedStyle` into logical category structs — propose a concrete set such as `BoxStyle` (margins/padding/border/width/height/display/position), `TextStyle` (color/font-*/line-height/text-*/letter-spacing/word-spacing), `BackgroundStyle` (background-*), `SurroundStyle`/`EffectsStyle` (outline/box-shadow/visibility), etc. Map EVERY major property group the codebase currently supports into exactly one category (build the table by reading the code above). State the rule: inherited properties cluster into inherited categories (e.g. TextStyle) and non-inherited into reset categories — mirroring Stylo's "inherited vs reset structs".
     - Each node holds `Arc<CategoryStyle>` (or `Rc` — discuss the single-thread-now / multi-thread-later trade-off and recommend one, noting `Arc` keeps the door open for parallel styling). Identical category values are SHARED across thousands of nodes (Style Sharing) via Arc clone instead of deep copy. Describe copy-on-write: mutating a category clones just that one Arc, leaving siblings untouched.
  3. **Static dispatch & macros**: Describe replacing runtime String lookups with compile-time resolution. Propose a macro (or build-time codegen) that, from a single property declaration list, generates: the property id enum, parse fn dispatch, initial value, and inherited/reset classification — eliminating per-lookup string compares in the cascade. Reference the existing `src/css/property.rs` table as the migration seed.
  4. **Bitflags for inheritance/dirty tracking**: Propose `bitflags`-based flags for "which categories are dirty / explicitly set / inherited" so cascade and restyle damage are bit-ops over a small word, not struct walks.
  5. **Migration plan (incremental, no big-bang)**: Concrete phased steps that DO NOT rewrite everything at once: (a) introduce category structs behind the existing `ComputedStyle` getters as an internal representation; (b) prototype + microbenchmark in `src/css/prototype/` (forward-reference, this is the next task) proving memory/speed win vs the current map on a synthetic N-thousand-node tree; (c) flip categories on one at a time keeping public getters stable; (d) remove the legacy map last. Emphasize public getter API stays stable throughout.
  6. **Alternatives considered / rejected**: (a) naive single big `HashMap<String,CssValue>` generalization — rejected: memory blowup + cache misses on large DOM (this is the status quo we are leaving); (b) one giant flat `ComputedStyle` struct copied per node — rejected: no sharing, huge per-node size; (c) interning only. Briefly say why each loses to categorized-Arc-sharing.
  7. **Consequences**: trade-offs (Arc refcount overhead, COW complexity) vs wins (shared memory, cache-friendly, parallel-ready).

CONSTRAINTS:
  - Be concrete and grounded in the real code you read — name actual structs/functions/files. Do not hand-wave.
  - This is the foundation other tasks build on; correctness of the category mapping matters most. If something in the code is ambiguous, write a `> NOTE (spec):` line in the doc rather than guessing silently.
  - Markdown only. Create the parent dir `docs/architecture/` as needed.

VERIFY before committing:
  - The file `docs/architecture/0001-computed-style-layout.md` exists and renders as valid Markdown.
  - `git status` shows ONLY that one new file (plus the new dir). NO Rust files, NO Cargo.toml, NO test changes. If anything else changed, revert it.
  - `cargo build` is NOT required (docs only) but run `cargo fmt --all -- --check` to confirm you accidentally touched no Rust.

COMMIT on branch agent/t0443-css-arch-adr with message:
  docs(architecture): ADR 0001 categorized Arc-shared ComputedStyle layout (t0443)
Then STOP. Report the file path and a one-paragraph summary of the proposed category split.
EOF
echo "$PROMPT" | setsid gemini -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta > "$LOG" 2>&1 &
echo "launched t0443 pid=$!"
