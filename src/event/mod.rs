use crate::dom::Dom;
use crate::infra::NodeId;
use std::cell::Cell;
use std::collections::HashMap;

/// DOM event phase constants.
pub const NONE: u16 = 0;
pub const CAPTURING_PHASE: u16 = 1;
pub const AT_TARGET: u16 = 2;
pub const BUBBLING_PHASE: u16 = 3;

/// DOM events as specified in SPEC S-25.
pub enum DomEvent {
    Click {
        node: NodeId,
        stopped: Cell<bool>,
        stopped_immediate: Cell<bool>,
        default_prevented: Cell<bool>,
        current_target: Cell<Option<NodeId>>,
        event_phase: Cell<u16>,
    },
    Input {
        node: NodeId,
        value: String,
        stopped: Cell<bool>,
        stopped_immediate: Cell<bool>,
        default_prevented: Cell<bool>,
        current_target: Cell<Option<NodeId>>,
        event_phase: Cell<u16>,
    },
    KeyDown {
        key: String,
        stopped: Cell<bool>,
        stopped_immediate: Cell<bool>,
        default_prevented: Cell<bool>,
        current_target: Cell<Option<NodeId>>,
        event_phase: Cell<u16>,
    },
}

impl DomEvent {
    /// Creates a new Click event.
    pub fn click(node: NodeId) -> Self {
        Self::Click {
            node,
            stopped: Cell::new(false),
            stopped_immediate: Cell::new(false),
            default_prevented: Cell::new(false),
            current_target: Cell::new(None),
            event_phase: Cell::new(NONE),
        }
    }

    /// Creates a new Input event.
    pub fn input(node: NodeId, value: String) -> Self {
        Self::Input {
            node,
            value,
            stopped: Cell::new(false),
            stopped_immediate: Cell::new(false),
            default_prevented: Cell::new(false),
            current_target: Cell::new(None),
            event_phase: Cell::new(NONE),
        }
    }

    /// Creates a new KeyDown event.
    pub fn key_down(key: String) -> Self {
        Self::KeyDown {
            key,
            stopped: Cell::new(false),
            stopped_immediate: Cell::new(false),
            default_prevented: Cell::new(false),
            current_target: Cell::new(None),
            event_phase: Cell::new(NONE),
        }
    }

    /// Prevents further propagation of the event in the capture and bubble phases.
    pub fn stop_propagation(&self) {
        match self {
            DomEvent::Click { stopped, .. } => stopped.set(true),
            DomEvent::Input { stopped, .. } => stopped.set(true),
            DomEvent::KeyDown { stopped, .. } => stopped.set(true),
        }
    }

    /// Prevents further propagation of the event and stops other listeners on the same element from running.
    pub fn stop_immediate_propagation(&self) {
        match self {
            DomEvent::Click {
                stopped_immediate, ..
            } => stopped_immediate.set(true),
            DomEvent::Input {
                stopped_immediate, ..
            } => stopped_immediate.set(true),
            DomEvent::KeyDown {
                stopped_immediate, ..
            } => stopped_immediate.set(true),
        }
    }

    /// Cancels the event if it is cancelable, without stopping further propagation of the event.
    pub fn prevent_default(&self) {
        match self {
            DomEvent::Click {
                default_prevented, ..
            } => default_prevented.set(true),
            DomEvent::Input {
                default_prevented, ..
            } => default_prevented.set(true),
            DomEvent::KeyDown {
                default_prevented, ..
            } => default_prevented.set(true),
        }
    }

    /// Returns whether `stop_propagation` or `stop_immediate_propagation` was called.
    pub fn is_stopped(&self) -> bool {
        match self {
            DomEvent::Click {
                stopped,
                stopped_immediate,
                ..
            } => stopped.get() || stopped_immediate.get(),
            DomEvent::Input {
                stopped,
                stopped_immediate,
                ..
            } => stopped.get() || stopped_immediate.get(),
            DomEvent::KeyDown {
                stopped,
                stopped_immediate,
                ..
            } => stopped.get() || stopped_immediate.get(),
        }
    }

    /// Returns whether `stop_immediate_propagation` was called.
    pub fn is_immediate_stopped(&self) -> bool {
        match self {
            DomEvent::Click {
                stopped_immediate, ..
            } => stopped_immediate.get(),
            DomEvent::Input {
                stopped_immediate, ..
            } => stopped_immediate.get(),
            DomEvent::KeyDown {
                stopped_immediate, ..
            } => stopped_immediate.get(),
        }
    }

    /// Returns whether `prevent_default` was called.
    pub fn is_default_prevented(&self) -> bool {
        match self {
            DomEvent::Click {
                default_prevented, ..
            } => default_prevented.get(),
            DomEvent::Input {
                default_prevented, ..
            } => default_prevented.get(),
            DomEvent::KeyDown {
                default_prevented, ..
            } => default_prevented.get(),
        }
    }

