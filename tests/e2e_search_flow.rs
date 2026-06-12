//! Headless, deterministic end-to-end (E2E) integration test for the search interaction flow.
//! This verifies the MVP interactive search journey:
//! 1. Render initial home page containing a form with an input.
//! 2. Find the input element's bounding rect and perform a hit-test (clicking) to focus it.
//! 3. Use `ShellInputManager` to set the text buffer of the input element.
//! 4. Simulate pressing Enter via `navigate_from_enter` with a custom ResourceLoader.
//! 5. Assert the resulting `Page` properties: navigate produced `Some(Page)`, structural text matches, etc.

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

#[test]
fn test_e2e_search_interaction_flow() {
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

    let base_url = Url::parse("https://example.com/").expect("Failed to parse base URL");
    let mut responses = HashMap::new();

    // Prepare canned result page HTML
    let expected_query_url = "GET:https://example.com/search?q=rust+lang";
    let canned_result_html = b"<html><body><h1>Search results: rust lang</h1></body></html>";
    responses.insert(expected_query_url.to_string(), canned_result_html.to_vec());

    let mock_loader = E2eMockLoader { responses };
    let viewport_width = 800.0;

    // Render the initial home page containing the form.
    let initial_page = render_page(home_html, &base_url, &mock_loader, viewport_width);

    // Locate the search input element in the DOM.
    let input_id = find_input_by_name(&initial_page.dom, "q")
        .expect("Could not find input element with name='q'");

    // 2. Find input box rect and hit-test to focus.
    let input_rect = find_box_rect(&initial_page.layout, input_id)
        .expect("Could not find layout box for the input element");

    // Let's assert that the box has non-zero size.
    assert!(input_rect.size.width > 0.0);
    assert!(input_rect.size.height > 0.0);

    // Pick a point inside the input's bounding box (specifically, the center).
    let click_x = input_rect.origin.x + input_rect.size.width / 2.0;
    let click_y = input_rect.origin.y + input_rect.size.height / 2.0;

    // Use hit_test to locate the NodeId from the layout tree at this point.
    let hit_node = hit_test(&initial_page.layout, click_x, click_y)
        .expect("Hit test did not find any node inside input rect");
    assert_eq!(
        hit_node, input_id,
        "Hit node should match the input node ID"
    );

    // 3. Drive ShellInputManager to focus and type a query.
    let mut input_manager = ShellInputManager::new();
    input_manager.handle_click(click_x as f64, click_y as f64, Some(hit_node));

    // Verify focus was successfully acquired.
    assert_eq!(
        input_manager.focused_element(),
        Some(input_id),
        "Input should be focused"
    );

    // Type the query: "rust lang" (we can use set_text_buffer to simulate typing,
    // which aligns with S-34/S-81 specifications and what human typed/toggled inputs model).
    input_manager.set_text_buffer(input_id, "rust lang".to_string());
    assert_eq!(input_manager.text_buffer(input_id), "rust lang");

    // 4. Set FormState values from input_manager.
    let mut form_state = FormState::new();
    form_state.set_value(input_id, input_manager.text_buffer(input_id));

    // 5. Call navigate_from_enter to simulate hitting Enter on the focused input.
    let result_page = navigate_from_enter(
        &initial_page.dom,
        input_id,
        &form_state,
        &base_url,
        &mock_loader,
        viewport_width,
    )
    .expect("navigate_from_enter should yield a navigated Page");

    // 6. Assert on the resulting navigated page DOM.
    let h1_id = find_element_by_tag(&result_page.dom, "h1")
        .expect("Result page should contain an <h1> element");
    let h1_text = result_page.dom.text_content(h1_id);
    assert_eq!(h1_text, "Search results: rust lang");
}
