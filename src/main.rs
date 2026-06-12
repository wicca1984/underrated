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

use std::sync::{Arc, Mutex};
use underrated::dom::{Dom, NodeData};
use underrated::forms::{self, FormState};
use underrated::infra::NodeId;
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

/// Returns the appropriate URL string for the given execution target.
fn url_for_target(target: &Target) -> String {
    match target {
        Target::Url(direct_url) => direct_url.clone(),
        Target::Search(query) => search_url("https://www.google.co.jp/", query)
            .unwrap_or_else(|| "https://www.google.co.jp/".to_string()),
        Target::Default => "https://www.google.co.jp/".to_string(),
    }
}

/// Walks up from the given node to find the nearest <a> ancestor with a non-empty href attribute.
fn find_link_href(dom: &Dom, node: NodeId) -> Option<String> {
    let mut current = node;
    loop {
        if let Some(NodeData::Element { name, .. }) = dom.data(current)
            && name.eq_ignore_ascii_case("a")
            && let Some(href) = dom.get_attribute(current, "href")
            && !href.is_empty()
        {
            return Some(href.to_string());
        }
        if let Some(parent) = dom.parent(current) {
            current = parent;
        } else {
            break;
        }
    }
    None
}

fn main() {
    let width: u32 = 800;
    let height: u32 = 600;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let target = resolve_target(&args);

    let url = url_for_target(&target);
    match &target {
        Target::Url(direct_url) => {
            println!("direct URL: {direct_url}");
        }
        Target::Search(_) => {
            println!("search URL: {url}");
        }
        Target::Default => {
            println!("default URL: {url}");
        }
    }

    // 2. Fetch it; fall back to the built-in sample when offline.
    let html = match fetch(&url) {
        Some(body) => {
            println!("fetched {} bytes from {url}", body.len());
            body
        }
        None => {
            println!("offline / fetch failed — showing the built-in sample page");
            format!("{}<style>{}</style>", SAMPLE_HTML, SAMPLE_CSS)
        }
    };

    // 3 + 4. Render to pixels and present in a window.
    let window = WinitWindow::new("underrated", width, height);
    let base_url = match Url::parse(&url) {
        Ok(parsed) => parsed,
        Err(_) => {
            eprintln!("failed to parse base URL: {url}");
            return;
        }
    };
    let loader = HttpLoader;

    let page = underrated::engine::render_page(&html, &base_url, &loader, width as f32);
    let browsing_context = underrated::engine::BrowsingContext::new(page);
    let input_manager = underrated::shell::ShellInputManager::new();
    let mut form_state = underrated::forms::FormState::new();
    form_state.set_current_url(&url);

    struct Session {
        browsing_context: underrated::engine::BrowsingContext,
        input_manager: underrated::shell::ShellInputManager,
        form_state: underrated::forms::FormState,
    }

    let session = Arc::new(Mutex::new(Session {
        browsing_context,
        input_manager,
        form_state,
    }));

    let session_draw = session.clone();
    let draw_closure = move || {
        if let Ok(session) = session_draw.lock() {
            let caret = session
                .browsing_context
                .focus_node
                .map(|node| (node, session.browsing_context.caret_index));
            let display_list = underrated::paint::build_display_list_with_caret(
                &session.browsing_context.page.layout,
                &session.browsing_context.page.dom,
                &session.browsing_context.page.styles,
                caret,
            );
            underrated::raster::rasterize(&display_list, width, height)
        } else {
            underrated::raster::Canvas::new(width, height)
        }
    };

    let session_event = session.clone();
    let base_url_clone = base_url.clone();
    let event_closure = move |event: underrated::shell::InputEvent| {
        if let Ok(mut session) = session_event.lock() {
            match event {
                underrated::shell::InputEvent::Click { x, y } => {
                    let clicked_node = underrated::layout::hit_test(
                        &session.browsing_context.page.layout,
                        x as f32,
                        y as f32,
                    );
                    session.input_manager.handle_click(x, y, clicked_node);
                    let focused = session.input_manager.focused_element();
                    session.browsing_context.set_focus(focused);
                    if let Some(focused_node) = session.browsing_context.focus_node {
                        let caret_pos = session.input_manager.caret_position(focused_node);
                        session.browsing_context.caret_index = caret_pos;
                    }

                    if let Some(node) = clicked_node
                        && let Some(href) = find_link_href(&session.browsing_context.page.dom, node)
                    {
                        let req = underrated::forms::NavigationRequest {
                            url: href,
                            method: underrated::forms::Method::Get,
                            body: String::new(),
                            content_type: None,
                        };
                        let new_page = underrated::engine::navigate(
                            &req,
                            &base_url_clone,
                            &underrated::loader::HttpLoader,
                            width as f32,
                        );
                        session.browsing_context.navigate(new_page);
                        session.input_manager.blur();
                    }
                }
                underrated::shell::InputEvent::Key { key } => {
                    if let Some(focused) = session.input_manager.focused_element() {
                        if key == "Enter" || key == "Return" {
                            if let Some(new_page) = underrated::engine::navigate_from_enter(
                                &session.browsing_context.page.dom,
                                focused,
                                &session.form_state,
                                &base_url_clone,
                                &underrated::loader::HttpLoader,
                                width as f32,
                            ) {
                                session.browsing_context.navigate(new_page);
                                session.input_manager.blur();
                            }
                        } else {
                            session.input_manager.handle_key(&key);
                            let text = session.input_manager.text_buffer(focused).to_string();
                            let caret_pos = session.input_manager.caret_position(focused);
                            session.form_state.set_value(focused, &text);
                            session
                                .browsing_context
                                .page
                                .dom
                                .set_attribute(focused, "value", &text);
                            session.browsing_context.caret_index = caret_pos;
                        }
                    }
                }
            }
        }
    };

    window.run_with_input(draw_closure, event_closure);
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

    #[test]
    fn test_url_for_target_default() {
        let target = Target::Default;
        assert_eq!(url_for_target(&target), "https://www.google.co.jp/");
    }

    #[test]
    fn test_url_for_target_search() {
        let target = Target::Search("query text".to_string());
        let expected = search_url("https://www.google.co.jp/", "query text").unwrap();
        assert_eq!(url_for_target(&target), expected);
    }

    #[test]
    fn test_url_for_target_url() {
        let target = Target::Url("https://example.com/some/path".to_string());
        assert_eq!(url_for_target(&target), "https://example.com/some/path");
    }

    #[test]
    fn test_find_link_href_basic() {
        let mut dom = Dom::new();
        // Create an <a> node with href="/result"
        let a_node = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![("href".into(), "/result".into())],
        });
        // Create a <span> node
        let span_node = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        // Append span_node under a_node
        dom.append_child(a_node, span_node);

        // find_link_href starting from span should find "/result"
        assert_eq!(find_link_href(&dom, span_node), Some("/result".to_string()));
        // find_link_href starting from a_node itself should find "/result"
        assert_eq!(find_link_href(&dom, a_node), Some("/result".to_string()));

        // Node with no <a> ancestor should return None
        let div_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        assert_eq!(find_link_href(&dom, div_node), None);

        // <a> with empty href should return None
        let a_empty = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![("href".into(), "".into())],
        });
        assert_eq!(find_link_href(&dom, a_empty), None);

        // <a> with missing href should return None
        let a_missing = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![],
        });
        assert_eq!(find_link_href(&dom, a_missing), None);

        // <a> in uppercase "A" should be matched case-insensitively
        let a_upper = dom.create_node(NodeData::Element {
            name: "A".into(),
            attrs: vec![("href".into(), "/uppercase".into())],
        });
        assert_eq!(
            find_link_href(&dom, a_upper),
            Some("/uppercase".to_string())
        );
    }
}
