use serde_json::Value;
use std::path::Path;

fn assert_structural_invariants(node: &Value) {
    let node_type = node["type"]
        .as_str()
        .expect("Node must have a 'type' field");
    match node_type {
        "element" => {
            let tag = node["tag"]
                .as_str()
                .expect("Element node must have a 'tag' string field");
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
            let _x = rect["x"].as_f64().expect("rect.x must be a number");
            let _y = rect["y"].as_f64().expect("rect.y must be a number");
            let width = rect["width"].as_f64().expect("rect.width must be a number");
            let height = rect["height"]
                .as_f64()
                .expect("rect.height must be a number");

            assert!(width >= 0.0, "rect.width of '{}' must be non-negative", tag);
            assert!(
                height >= 0.0,
                "rect.height of '{}' must be non-negative",
                tag
            );

            let children = node["children"]
                .as_array()
                .expect("Element node must have a 'children' array");
            for child in children {
                assert_structural_invariants(child);
            }
        }
        "text" => {
            let text = node["text"]
                .as_str()
                .expect("Text node must have a 'text' string field");
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

fn collect_text_nodes<'a>(node: &'a Value, results: &mut Vec<&'a str>) {
    if node["type"] == "text" {
        if let Some(text) = node["text"].as_str() {
            results.push(text);
        }
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            collect_text_nodes(child, results);
        }
    }
}

#[test]
fn test_oracle_snapshot_fixtures() {
    let fixtures: Vec<(&str, Box<dyn Fn(&Value)>)> = vec![
        (
            "01_single_block_text.html",
            Box::new(|snapshot: &Value| {
                // Root should be html
                assert_eq!(snapshot["tag"], "html");

                // Check that the div element exists
                let div = find_element_by_tag(snapshot, "div")
                    .expect("div element must exist in 01_single_block_text");

                // Check text inside div is Hello World
                let mut text_nodes = Vec::new();
                collect_text_nodes(div, &mut text_nodes);
                assert_eq!(text_nodes, vec!["Hello World"]);

                // Width should be approx 400.0, height approx 100.0
                let width = div["rect"]["width"].as_f64().unwrap();
                let height = div["rect"]["height"].as_f64().unwrap();
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
            }),
        ),
        (
            "02_nested_blocks_inline_css.html",
            Box::new(|snapshot: &Value| {
                assert_eq!(snapshot["tag"], "html");

                // Find outer and inner div by finding all divs
                let mut divs = Vec::new();
                find_elements_by_tag(snapshot, "div", &mut divs);
                assert_eq!(divs.len(), 2, "Should have exactly 2 div elements");

                let outer_div = divs[0];
                let inner_div = divs[1];

                // Check attributes
                assert!(
                    outer_div["attrs"]["style"]
                        .as_str()
                        .unwrap()
                        .contains("width: 500px")
                );
                assert!(
                    inner_div["attrs"]["style"]
                        .as_str()
                        .unwrap()
                        .contains("width: 250px")
                );

                // Check rects
                let outer_width = outer_div["rect"]["width"].as_f64().unwrap();
                let inner_width = inner_div["rect"]["width"].as_f64().unwrap();
                let inner_height = inner_div["rect"]["height"].as_f64().unwrap();

                assert!((outer_width - 500.0).abs() < 1.0);
                assert!((inner_width - 250.0).abs() < 1.0);
                assert!((inner_height - 150.0).abs() < 1.0);

                // Inner div should be nested within the hierarchy
                // Verify inner_div is indeed in outer_div's children
                let outer_children = outer_div["children"].as_array().unwrap();
                let found_inner = outer_children
                    .iter()
                    .any(|c| c["tag"] == "div" && c["rect"]["width"] == inner_div["rect"]["width"]);
                assert!(found_inner, "inner_div must be nested under outer_div");
            }),
        ),
        (
            "03_paragraph_text_wrap.html",
            Box::new(|snapshot: &Value| {
                assert_eq!(snapshot["tag"], "html");

                let p = find_element_by_tag(snapshot, "p").expect("p element must exist");
                let p_width = p["rect"]["width"].as_f64().unwrap();
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
            }),
        ),
        (
            "04_list_items.html",
            Box::new(|snapshot: &Value| {
                assert_eq!(snapshot["tag"], "html");

                let ul = find_element_by_tag(snapshot, "ul").expect("ul element must exist");
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
            }),
        ),
        (
            "05_display_none.html",
            Box::new(|snapshot: &Value| {
                assert_eq!(snapshot["tag"], "html");

                // Element with class .hidden has display: none
                let hidden = find_element_by_tag(snapshot, "div")
                    .and_then(|_c| {
                        let mut divs = Vec::new();
                        find_elements_by_tag(snapshot, "div", &mut divs);
                        divs.into_iter().find(|d| d["attrs"]["class"] == "hidden")
                    })
                    .expect("hidden div element should exist in DOM");

                let hidden_width = hidden["rect"]["width"].as_f64().unwrap();
                let hidden_height = hidden["rect"]["height"].as_f64().unwrap();
                assert_eq!(hidden_width, 0.0);
                assert_eq!(hidden_height, 0.0);

                let visible = find_element_by_tag(snapshot, "div")
                    .and_then(|_c| {
                        let mut divs = Vec::new();
                        find_elements_by_tag(snapshot, "div", &mut divs);
                        divs.into_iter().find(|d| d["attrs"]["class"] == "visible")
                    })
                    .expect("visible div element should exist in DOM");

                let visible_width = visible["rect"]["width"].as_f64().unwrap();
                let visible_height = visible["rect"]["height"].as_f64().unwrap();
                assert!((visible_width - 200.0).abs() < 1.0);
                assert!((visible_height - 100.0).abs() < 1.0);
            }),
        ),
        (
            "06_vertical_stack.html",
            Box::new(|snapshot: &Value| {
                assert_eq!(snapshot["tag"], "html");

                let mut divs = Vec::new();
                find_elements_by_tag(snapshot, "div", &mut divs);
                assert_eq!(divs.len(), 2);

                let item1 = divs[0];
                let item2 = divs[1];

                assert_eq!(item1["attrs"]["class"], "item1");
                assert_eq!(item2["attrs"]["class"], "item2");

                let y1 = item1["rect"]["y"].as_f64().unwrap();
                let h1 = item1["rect"]["height"].as_f64().unwrap();
                let y2 = item2["rect"]["y"].as_f64().unwrap();

                assert!((h1 - 50.0).abs() < 1.0);
                // Stacked vertically: item2 must start below item1
                assert!(
                    y2 >= y1 + h1 - 0.1,
                    "item2 y ({}) should be below item1 bottom ({})",
                    y2,
                    y1 + h1
                );
            }),
        ),
    ];

    for (filename, assertion) in fixtures {
        let path = Path::new("tests/oracle/fixtures").join(filename);
        let html = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path.display(), e));

        // Render with width=800, height=600 (height parameter is kept for interface matching)
        let snapshot = underrated::oracle::export_snapshot(&html, "", 800, 600);

        // Assert top-level keys and recursive schema/structural validity
        assert_structural_invariants(&snapshot);

        // Assert specific fixture behavior
        assertion(&snapshot);
    }
}