    /// Returns the target node of the event, if any.
    pub fn target(&self) -> Option<NodeId> {
        match self {
            DomEvent::Click { node, .. } => Some(*node),
            DomEvent::Input { node, .. } => Some(*node),
            DomEvent::KeyDown { .. } => None,
        }
    }

    /// Returns the current target node of the event during propagation.
    pub fn current_target(&self) -> Option<NodeId> {
        match self {
            DomEvent::Click { current_target, .. } => current_target.get(),
            DomEvent::Input { current_target, .. } => current_target.get(),
            DomEvent::KeyDown { current_target, .. } => current_target.get(),
        }
    }

    /// Returns the current phase of the event during propagation.
    pub fn event_phase(&self) -> u16 {
        match self {
            DomEvent::Click { event_phase, .. } => event_phase.get(),
            DomEvent::Input { event_phase, .. } => event_phase.get(),
            DomEvent::KeyDown { event_phase, .. } => event_phase.get(),
        }
    }

    /// Returns whether the event bubbles.
    pub fn bubbles(&self) -> bool {
        true
    }

    fn set_event_phase(&self, phase: u16) {
        match self {
            DomEvent::Click { event_phase, .. } => event_phase.set(phase),
            DomEvent::Input { event_phase, .. } => event_phase.set(phase),
            DomEvent::KeyDown { event_phase, .. } => event_phase.set(phase),
        }
    }

    fn set_current_target(&self, node: Option<NodeId>) {
        match self {
            DomEvent::Click { current_target, .. } => current_target.set(node),
            DomEvent::Input { current_target, .. } => current_target.set(node),
            DomEvent::KeyDown { current_target, .. } => current_target.set(node),
        }
    }
}

/// Type for event listeners.
pub type EventListener = Box<dyn FnMut(&DomEvent)>;

/// Registry mapping NodeId to event listeners.
pub struct EventRegistry {
    /// Mapping from NodeId to a list of (use_capture, listener).
    listeners: HashMap<NodeId, Vec<(bool, EventListener)>>,
}

