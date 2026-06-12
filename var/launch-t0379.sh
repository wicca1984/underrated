#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0379
LOG=/workspaces/toy-browser/var/log/t0379.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0379 — wire "click a search-result link -> navigate to its href" into the interactive session loop.

Target file: src/main.rs ONLY. Touch ONLY src/main.rs. Do NOT edit engine/, forms/, dom/, paint/, layout/, shell/, or any other module or worktree. Use ONLY existing PUBLIC APIs from those modules. If something genuinely requires editing another module, leave a `// TODO(spec): ...` comment in src/main.rs and stop — do not edit other files.

Context (already on main, all public):
- src/main.rs `main()` builds a `Session { browsing_context, input_manager, form_state }` behind `Arc<Mutex<..>>` and registers an `event_closure` handling `underrated::shell::InputEvent::{Click{x,y}, Key{key}}`. The Click arm already calls `underrated::layout::hit_test(&session.browsing_context.page.layout, x as f32, y as f32) -> Option<NodeId>` to get the clicked node, then does focus handling for form inputs.
- DOM accessors: `dom.parent(node) -> Option<NodeId>`; `dom.data(node) -> Option<&NodeData>` where `NodeData::Element { name, .. }` gives the lowercase-ish tag name; `dom.get_attribute(node, "href") -> Option<&str>`. (Mirror the ancestor-walk-by-tag pattern used in src/forms/mod.rs around `find_form_for_button`.)
- Navigation: `underrated::forms::NavigationRequest { url: String, method: underrated::forms::Method, body: String, content_type: Option<String> }` is constructible (all fields pub; `Method::Get`). `underrated::engine::navigate(req: &NavigationRequest, base: &Url, loader: &dyn ResourceLoader, viewport_width: f32) -> Page` fetches + renders and resolves `req.url` against `base`. `browsing_context.navigate(new_page)` swaps the page (resets focus/caret/scroll).

Implement (ALL in src/main.rs):
1. Add a private helper:
   `fn find_link_href(dom: &underrated::dom::Dom, node: underrated::infra::NodeId) -> Option<String>`
   that walks from `node` up through ancestors (`dom.parent`) and, for the first ancestor (including `node` itself) that is an `<a>` element (`NodeData::Element { name, .. }` with name equal to "a", case-insensitive) carrying a NON-empty `href` attribute, returns that href as an owned `String`. Returns `None` if no such ancestor exists. (Match the import path actually used in main.rs for `NodeId` — it is `underrated::infra::NodeId` or already imported; reuse the existing import.)
2. In the Click arm of `event_closure`, AFTER the existing form-focus handling, when `hit_test` returned `Some(node)`: call `find_link_href`. If it returns `Some(href)`, construct a GET `NavigationRequest { url: href, method: Method::Get, body: String::new(), content_type: None }`, call `underrated::engine::navigate(&req, &base_url_clone, &underrated::loader::HttpLoader, width as f32)` to get the new `Page`, then `session.browsing_context.navigate(new_page)` and `session.input_manager.blur()`. Keep the existing form-input focus behavior intact for clicks that are NOT on links (a click that hits a link should navigate; a click that hits a form input should focus — if a node is both, prefer the form-input focus already coded OR link navigation, but do not break the existing input focus path for non-link clicks).
   - Reuse the already-captured `base_url_clone` and `width` bindings used elsewhere in the closure. Do not introduce a second loader type — use `underrated::loader::HttpLoader` as the rest of the file does.
3. Keep everything panic-free (AGENTS.md I-6): NO `unwrap()`/`expect()` in non-test code. Use `if let` / `match`.

Add a unit test in the existing `#[cfg(test)] mod tests` block in src/main.rs:
- `fn test_find_link_href_*`: build a small `Dom` with an `<a href="/result">...<span>text</span></a>` structure (use the dom test-construction helpers already used by other tests, or `Dom`/`create_node` as available), assert `find_link_href` from the inner span node returns `Some("/result".to_string())`, and that a node with no `<a>` ancestor returns `None`, and that an `<a>` with empty/missing href returns `None`.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(main): navigate on click of result-page links via find_link_href (t0379)"
Then print "T0379 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
