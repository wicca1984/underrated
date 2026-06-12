#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0415
LOG=/workspaces/toy-browser/var/log/t0415.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0415 — determine and retain the document quirks mode from the DOCTYPE token. Touch ONLY files under src/html/ (specifically src/html/tree.rs, plus you MAY add the QuirksMode enum in that same file). Do NOT edit src/dom/ or any other module. If the dom-node side truly needs changing, leave a `// TODO(spec): ...` and report instead — do NOT modify NodeData::Document.

Context (read before coding):
- The HTML tree builder lives in src/html/tree.rs: `pub fn parse_document(input: InputStream) -> Dom` and `pub struct TreeBuilder { ... }`. The DOCTYPE is handled inside TreeBuilder where there is currently `// TODO(spec): handle quirks mode` around line 131, matching `Token::Doctype { name, public_id, system_id, force_quirks, .. }`.
- The Doctype token (src/html/tokenizer.rs) carries: `name: Option<String>`, `public_id: Option<String>`, `system_id: Option<String>`, `force_quirks: bool`. Read these — do NOT edit tokenizer.rs.

What to implement (all within src/html/tree.rs):
1. Add `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub enum QuirksMode { #[default] NoQuirks, Quirks, LimitedQuirks }`.
2. Add a pure function that maps a DOCTYPE to a QuirksMode following the HTML Standard "quirks mode" algorithm (the relevant, well-defined subset):
   `fn quirks_mode_for_doctype(name: Option<&str>, public_id: Option<&str>, system_id: Option<&str>, force_quirks: bool) -> QuirksMode`.
   Implement the standard rules:
   - If `force_quirks` is true => Quirks.
   - If `name` is not "html" (ASCII case-insensitive) => Quirks.
   - Compare `public_id` ASCII-case-insensitively against the known prefixes/exact strings that force Quirks (e.g. starts-with "-//W3C//DTD HTML 4.01 Frameset//", "-//W3C//DTD HTML 4.01 Transitional//", exact "-//W3O//DTD W3 HTML Strict 3.0//EN//", "-//IETF//DTD HTML//", the long list of legacy public identifiers per the spec) => Quirks. Implement the spec's documented set of public-id prefixes and the exact-match list; it is acceptable to cover the spec's enumerated set.
   - The two public-id prefixes "-//W3C//DTD XHTML 1.0 Frameset//" and "-//W3C//DTD XHTML 1.0 Transitional//" => LimitedQuirks; and if a `system_id` is present, the HTML 4.01 Frameset/Transitional public-id prefixes also yield LimitedQuirks instead of Quirks (apply the spec's system-id-present nuance).
   - Otherwise => NoQuirks.
   Keep the matching ASCII-case-insensitive. Use helper(s) for the prefix/exact lists to stay readable.
3. Add a `quirks_mode: QuirksMode` field to `TreeBuilder` (default NoQuirks via `QuirksMode::default()` at construction). When the DOCTYPE token is processed (the existing arm with the TODO), compute and store `self.quirks_mode = quirks_mode_for_doctype(...)`. Keep the existing Doctype node creation/append as-is.
4. Expose the result so it is observable. Since `parse_document` returns `Dom` and wiring the flag onto the dom Document node crosses into src/dom/ (OUT OF SCOPE), leave a `// TODO(spec): surface quirks_mode onto the Document node / Dom once a dom-side field exists` and instead add a thin `pub fn parse_document_with_quirks(input: InputStream) -> (Dom, QuirksMode)` in src/html/tree.rs that runs the same build and also returns the computed `quirks_mode` (have `parse_document` delegate to it and drop the QuirksMode). This keeps everything in src/html/ and makes the value testable.

Keep it panic-free: no unwrap/expect/panicking indexing in non-test code. Document public items with `///` doc comments.

Tests — add to a `#[cfg(test)] mod tests` in src/html/tree.rs (do not delete existing tests). Use `parse_document_with_quirks` with small HTML inputs (build `InputStream` the same way existing tree.rs tests do):
- No DOCTYPE at all => Quirks (per spec, missing doctype in the initial mode triggers quirks). If the existing initial-mode handling differs, assert the actual builder behavior and note it; primarily test the `quirks_mode_for_doctype` function directly for the cases below.
- `<!DOCTYPE html>` => NoQuirks.
- A DOCTYPE with `force_quirks` true => Quirks.
- name other than "html" (e.g. `<!DOCTYPE foo>`) => Quirks.
- public_id "-//W3C//DTD HTML 4.01 Transitional//EN" with NO system_id => Quirks; WITH a system_id present => LimitedQuirks.
- public_id "-//W3C//DTD XHTML 1.0 Transitional//EN" => LimitedQuirks.
Prefer calling `quirks_mode_for_doctype(...)` directly for precise unit coverage of each branch.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(html): determine and retain document quirks mode from DOCTYPE (t0415)"
Then print "T0415 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
