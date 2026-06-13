//! E2E Winit Session example.
//!
//! Simulates driving a typed-search-and-submit session through the same shared
//! `BrowserSession` glue that the production live app uses, converting winit-like
//! events to high-level input intents.

use std::collections::HashMap;
use underrated::app::BrowserSession;
use underrated::dom::{Dom, NodeData};
use underrated::engine::{BrowsingContext, render_page};
use underrated::forms::FormState;
use underrated::infra::NodeId;
use underrated::layout::find_box_rect;
use underrated::loader::{HttpMethod, LoadError, LoaderResponse, ResourceLoader};
use underrated::shell::{InputEvent, ShellInputManager, map_window_event};
use underrated::url::Url;

/// A custom mock loader that responds with our canned search result page.
struct E2eMockLoader {
    responses: HashMap<String, Vec<u8>>,
}

impl ResourceLoader for E2eMockLoader {
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

/// Helper function to locate an input element by its "name" attribute value.
fn find_input_by_name(dom: &Dom, name_attr: &str) -> Option<NodeId> {
    let doc = dom.document();
    for node_id in dom.descendants(doc) {
        if let Some(NodeData::Element { name, attrs }) = dom.data(node_id)
            && name.eq_ignore_ascii_case("input")
            && attrs
                .iter()
                .any(|(k, v)| k.eq_ignore_ascii_case("name") && v == name_attr)
        {
            return Some(node_id);
        }
    }
    None
}

/// Helper to write PNG bytes to the specified path, creating directories as needed.
fn save_png(out_path: &str, png_bytes: &[u8]) {
    if let Some(parent) = std::path::Path::new(out_path).parent()
        && !parent.exists()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        eprintln!(
            "Error: failed to create directory {}: {}",
            parent.display(),
            e
        );
        std::process::exit(1);
    }

    if let Err(e) = std::fs::write(out_path, png_bytes) {
        eprintln!("Error writing PNG file '{out_path}': {e}");
        std::process::exit(1);
    }
}

