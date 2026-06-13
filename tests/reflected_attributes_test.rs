use underrated::dom::{Dom, NodeData};
use underrated::script::BoaHost;

#[test]
fn test_title_empty_by_default() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.title;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "");
}

#[test]
fn test_title_reflect_set_attribute() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.setAttribute('title', 'Welcome to my site');
        el.title;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "Welcome to my site");
}

#[test]
fn test_title_setter_updates_attribute() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.title = 'Hello World';
        el.getAttribute('title');
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "Hello World");
}

#[test]
fn test_title_round_trip() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.title = 'round-trip-title';
        el.title;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "round-trip-title");
}

#[test]
fn test_slot_empty_by_default() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.slot;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "");
}

#[test]
fn test_slot_reflect_set_attribute() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.setAttribute('slot', 'my-slot');
        el.slot;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "my-slot");
}

#[test]
fn test_slot_setter_updates_attribute() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.slot = 'changed-slot';
        el.getAttribute('slot');
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "changed-slot");
}

#[test]
fn test_nonce_empty_by_default() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.nonce;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "");
}

#[test]
fn test_nonce_reflect_set_attribute() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.setAttribute('nonce', 'abc123nonce');
        el.nonce;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "abc123nonce");
}

#[test]
fn test_nonce_setter_updates_attribute() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.nonce = 'xyz987nonce';
        el.getAttribute('nonce');
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "xyz987nonce");
}

#[test]
fn test_tabindex_empty_by_default() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.tabIndex;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "-1");
}

#[test]
fn test_tabindex_reflect_set_attribute() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![
            ("id".into(), "target".into()),
            ("tabindex".into(), "3".into()),
        ],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.tabIndex;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "3");
}

#[test]
fn test_tabindex_invalid_parses_to_negative_one() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![
            ("id".into(), "target".into()),
            ("tabindex".into(), "abc".into()),
        ],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.tabIndex;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "-1");
}

#[test]
fn test_tabindex_setter_updates_attribute() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.tabIndex = 5;
        el.getAttribute('tabindex');
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "5");
}

#[test]
fn test_tabindex_setter_coercion() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.tabIndex = 12.7;
        const res1 = el.getAttribute('tabindex');
        el.tabIndex = 'abc';
        const res2 = el.getAttribute('tabindex');
        el.tabIndex = true;
        const res3 = el.getAttribute('tabindex');
        [res1, res2, res3].join('|');
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "12|0|1");
}

#[test]
fn test_classname_empty_by_default() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.className;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "");
}

#[test]
fn test_classname_reflect_set_attribute() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.setAttribute('class', 'foo bar');
        el.className;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "foo bar");
}

#[test]
fn test_classname_setter_updates_attribute() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.className = 'my-custom-class';
        el.getAttribute('class');
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "my-custom-class");
}

#[test]
fn test_classname_round_trip() {
    let mut dom = Dom::new();
    let doc = dom.document();

    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("id".into(), "target".into())],
    });
    dom.append_child(doc, div);

    let mut host = BoaHost::new();
    let script = r#"
        const el = document.getElementById('target');
        el.className = 'round-trip';
        el.className;
    "#;

    let res = host.eval_with_dom(script, &mut dom).unwrap();
    assert_eq!(res, "round-trip");
}
