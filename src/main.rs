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
use underrated::engine::render_page_to_canvas;
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

#[derive(Debug, PartialEq, Eq)]
enum Target {
    Url(String),
    Search(String),
    Default,
}

/// Resolves the command line arguments to determine the execution target.
/// Expects user-provided arguments (excluding the executable name).
fn resolve_target(args: &[String]) -> Target {
    if let Some(arg) = args.first() {
        match Url::parse(arg) {
            Ok(parsed) if parsed.scheme == "http" || parsed.scheme == "https" => {
                Target::Url(arg.clone())
            }
            _ => Target::Search(arg.clone()),
        }
    } else {
        Target::Default
    }
}

fn main() {
    let width: u32 = 800;
    let height: u32 = 600;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let target = resolve_target(&args);

    let url = match target {
        Target::Url(direct_url) => {
            println!("direct URL: {direct_url}");
            direct_url
        }
        Target::Search(query) => {
            let url = search_url("https://example.com/", &query)
                .unwrap_or_else(|| "https://example.com/".to_string());
            println!("search URL: {url}");
            url
        }
        Target::Default => {
            let url = search_url("https://example.com/", "hello underrated")
                .unwrap_or_else(|| "https://example.com/".to_string());
            println!("search URL: {url}");
            url
        }
    };

    // 2. Fetch it; fall back to the built-in sample when offline.
    let html = match fetch(&url) {
        Some(body) => {
            println!("fetched {} bytes from {url}", body.len());
            // The fetched page carries its own <style>; we don't extract it yet,
            // so render the structure with no author CSS. // TODO(spec): hoist
            // <style> from the DOM into the stylesheet.
            body
        }
        None => {
            println!("offline / fetch failed — showing the built-in sample page");
            format!("{}<style>{}</style>", SAMPLE_HTML, SAMPLE_CSS)
        }
    };

    // 3 + 4. Render to pixels and present in a window.
    let window = WinitWindow::new("underrated", width, height);
    let base_url = Url::parse(&url).unwrap();
    let loader = HttpLoader;
    window.run(move || render_page_to_canvas(&html, &base_url, &loader, width, height));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_target_empty() {
        let args: Vec<String> = vec![];
        assert_eq!(resolve_target(&args), Target::Default);
    }

    #[test]
    fn test_resolve_target_absolute_https() {
        let args = vec!["https://example.com/foo?bar=baz".to_string()];
        assert_eq!(
            resolve_target(&args),
            Target::Url("https://example.com/foo?bar=baz".to_string())
        );
    }

    #[test]
    fn test_resolve_target_absolute_http() {
        let args = vec!["http://localhost:8080/".to_string()];
        assert_eq!(
            resolve_target(&args),
            Target::Url("http://localhost:8080/".to_string())
        );
    }

    #[test]
    fn test_resolve_target_plain_query_string() {
        let args = vec!["hello underrated browser".to_string()];
        assert_eq!(
            resolve_target(&args),
            Target::Search("hello underrated browser".to_string())
        );
    }

    #[test]
    fn test_resolve_target_capitalized_scheme() {
        let args = vec!["HTTPS://Example.Com/".to_string()];
        assert_eq!(
            resolve_target(&args),
            Target::Url("HTTPS://Example.Com/".to_string())
        );
    }

    #[test]
    fn test_resolve_target_unsupported_scheme() {
        let args = vec!["ftp://example.com/".to_string()];
        assert_eq!(
            resolve_target(&args),
            Target::Search("ftp://example.com/".to_string())
        );
    }
}