impl Default for EventRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EventRegistry {
    /// Creates a new empty EventRegistry.
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }
    }

    /// Registers an event listener on a node.
    pub fn add_listener(
        &mut self,
        node: NodeId,
        use_capture: bool,
        listener: Box<dyn FnMut(&DomEvent)>,
    ) {
        self.listeners
            .entry(node)
            .or_default()
            .push((use_capture, listener));
    }

    /// Dispatches an event according to the DOM event model (capture -> target -> bubble).
    /// Returns true if the event was not canceled (preventDefault not called).
    pub fn dispatch(&mut self, dom: &Dom, event: &DomEvent) -> bool {
        let target = match event.target() {
            Some(t) => t,
            None => dom.document(),
        };
        self.dispatch_to(dom, event, target)
    }

    /// Dispatches an event with an explicit target.
    pub fn dispatch_to(&mut self, dom: &Dom, event: &DomEvent, target: NodeId) -> bool {
        // Build ancestor chain (excluding target)
        let mut ancestors = Vec::new();
        let mut curr = dom.parent(target);
        while let Some(p) = curr {
            ancestors.push(p);
            curr = dom.parent(p);
        }

        // spec: capture phase (from root down to parent)
        if !ancestors.is_empty() {
            event.set_event_phase(CAPTURING_PHASE);
            for &node in ancestors.iter().rev() {
                if event.is_stopped() {
                    break;
                }
                event.set_current_target(Some(node));
                self.trigger(node, event, true);
            }
        }

        // spec: target phase (at the target)
        if !event.is_stopped() {
            event.set_event_phase(AT_TARGET);
            event.set_current_target(Some(target));
            self.trigger(target, event, true);
            self.trigger(target, event, false);
        }

        // spec: bubble phase (from parent of target up to root)
        if event.bubbles() && !ancestors.is_empty() {
            event.set_event_phase(BUBBLING_PHASE);
            for &node in ancestors.iter() {
                if event.is_stopped() {
                    break;
                }
                event.set_current_target(Some(node));
                self.trigger(node, event, false);
            }
        }

        // Reset phase and current target
        event.set_event_phase(NONE);
        event.set_current_target(None);

        !event.is_default_prevented()
    }

    /// Helper to trigger listeners on a specific node for a specific phase.
    fn trigger(&mut self, node: NodeId, event: &DomEvent, capture: bool) {
        if let Some(node_listeners) = self.listeners.get_mut(&node) {
            for (is_capture, listener) in node_listeners.iter_mut() {
                if event.is_immediate_stopped() {
                    break;
                }
                if *is_capture == capture {
                    listener(event);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::NodeData;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_dispatch_bubble_capture() {
        let mut dom = Dom::new();
        let root = dom.document();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        let child = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![],
        });
        dom.append_child(root, parent);
        dom.append_child(parent, child);

        let mut registry = EventRegistry::new();
        let log = Rc::new(RefCell::new(Vec::new()));

        // Add listeners in various phases
        {
            let log = log.clone();
            registry.add_listener(
                root,
                true,
                Box::new(move |_| log.borrow_mut().push("root capture")),
            );
        }
        {
            let log = log.clone();
            registry.add_listener(
                root,
                false,
                Box::new(move |_| log.borrow_mut().push("root bubble")),
            );
        }
        {
            let log = log.clone();
            registry.add_listener(
                parent,
                true,
                Box::new(move |_| log.borrow_mut().push("parent capture")),
            );
        }
        {
            let log = log.clone();
            registry.add_listener(
                parent,
                false,
                Box::new(move |_| log.borrow_mut().push("parent bubble")),
            );
        }
        {
            let log = log.clone();
            registry.add_listener(
                child,
                false,
                Box::new(move |_| log.borrow_mut().push("child bubble")),
            );
        }
        {
            let log = log.clone();
            registry.add_listener(
                child,
                true,
                Box::new(move |_| log.borrow_mut().push("child capture")),
            );
        }

        let event = DomEvent::click(child);
        registry.dispatch(&dom, &event);

        assert_eq!(
            *log.borrow(),
            vec![
                "root capture",
                "parent capture",
                "child capture",
                "child bubble",
                "parent bubble",
                "root bubble"
            ]
        );
    }

    #[test]
    fn test_stop_propagation() {
        let mut dom = Dom::new();
        let root = dom.document();
        let child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(root, child);

        let mut registry = EventRegistry::new();
        let log = Rc::new(RefCell::new(Vec::new()));

        {
            let log = log.clone();
            registry.add_listener(
                child,
                false,
                Box::new(move |e| {
                    log.borrow_mut().push("child");
                    e.stop_propagation();
                }),
            );
        }
        {
            let log = log.clone();
            registry.add_listener(
                root,
                false,
                Box::new(move |_| log.borrow_mut().push("root")),
            );
        }

        let event = DomEvent::click(child);
        registry.dispatch(&dom, &event);

        assert_eq!(*log.borrow(), vec!["child"]);
    }

    #[test]
    fn test_prevent_default() {
        let dom = Dom::new();
        let root = dom.document();
        let mut registry = EventRegistry::new();

        registry.add_listener(root, false, Box::new(move |e| e.prevent_default()));

        let event = DomEvent::click(root);
        let result = registry.dispatch(&dom, &event);

        assert!(!result);
        assert!(event.is_default_prevented());
    }

    #[test]
    fn test_stop_immediate_propagation() {
        let mut dom = Dom::new();
        let root = dom.document();
        let child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(root, child);

        let mut registry = EventRegistry::new();
        let log = Rc::new(RefCell::new(Vec::new()));

        {
            let log = log.clone();
            registry.add_listener(
                child,
                false,
                Box::new(move |e| {
                    log.borrow_mut().push("child first");
                    e.stop_immediate_propagation();
                }),
            );
        }
        {
            let log = log.clone();
            registry.add_listener(
                child,
                false,
                Box::new(move |_| {
                    log.borrow_mut().push("child second");
                }),
            );
        }
        {
            let log = log.clone();
            registry.add_listener(
                root,
                false,
                Box::new(move |_| {
                    log.borrow_mut().push("root");
                }),
            );
        }

        let event = DomEvent::click(child);
        registry.dispatch(&dom, &event);

        assert_eq!(*log.borrow(), vec!["child first"]);
    }

    #[test]
    fn test_event_phase_and_current_target() {
        let mut dom = Dom::new();
        let root = dom.document();
        let child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(root, child);

        let mut registry = EventRegistry::new();
        let log = Rc::new(RefCell::new(Vec::new()));

        {
            let log = log.clone();
            registry.add_listener(
                root,
                true,
                Box::new(move |e| {
                    log.borrow_mut().push((e.event_phase(), e.current_target()));
                }),
            );
        }
        {
            let log = log.clone();
            registry.add_listener(
                child,
                false,
                Box::new(move |e| {
                    log.borrow_mut().push((e.event_phase(), e.current_target()));
                }),
            );
        }
        {
            let log = log.clone();
            registry.add_listener(
                root,
                false,
                Box::new(move |e| {
                    log.borrow_mut().push((e.event_phase(), e.current_target()));
                }),
            );
        }

        let event = DomEvent::click(child);
        assert_eq!(event.event_phase(), NONE);
        assert_eq!(event.current_target(), None);

        registry.dispatch(&dom, &event);

        assert_eq!(
            *log.borrow(),
            vec![
                (CAPTURING_PHASE, Some(root)),
                (AT_TARGET, Some(child)),
                (BUBBLING_PHASE, Some(root))
            ]
        );

        assert_eq!(event.event_phase(), NONE);
        assert_eq!(event.current_target(), None);
    }
}
