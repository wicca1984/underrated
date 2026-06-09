use std::cell::RefCell;
use std::rc::Rc;
use underrated::dom::{Dom, NodeData};
use underrated::event::{
    AT_TARGET, BUBBLING_PHASE, CAPTURING_PHASE, DomEvent, EventRegistry, NONE,
};

#[test]
fn test_integration_event_propagation_phases_and_stops() {
    // 1. Create a tree: root -> parent -> child
    let mut dom = Dom::new();
    let root = dom.document();
    let parent = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![],
    });
    let child = dom.create_node(NodeData::Element {
        name: "span".into(),
        attrs: vec![],
    });
    dom.append_child(root, parent);
    dom.append_child(parent, child);

    let mut registry = EventRegistry::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    // 2. Add listeners to trace propagation phases
    {
        let log = log.clone();
        registry.add_listener(
            root,
            true, // capture
            Box::new(move |e| {
                assert_eq!(e.event_phase(), CAPTURING_PHASE);
                assert_eq!(e.current_target(), Some(root));
                log.borrow_mut().push("root_capture");
            }),
        );
    }
    {
        let log = log.clone();
        registry.add_listener(
            parent,
            true, // capture
            Box::new(move |e| {
                assert_eq!(e.event_phase(), CAPTURING_PHASE);
                assert_eq!(e.current_target(), Some(parent));
                log.borrow_mut().push("parent_capture");
            }),
        );
    }
    {
        let log = log.clone();
        registry.add_listener(
            child,
            true, // capture (runs at target phase)
            Box::new(move |e| {
                assert_eq!(e.event_phase(), AT_TARGET);
                assert_eq!(e.current_target(), Some(child));
                log.borrow_mut().push("child_capture");
            }),
        );
    }
    {
        let log = log.clone();
        registry.add_listener(
            child,
            false, // bubble (runs at target phase)
            Box::new(move |e| {
                assert_eq!(e.event_phase(), AT_TARGET);
                assert_eq!(e.current_target(), Some(child));
                log.borrow_mut().push("child_bubble");
            }),
        );
    }
    {
        let log = log.clone();
        registry.add_listener(
            parent,
            false, // bubble
            Box::new(move |e| {
                assert_eq!(e.event_phase(), BUBBLING_PHASE);
                assert_eq!(e.current_target(), Some(parent));
                log.borrow_mut().push("parent_bubble");
            }),
        );
    }
    {
        let log = log.clone();
        registry.add_listener(
            root,
            false, // bubble
            Box::new(move |e| {
                assert_eq!(e.event_phase(), BUBBLING_PHASE);
                assert_eq!(e.current_target(), Some(root));
                log.borrow_mut().push("root_bubble");
            }),
        );
    }

    let event = DomEvent::click(child);
    assert_eq!(event.event_phase(), NONE);
    assert_eq!(event.current_target(), None);

    let not_canceled = registry.dispatch(&dom, &event);

    // Verify propagation completed and default was not prevented
    assert!(not_canceled);
    assert_eq!(event.event_phase(), NONE);
    assert_eq!(event.current_target(), None);

    assert_eq!(
        *log.borrow(),
        vec![
            "root_capture",
            "parent_capture",
            "child_capture",
            "child_bubble",
            "parent_bubble",
            "root_bubble"
        ]
    );
}

#[test]
fn test_integration_stop_propagation_in_capture() {
    let mut dom = Dom::new();
    let root = dom.document();
    let parent = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![],
    });
    let child = dom.create_node(NodeData::Element {
        name: "span".into(),
        attrs: vec![],
    });
    dom.append_child(root, parent);
    dom.append_child(parent, child);

    let mut registry = EventRegistry::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    // Root capture listener stops propagation
    {
        let log = log.clone();
        registry.add_listener(
            root,
            true,
            Box::new(move |e| {
                log.borrow_mut().push("root_capture");
                e.stop_propagation();
            }),
        );
    }
    {
        let log = log.clone();
        registry.add_listener(
            parent,
            true,
            Box::new(move |_| {
                log.borrow_mut().push("parent_capture");
            }),
        );
    }

    let event = DomEvent::click(child);
    registry.dispatch(&dom, &event);

    // Only root capture should run because it stopped propagation
    assert_eq!(*log.borrow(), vec!["root_capture"]);
}
