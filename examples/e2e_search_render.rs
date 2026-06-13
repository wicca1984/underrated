//! E2E Search and Render example.
//!
//! Simulates focusing an input, typing a query, pressing Enter,
//! navigating to the results page, and rendering both pages to PNG.

use std::collections::HashMap;
use underrated::dom::{Dom, NodeData};
use underrated::engine::{navigate_from_enter, render_page};
use underrated::forms::FormState;
use underrated::infra::NodeId;
use underrated::layout::{find_box_rect, hit_test};
use underrated::loader::{HttpMethod, LoadError, LoaderResponse, ResourceLoader};
use underrated::shell::ShellInputManager;
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

/// Helper function to locate a node by its HTML element tag name.
fn find_element_by_tag(dom: &Dom, tag_name: &str) -> Option<NodeId> {
    let doc = dom.document();
    for node_id in dom.descendants(doc) {
        if let Some(NodeData::Element { name, .. }) = dom.data(node_id)
            && name.eq_ignore_ascii_case(tag_name)
        {
            return Some(node_id);
        }
    }
    None
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
    let expected_query_url = "GET:https://example.com/search?q=rust+lang";
    let canned_result_html = b"<html><body><h1>Search results: rust lang</h1></body></html>";
    responses.insert(expected_query_url.to_string(), canned_result_html.to_vec());

    let mock_loader = E2eMockLoader { responses };
    let viewport_width = 800.0;

    // Render the initial home page containing the form.
    let initial_page = render_page(home_html, &base_url, &mock_loader, viewport_width);

    // Rasterize the initial page and save to var/e2e-search-home.png
    let initial_display_list = underrated::paint::build_display_list(
        &initial_page.layout,
        &initial_page.dom,
        &initial_page.styles,
    );
    let initial_canvas = underrated::raster::rasterize(&initial_display_list, 800, 600);
    let initial_png_bytes = underrated::image::encode_png(&initial_canvas);
    save_png("var/e2e-search-home.png", &initial_png_bytes);

    // Locate the search input element in the DOM.
    let Some(input_id) = find_input_by_name(&initial_page.dom, "q") else {
        eprintln!("Error: Could not find input element with name='q'");
        std::process::exit(1);
    };

    // Find input box rect and hit-test to focus.
    let Some(input_rect) = find_box_rect(&initial_page.layout, input_id) else {
        eprintln!("Error: Could not find layout box for the input element");
        std::process::exit(1);
    };

    if input_rect.size.width <= 0.0 || input_rect.size.height <= 0.0 {
        eprintln!("Error: Input element has non-positive size");
        std::process::exit(1);
    }

    // Pick a point inside the input's bounding box (specifically, the center).
    let click_x = input_rect.origin.x + input_rect.size.width / 2.0;
    let click_y = input_rect.origin.y + input_rect.size.height / 2.0;

    // Use hit_test to locate the NodeId from the layout tree at this point.
    let Some(hit_node) = hit_test(&initial_page.layout, click_x, click_y) else {
        eprintln!("Error: Hit test did not find any node inside input rect");
        std::process::exit(1);
    };

    if hit_node != input_id {
        eprintln!("Error: Hit node ID does not match the input element ID");
        std::process::exit(1);
    }

    // Drive ShellInputManager to focus and type a query.
    let mut input_manager = ShellInputManager::new();
    input_manager.handle_click(click_x as f64, click_y as f64, Some(hit_node));

    // Verify focus was successfully acquired.
    if input_manager.focused_element() != Some(input_id) {
        eprintln!("Error: Input element was not successfully focused");
        std::process::exit(1);
    }

    // Type the query: "rust lang"
    input_manager.set_text_buffer(input_id, "rust lang".to_string());
    if input_manager.text_buffer(input_id) != "rust lang" {
        eprintln!("Error: Input text buffer was not updated correctly");
        std::process::exit(1);
    }

    // Set FormState values from input_manager.
    let mut form_state = FormState::new();
    form_state.set_value(input_id, input_manager.text_buffer(input_id));

    // Call navigate_from_enter to simulate hitting Enter on the focused input.
    let Some(result_page) = navigate_from_enter(
        &initial_page.dom,
        input_id,
        &form_state,
        &base_url,
        &mock_loader,
        viewport_width,
    ) else {
        eprintln!("Error: navigate_from_enter did not return a Page");
        std::process::exit(1);
    };

    // Rasterize the result page and save to var/e2e-search-result.png
    let result_display_list = underrated::paint::build_display_list(
        &result_page.layout,
        &result_page.dom,
        &result_page.styles,
    );
    let result_canvas = underrated::raster::rasterize(&result_display_list, 800, 600);
    let result_png_bytes = underrated::image::encode_png(&result_canvas);
    save_png("var/e2e-search-result.png", &result_png_bytes);

    // Verify on the resulting navigated page DOM.
    let Some(h1_id) = find_element_by_tag(&result_page.dom, "h1") else {
        eprintln!("Error: Result page does not contain an h1 element");
        std::process::exit(1);
    };
    let h1_text = result_page.dom.text_content(h1_id);
    if h1_text != "Search results: rust lang" {
        eprintln!(
            "Error: h1 text was '{}', expected 'Search results: rust lang'",
            h1_text
        );
        std::process::exit(1);
    }

    println!("wrote home page rendering to: var/e2e-search-home.png");
    println!("wrote result page rendering to: var/e2e-search-result.png");
    println!("confirmation: h1 text content equals \"Search results: rust lang\"");
}
