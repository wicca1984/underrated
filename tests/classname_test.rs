use underrated::dom::{Dom, NodeData};
use underrated::script::BoaHost;

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
