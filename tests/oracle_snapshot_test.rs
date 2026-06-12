use serde_json::Value;
use std::path::Path;

fn assert_structural_invariants(node: &Value) {
    let node_type = node["type"]
        .as_str()
        .unwrap_or_else(|| panic!("Node must have a 'type' field"));
    match node_type {
        "element" => {
            let tag = node["tag"]
                .as_str()
                .unwrap_or_else(|| panic!("Element node must have a 'tag' string field"));
            assert_eq!(
                tag,
                tag.to_lowercase(),
                "Tag name must be lowercase in element: {}",
                tag
            );

            assert!(
                node["attrs"].is_object(),
                "Element node '{}' must have an 'attrs' object field",
                tag
            );

            let rect = &node["rect"];
            assert!(
                rect.is_object(),
                "Element node '{}' must have a 'rect' object field",
                tag
            );
            let _x = rect["x"]
                .as_f64()
                .unwrap_or_else(|| panic!("rect.x must be a number"));
            let _y = rect["y"]
                .as_f64()
                .unwrap_or_else(|| panic!("rect.y must be a number"));
            let width = rect["width"]
                .as_f64()
                .unwrap_or_else(|| panic!("rect.width must be a number"));
            let height = rect["height"]
                .as_f64()
                .unwrap_or_else(|| panic!("rect.height must be a number"));

            assert!(width >= 0.0, "rect.width of '{}' must be non-negative", tag);
            assert!(
                height >= 0.0,
                "rect.height of '{}' must be non-negative",
                tag
            );

            let children = node["children"]
                .as_array()
                .unwrap_or_else(|| panic!("Element node must have a 'children' array"));
            for child in children {
                assert_structural_invariants(child);
            }
        }
        "text" => {
            let text = node["text"]
                .as_str()
                .unwrap_or_else(|| panic!("Text node must have a 'text' string field"));
            assert!(
                !text.trim().is_empty(),
                "Text node must not contain only whitespace"
            );
        }
        other => panic!("Unknown node type: {}", other),
    }
}

fn find_element_by_tag<'a>(node: &'a Value, tag_name: &str) -> Option<&'a Value> {
    if node["type"] == "element" && node["tag"] == tag_name {
        return Some(node);
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            if let Some(found) = find_element_by_tag(child, tag_name) {
                return Some(found);
            }
        }
    }
    None
}

fn find_elements_by_tag<'a>(node: &'a Value, tag_name: &str, results: &mut Vec<&'a Value>) {
    if node["type"] == "element" && node["tag"] == tag_name {
        results.push(node);
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            find_elements_by_tag(child, tag_name, results);
        }
    }
}

fn find_element_by_class<'a>(node: &'a Value, class_name: &str) -> Option<&'a Value> {
    if node["type"] == "element" {
        let has_class = node["attrs"]
            .as_object()
            .and_then(|attrs| attrs.get("class"))
            .and_then(|v| v.as_str())
            .map(|class| class.split_whitespace().any(|c| c == class_name))
            .unwrap_or(false);
        if has_class {
            return Some(node);
        }
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            if let Some(found) = find_element_by_class(child, class_name) {
                return Some(found);
            }
        }
    }
    None
}

fn find_elements_by_class<'a>(node: &'a Value, class_name: &str, results: &mut Vec<&'a Value>) {
    if node["type"] == "element" {
        let has_class = node["attrs"]
            .as_object()
            .and_then(|attrs| attrs.get("class"))
            .and_then(|v| v.as_str())
            .map(|class| class.split_whitespace().any(|c| c == class_name))
            .unwrap_or(false);
        if has_class {
            results.push(node);
        }
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            find_elements_by_class(child, class_name, results);
        }
    }
}

fn find_element_by_attr<'a>(
    node: &'a Value,
    attr_name: &str,
    attr_value: &str,
) -> Option<&'a Value> {
    if node["type"] == "element" {
        let has_attr = node["attrs"]
            .as_object()
            .and_then(|attrs| attrs.get(attr_name))
            .and_then(|v| v.as_str())
            .map(|val| val == attr_value)
            .unwrap_or(false);
        if has_attr {
            return Some(node);
        }
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            if let Some(found) = find_element_by_attr(child, attr_name, attr_value) {
                return Some(found);
            }
        }
    }
    None
}

