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

#[test]
fn test_box_model_resolutions() {
    let mut dom = Dom::new();
    let doc = dom.document();
    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![],
    });
    dom.append_child(doc, div);

    let stylesheet = parse_stylesheet(
        "
        div {
            margin: 10px 20px;
            padding: 4px;
            border-width: 1px 2px 3px 4px;
            box-sizing: border-box;
            display: inline-block;
        }
        ",
    );

    let styles = compute_styles_with_viewport(&dom, &stylesheet, 1024.0);
    let div_style = styles.get(&div).unwrap();

    // Check margin-top/bottom=10px, margin-left/right=20px
    assert_eq!(
        div_style.get("margin-top"),
        Some(&CssValue::Length(
            10.0,
            underrated::css::values::LengthUnit::Px
        ))
    );
    assert_eq!(
        div_style.get("margin-right"),
        Some(&CssValue::Length(
            20.0,
            underrated::css::values::LengthUnit::Px
        ))
    );
    assert_eq!(
        div_style.get("margin-bottom"),
        Some(&CssValue::Length(
            10.0,
            underrated::css::values::LengthUnit::Px
        ))
    );
    assert_eq!(
        div_style.get("margin-left"),
        Some(&CssValue::Length(
            20.0,
            underrated::css::values::LengthUnit::Px
        ))
    );

    // Check padding expansion (1-value form)
    assert_eq!(
        div_style.get("padding-top"),
        Some(&CssValue::Length(
            4.0,
            underrated::css::values::LengthUnit::Px
        ))
    );
    assert_eq!(
        div_style.get("padding-right"),
        Some(&CssValue::Length(
            4.0,
            underrated::css::values::LengthUnit::Px
        ))
    );
    assert_eq!(
        div_style.get("padding-bottom"),
        Some(&CssValue::Length(
            4.0,
            underrated::css::values::LengthUnit::Px
        ))
    );
    assert_eq!(
        div_style.get("padding-left"),
        Some(&CssValue::Length(
            4.0,
            underrated::css::values::LengthUnit::Px
        ))
    );

    // Check border-width (4-value form)
    assert_eq!(
        div_style.get("border-top-width"),
        Some(&CssValue::Length(
            1.0,
            underrated::css::values::LengthUnit::Px
        ))
    );
    assert_eq!(
        div_style.get("border-right-width"),
        Some(&CssValue::Length(
            2.0,
            underrated::css::values::LengthUnit::Px
        ))
    );
    assert_eq!(
        div_style.get("border-bottom-width"),
        Some(&CssValue::Length(
            3.0,
            underrated::css::values::LengthUnit::Px
        ))
    );
    assert_eq!(
        div_style.get("border-left-width"),
        Some(&CssValue::Length(
            4.0,
            underrated::css::values::LengthUnit::Px
        ))
    );

    // Check display & box-sizing
    assert_eq!(
        div_style.get("display"),
        Some(&CssValue::Keyword("inline-block".to_string()))
    );
    assert_eq!(
        div_style.get("box-sizing"),
        Some(&CssValue::Keyword("border-box".to_string()))
    );
}
