//! Focus management and keyboard event routing to the focused node.

use super::Dom;
use crate::infra::NodeId;
use crate::selector::{get_node_state, set_node_state};

impl Dom {
    /// Returns true if the node is valid and connected to the Document root.
    pub fn is_connected(&self, node: NodeId) -> bool {
        if self.arena.get(node).is_none() {
            return false;
        }
        let mut curr = node;
        while let Some(parent) = self.parent(curr) {
            curr = parent;
        }
        curr == self.document
    }

    /// Returns the NodeId of the currently focused node, if any.
    /// Ensures that the node is valid and connected to the Document root.
    pub fn focused_node(&self) -> Option<NodeId> {
        let focused = self.focused_node.get();
        if let Some(node) = focused
            && self.is_connected(node)
        {
            return Some(node);
        }
        None
    }

    /// Focuses the given node.
    /// Updates the focus state of both the old (blurred) and new (focused) nodes in NodeState.
    pub fn focus(&self, node: NodeId) {
        // Verify if node exists and is connected to the Document root.
        if !self.is_connected(node) {
            return;
        }

        let prev = self.focused_node.get();
        if prev == Some(node) {
            return; // Already focused
        }

        // Blur previous node if existed and is connected
        if let Some(prev_node) = prev
            && self.is_connected(prev_node)
        {
            let mut state = get_node_state(prev_node);
            state.focus = false;
            set_node_state(prev_node, state);
        }

        // Focus new node
        let mut state = get_node_state(node);
        state.focus = true;
        set_node_state(node, state);

        self.focused_node.set(Some(node));
    }

    /// Blurs the currently focused node.
    pub fn blur(&self) {
        if let Some(prev_node) = self.focused_node.get()
            && self.is_connected(prev_node)
        {
            let mut state = get_node_state(prev_node);
            state.focus = false;
            set_node_state(prev_node, state);
        }
        self.focused_node.set(None);
    }

    /// Blurs the given node if it is currently focused.
    pub fn blur_node(&self, node: NodeId) {
        if self.focused_node.get() == Some(node) {
            self.blur();
        }
    }

    /// Dispatches a keyboard event to the focused node.
    /// If no node is focused, routes it to the Document root.
    pub fn dispatch_keyboard_event(
        &self,
        registry: &mut crate::event::EventRegistry,
        event: &crate::event::DomEvent,
    ) -> bool {
        self.routing_keyboard_event.set(true);
        let res = registry.dispatch(self, event);
        self.routing_keyboard_event.set(false);
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{Dom, NodeData};
    use crate::event::{DomEvent, EventRegistry};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_focus_and_blur() {
        let mut dom = Dom::new();
        let doc_id = dom.document();

        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, input_id);

        let button_id = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, button_id);

        // Initially no focus
        assert_eq!(dom.focused_node(), None);
        assert!(!get_node_state(input_id).focus);
        assert!(!get_node_state(button_id).focus);

        // Focus the input
        dom.focus(input_id);
        assert_eq!(dom.focused_node(), Some(input_id));
        assert!(get_node_state(input_id).focus);
        assert!(!get_node_state(button_id).focus);

        // Focus the button (input should blur)
        dom.focus(button_id);
        assert_eq!(dom.focused_node(), Some(button_id));
        assert!(!get_node_state(input_id).focus);
        assert!(get_node_state(button_id).focus);

        // Blur the button
        dom.blur();
        assert_eq!(dom.focused_node(), None);
        assert!(!get_node_state(input_id).focus);
        assert!(!get_node_state(button_id).focus);
    }

    #[test]
    fn test_disconnected_node_focus() {
        let mut dom = Dom::new();
        let doc_id = dom.document();

        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });

        // Try to focus disconnected node - should do nothing
        dom.focus(input_id);
        assert_eq!(dom.focused_node(), None);
        assert!(!get_node_state(input_id).focus);

        // Append to doc, now focusable
        dom.append_child(doc_id, input_id);
        dom.focus(input_id);
        assert_eq!(dom.focused_node(), Some(input_id));
        assert!(get_node_state(input_id).focus);

        // Remove node from DOM, should no longer be focused
        dom.remove_child(doc_id, input_id);
        assert_eq!(dom.focused_node(), None);
    }

    #[test]
    fn test_keyboard_event_routing() {
        let mut dom = Dom::new();
        let doc_id = dom.document();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, div_id);

        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        dom.append_child(div_id, input_id);

        let mut registry = EventRegistry::new();

        let div_received = Rc::new(RefCell::new(Vec::new()));
        let div_received_clone = div_received.clone();
        registry.add_listener(
            div_id,
            false,
            Box::new(move |event| {
                if let DomEvent::KeyDown { key, .. } = event {
                    div_received_clone.borrow_mut().push(key.clone());
                }
            }),
        );

        let input_received = Rc::new(RefCell::new(Vec::new()));
        let input_received_clone = input_received.clone();
        registry.add_listener(
            input_id,
            false,
            Box::new(move |event| {
                if let DomEvent::KeyDown { key, .. } = event {
                    input_received_clone.borrow_mut().push(key.clone());
                }
            }),
        );

        // Route a KeyDown event with no focus (should route to document and bubble, but not hit div or input since they are descendants)
        let event = DomEvent::key_down("Enter".to_string());
        dom.dispatch_keyboard_event(&mut registry, &event);

        assert!(div_received.borrow().is_empty());
        assert!(input_received.borrow().is_empty());

        // Focus the input and dispatch KeyDown
        dom.focus(input_id);
        let event2 = DomEvent::key_down("Escape".to_string());
        dom.dispatch_keyboard_event(&mut registry, &event2);

        // Should hit input (target) and bubble to div (ancestor)
        assert_eq!(input_received.borrow().as_slice(), &["Escape".to_string()]);
        assert_eq!(div_received.borrow().as_slice(), &["Escape".to_string()]);
    }
}