fn assert_centered(node: &Value, viewport_width: f64, tolerance_px: f64) {
    let tag = node["tag"]
        .as_str()
        .unwrap_or_else(|| panic!("Node must have a 'tag' string field"));
    let rect = &node["rect"];
    assert!(
        rect.is_object(),
        "Element node '{}' must have a 'rect' object field",
        tag
    );
    let x = rect["x"]
        .as_f64()
        .unwrap_or_else(|| panic!("rect.x of '{}' must be a number", tag));
    let w = rect["width"]
        .as_f64()
        .unwrap_or_else(|| panic!("rect.width of '{}' must be a number", tag));

    let center = x + w / 2.0;
    let expected_center = viewport_width / 2.0;
    assert!(
        (center - expected_center).abs() <= tolerance_px,
        "Element '{}' is not centered: computed center is {}, expected {}, tolerance is {}",
        tag,
        center,
        expected_center,
        tolerance_px
    );
}

fn assert_max_width(node: &Value, viewport_width: f64, ratio: f64) {
    let tag = node["tag"]
        .as_str()
        .unwrap_or_else(|| panic!("Node must have a 'tag' string field"));
    let rect = &node["rect"];
    assert!(
        rect.is_object(),
        "Element node '{}' must have a 'rect' object field",
        tag
    );
    let w = rect["width"]
        .as_f64()
        .unwrap_or_else(|| panic!("rect.width of '{}' must be a number", tag));

    let limit = viewport_width * ratio;
    assert!(
        w <= limit,
        "Element '{}' width ({}) exceeds maximum width limit ({}) for ratio {}",
        tag,
        w,
        limit,
        ratio
    );
}

fn collect_text_nodes<'a>(node: &'a Value, results: &mut Vec<&'a str>) {
    if node["type"] == "text" {
        let text = node["text"]
            .as_str()
            .unwrap_or_else(|| panic!("Text node must have a 'text' string field"));
        results.push(text);
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            collect_text_nodes(child, results);
        }
    }
}

fn load_fixture_snapshot(filename: &str) -> Value {
    let path = Path::new("tests/oracle/fixtures").join(filename);
    let html = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path.display(), e));

    let snapshot = underrated::oracle::export_snapshot(&html, "", 800, 600);
    assert_structural_invariants(&snapshot);
    snapshot
}

#[test]
fn test_fixture_01_single_block_text() {
    let snapshot = load_fixture_snapshot("01_single_block_text.html");
    // Root should be html
    assert_eq!(snapshot["tag"], "html");

    // Check that the div element exists
    let div = find_element_by_tag(&snapshot, "div")
        .unwrap_or_else(|| panic!("div element must exist in 01_single_block_text"));

    // Check text inside div is Hello World
    let mut text_nodes = Vec::new();
    collect_text_nodes(div, &mut text_nodes);
    assert_eq!(text_nodes, vec!["Hello World"]);

    // Width should be approx 400.0, height approx 100.0
    let width = div["rect"]["width"]
        .as_f64()
        .unwrap_or_else(|| panic!("rect.width must be a number"));
    let height = div["rect"]["height"]
        .as_f64()
        .unwrap_or_else(|| panic!("rect.height must be a number"));
    assert!(
        (width - 400.0).abs() < 1.0,
        "div width {} not close to 400",
        width
    );
    assert!(
        (height - 100.0).abs() < 1.0,
        "div height {} not close to 100",
        height
    );
}

#[test]
fn test_fixture_02_nested_blocks_inline_css() {
    let snapshot = load_fixture_snapshot("02_nested_blocks_inline_css.html");
    assert_eq!(snapshot["tag"], "html");

    // Find outer and inner div by finding all divs
    let mut divs = Vec::new();
    find_elements_by_tag(&snapshot, "div", &mut divs);
    assert_eq!(divs.len(), 2, "Should have exactly 2 div elements");

    let outer_div = divs[0];
    let inner_div = divs[1];

    // Check attributes
    let outer_style = outer_div["attrs"]["style"]
        .as_str()
        .unwrap_or_else(|| panic!("outer div style must be a string"));
    let inner_style = inner_div["attrs"]["style"]
        .as_str()
        .unwrap_or_else(|| panic!("inner div style must be a string"));
    assert!(outer_style.contains("width: 500px"));
    assert!(inner_style.contains("width: 250px"));

    // Check rects
    let outer_width = outer_div["rect"]["width"]
        .as_f64()
        .unwrap_or_else(|| panic!("outer div width must be a number"));
    let inner_width = inner_div["rect"]["width"]
        .as_f64()
        .unwrap_or_else(|| panic!("inner div width must be a number"));
    let inner_height = inner_div["rect"]["height"]
        .as_f64()
        .unwrap_or_else(|| panic!("inner div height must be a number"));

    assert!((outer_width - 500.0).abs() < 1.0);
    assert!((inner_width - 250.0).abs() < 1.0);
    assert!((inner_height - 150.0).abs() < 1.0);

    // Inner div should be nested within the hierarchy
    // Verify inner_div is indeed in outer_div's children
    let outer_children = outer_div["children"]
        .as_array()
        .unwrap_or_else(|| panic!("outer div children must be an array"));
    let found_inner = outer_children
        .iter()
        .any(|c| c["tag"] == "div" && c["rect"]["width"] == inner_div["rect"]["width"]);
    assert!(found_inner, "inner_div must be nested under outer_div");
}

