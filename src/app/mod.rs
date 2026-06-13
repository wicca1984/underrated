//! Core application layer managing the web browser session.
//!
//! Provides the `BrowserSession` struct which glues together browsing context,
//! input management, forms state, window sizing, and rendering.

use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;

/// Walks up from the given node to find the nearest `<a>` ancestor with a non-empty href attribute.
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

/// Walks up from the given node to find the nearest `<button>` or `<input>` of type `submit` or `button`.
fn find_submit_button(dom: &Dom, node: NodeId) -> Option<NodeId> {
    let mut current = node;
    loop {
        if let Some(NodeData::Element { name, .. }) = dom.data(current) {
            if name.eq_ignore_ascii_case("button") {
                return Some(current);
            }
            if name.eq_ignore_ascii_case("input")
                && let Some(input_type) = dom.get_attribute(current, "type")
                && (input_type.eq_ignore_ascii_case("submit")
                    || input_type.eq_ignore_ascii_case("button"))
            {
                return Some(current);
            }
        }
        if let Some(parent) = dom.parent(current) {
            current = parent;
        } else {
            break;
        }
    }
    None
}

/// A structured, testable browser session wrapping page, style, layout, rasterization,
/// and winit-compatible input dispatch glue.
pub struct BrowserSession {
    pub browsing_context: crate::engine::BrowsingContext,
    pub input_manager: crate::shell::ShellInputManager,
    pub form_state: crate::forms::FormState,
    pub base_url: crate::url::Url,
    pub width: u32,
    pub height: u32,
}

