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
