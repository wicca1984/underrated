use crate::dom::Dom;
use crate::infra::NodeId;
use std::cell::Cell;
use std::collections::HashMap;

/// Phases of event propagation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    Capture,
    Target,
    Bubble,
}

/// DOM events as specified in SPEC S-25.
pub enum DomEvent {
    Click {
        node: NodeId,
        stopped: Cell<bool>,
        default_prevented: Cell<bool>,
    },
    Input {
        node: NodeId,
        value: String,
        stopped: Cell<bool>,
        default_prevented: Cell<bool>,
    },
    KeyDown {
        key: String,
        stopped: Cell<bool>,
        default_prevented: Cell<bool>,
    },
}

impl DomEvent {
    /// Creates a new Click event.
    pub fn click(node: NodeId) -> Self {
        Self::Click {
            node,
            stopped: Cell::new(false),
            default_prevented: Cell::new(false),
        }
    }

    /// Creates a new Input event.
    pub fn input(node: NodeId, value: String) -> Self {
        Self::Input {
            node,
            value,
            stopped: Cell::new(false),
            default_prevented: Cell::new(false),
        }
    }

    /// Creates a new KeyDown event.
    pub fn key_down(key: String) -> Self {
        Self::KeyDown {
            key,
            stopped: Cell::new(false),
            default_prevented: Cell::new(false),
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

    /// Returns whether `stop_propagation` was called.
    pub fn is_stopped(&self) -> bool {
        match self {
            DomEvent::Click { stopped, .. } => stopped.get(),
            DomEvent::Input { stopped, .. } => stopped.get(),
            DomEvent::KeyDown { stopped, .. } => stopped.get(),
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

        // Build ancestor chain (excluding target)
        let mut ancestors = Vec::new();
        let mut curr = dom.parent(target);
        while let Some(p) = curr {
            ancestors.push(p);
            curr = dom.parent(p);
        }

        // 1. Capture phase: from root down to parent of target
        for &node in ancestors.iter().rev() {
            if event.is_stopped() {
                return !event.is_default_prevented();
            }
            self.trigger(node, event, true);
        }

        // 2. Target phase: at the target (both capture and non-capture listeners)
        if !event.is_stopped() {
            self.trigger(target, event, true);
            if !event.is_stopped() {
                self.trigger(target, event, false);
            }
        }

        // 3. Bubble phase: from parent of target up to root
        for &node in ancestors.iter() {
            if event.is_stopped() {
                break;
            }
            self.trigger(node, event, false);
        }

        !event.is_default_prevented()
    }

    /// Helper to trigger listeners on a specific node for a specific phase.
    fn trigger(&mut self, node: NodeId, event: &DomEvent, capture: bool) {
        if let Some(node_listeners) = self.listeners.get_mut(&node) {
            for (is_capture, listener) in node_listeners.iter_mut() {
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
}
