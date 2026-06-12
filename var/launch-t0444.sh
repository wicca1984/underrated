#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0444
LOG=/workspaces/toy-browser/var/log/t0444.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the code, run the checks, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7. NEVER use unwrap()/expect() in non-test code (I-6). NEVER add a new external crate dependency to Cargo.toml — use only std.

Task t0444 — MS-CSS-Architecture, Phase B: build an ISOLATED PROTOTYPE + MICROBENCHMARK that PROVES the categorized, Arc-shared ComputedStyle design from ADR 0001. This is a self-contained, additive task. Do NOT modify the existing style engine, layout, paint, or the real `ComputedStyle`. You create ONE new module directory `src/css/prototype/` and add exactly ONE line to `src/css/mod.rs` to register it.

STEP 0 — READ the ADR you are implementing (authoritative design source):
  - `/workspaces/wt/t0443/docs/architecture/0001-computed-style-layout.md`
    Read its category mapping (InheritedText / InheritedList / InheritedTable / InheritedUI / InheritedEffects and ResetBox / ResetSurround / ResetBackground / ResetFlex / ResetTable / ResetEffects) and the Arc / copy-on-write sharing model. You will implement a faithful but TRACTABLE subset, not all 100+ properties.
  - For grounding, skim `src/style/mod.rs` (current `ComputedStyle` = per-node `HashMap<String, CssValue>`, this is the BASELINE you compare against) and `src/css/property.rs` (inherited/initial metadata).

STEP 1 — IMPLEMENT the prototype in `src/css/prototype/mod.rs` (you may split into a couple of files under that dir):
  Implement a REPRESENTATIVE subset of the ADR's categories — enough to prove the thesis. Concretely implement at minimum:
    - `InheritedText { color: String, font_family: String, font_size: u32, line_height: u32 }`  (an inherited category)
    - `ResetBox { display: u8, width: i32, height: i32, position: u8 }`                          (a reset category)
  Each implements `Default` (= the category's initial values) and derives `Debug, Clone, PartialEq`.
  Define the prototype style node:
    ```rust
    use std::sync::Arc;
    #[derive(Debug, Clone, PartialEq)]
    pub struct ProtoComputedStyle {
        pub inherited_text: Arc<InheritedText>,
        pub reset_box: Arc<ResetBox>,
    }
    ```
  Provide:
    - `ProtoComputedStyle::initial() -> Self` returning a style whose categories point at process-wide SHARED initial `Arc`s (use `std::sync::OnceLock<Arc<InheritedText>>` / `OnceLock<Arc<ResetBox>>` so EVERY default node shares ONE allocation per category — this is the Style-Sharing win).
    - `inherit_from(parent: &ProtoComputedStyle) -> Self`: child clones the parent's `inherited_text` Arc (zero-alloc inheritance) and gets a fresh shared-initial `reset_box`.
    - A copy-on-write mutator, e.g. `set_color(&mut self, color: String)` that does `Arc::make_mut(&mut self.inherited_text).color = color;` — proving COW: only nodes that diverge allocate.

STEP 2 — MICROBENCHMARK as a `#[cfg(test)]` module inside the prototype dir (NOT a Cargo bench, NO criterion). It must:
  - Build a synthetic tree of N = 10_000 nodes where the vast majority keep inherited text identical to a root (only a small fraction, e.g. every 100th node, diverges via `set_color`).
  - BASELINE: build the same N nodes using `std::collections::HashMap<String, String>` per node carrying the same ~8 inherited+reset properties (mimicking today's per-node map representation).
  - Measure with `std::time::Instant` the build time of each approach, and a MEMORY PROXY for each:
      * prototype: count DISTINCT `InheritedText` allocations = 1 shared initial + number of diverged nodes (assert it is far below N, e.g. <= N/50 + 2). Use `Arc::strong_count` and/or pointer identity (`Arc::as_ptr`) collected into a `HashSet<usize>` to count unique allocations.
      * baseline: N separate HashMaps (inherent — just state the count = N).
  - Print a human-readable comparison line via `println!` (visible with `cargo test -- --nocapture`), e.g.
      `proto: distinct InheritedText allocs = X (of 10000 nodes), build = ?ms ; baseline: 10000 maps, build = ?ms`
  - Add `#[test]` assertions that ENCODE the thesis so the win is machine-checked, not just printed:
      * `assert!(distinct_inherited_allocs <= N/50 + 2);`  (sharing collapses 10k nodes to a handful of allocations)
      * a sanity assert that COW divergence actually changed only the diverged nodes' color and left shared nodes untouched.
  Keep N modest enough that the test runs in well under a few seconds in CI debug mode.

STEP 3 — REGISTER the module: add `pub mod prototype;` to `src/css/mod.rs` (alphabetical placement near the other `pub mod` lines). That ONE line is the only edit outside `src/css/prototype/`.

CONSTRAINTS / SCOPE GUARD:
  - `git status` before commit must show ONLY: new files under `src/css/prototype/` and the one-line change to `src/css/mod.rs`. NOTHING else. If anything else changed, revert it.
  - No new Cargo dependency. std only. No unwrap()/expect() outside #[cfg(test)] code.
  - Do not touch the real `ComputedStyle`, layout, paint, or any existing test.

VERIFY before committing (all must pass):
  - `cargo fmt --all`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test css::prototype -- --nocapture`  (the benchmark test prints its comparison and passes its assertions)
  - `cargo build`

COMMIT on branch agent/t0444-css-prototype-bench with message:
  feat(css): Arc-shared ComputedStyle prototype + Style-Sharing microbenchmark (t0444)
Then STOP. Report: the printed benchmark comparison line, the distinct-allocation count vs N, and confirm git status scope.
EOF
echo "$PROMPT" | setsid gemini -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta > "$LOG" 2>&1 &
echo "launched t0444 pid=$!"