#[test]
fn test_fixture_03_paragraph_text_wrap() {
    let snapshot = load_fixture_snapshot("03_paragraph_text_wrap.html");
    assert_eq!(snapshot["tag"], "html");

    let p = find_element_by_tag(&snapshot, "p").unwrap_or_else(|| panic!("p element must exist"));
    let p_width = p["rect"]["width"]
        .as_f64()
        .unwrap_or_else(|| panic!("p width must be a number"));
    assert!(
        (p_width - 100.0).abs() < 1.0,
        "Paragraph width should be 100.0, got {}",
        p_width
    );

    // Confirm text content is carried through
    let mut text_nodes = Vec::new();
    collect_text_nodes(p, &mut text_nodes);
    assert_eq!(text_nodes.len(), 1);
    assert!(text_nodes[0].contains("This is a short paragraph designed to wrap"));
}

#[test]
fn test_fixture_04_list_items() {
    let snapshot = load_fixture_snapshot("04_list_items.html");
    assert_eq!(snapshot["tag"], "html");

    let ul =
        find_element_by_tag(&snapshot, "ul").unwrap_or_else(|| panic!("ul element must exist"));
    let mut lis = Vec::new();
    find_elements_by_tag(ul, "li", &mut lis);
    assert_eq!(
        lis.len(),
        3,
        "ul element must contain exactly 3 li elements"
    );

    let mut text_nodes = Vec::new();
    collect_text_nodes(ul, &mut text_nodes);
    assert_eq!(text_nodes, vec!["First Item", "Second Item", "Third Item"]);
}

#[test]
fn test_fixture_05_display_none() {
    let snapshot = load_fixture_snapshot("05_display_none.html");
    assert_eq!(snapshot["tag"], "html");

    // Element with class .hidden has display: none
    let hidden = find_element_by_tag(&snapshot, "div")
        .and_then(|_| {
            let mut divs = Vec::new();
            find_elements_by_tag(&snapshot, "div", &mut divs);
            divs.into_iter().find(|d| d["attrs"]["class"] == "hidden")
        })
        .unwrap_or_else(|| panic!("hidden div element should exist in DOM"));

    let hidden_width = hidden["rect"]["width"]
        .as_f64()
        .unwrap_or_else(|| panic!("hidden width must be a number"));
    let hidden_height = hidden["rect"]["height"]
        .as_f64()
        .unwrap_or_else(|| panic!("hidden height must be a number"));
    assert_eq!(hidden_width, 0.0);
    assert_eq!(hidden_height, 0.0);

    let visible = find_element_by_tag(&snapshot, "div")
        .and_then(|_| {
            let mut divs = Vec::new();
            find_elements_by_tag(&snapshot, "div", &mut divs);
            divs.into_iter().find(|d| d["attrs"]["class"] == "visible")
        })
        .unwrap_or_else(|| panic!("visible div element should exist in DOM"));

    let visible_width = visible["rect"]["width"]
        .as_f64()
        .unwrap_or_else(|| panic!("visible width must be a number"));
    let visible_height = visible["rect"]["height"]
        .as_f64()
        .unwrap_or_else(|| panic!("visible height must be a number"));
    assert!((visible_width - 200.0).abs() < 1.0);
    assert!((visible_height - 100.0).abs() < 1.0);
}

#[test]
fn test_fixture_06_vertical_stack() {
    let snapshot = load_fixture_snapshot("06_vertical_stack.html");
    assert_eq!(snapshot["tag"], "html");

    let mut divs = Vec::new();
    find_elements_by_tag(&snapshot, "div", &mut divs);
    assert_eq!(divs.len(), 2);

    let item1 = divs[0];
    let item2 = divs[1];

    assert_eq!(item1["attrs"]["class"], "item1");
    assert_eq!(item2["attrs"]["class"], "item2");

    let y1 = item1["rect"]["y"]
        .as_f64()
        .unwrap_or_else(|| panic!("y1 must be a number"));
    let h1 = item1["rect"]["height"]
        .as_f64()
        .unwrap_or_else(|| panic!("h1 must be a number"));
    let y2 = item2["rect"]["y"]
        .as_f64()
        .unwrap_or_else(|| panic!("y2 must be a number"));

    assert!((h1 - 50.0).abs() < 1.0);
    // Stacked vertically: item2 must start below item1
    assert!(
        y2 >= y1 + h1 - 0.1,
        "item2 y ({}) should be below item1 bottom ({})",
        y2,
        y1 + h1
    );
}

