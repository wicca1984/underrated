#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0402
LOG=/workspaces/toy-browser/var/log/t0402.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0402 — implement the HTML5 "in frameset" and "after frameset" tree-construction insertion modes. Touch ONLY src/html/tree.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in tree.rs if something truly needs another module.

Background (read before coding):
- In src/html/tree.rs the dispatcher (around lines 89-111) currently has:
    InsertionMode::InFrameset => self.handle_in_body(token), // TODO(spec)
    InsertionMode::AfterFrameset => self.handle_after_after_body(token)?  -- actually it delegates to handle_in_body, a stub.
  Replace these two stubs with real handlers `handle_in_frameset` and `handle_after_frameset`.
- Model your code on the EXISTING `handle_after_body` and `handle_after_after_body` functions already in this file (read them first to match the exact token-matching idiom, the `self.dom.create_node`, `self.dom.append_child`, `self.stack_of_open_elements`, and `is_html_whitespace` helpers).

Spec to implement (HTML §13.2.6.4.7 "in frameset" and §13.2.6.4.8 "after frameset"):

handle_in_frameset(token):
- Character that IS html whitespace: insert the character (mirror how other modes insert a single whitespace character; if there is an existing `insert_character`/`insert_text` helper use it, otherwise append a Text node to current node).
- Comment: insert a comment node (append Comment to current node — the node at the top of the stack of open elements).
- Doctype: parse error, ignore.
- StartTag "html": process using the rules for "in body" (self.handle_in_body(...)).
- StartTag "frameset": insert an HTML element for the token (use the same element-insertion path the other start-tag handlers use; if a helper like `insert_html_element` / `insert_element_for_token` exists, use it).
- EndTag "frameset": if the current node is the root html element, parse error, ignore. Otherwise pop the current node off the stack of open elements. Then (if not in fragment case and the new current node is not a frameset) switch insertion mode to AfterFrameset. Keep the fragment-case nuance as a `// TODO(spec): fragment case` if the parser has no fragment flag.
- StartTag "frame": insert an HTML element for the token, then immediately pop it (frame is void / acknowledge self-closing).
- StartTag "noframes": process using the "in head" rules (self.handle_in_head(...)).
- Eof: stop parsing (no-op; optionally parse error if current node is not root html — leave a comment).
- Anything else: parse error, ignore the token.

handle_after_frameset(token):
- Character that IS html whitespace: insert the character.
- Comment: insert a comment node.
- Doctype: parse error, ignore.
- StartTag "html": process using "in body" rules.
- EndTag "html": switch insertion mode to AfterAfterFrameset.
- StartTag "noframes": process using "in head" rules.
- Eof: stop parsing.
- Anything else: parse error, ignore.

Then update the dispatcher arms:
    InsertionMode::InFrameset => self.handle_in_frameset(token),
    InsertionMode::AfterFrameset => self.handle_after_frameset(token),
Keep AfterAfterFrameset as-is (already delegates to handle_after_after_body).

IMPORTANT: Use ONLY helpers/methods that already exist in tree.rs (read the file to find the exact names for element insertion, character insertion, current-node access, and stack pop). Do not invent new infrastructure or new public APIs. If a needed primitive genuinely does not exist, implement the minimal logic inline using `self.dom` + `self.stack_of_open_elements` exactly as the neighboring handlers do, and leave a `// TODO(spec):` note. No unwrap/expect/panicking indexing in non-test code (use `.last()`, `if let`, etc.).

Tests — add to the existing `#[cfg(test)] mod tests` in src/html/tree.rs (do NOT modify/delete existing tests; mirror the style of the existing insertion-mode tests, e.g. the in-column-group / in-select tests added recently):
- A `<frameset>...<frame>...</frameset>` document parses without panicking and builds the expected frameset/frame element structure.
- After the outer `</frameset>`, a following `<noframes>` or whitespace is handled in the after-frameset mode without panicking.
Keep tests deterministic and use the existing parse helpers (e.g. `parse_document` + `InputStream::from_utf8`) that other tests in this file use.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(html): implement the in-frameset and after-frameset insertion modes (t0402)"
Then print "T0402 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
