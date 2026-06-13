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

#[test]
fn test_object_position_style() {
    let mut dom = Dom::new();
    let doc = dom.document();
    let img = dom.create_node(NodeData::Element {
        name: "img".into(),
        attrs: vec![("style".into(), "object-position: center;".into())],
    });
    dom.append_child(doc, img);

    let stylesheet = parse_stylesheet("img { object-fit: contain; }");
    let styles = compute_styles_with_viewport(&dom, &stylesheet, 1024.0);

    let img_style = styles.get(&img).unwrap();
    assert_eq!(
        img_style.get("object-position"),
        Some(&CssValue::Keyword("center".to_string()))
    );
    assert_eq!(
        img_style.get("object-fit"),
        Some(&CssValue::Keyword("contain".to_string()))
    );
}

#[test]
fn test_scroll_behavior_style() {
    let mut dom = Dom::new();
    let doc = dom.document();
    let div1 = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("style".into(), "scroll-behavior: smooth;".into())],
    });
    let div2 = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("style".into(), "scroll-behavior: auto;".into())],
    });
    let div3 = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("style".into(), "scroll-behavior: invalid-value;".into())],
    });
    dom.append_child(doc, div1);
    dom.append_child(doc, div2);
    dom.append_child(doc, div3);

    let stylesheet = parse_stylesheet("");
    let styles = compute_styles_with_viewport(&dom, &stylesheet, 1024.0);

    let div1_style = styles.get(&div1).unwrap();
    assert_eq!(
        div1_style.get("scroll-behavior"),
        Some(&CssValue::Keyword("smooth".to_string()))
    );

    let div2_style = styles.get(&div2).unwrap();
    assert_eq!(
        div2_style.get("scroll-behavior"),
        Some(&CssValue::Keyword("auto".to_string()))
    );

    let div3_style = styles.get(&div3).unwrap();
    // Invalid value should be dropped/not set, so getting it returns None.
    assert_eq!(div3_style.get("scroll-behavior"), None);
}

#[test]
fn test_user_select_style() {
    let mut dom = Dom::new();
    let doc = dom.document();
    let div1 = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("style".into(), "user-select: none;".into())],
    });
    let div2 = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("style".into(), "user-select: text;".into())],
    });
    let div3 = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("style".into(), "user-select: invalid-value;".into())],
    });
    dom.append_child(doc, div1);
    dom.append_child(doc, div2);
    dom.append_child(doc, div3);

    let stylesheet = parse_stylesheet("");
    let styles = compute_styles_with_viewport(&dom, &stylesheet, 1024.0);

    let div1_style = styles.get(&div1).unwrap();
    assert_eq!(
        div1_style.get("user-select"),
        Some(&CssValue::Keyword("none".to_string()))
    );

    let div2_style = styles.get(&div2).unwrap();
    assert_eq!(
        div2_style.get("user-select"),
        Some(&CssValue::Keyword("text".to_string()))
    );

    let div3_style = styles.get(&div3).unwrap();
    // Invalid value should be dropped/not set, so getting it returns None.
    assert_eq!(div3_style.get("user-select"), None);
}

#[test]
fn test_accent_color_and_caret_color_style() {
    let mut dom = Dom::new();
    let doc = dom.document();
    let div1 = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![(
            "style".into(),
            "accent-color: red; caret-color: #00ff00;".into(),
        )],
    });
    let div2 = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![(
            "style".into(),
            "accent-color: auto; caret-color: auto;".into(),
        )],
    });
    let div3 = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![(
            "style".into(),
            "accent-color: invalid-val; caret-color: invalid-val;".into(),
        )],
    });
    dom.append_child(doc, div1);
    dom.append_child(doc, div2);
    dom.append_child(doc, div3);

    let stylesheet = parse_stylesheet("");
    let styles = compute_styles_with_viewport(&dom, &stylesheet, 1024.0);

    let div1_style = styles.get(&div1).unwrap();
    assert_eq!(
        div1_style.get("accent-color"),
        Some(&CssValue::Color(Color::Rgba(255, 0, 0, 255)))
    );
    assert_eq!(
        div1_style.get("caret-color"),
        Some(&CssValue::Color(Color::Rgba(0, 255, 0, 255)))
    );

    let div2_style = styles.get(&div2).unwrap();
    assert_eq!(
        div2_style.get("accent-color"),
        Some(&CssValue::Keyword("auto".to_string()))
    );
    assert_eq!(
        div2_style.get("caret-color"),
        Some(&CssValue::Keyword("auto".to_string()))
    );

    let div3_style = styles.get(&div3).unwrap();
    // Invalid value should be dropped/not set, so getting it returns the fallback auto.
    assert_eq!(
        div3_style.get("accent-color"),
        Some(&CssValue::Keyword("auto".to_string()))
    );
    assert_eq!(
        div3_style.get("caret-color"),
        Some(&CssValue::Keyword("auto".to_string()))
    );
}