#[test]
fn test_fixture_visual_paint_gate() {
    let path = std::path::Path::new("tests/oracle/fixtures").join("01_single_block_text.html");
    let html = std::fs::read_to_string(&path).unwrap();

    let base_url = underrated::url::Url::parse("http://localhost/").unwrap();
    struct DummyLoader;
    impl underrated::loader::ResourceLoader for DummyLoader {
        fn load(
            &self,
            _url: &underrated::url::Url,
        ) -> Result<Vec<u8>, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
        fn load_request(
            &self,
            _url: &underrated::url::Url,
            _method: underrated::loader::HttpMethod,
            _body: &[u8],
            _content_type: Option<&str>,
        ) -> Result<underrated::loader::LoaderResponse, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
    }

    let canvas =
        underrated::engine::render_page_to_canvas(&html, &base_url, &DummyLoader, 800, 600);
    assert_eq!(canvas.width, 800);
    assert_eq!(canvas.height, 600);

    let has_drawn_pixels = canvas.pixels.iter().any(|&p| p != 0 && p != 0xFFFFFFFF);
    assert!(
        has_drawn_pixels,
        "Canvas should have drawn some non-white pixels"
    );
}

#[test]
fn test_fixture_07_google_mock() {
    let snapshot = load_fixture_snapshot("07_google_mock.html");
    assert_eq!(snapshot["tag"], "html");

    let mut scripts = Vec::new();
    find_elements_by_tag(&snapshot, "script", &mut scripts);
    for script in scripts {
        let width = script["rect"]["width"].as_f64().unwrap();
        let height = script["rect"]["height"].as_f64().unwrap();
        assert_eq!(width, 0.0);
        assert_eq!(height, 0.0);
    }

    let search_box = find_element_by_class(&snapshot, "search-box")
        .unwrap_or_else(|| panic!("search-box element must exist"));
    let sb_x = search_box["rect"]["x"].as_f64().unwrap();
    let sb_w = search_box["rect"]["width"].as_f64().unwrap();
    println!(
        "DIAGNOSTIC: .search-box rect x={}, width={}, center={}",
        sb_x,
        sb_w,
        sb_x + sb_w / 2.0
    );
    assert_centered(search_box, 800.0, 4.0);

    let logo = find_element_by_class(&snapshot, "logo")
        .unwrap_or_else(|| panic!("logo element must exist"));
    let logo_x = logo["rect"]["x"].as_f64().unwrap();
    let logo_w = logo["rect"]["width"].as_f64().unwrap();
    println!(
        "DIAGNOSTIC: .logo rect x={}, width={}, center={}",
        logo_x,
        logo_w,
        logo_x + logo_w / 2.0
    );
    assert_centered(logo, 800.0, 4.0);

    let mut buttons = Vec::new();
    find_elements_by_tag(&snapshot, "button", &mut buttons);
    assert_eq!(buttons.len(), 2, "Should find exactly two buttons");
    for (i, button) in buttons.iter().enumerate() {
        let b_w = button["rect"]["width"].as_f64().unwrap();
        println!("DIAGNOSTIC: button {} rect width={}", i + 1, b_w);
        assert_max_width(button, 800.0, 0.4);
    }

    let path = std::path::Path::new("tests/oracle/fixtures").join("07_google_mock.html");
    let html = std::fs::read_to_string(&path).unwrap();

    let base_url = underrated::url::Url::parse("http://localhost/").unwrap();
    struct DummyLoader;
    impl underrated::loader::ResourceLoader for DummyLoader {
        fn load(
            &self,
            _url: &underrated::url::Url,
        ) -> Result<Vec<u8>, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
        fn load_request(
            &self,
            _url: &underrated::url::Url,
            _method: underrated::loader::HttpMethod,
            _body: &[u8],
            _content_type: Option<&str>,
        ) -> Result<underrated::loader::LoaderResponse, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
    }

    let canvas =
        underrated::engine::render_page_to_canvas(&html, &base_url, &DummyLoader, 800, 600);
    assert_eq!(canvas.width, 800);
    assert_eq!(canvas.height, 600);

    let has_drawn_pixels = canvas.pixels.iter().any(|&p| p != 0 && p != 0xFFFFFFFF);
    assert!(
        has_drawn_pixels,
        "Canvas should have drawn some non-white pixels"
    );
}