fn main() {
    // 1. Home page HTML with a search form. We use custom CSS to ensure a deterministic non-zero input rect.
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

    let base_url = match Url::parse("https://example.com/") {
        Ok(url) => url,
        Err(e) => {
            eprintln!("Error: failed to parse base URL: {e:?}");
            std::process::exit(1);
        }
    };

    let mut responses = HashMap::new();

    // Prepare canned result page HTML
    let expected_query_url = "GET:https://example.com/search?q=rust";
    let canned_result_html = b"<html><body><h1>Search results: rust</h1></body></html>";
    responses.insert(expected_query_url.to_string(), canned_result_html.to_vec());

    let mock_loader = E2eMockLoader { responses };
    let viewport_width = 800;
    let viewport_height = 600;

    // Render the initial home page containing the form.
    let initial_page = render_page(home_html, &base_url, &mock_loader, viewport_width as f32);
    let browsing_context = BrowsingContext::new(initial_page);
    let input_manager = ShellInputManager::new();
    let form_state = FormState::new();

    // Construct the BrowserSession over the search page.
    let mut session = BrowserSession::new(
        browsing_context,
        input_manager,
        form_state,
        base_url,
        viewport_width,
        viewport_height,
    );

    // Locate the search input element in the DOM.
    let Some(input_id) = find_input_by_name(&session.browsing_context.page.dom, "q") else {
        eprintln!("Error: Could not find input element with name='q'");
        std::process::exit(1);
    };

    // Find input box rect and hit-test to focus.
    let Some(input_rect) = find_box_rect(&session.browsing_context.page.layout, input_id) else {
        eprintln!("Error: Could not find layout box for the input element");
        std::process::exit(1);
    };

    if input_rect.size.width <= 0.0 || input_rect.size.height <= 0.0 {
        eprintln!("Error: Input element has non-positive size");
        std::process::exit(1);
    }

    // Pick a point inside the input's bounding box (center).
    let click_x = input_rect.origin.x + input_rect.size.width / 2.0;
    let click_y = input_rect.origin.y + input_rect.size.height / 2.0;

    // CLICK to focus the input: construct a real winit MouseInput event.
    // Pass it through map_window_event to obtain the Click InputEvent.
    let device_id = winit::event::DeviceId::dummy();
    let mouse_event = winit::event::WindowEvent::MouseInput {
        device_id,
        state: winit::event::ElementState::Pressed,
        button: winit::event::MouseButton::Left,
    };
    let cursor_pos = (click_x as f64, click_y as f64);
    let click_input_event = match map_window_event(&mouse_event, cursor_pos) {
        Some(evt) => evt,
        None => {
            eprintln!("Error: map_window_event returned None for mouse input");
            std::process::exit(1);
        }
    };

    // Feed the click through our session's event handling glue.
    session.handle_input_event(click_input_event, &mock_loader);

    // Assert that the input successfully acquired focus.
    if session.browsing_context.focus_node != Some(input_id) {
        eprintln!("Error: BrowserSession's browsing_context focus_node was not set to input_id");
        std::process::exit(1);
    }
    if session.input_manager.focused_element() != Some(input_id) {
        eprintln!("Error: BrowserSession's input_manager focused_element was not set to input_id");
        std::process::exit(1);
    }

    // TYPE "rust" character-by-character.
    // NOTE: winit KeyEvent CANNOT be constructed outside winit — its platform_specific field
    // is private — so for keystrokes we must build underrated::shell::InputEvent::Key values
    // directly. This is the exact type map_window_event yields for keys, so the shared
    // event routing glue path is still fully exercised.
    let keys_to_type = vec!["r", "u", "s", "t"];
    for key in keys_to_type {
        let key_evt = InputEvent::Key {
            key: key.to_string(),
        };
        session.handle_input_event(key_evt, &mock_loader);
    }

    // After typing, render and save state to var/e2e-winit-typed.png.
    // This PNG must visibly show the typed text inside the search box.
    let typed_canvas = session.render();
    let typed_png_bytes = underrated::image::encode_png(&typed_canvas);
    save_png("var/e2e-winit-typed.png", &typed_png_bytes);

    // Verify input value attribute equals "rust" after typing.
    let Some(typed_value) = session
        .browsing_context
        .page
        .dom
        .get_attribute(input_id, "value")
    else {
        eprintln!("Error: 'value' attribute not found on input element after typing");
        std::process::exit(1);
    };
    if typed_value != "rust" {
        eprintln!(
            "Error: expected typed value to be 'rust', but got '{}'",
            typed_value
        );
        std::process::exit(1);
    }

    // SUBMIT the form by sending a Return/Enter key.
    let submit_key_event = InputEvent::Key {
        key: "Enter".to_string(),
    };
    session.handle_input_event(submit_key_event, &mock_loader);

    // After Enter, render and save state to var/e2e-winit-results.png.
    // This PNG must show the navigated results page.
    let results_canvas = session.render();
    let results_png_bytes = underrated::image::encode_png(&results_canvas);
    save_png("var/e2e-winit-results.png", &results_png_bytes);

    // Verify the page changed to the expected results page DOM.
    let results_doc = session.browsing_context.page.dom.document();
    let mut h1_text = None;
    for node_id in session.browsing_context.page.dom.descendants(results_doc) {
        if let Some(NodeData::Element { name, .. }) =
            session.browsing_context.page.dom.data(node_id)
            && name.eq_ignore_ascii_case("h1")
        {
            h1_text = Some(session.browsing_context.page.dom.text_content(node_id));
            break;
        }
    }

    let Some(h1_text) = h1_text else {
        eprintln!("Error: Result page does not contain an h1 element");
        std::process::exit(1);
    };
    if h1_text != "Search results: rust" {
        eprintln!(
            "Error: h1 text was '{}', expected 'Search results: rust'",
            h1_text
        );
        std::process::exit(1);
    }

    println!("Success: drove E2E typed search session through BrowserSession!");
    println!("Saved: var/e2e-winit-typed.png");
    println!("Saved: var/e2e-winit-results.png");
}
