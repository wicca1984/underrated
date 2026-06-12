#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0428
LOG=/workspaces/toy-browser/var/log/t0428.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic as the existing tests do).

Task t0428 — Expose the `Element.getBoundingClientRect()` DOM method to the JS execution context. Touch ONLY `src/script/mod.rs` (plus you MAY add a test in that same file). Do NOT edit any other file under src/, and do NOT edit `src/dom/rect.rs`.

WHY this is new (do not duplicate): `src/dom/rect.rs` already defines `DomRect` and `Dom::get_bounding_client_rect(&self, node) -> DomRect` (currently a DOM-side placeholder returning a zero rect, with a `// TODO(spec):` to later read the laid-out box). There is, however, NO JS-side binding: a page running script today cannot call `element.getBoundingClientRect()`. Your task is the script-layer wiring/preparation ONLY — expose the method so it returns a DOMRect-shaped object to JS by calling the EXISTING `dom.get_bounding_client_rect(...)`. Do NOT try to make it read real layout (that is a separate future task; keep using the existing dom method as-is).

Read these IN FULL before writing anything:
- `src/script/mod.rs` around the `__dom_bridge__` ObjectInitializer (search `getComputedStyleValue`): study `bridge_get_computed_style_value` (its `NativeFunction::from_fn_ptr` registration via `.function(..., JsString::from("getComputedStyleValue"), 2)`, how it resolves the element by `__key__`, how it accesses the shared Dom, and how it returns a value to JS). Copy this pattern EXACTLY for your new native function.
- The JS-side method dispatch (search `getComputedStyleValue` again near line ~1984, and how methods like `setAttribute`/`getAttribute` are exposed on the element wrapper object) to see how a JS method forwards to `bridge.<name>(this.__key__, ...)`.
- `src/dom/rect.rs`: note `DomRect::get_bounding_client_rect`/`Dom::get_bounding_client_rect` signature and `DomRect::serialize()` which returns a serde_json object with keys x,y,width,height,top,right,bottom,left.

Steps:
1. Add a native bridge function (e.g. `bridge_get_bounding_client_rect`) mirroring `bridge_get_computed_style_value`: take the element `__key__` arg, resolve the NodeId, call the shared `dom.get_bounding_client_rect(node)`, then build and return a JS object exposing the DOMRect fields (x, y, width, height, top, right, bottom, left). Use the same JS-value construction approach already used elsewhere in this file for returning objects (e.g. build an `ObjectInitializer`/`JsObject` with number properties, OR if the codebase returns JSON strings across the bridge, serialize via `DomRect::serialize()` and parse on the JS side — MATCH whatever existing convention `getComputedStyleValue`/other object-returning bridges use; discover it, do not assume).
2. Register it on the `__dom_bridge__` object with `.function(NativeFunction::from_fn_ptr(bridge_get_bounding_client_rect), JsString::from("getBoundingClientRect"), 1)`.
3. Expose `getBoundingClientRect` as a method on the element wrapper so JS `element.getBoundingClientRect()` forwards to `bridge.getBoundingClientRect(this.__key__)` and returns the rect object. Follow the exact wrapper-method pattern already present for other element methods.
4. Add ONE `#[test]` in `src/script/mod.rs` (reuse the existing test harness/setup pattern in that file): load a tiny HTML with an element, run JS that calls `element.getBoundingClientRect()` and reads `.width`/`.height`/`.x`/`.y`, and assert they are the values returned by the current dom stub (0.0 each). The test guards the wiring (method exists, is callable, returns a DOMRect-shaped object), NOT real layout values. Put a short `//` comment above the test naming what it guards.

Keep I-6: no `unwrap`/`expect` in non-test code paths — handle missing element/key gracefully (return undefined/null or a zero rect, matching how `bridge_get_computed_style_value` handles a missing element).

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(script): expose Element.getBoundingClientRect() binding returning a DOMRect object (t0428)"
Then print "T0428 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