/// This test acts as a C-1 regression floor for the real Google homepage layout.
/// It verifies that the search input is centered in the viewport, and that the two
/// inline-block buttons (btnG, btnI) flow side by side and are centered *as a pair*
/// (their combined bounding box is centered — individually centering each would force
/// them to overlap at the center), each shrinking-to-fit under 40% of the viewport.
#[test]
fn test_fixture_08_google_real() {
    let snapshot = load_fixture_snapshot("08_google_real.html");
    assert_eq!(snapshot["tag"], "html");

    // Find the search input by name="q" (or class "lst")
    let q_input = find_element_by_attr(&snapshot, "name", "q")
        .or_else(|| find_element_by_class(&snapshot, "lst"))
        .unwrap_or_else(|| panic!("search input 'q' must exist"));

    let q_x = q_input["rect"]["x"].as_f64().unwrap();
    let q_w = q_input["rect"]["width"].as_f64().unwrap();
    println!(
        "DIAGNOSTIC: search input q rect x={}, width={}, center={}",
        q_x,
        q_w,
        q_x + q_w / 2.0
    );
    assert_centered(q_input, 800.0, 4.0);

    // Find BOTH <input class="lsb"> buttons (btnG, btnI)
    let mut lsb_buttons = Vec::new();
    find_elements_by_class(&snapshot, "lsb", &mut lsb_buttons);
    assert_eq!(
        lsb_buttons.len(),
        2,
        "Should find exactly two 'lsb' buttons"
    );

    let mut min_left = f64::INFINITY;
    let mut max_right = f64::NEG_INFINITY;
    let mut spans: Vec<(f64, f64)> = Vec::new();
    for (i, button) in lsb_buttons.iter().enumerate() {
        let b_x = button["rect"]["x"].as_f64().unwrap();
        let b_w = button["rect"]["width"].as_f64().unwrap();
        let name = button["attrs"]["name"].as_str().unwrap_or("unknown");
        println!(
            "DIAGNOSTIC: lsb button {} '{}' rect x={}, width={}, center={}",
            i + 1,
            name,
            b_x,
            b_w,
            b_x + b_w / 2.0
        );
        // Each button is shrink-to-fit (well under 40% of the viewport).
        assert_max_width(button, 800.0, 0.4);
        min_left = min_left.min(b_x);
        max_right = max_right.max(b_x + b_w);
        spans.push((b_x, b_x + b_w));
    }

    // The two inline-block buttons flow side by side; as a centered pair their combined
    // bounding box is centered in the viewport. Asserting each button individually centered
    // would be wrong: it could only hold if the buttons overlapped at the center.
    let group_center = (min_left + max_right) / 2.0;
    assert!(
        (group_center - 400.0).abs() <= 4.0,
        "Button pair must be centered as a group: combined center is {}, expected 400, tolerance 4",
        group_center
    );

    // The buttons must not overlap horizontally (inline-block side-by-side flow).
    spans.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("button x must be comparable"));
    for w in spans.windows(2) {
        assert!(
            w[0].1 <= w[1].0 + 0.5,
            "Adjacent buttons must not overlap: left ends at {}, right starts at {}",
            w[0].1,
            w[1].0
        );
    }

    // Assert every <input type="hidden"> has rect width == 0.0 (regression guard for t0336 display:none)
    let mut inputs = Vec::new();
    find_elements_by_tag(&snapshot, "input", &mut inputs);
    let mut hidden_count = 0;
    for input in inputs {
        let is_hidden = input["attrs"]["type"].as_str() == Some("hidden");
        if is_hidden {
            let width = input["rect"]["width"]
                .as_f64()
                .expect("rect width must be a number");
            assert_eq!(
                width, 0.0,
                "Hidden input (name: {:?}) must have a width of 0.0, got {}",
                input["attrs"]["name"], width
            );
            hidden_count += 1;
        }
    }
    assert!(
        hidden_count > 0,
        "Should have found at least one hidden input"
    );

    // TODO(spec): td percent-width cell collapse (real-google residual B) not yet asserted
}

