#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0383
LOG=/workspaces/toy-browser/var/log/t0383.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0383 — statically parse and retain `role` and `aria-*` attributes on DOM nodes (groundwork for a future a11y tree). NO rendering, NO behavior change — pure structural retention.

Targets: src/dom/ (the node/element struct) and src/html/ (the attribute parsing path). Touch ONLY files under src/dom/ and src/html/. Do NOT edit layout/, paint/, style/, engine/, script/, main.rs, or any other module. If something genuinely requires another module, leave a `// TODO(spec): ...` comment in the closest target file and stop.

FIRST verify nothing already does this: grep src/dom and src/html for `aria` and `role`. The orchestrator confirmed there is currently NO aria/role handling — but double-check before adding, and if you find existing support, STOP and instead just add any missing tests.

Background (read before coding):
- Read src/dom/ to find the element representation. HTML elements store their attributes somewhere — find the existing attribute storage (likely a map/Vec of (name,value) on the element node). Most likely attributes are ALREADY retained generically by the HTML parser (e.g. `el.attr("id")`, `el.get_attribute(...)`). If attributes are already stored generically, then `role` and `aria-*` are ALREADY retained as raw attributes and NO parsing change is needed — in that case the task is to add a small typed accessor + tests, NOT to re-plumb storage.
- Read src/html/ to confirm the parser keeps arbitrary attributes (it almost certainly does).

Implement (minimal, idiomatic, matching surrounding code):
1. Confirm `role` and `aria-*` attributes survive parsing into the DOM (they should, via generic attribute retention). If they do NOT (parser drops unknown attrs), fix the parser in src/html/ to retain them — but do NOT special-case them beyond generic retention.
2. Add convenience accessors on the DOM element type (in src/dom/) — match the existing accessor style in that file:
   - `pub fn role(&self) -> Option<&str>` — returns the value of the `role` attribute if present.
   - `pub fn aria(&self, name: &str) -> Option<&str>` — returns the value of attribute `aria-{name}` (e.g. `el.aria("label")` reads `aria-label`). Build the lookup key as `format!("aria-{name}")` and read via the existing attribute getter.
   These are thin wrappers over the existing attribute accessor — do not duplicate storage.
3. Do NOT add a separate parallel data structure, a11y tree, or any rendering. Pure retention + accessors only.
4. Panic-free (AGENTS.md I-6): no unwrap()/expect()/panicking indexing in non-test code.

Add unit tests in the appropriate `#[cfg(test)] mod tests` block under src/dom/ (reuse the existing DOM/parse test helper that turns an HTML string into a DOM — find how other tests in src/dom or src/html parse `<div ...>` and read attributes, and copy that pattern):
- `test_role_attribute_retained`: parse `<div role="button">x</div>`; the div element's `role()` returns `Some("button")`.
- `test_aria_attribute_retained`: parse `<div aria-label="Close" aria-hidden="true">x</div>`; `aria("label")` returns `Some("Close")` and `aria("hidden")` returns `Some("true")`.
- `test_role_absent`: parse `<div>x</div>`; `role()` returns `None` and `aria("label")` returns `None`.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(dom): retain role and aria-* attributes with typed accessors (t0383)"
Then print "T0383 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
