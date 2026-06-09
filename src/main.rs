//! Binary entry point — a tiny end-to-end demo of the `underrated` engine:
//!
//! 1. build a search-form DOM and submit it → a search URL (`forms`),
//! 2. fetch that URL over HTTP (`loader::HttpLoader`),
//! 3. parse + style + layout + paint + rasterize the response (`engine`),
//! 4. show the pixels in a native window (`shell`).
//!
//! Run on a desktop with a display: `cargo run`. Headless (CI / dev container)
//! has no window and may have no network — every step degrades gracefully
//! (no panics) and falls back to a built-in sample page.

use underrated::dom::{Dom, NodeData};
use underrated::engine::render_to_canvas;
use underrated::forms::{self, FormState};
use underrated::loader::{HttpLoader, ResourceLoader};
use underrated::shell::WinitWindow;
use underrated::url::Url;

/// Built-in page shown when there is no network (so `cargo run` always shows
/// something). Exercises block layout, background colors and text rendering.
const SAMPLE_HTML: &str = "<!DOCTYPE html><html><body>\
    <div class=\"banner\"></div>\
    <p>underrated: rendered locally</p>\
    </body></html>";
const SAMPLE_CSS: &str = "body { margin: 0; } \
    .banner { width: 800px; height: 60px; background-color: rgb(40, 120, 220); } \
    p { color: rgb(20, 140, 60); }";

/// Builds a `<form action=action method=get><input name=q></form>` DOM and
/// returns the navigation URL produced by submitting it with `query`.
fn search_url(action: &str, query: &str) -> Option<String> {
    let mut dom = Dom::new();
    let form = dom.create_node(NodeData::Element {
        name: "form".into(),
        attrs: vec![
            ("action".into(), action.into()),
            ("method".into(), "get".into()),
        ],
    });
    let input = dom.create_node(NodeData::Element {
        name: "input".into(),
        attrs: vec![("name".into(), "q".into())],
    });
    dom.append_child(form, input);

    let mut state = FormState::new();
    state.set_value(input, query);
    forms::submit(&dom, form, &state).map(|req| req.url)
}

/// Tries to fetch `url` over HTTP and return the body as a UTF-8 string.
fn fetch(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let bytes = HttpLoader.load(&parsed).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn main() {
    let width: u32 = 800;
    let height: u32 = 600;

    // 1. Search: build the form and produce the query URL.
    let url = search_url("https://example.com/", "hello underrated")
        .unwrap_or_else(|| "https://example.com/".to_string());
    println!("search URL: {url}");

    // 2. Fetch it; fall back to the built-in sample when offline.
    let (html, css) = match fetch(&url) {
        Some(body) => {
            println!("fetched {} bytes from {url}", body.len());
            // The fetched page carries its own <style>; we don't extract it yet,
            // so render the structure with no author CSS. // TODO(spec): hoist
            // <style> from the DOM into the stylesheet.
            (body, String::new())
        }
        None => {
            println!("offline / fetch failed — showing the built-in sample page");
            (SAMPLE_HTML.to_string(), SAMPLE_CSS.to_string())
        }
    };

    // 3 + 4. Render to pixels and present in a window.
    let window = WinitWindow::new("underrated", width, height);
    window.run(move || render_to_canvas(&html, &css, width, height));
}