// TODO(spec): B-3 verification — proves relative-URL <img> resolves against page base, fetches via loader,
// and blits. Real external fetch (HttpLoader/network) and placeholder-on-failure rendering are out of scope.
#[test]
fn test_b3_relative_url_image_blits() {
    let mut img_canvas = underrated::raster::Canvas::new(40, 20);
    img_canvas.pixels.fill(0xFF0000FF);
    let png = underrated::image::encode_png(&img_canvas);
    assert!(!png.is_empty(), "PNG encoding must not be empty");

    struct StubLoader {
        png: Vec<u8>,
    }
    impl underrated::loader::ResourceLoader for StubLoader {
        fn load(
            &self,
            url: &underrated::url::Url,
        ) -> Result<Vec<u8>, underrated::loader::LoadError> {
            if url.serialize() == "https://www.example.com/images/logo.png" {
                Ok(self.png.clone())
            } else {
                Err(underrated::loader::LoadError::NotFound)
            }
        }
        fn load_request(
            &self,
            _url: &underrated::url::Url,
            _method: underrated::loader::HttpMethod,
            _body: &[u8],
            _content_type: Option<&str>,
        ) -> Result<underrated::loader::LoaderResponse, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
    }

    let html = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <img src="/images/logo.png" width="40" height="20">
        </body>
        </html>
    "#;
    let base_url = underrated::url::Url::parse("https://www.example.com/").unwrap();
    let stub_loader = StubLoader { png };

    let canvas_positive =
        underrated::engine::render_page_to_canvas(html, &base_url, &stub_loader, 200, 100);

    let blue_pixel_count_positive = canvas_positive
        .pixels
        .iter()
        .filter(|&&p| p == 0xFF0000FF)
        .count();

    assert!(
        blue_pixel_count_positive > 100,
        "Rendered canvas should have drawn blue pixels. Found count: {}",
        blue_pixel_count_positive
    );

    struct DummyLoader;
    impl underrated::loader::ResourceLoader for DummyLoader {
        fn load(
            &self,
            _url: &underrated::url::Url,
        ) -> Result<Vec<u8>, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
        fn load_request(
            &self,
            _url: &underrated::url::Url,
            _method: underrated::loader::HttpMethod,
            _body: &[u8],
            _content_type: Option<&str>,
        ) -> Result<underrated::loader::LoaderResponse, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
    }

    let canvas_negative =
        underrated::engine::render_page_to_canvas(html, &base_url, &DummyLoader, 200, 100);

    let blue_pixel_count_negative = canvas_negative
        .pixels
        .iter()
        .filter(|&&p| p == 0xFF0000FF)
        .count();

    assert_eq!(
        blue_pixel_count_negative, 0,
        "Negative control canvas must have zero blue pixels. Found: {}",
        blue_pixel_count_negative
    );
}

#[test]
fn test_fixture_09_wiki_article() {
    let snapshot = load_fixture_snapshot("09_wiki_article.html");
    assert_eq!(snapshot["tag"], "html");

    // Headings verification
    let h1 = find_element_by_tag(&snapshot, "h1")
        .unwrap_or_else(|| panic!("h1 element must exist in 09_wiki_article"));
    let h1_w = h1["rect"]["width"].as_f64().unwrap();
    let h1_h = h1["rect"]["height"].as_f64().unwrap();
    assert!(h1_w > 0.0, "h1 width must be positive, got {}", h1_w);
    assert!(h1_h > 0.0, "h1 height must be positive, got {}", h1_h);

    let mut h2s = Vec::new();
    find_elements_by_tag(&snapshot, "h2", &mut h2s);
    assert_eq!(h2s.len(), 3, "Should find exactly 3 h2 headings");
    for h2 in &h2s {
        let h2_w = h2["rect"]["width"].as_f64().unwrap();
        let h2_h = h2["rect"]["height"].as_f64().unwrap();
        assert!(h2_w > 0.0, "h2 width must be positive");
        assert!(h2_h > 0.0, "h2 height must be positive");
    }

    // Paragraphs verification
    let mut ps = Vec::new();
    find_elements_by_tag(&snapshot, "p", &mut ps);
    assert_eq!(ps.len(), 2, "Should find exactly 2 paragraphs");

    let p1_y = ps[0]["rect"]["y"].as_f64().unwrap();
    let p1_h = ps[0]["rect"]["height"].as_f64().unwrap();
    let p2_y = ps[1]["rect"]["y"].as_f64().unwrap();
    assert!(
        p2_y >= p1_y + p1_h - 0.1,
        "Paragraph 2 (y={}) should be below Paragraph 1 bottom (y={})",
        p2_y,
        p1_y + p1_h
    );

    // Lists and list items verification
    let ul =
        find_element_by_tag(&snapshot, "ul").unwrap_or_else(|| panic!("ul element must exist"));
    let mut lis = Vec::new();
    find_elements_by_tag(ul, "li", &mut lis);
    assert_eq!(lis.len(), 3, "Should find exactly 3 list items");
    for li in &lis {
        let li_w = li["rect"]["width"].as_f64().unwrap();
        let li_h = li["rect"]["height"].as_f64().unwrap();
        assert!(li_w > 0.0, "list item width must be positive");
        assert!(li_h > 0.0, "list item height must be positive");
    }

    // Inline elements verification (b, i, a)
    let b = find_element_by_tag(&snapshot, "b").unwrap_or_else(|| panic!("b element must exist"));
    let i = find_element_by_tag(&snapshot, "i").unwrap_or_else(|| panic!("i element must exist"));
    let a = find_element_by_tag(&snapshot, "a").unwrap_or_else(|| panic!("a element must exist"));

    assert!(b["rect"]["width"].as_f64().unwrap() > 0.0);
    assert!(i["rect"]["width"].as_f64().unwrap() > 0.0);
    assert!(a["rect"]["width"].as_f64().unwrap() > 0.0);

    // Infobox column and main-content column verification
    let infobox = find_element_by_class(&snapshot, "infobox")
        .unwrap_or_else(|| panic!("infobox element must exist"));
    let info_w = infobox["rect"]["width"].as_f64().unwrap();
    let info_h = infobox["rect"]["height"].as_f64().unwrap();
    assert_eq!(
        info_w, 262.0,
        "Infobox width should be 262.0 (240px width + 20px padding + 2px border)"
    );
    assert!(info_h > 0.0, "Infobox height must be positive");

    let main_content = find_element_by_class(&snapshot, "main-content")
        .unwrap_or_else(|| panic!("main-content element must exist"));
    let main_w = main_content["rect"]["width"].as_f64().unwrap();
    assert_eq!(
        main_w, 480.0,
        "Main content width should match specified 480.0"
    );
}

