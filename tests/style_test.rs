use underrated::css::parser::parse_stylesheet;
use underrated::css::values::{Color, CssValue};
use underrated::dom::{Dom, NodeData};
use underrated::style::compute_styles_with_viewport;

#[test]
fn test_inline_style_attribute() {
    let mut dom = Dom::new();
    let doc = dom.document();
    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("style".into(), "color: red;".into())],
    });
    dom.append_child(doc, div);

    let stylesheet = parse_stylesheet("div { color: blue; }");
    let styles = compute_styles_with_viewport(&dom, &stylesheet, 1024.0);

    let div_style = styles.get(&div).unwrap();
    if let Some(CssValue::Color(c)) = div_style.get("color") {
        // inline style should beat author rule
        assert_eq!(c, &Color::Rgba(255, 0, 0, 255));
    } else {
        panic!(
            "Expected color red from inline style, got {:?}",
            div_style.get("color")
        );
    }
}

#[test]
fn test_media_query_max_width() {
    let mut dom = Dom::new();
    let doc = dom.document();
    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![],
    });
    dom.append_child(doc, div);

    let stylesheet = parse_stylesheet(
        "
        div { color: blue; }
        @media (max-width: 600px) {
            div { color: red; }
        }
    ",
    );

    // At 800px width, @media should NOT apply
    let styles = compute_styles_with_viewport(&dom, &stylesheet, 800.0);
    let div_style = styles.get(&div).unwrap();
    if let Some(CssValue::Color(c)) = div_style.get("color") {
        assert_eq!(c, &Color::Rgba(0, 0, 255, 255)); // remains blue
    } else {
        panic!("Expected color blue");
    }

    // At 500px width, @media SHOULD apply
    let styles = compute_styles_with_viewport(&dom, &stylesheet, 500.0);
    let div_style = styles.get(&div).unwrap();
    if let Some(CssValue::Color(c)) = div_style.get("color") {
        assert_eq!(c, &Color::Rgba(255, 0, 0, 255)); // becomes red
    } else {
        panic!("Expected color red");
    }
}

#[test]
fn test_media_query_min_width() {
    let mut dom = Dom::new();
    let doc = dom.document();
    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![],
    });
    dom.append_child(doc, div);

    let stylesheet = parse_stylesheet(
        "
        div { color: blue; }
        @media (min-width: 600px) {
            div { color: red; }
        }
    ",
    );

    // At 500px width, @media should NOT apply
    let styles = compute_styles_with_viewport(&dom, &stylesheet, 500.0);
    let div_style = styles.get(&div).unwrap();
    if let Some(CssValue::Color(c)) = div_style.get("color") {
        assert_eq!(c, &Color::Rgba(0, 0, 255, 255));
    }

    // At 800px width, @media SHOULD apply
    let styles = compute_styles_with_viewport(&dom, &stylesheet, 800.0);
    let div_style = styles.get(&div).unwrap();
    if let Some(CssValue::Color(c)) = div_style.get("color") {
        assert_eq!(c, &Color::Rgba(255, 0, 0, 255));
    }
}