impl BrowserSession {
    /// Constructs a new `BrowserSession`.
    pub fn new(
        browsing_context: crate::engine::BrowsingContext,
        input_manager: crate::shell::ShellInputManager,
        form_state: crate::forms::FormState,
        base_url: crate::url::Url,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            browsing_context,
            input_manager,
            form_state,
            base_url,
            width,
            height,
        }
    }

    /// Renders the current browsing context layout to a raster Canvas.
    pub fn render(&self) -> crate::raster::Canvas {
        let caret = self
            .browsing_context
            .focus_node
            .map(|node| (node, self.browsing_context.caret_index));
        let display_list = crate::paint::build_display_list_with_caret(
            &self.browsing_context.page.layout,
            &self.browsing_context.page.dom,
            &self.browsing_context.page.styles,
            caret,
        );
        crate::raster::rasterize(&display_list, self.width, self.height)
    }

    /// Dispatches and processes an input event (clicks, text input, navigation),
    /// using the provided resource loader for dynamic document navigation.
    pub fn handle_input_event(
        &mut self,
        event: crate::shell::InputEvent,
        loader: &dyn crate::loader::ResourceLoader,
    ) {
        match event {
            crate::shell::InputEvent::Click { x, y } => {
                let clicked_node =
                    crate::layout::hit_test(&self.browsing_context.page.layout, x as f32, y as f32);
                self.input_manager.handle_click(x, y, clicked_node);
                let focused = self.input_manager.focused_element();
                self.browsing_context.set_focus(focused);
                if let Some(focused_node) = self.browsing_context.focus_node {
                    let caret_pos = self.input_manager.caret_position(focused_node);
                    self.browsing_context.caret_index = caret_pos;
                }

                if let Some(node) = clicked_node {
                    if let Some(href) = find_link_href(&self.browsing_context.page.dom, node) {
                        let req = crate::forms::NavigationRequest {
                            url: href,
                            method: crate::forms::Method::Get,
                            body: String::new(),
                            content_type: None,
                        };
                        let new_page = crate::engine::navigate(
                            &req,
                            &self.base_url,
                            loader,
                            self.width as f32,
                        );
                        self.base_url = new_page.url.clone();
                        self.form_state.set_current_url(&new_page.url.serialize());
                        self.browsing_context.navigate(new_page);
                        self.input_manager.blur();
                    } else if let Some(submit_btn) =
                        find_submit_button(&self.browsing_context.page.dom, node)
                        && let Some(new_page) = crate::engine::navigate_from_enter(
                            &self.browsing_context.page.dom,
                            submit_btn,
                            &self.form_state,
                            &self.base_url,
                            loader,
                            self.width as f32,
                        )
                    {
                        self.base_url = new_page.url.clone();
                        self.form_state.set_current_url(&new_page.url.serialize());
                        self.browsing_context.navigate(new_page);
                        self.input_manager.blur();
                    }
                }
            }
            crate::shell::InputEvent::Key { key } => {
                if let Some(focused) = self.input_manager.focused_element() {
                    if key == "Enter" || key == "Return" {
                        if let Some(new_page) = crate::engine::navigate_from_enter(
                            &self.browsing_context.page.dom,
                            focused,
                            &self.form_state,
                            &self.base_url,
                            loader,
                            self.width as f32,
                        ) {
                            self.base_url = new_page.url.clone();
                            self.form_state.set_current_url(&new_page.url.serialize());
                            self.browsing_context.navigate(new_page);
                            self.input_manager.blur();
                        }
                    } else {
                        self.input_manager.handle_key(&key);
                        let text = self.input_manager.text_buffer(focused).to_string();
                        let caret_pos = self.input_manager.caret_position(focused);
                        self.form_state.set_value(focused, &text);
                        self.browsing_context
                            .page
                            .dom
                            .set_attribute(focused, "value", &text);
                        self.browsing_context.caret_index = caret_pos;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{Dom, NodeData};
    use crate::engine::{BrowsingContext, render_page};
    use crate::forms::FormState;
    use crate::loader::{HttpMethod, LoadError, LoaderResponse, ResourceLoader};
    use crate::shell::{InputEvent, ShellInputManager};
    use crate::url::Url;
    use std::collections::HashMap;

    struct TestMockLoader {
        responses: HashMap<String, Vec<u8>>,
    }

    impl ResourceLoader for TestMockLoader {
        fn load(&self, _url: &Url) -> Result<Vec<u8>, LoadError> {
            Err(LoadError::NotFound)
        }

        fn load_request(
            &self,
            url: &Url,
            method: HttpMethod,
            body: &[u8],
            _content_type: Option<&str>,
        ) -> Result<LoaderResponse, LoadError> {
            let serialized_url = url.serialize();
            let key = match method {
                HttpMethod::Get => format!("GET:{}", serialized_url),
                HttpMethod::Post => {
                    let body_str = String::from_utf8_lossy(body);
                    format!("POST:{}:{}", serialized_url, body_str)
                }
            };
            if let Some(bytes) = self.responses.get(&key) {
                Ok(LoaderResponse {
                    bytes: bytes.clone(),
                    content_type: "text/html".to_string(),
                    charset: Some("utf-8".to_string()),
                })
            } else {
                Err(LoadError::NotFound)
            }
        }
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

    #[test]
    fn test_browser_session_input_handling() {
        let home_html = r#"
            <!DOCTYPE html>
            <html>
            <head>
            <style>
              body { margin: 0; padding: 0; }
              form { margin: 0; padding: 0; }
              input { display: block; width: 200px; height: 40px; }
            </style>
            </head>
            <body>
              <form action="/search" method="get">
                <input name="q" id="search-input">
                <input type="submit" value="Search">
              </form>
            </body>
            </html>
        "#;

        let base_url = Url::parse("https://example.com/").unwrap();
        let mut responses = HashMap::new();
        responses.insert(
            "GET:https://example.com/search?q=abc".to_string(),
            b"<html><body><h1>Results for abc</h1></body></html>".to_vec(),
        );
        let loader = TestMockLoader { responses };

        let page = render_page(home_html, &base_url, &loader, 800.0);
        let browsing_context = BrowsingContext::new(page);
        let input_manager = ShellInputManager::new();
        let form_state = FormState::new();

        let mut session = BrowserSession::new(
            browsing_context,
            input_manager,
            form_state,
            base_url,
            800,
            600,
        );

        // Find the input element "q"
        let doc = session.browsing_context.page.dom.document();
        let mut input_id = None;
        for node_id in session.browsing_context.page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, attrs }) =
                session.browsing_context.page.dom.data(node_id)
                && name.eq_ignore_ascii_case("input")
                && attrs.iter().any(|(k, v)| k == "name" && v == "q")
            {
                input_id = Some(node_id);
                break;
            }
        }
        let input_id = input_id.expect("Could not find input element");

        // Click on the input element to focus it.
        let input_rect =
            crate::layout::find_box_rect(&session.browsing_context.page.layout, input_id)
                .expect("Could not find input layout rect");
        let click_x = input_rect.origin.x + 10.0;
        let click_y = input_rect.origin.y + 10.0;

        // Dispatch a click event to focus the input
        session.handle_input_event(
            InputEvent::Click {
                x: click_x as f64,
                y: click_y as f64,
            },
            &loader,
        );

        // Verify focus and caret
        assert_eq!(session.browsing_context.focus_node, Some(input_id));
        assert_eq!(session.input_manager.focused_element(), Some(input_id));
        assert_eq!(session.browsing_context.caret_index, 0);

        // Type characters: "a", "b", "c"
        session.handle_input_event(
            InputEvent::Key {
                key: "a".to_string(),
            },
            &loader,
        );
        session.handle_input_event(
            InputEvent::Key {
                key: "b".to_string(),
            },
            &loader,
        );
        session.handle_input_event(
            InputEvent::Key {
                key: "c".to_string(),
            },
            &loader,
        );

        // Verify values
        assert_eq!(session.input_manager.text_buffer(input_id), "abc");
        assert_eq!(
            session
                .browsing_context
                .page
                .dom
                .get_attribute(input_id, "value"),
            Some("abc")
        );
        assert_eq!(session.browsing_context.caret_index, 3);

        // Trigger enter navigation
        session.handle_input_event(
            InputEvent::Key {
                key: "Enter".to_string(),
            },
            &loader,
        );

        // Verify navigation happened and the page loaded is the results page
        let doc_after = session.browsing_context.page.dom.document();
        let mut h1_found = false;
        for node_id in session.browsing_context.page.dom.descendants(doc_after) {
            if let Some(NodeData::Element { name, .. }) =
                session.browsing_context.page.dom.data(node_id)
                && name.eq_ignore_ascii_case("h1")
            {
                let text = session.browsing_context.page.dom.text_content(node_id);
                assert_eq!(text, "Results for abc");
                h1_found = true;
                break;
            }
        }
        assert!(h1_found, "h1 element not found on navigated results page");
    }

    #[test]
    fn test_find_submit_button() {
        let mut dom = Dom::new();
        // Create <button>
        let btn_node = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![],
        });
        // Create a <span> inside <button>
        let span_node = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(btn_node, span_node);

        // should find button from span
        assert_eq!(find_submit_button(&dom, span_node), Some(btn_node));
        // should find button from button itself
        assert_eq!(find_submit_button(&dom, btn_node), Some(btn_node));

        // Create <input type="submit">
        let input_submit = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![("type".into(), "submit".into())],
        });
        assert_eq!(find_submit_button(&dom, input_submit), Some(input_submit));

        // Create <input type="button">
        let input_button = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![("type".into(), "button".into())],
        });
        assert_eq!(find_submit_button(&dom, input_button), Some(input_button));

        // Create <input type="text"> (should be None)
        let input_text = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![("type".into(), "text".into())],
        });
        assert_eq!(find_submit_button(&dom, input_text), None);

        // Create <div_node> (should be None)
        let div_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        assert_eq!(find_submit_button(&dom, div_node), None);
    }

    #[test]
    fn test_browser_session_submit_button_click() {
        let home_html = r#"
            <!DOCTYPE html>
            <html>
            <head>
            <style>
              body { margin: 0; padding: 0; }
              form { margin: 0; padding: 0; }
              input { display: block; width: 200px; height: 40px; }
              button { display: block; width: 200px; height: 40px; }
            </style>
            </head>
            <body>
              <form action="/result" method="get">
                <input name="q" id="search-input">
                <button type="submit" id="btn-submit">Search</button>
              </form>
            </body>
            </html>
        "#;

        let base_url = Url::parse("https://example.com/").unwrap();
        let mut responses = HashMap::new();
        responses.insert(
            "GET:https://example.com/result?q=xyz".to_string(),
            b"<html><body><h1>Results for xyz</h1></body></html>".to_vec(),
        );
        let loader = TestMockLoader { responses };

        let page = render_page(home_html, &base_url, &loader, 800.0);
        let browsing_context = BrowsingContext::new(page);
        let input_manager = ShellInputManager::new();
        let form_state = FormState::new();

        let mut session = BrowserSession::new(
            browsing_context,
            input_manager,
            form_state,
            base_url,
            800,
            600,
        );

        // Find the input element "q"
        let doc = session.browsing_context.page.dom.document();
        let mut input_id = None;
        let mut button_id = None;
        for node_id in session.browsing_context.page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, attrs }) =
                session.browsing_context.page.dom.data(node_id)
            {
                if name.eq_ignore_ascii_case("input")
                    && attrs.iter().any(|(k, v)| k == "name" && v == "q")
                {
                    input_id = Some(node_id);
                } else if name.eq_ignore_ascii_case("button") {
                    button_id = Some(node_id);
                }
            }
        }
        let input_id = input_id.expect("Could not find input element");
        let button_id = button_id.expect("Could not find button element");

        // Click on the input element to focus it.
        let input_rect =
            crate::layout::find_box_rect(&session.browsing_context.page.layout, input_id)
                .expect("Could not find input layout rect");
        let click_x = input_rect.origin.x + 10.0;
        let click_y = input_rect.origin.y + 10.0;

        session.handle_input_event(
            InputEvent::Click {
                x: click_x as f64,
                y: click_y as f64,
            },
            &loader,
        );

        // Type characters: "x", "y", "z"
        session.handle_input_event(
            InputEvent::Key {
                key: "x".to_string(),
            },
            &loader,
        );
        session.handle_input_event(
            InputEvent::Key {
                key: "y".to_string(),
            },
            &loader,
        );
        session.handle_input_event(
            InputEvent::Key {
                key: "z".to_string(),
            },
            &loader,
        );

        // Find the button rect and click it
        let button_rect =
            crate::layout::find_box_rect(&session.browsing_context.page.layout, button_id)
                .expect("Could not find button layout rect");
        let btn_click_x = button_rect.origin.x + 10.0;
        let btn_click_y = button_rect.origin.y + 10.0;

        session.handle_input_event(
            InputEvent::Click {
                x: btn_click_x as f64,
                y: btn_click_y as f64,
            },
            &loader,
        );

        // Verify navigation happened to the result page
        let doc_after = session.browsing_context.page.dom.document();
        let mut h1_found = false;
        for node_id in session.browsing_context.page.dom.descendants(doc_after) {
            if let Some(NodeData::Element { name, .. }) =
                session.browsing_context.page.dom.data(node_id)
                && name.eq_ignore_ascii_case("h1")
            {
                let text = session.browsing_context.page.dom.text_content(node_id);
                assert_eq!(text, "Results for xyz");
                h1_found = true;
                break;
            }
        }
        assert!(h1_found, "h1 element not found on navigated results page");
    }

    #[test]
    fn test_browser_session_submit_input_click() {
        let home_html = r#"
            <!DOCTYPE html>
            <html>
            <head>
            <style>
              body { margin: 0; padding: 0; }
              form { margin: 0; padding: 0; }
              input { display: block; width: 200px; height: 40px; }
            </style>
            </head>
            <body>
              <form action="/result" method="get">
                <input name="q" id="search-input">
                <input type="submit" id="input-submit" value="Search">
              </form>
            </body>
            </html>
        "#;

        let base_url = Url::parse("https://example.com/").unwrap();
        let mut responses = HashMap::new();
        responses.insert(
            "GET:https://example.com/result?q=123".to_string(),
            b"<html><body><h1>Results for 123</h1></body></html>".to_vec(),
        );
        let loader = TestMockLoader { responses };

        let page = render_page(home_html, &base_url, &loader, 800.0);
        let browsing_context = BrowsingContext::new(page);
        let input_manager = ShellInputManager::new();
        let form_state = FormState::new();

        let mut session = BrowserSession::new(
            browsing_context,
            input_manager,
            form_state,
            base_url,
            800,
            600,
        );

        // Find input "q" and input "submit"
        let doc = session.browsing_context.page.dom.document();
        let mut input_id = None;
        let mut submit_id = None;
        for node_id in session.browsing_context.page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, attrs }) =
                session.browsing_context.page.dom.data(node_id)
                && name.eq_ignore_ascii_case("input")
            {
                if attrs.iter().any(|(k, v)| k == "name" && v == "q") {
                    input_id = Some(node_id);
                } else if attrs.iter().any(|(k, v)| k == "type" && v == "submit") {
                    submit_id = Some(node_id);
                }
            }
        }
        let input_id = input_id.expect("Could not find input element");
        let submit_id = submit_id.expect("Could not find submit element");

        // Focus input
        let input_rect =
            crate::layout::find_box_rect(&session.browsing_context.page.layout, input_id)
                .expect("Could not find input layout rect");
        let click_x = input_rect.origin.x + 10.0;
        let click_y = input_rect.origin.y + 10.0;

        session.handle_input_event(
            InputEvent::Click {
                x: click_x as f64,
                y: click_y as f64,
            },
            &loader,
        );

        // Type characters: "1", "2", "3"
        session.handle_input_event(
            InputEvent::Key {
                key: "1".to_string(),
            },
            &loader,
        );
        session.handle_input_event(
            InputEvent::Key {
                key: "2".to_string(),
            },
            &loader,
        );
        session.handle_input_event(
            InputEvent::Key {
                key: "3".to_string(),
            },
            &loader,
        );

        // Click submit
        let submit_rect =
            crate::layout::find_box_rect(&session.browsing_context.page.layout, submit_id)
                .expect("Could not find submit layout rect");
        let sub_click_x = submit_rect.origin.x + 10.0;
        let sub_click_y = submit_rect.origin.y + 10.0;

        session.handle_input_event(
            InputEvent::Click {
                x: sub_click_x as f64,
                y: sub_click_y as f64,
            },
            &loader,
        );

        // Verify results page is navigated
        let doc_after = session.browsing_context.page.dom.document();
        let mut h1_found = false;
        for node_id in session.browsing_context.page.dom.descendants(doc_after) {
            if let Some(NodeData::Element { name, .. }) =
                session.browsing_context.page.dom.data(node_id)
                && name.eq_ignore_ascii_case("h1")
            {
                let text = session.browsing_context.page.dom.text_content(node_id);
                assert_eq!(text, "Results for 123");
                h1_found = true;
                break;
            }
        }
        assert!(h1_found, "h1 element not found on navigated results page");
    }

    #[test]
    fn test_browser_session_button_input_click() {
        let home_html = r#"
            <!DOCTYPE html>
            <html>
            <head>
            <style>
              body { margin: 0; padding: 0; }
              form { margin: 0; padding: 0; }
              input { display: block; width: 200px; height: 40px; }
            </style>
            </head>
            <body>
              <form action="/result" method="get">
                <input name="q" id="search-input">
                <input type="button" id="input-btn" value="Search">
              </form>
            </body>
            </html>
        "#;

        let base_url = Url::parse("https://example.com/").unwrap();
        let mut responses = HashMap::new();
        responses.insert(
            "GET:https://example.com/result?q=456".to_string(),
            b"<html><body><h1>Results for 456</h1></body></html>".to_vec(),
        );
        let loader = TestMockLoader { responses };

        let page = render_page(home_html, &base_url, &loader, 800.0);
        let browsing_context = BrowsingContext::new(page);
        let input_manager = ShellInputManager::new();
        let form_state = FormState::new();

        let mut session = BrowserSession::new(
            browsing_context,
            input_manager,
            form_state,
            base_url,
            800,
            600,
        );

        // Find input "q" and input "button"
        let doc = session.browsing_context.page.dom.document();
        let mut input_id = None;
        let mut submit_id = None;
        for node_id in session.browsing_context.page.dom.descendants(doc) {
            if let Some(NodeData::Element { name, attrs }) =
                session.browsing_context.page.dom.data(node_id)
                && name.eq_ignore_ascii_case("input")
            {
                if attrs.iter().any(|(k, v)| k == "name" && v == "q") {
                    input_id = Some(node_id);
                } else if attrs.iter().any(|(k, v)| k == "type" && v == "button") {
                    submit_id = Some(node_id);
                }
            }
        }
        let input_id = input_id.expect("Could not find input element");
        let submit_id = submit_id.expect("Could not find button input element");

        // Focus input
        let input_rect =
            crate::layout::find_box_rect(&session.browsing_context.page.layout, input_id)
                .expect("Could not find input layout rect");
        let click_x = input_rect.origin.x + 10.0;
        let click_y = input_rect.origin.y + 10.0;

        session.handle_input_event(
            InputEvent::Click {
                x: click_x as f64,
                y: click_y as f64,
            },
            &loader,
        );

        // Type characters: "4", "5", "6"
        session.handle_input_event(
            InputEvent::Key {
                key: "4".to_string(),
            },
            &loader,
        );
        session.handle_input_event(
            InputEvent::Key {
                key: "5".to_string(),
            },
            &loader,
        );
        session.handle_input_event(
            InputEvent::Key {
                key: "6".to_string(),
            },
            &loader,
        );

        // Click submit input
        let submit_rect =
            crate::layout::find_box_rect(&session.browsing_context.page.layout, submit_id)
                .expect("Could not find button input layout rect");
        let sub_click_x = submit_rect.origin.x + 10.0;
        let sub_click_y = submit_rect.origin.y + 10.0;

        session.handle_input_event(
            InputEvent::Click {
                x: sub_click_x as f64,
                y: sub_click_y as f64,
            },
            &loader,
        );

        // Verify results page is navigated
        let doc_after = session.browsing_context.page.dom.document();
        let mut h1_found = false;
        for node_id in session.browsing_context.page.dom.descendants(doc_after) {
            if let Some(NodeData::Element { name, .. }) =
                session.browsing_context.page.dom.data(node_id)
                && name.eq_ignore_ascii_case("h1")
            {
                let text = session.browsing_context.page.dom.text_content(node_id);
                assert_eq!(text, "Results for 456");
                h1_found = true;
                break;
            }
        }
        assert!(h1_found, "h1 element not found on navigated results page");
    }
}