#[test]
fn test_fixture_09_wiki_infobox_internal_layout() {
    let snapshot = load_fixture_snapshot("09_wiki_article.html");

    // Locate the infobox element via find_element_by_class. Read its rect.
    let infobox = find_element_by_class(&snapshot, "infobox")
        .unwrap_or_else(|| panic!("infobox element must exist"));
    let info_x = infobox["rect"]["x"].as_f64().unwrap();
    let _info_y = infobox["rect"]["y"].as_f64().unwrap();
    let info_w = infobox["rect"]["width"].as_f64().unwrap();
    let info_h = infobox["rect"]["height"].as_f64().unwrap();

    assert!(
        info_h > 0.0,
        "Infobox height must be positive, got {}",
        info_h
    );

    // Locate the infobox-title via find_element_by_class.
    let title = find_element_by_class(&snapshot, "infobox-title")
        .unwrap_or_else(|| panic!("infobox-title element must exist"));
    let title_x = title["rect"]["x"].as_f64().unwrap();
    let title_y = title["rect"]["y"].as_f64().unwrap();
    let title_w = title["rect"]["width"].as_f64().unwrap();
    let title_h = title["rect"]["height"].as_f64().unwrap();

    // Assert each row and the title has positive width and positive height.
    assert!(
        title_w > 0.0,
        "Title width must be positive, got {}",
        title_w
    );
    assert!(
        title_h > 0.0,
        "Title height must be positive, got {}",
        title_h
    );

    // Collect the three infobox-row elements via find_elements_by_class.
    let mut rows = Vec::new();
    find_elements_by_class(&snapshot, "infobox-row", &mut rows);
    assert_eq!(rows.len(), 3, "Should find exactly 3 infobox rows");

    // Assert vertical stacking and positive dimensions for each row
    let mut last_y = title_y;
    let mut last_bottom = title_y + title_h;

    // Title vertical stacking: title sits above first row
    let first_row_y = rows[0]["rect"]["y"].as_f64().unwrap();
    assert!(
        title_y < first_row_y,
        "Title y ({}) must be strictly less than first row y ({})",
        title_y,
        first_row_y
    );

    // Verify row layout
    for (idx, row) in rows.iter().enumerate() {
        let r_y = row["rect"]["y"].as_f64().unwrap();
        let r_w = row["rect"]["width"].as_f64().unwrap();
        let r_h = row["rect"]["height"].as_f64().unwrap();

        assert!(r_w > 0.0, "Row {} width must be positive, got {}", idx, r_w);
        assert!(
            r_h > 0.0,
            "Row {} height must be positive, got {}",
            idx,
            r_h
        );

        // Strict increasing y order
        if idx > 0 {
            assert!(
                last_y < r_y,
                "Row {} y ({}) must be strictly greater than previous row y ({})",
                idx,
                r_y,
                last_y
            );
        }

        // Must not horizontally overlap sibling: next element's y is at or below previous element's bottom - 0.5
        assert!(
            r_y >= last_bottom - 0.5,
            "Element {} y ({}) must be at or below previous bottom ({}) - 0.5",
            idx,
            r_y,
            last_bottom
        );

        last_y = r_y;
        last_bottom = r_y + r_h;
    }

    // Assert CONTAINMENT: each of the title and the three rows has x >= infobox.x - 0.5
    // and (x + width) <= (infobox.x + infobox.width) + 0.5
    assert!(
        title_x >= info_x - 0.5,
        "Title x ({}) must be >= infobox.x ({}) - 0.5",
        title_x,
        info_x
    );
    assert!(
        (title_x + title_w) <= (info_x + info_w) + 0.5,
        "Title right ({}) must be <= infobox right ({}) + 0.5",
        title_x + title_w,
        info_x + info_w
    );

    for (idx, row) in rows.iter().enumerate() {
        let r_x = row["rect"]["x"].as_f64().unwrap();
        let r_w = row["rect"]["width"].as_f64().unwrap();

        assert!(
            r_x >= info_x - 0.5,
            "Row {} x ({}) must be >= infobox.x ({}) - 0.5",
            idx,
            r_x,
            info_x
        );
        assert!(
            (r_x + r_w) <= (info_x + info_w) + 0.5,
            "Row {} right ({}) must be <= infobox right ({}) + 0.5",
            idx,
            r_x + r_w,
            info_x + info_w
        );
    }
}

