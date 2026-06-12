#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0387
LOG=/workspaces/toy-browser/var/log/t0387.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0387 — add a typed accessor for the HTML `loading` attribute (lazy-loading hint) on DOM nodes, mirroring the existing `role` / `aria` typed-accessor pattern. Touch ONLY files under src/dom/. Do NOT edit html/, paint/, layout/, style/, engine/, loader/, main.rs, or any other module. If something genuinely requires another module, leave a `// TODO(spec): ...` comment in the closest src/dom/ file and stop.

Background (read before coding):
- Read src/dom/mod.rs around lines 30-50 and 160-180. There is an existing two-layer typed-accessor pattern added for `role`/`aria`:
  - A `NodeData`/element-level accessor (~line 34) `pub fn role(&self) -> Option<&str>` that scans the element's attributes.
  - A `Dom`-level accessor (~line 167) `pub fn role(&self, node: NodeId) -> Option<&str>` that calls `self.get_attribute(node, "role")`.
- The generic `loading` attribute is ALREADY parsed and stored by the HTML parser into the element's attribute map (like `role`). You are NOT adding parsing in html/ — you are exposing a typed accessor that reads the already-stored attribute. Confirm by reading how `role` reads its attribute.
- HTML spec for `<img loading>` and `<iframe loading>`: the attribute is an enumerated attribute with keywords `lazy` and `eager`. The missing-value default and the invalid-value default are BOTH `eager`. (https://html.spec.whatwg.org/multipage/urls-and-fetching.html#lazy-loading-attributes) Matching is ASCII-case-insensitive.

Implement (minimal, idiomatic, matching surrounding code) in src/dom/ (mod.rs, mirroring role/aria exactly):
1. Add a small public enum near the other DOM types:
   `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum ImageLoading { Eager, Lazy }`
   with a private/pub helper `fn parse_loading(value: Option<&str>) -> ImageLoading` that returns `ImageLoading::Lazy` only when the trimmed value ASCII-case-insensitively equals "lazy", and `ImageLoading::Eager` otherwise (covers missing, "eager", and any invalid value per spec).
2. Add a `NodeData`/element-level accessor `pub fn loading(&self) -> ImageLoading` mirroring `role()`'s attribute scan, feeding the raw attribute value into `parse_loading`.
3. Add a `Dom`-level accessor `pub fn loading(&self, node: NodeId) -> ImageLoading` mirroring `role(node)`, using `self.get_attribute(node, "loading")` and `parse_loading`.
4. Panic-free (AGENTS.md I-6): no unwrap()/expect()/panicking indexing in non-test code.

NOTE on semantics (do NOT implement fetch behavior): per the project milestone, headless rendering treats `lazy` the same as `eager` for actual fetching. This task only PARSES and RETAINS the hint as a typed value. Leave a `// TODO(spec): loading=lazy currently behaves as eager (no viewport-proximity deferral); see src/loader` comment at the enum.

Add unit tests in the existing `#[cfg(test)] mod tests` block in src/dom/mod.rs (copy the `test_role_attribute_retained` pattern that builds a small DOM and queries via both accessors):
- `test_loading_lazy_retained`: `<img loading="lazy">` -> both the NodeData accessor and the Dom accessor return `ImageLoading::Lazy`.
- `test_loading_eager_explicit`: `<img loading="eager">` -> `ImageLoading::Eager`.
- `test_loading_default_eager`: `<img>` with no loading attribute -> `ImageLoading::Eager`.
- `test_loading_invalid_is_eager`: `<img loading="garbage">` -> `ImageLoading::Eager`.
- `test_loading_case_insensitive`: `<img loading="LAZY">` -> `ImageLoading::Lazy`.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(dom): expose typed loading (lazy/eager) attribute accessor (t0387)"
Then print "T0387 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