#[test]
fn test_fixture_10_news_article() {
    let snapshot = load_fixture_snapshot("10_news_article.html");
    assert_eq!(snapshot["tag"], "html");

    // Headings verification
    let h1 = find_element_by_tag(&snapshot, "h1")
        .unwrap_or_else(|| panic!("h1 element must exist in 10_news_article"));
    let h1_w = h1["rect"]["width"].as_f64().unwrap();
    let h1_h = h1["rect"]["height"].as_f64().unwrap();
    assert!(h1_w > 0.0, "h1 width must be positive, got {}", h1_w);
    assert!(h1_h > 0.0, "h1 height must be positive, got {}", h1_h);

    let h2 = find_element_by_tag(&snapshot, "h2")
        .unwrap_or_else(|| panic!("h2 element must exist in 10_news_article"));
    let h2_w = h2["rect"]["width"].as_f64().unwrap();
    let h2_h = h2["rect"]["height"].as_f64().unwrap();
    assert!(h2_w > 0.0, "h2 width must be positive, got {}", h2_w);
    assert!(h2_h > 0.0, "h2 height must be positive, got {}", h2_h);

    // Paragraphs verification
    let mut ps = Vec::new();
    find_elements_by_tag(&snapshot, "p", &mut ps);
    assert_eq!(ps.len(), 4, "Should find exactly 4 paragraphs");

    // Assert that consecutive paragraphs are in top-to-bottom document order
    for i in 0..ps.len() - 1 {
        let p_curr_y = ps[i]["rect"]["y"].as_f64().unwrap();
        let p_curr_h = ps[i]["rect"]["height"].as_f64().unwrap();
        let p_next_y = ps[i + 1]["rect"]["y"].as_f64().unwrap();
        assert!(
            p_next_y >= p_curr_y + p_curr_h - 0.1,
            "Paragraph {} (y={}) should be below Paragraph {} bottom (y={})",
            i + 1,
            p_next_y,
            i,
            p_curr_y + p_curr_h
        );
    }

    // Lists and list items verification
    let ul =
        find_element_by_tag(&snapshot, "ul").unwrap_or_else(|| panic!("ul element must exist"));
    let mut lis = Vec::new();
    find_elements_by_tag(ul, "li", &mut lis);
    assert_eq!(lis.len(), 3, "Should find exactly 3 list items");
    for li in &lis {
        let li_w = li["rect"]["width"].as_f64().unwrap();
        let li_h = li["rect"]["height"].as_f64().unwrap();
        assert!(li_w > 0.0, "list item width must be positive");
        assert!(li_h > 0.0, "list item height must be positive");
    }

    // Container width verification
    let container = find_element_by_class(&snapshot, "container")
        .unwrap_or_else(|| panic!("container element must exist"));
    let container_w = container["rect"]["width"].as_f64().unwrap();
    assert_eq!(
        container_w, 600.0,
        "Container width should match specified 600.0"
    );
}
