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

    /// Returns true if the node is disabled according to HTML specification rules.
    /// An element is disabled if it is a disabled-capable element (button, input, select, textarea,
    /// optgroup, option, fieldset) and has the `disabled` attribute,
    /// or if it is a descendant of a disabled `<fieldset>` (with certain exceptions for the first `<legend>` child).
    pub fn is_disabled(&self, node: NodeId) -> bool {
        let data = match self.data(node) {
            Some(data) => data,
            None => return false,
        };
        let name = match data {
            crate::dom::NodeData::Element { name, .. } => name.as_str(),
            _ => return false,
        };

        // 1. Check if the element itself is disabled-capable and has the "disabled" attribute.
        let is_capable = matches!(
            name,
            "button" | "input" | "select" | "textarea" | "optgroup" | "option" | "fieldset"
        );
        if is_capable && self.get_attribute(node, "disabled").is_some() {
            return true;
        }

        // 2. Check if the element is a descendant of a disabled fieldset.
        let mut current = node;
        while let Some(parent) = self.parent(current) {
            if let Some(parent_data) = self.data(parent)
                && let crate::dom::NodeData::Element {
                    name: parent_name, ..
                } = parent_data
                && parent_name == "fieldset"
                && self.get_attribute(parent, "disabled").is_some()
            {
                // It is a descendant of a disabled fieldset.
                // Is it inside the fieldset's first legend?
                if self.is_inside_first_legend_of_fieldset(parent, node) {
                    return false;
                }
                return true;
            }
            current = parent;
        }

        false
    }

    fn is_inside_first_legend_of_fieldset(&self, fieldset: NodeId, node: NodeId) -> bool {
        // Find the first legend child of the fieldset
        let first_legend = self.children(fieldset).iter().find(|&&child| {
            if let Some(crate::dom::NodeData::Element { name, .. }) = self.data(child) {
                name == "legend"
            } else {
                false
            }
        });

        if let Some(&legend) = first_legend {
            if node == legend {
                return true;
            }
            let mut curr = node;
            while let Some(p) = self.parent(curr) {
                if p == legend {
                    return true;
                }
                if p == fieldset {
                    break;
                }
                curr = p;
            }
        }
        false
    }

    /// Returns the document element of the DOM, if it exists and is connected.
    pub fn document_element(&self) -> Option<NodeId> {
        self.descendants_iter(self.document).find(|&node| {
            if let Some(crate::dom::NodeData::Element { name, .. }) = self.data(node) {
                name == "html"
            } else {
                false
            }
        })
    }

    /// Returns the body element of the DOM, if it exists and is connected.
    pub fn body_element(&self) -> Option<NodeId> {
        self.descendants_iter(self.document).find(|&node| {
            if let Some(crate::dom::NodeData::Element { name, .. }) = self.data(node) {
                name == "body" || name == "frameset"
            } else {
                false
            }
        })
    }

    /// Returns the NodeId of the active element.
    /// According to the HTML specification, it is the focused node if it is connected.
    /// Otherwise, it falls back to the `body` element (or `frameset`), or the `html` element (document element).
    /// If none of those exist or are connected, it returns `None`.
    pub fn active_element(&self) -> Option<NodeId> {
        if let Some(focused) = self.focused_node() {
            return Some(focused);
        }

        if let Some(body) = self.body_element()
            && self.is_connected(body)
        {
            return Some(body);
        }

        if let Some(doc_elem) = self.document_element()
            && self.is_connected(doc_elem)
        {
            return Some(doc_elem);
        }

        None
    }

    /// Returns true if the document or any element within it currently has focus.
    pub fn has_focus(&self) -> bool {
        self.focused_node().is_some()
    }

    /// Returns true if the node is focusable.
    /// According to the HTML specification, an element is focusable if it is connected to the document,
    /// is not disabled, is not hidden, and is either a default focusable element (like `input`, `button`,
    /// `select`, `textarea`, or `a` with `href`), or has a valid `tabindex` attribute.
    pub fn is_focusable(&self, node: NodeId) -> bool {
        // Must be connected
        if !self.is_connected(node) {
            return false;
        }

        let data = match self.data(node) {
            Some(data) => data,
            None => return false,
        };

        // If it's not an element, it's not focusable (e.g. text nodes, document)
        if !matches!(data, crate::dom::NodeData::Element { .. }) {
            return false;
        }

        // Must not be hidden
        if self.hidden(node) {
            return false;
        }

        // Must not be disabled
        if self.is_disabled(node) {
            return false;
        }

        // If a valid tabindex is explicitly specified, it is focusable.
        if self.tabindex(node).is_some() {
            return true;
        }

        // Check contenteditable attribute
        if let Some(val) = self.get_attribute(node, "contenteditable")
            && !val.trim().eq_ignore_ascii_case("false")
        {
            return true;
        }

        // Check default focusable elements
        if let crate::dom::NodeData::Element { name, attrs } = data {
            match name.as_str() {
                "input" | "button" | "select" | "textarea" | "iframe" => true,
                "a" | "link" => {
                    // must have href attribute
                    attrs.iter().any(|(k, _)| k == "href")
                }
                "area" => attrs.iter().any(|(k, _)| k == "href"),
                "summary" => {
                    if let Some(parent) = self.parent(node)
                        && let Some(crate::dom::NodeData::Element {
                            name: parent_name, ..
                        }) = self.data(parent)
                    {
                        parent_name == "details"
                    } else {
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Returns the focusable nodes sorted in sequential focus navigation order.
    ///
    /// According to the HTML specification:
    /// 1. Elements with `tabindex` > 0 are sorted first, in increasing order of their `tabindex`.
    ///    Elements with the same positive `tabindex` are sorted in document order.
    /// 2. Elements with `tabindex` == 0 and elements that are focusable by default (which behave
    ///    as if they have `tabindex` 0) are sorted next, in document order.
    /// 3. Elements with negative `tabindex` are focusable but are excluded from sequential navigation.
    pub fn sequential_focus_navigation_order(&self) -> Vec<NodeId> {
        let all_focusable = std::iter::once(self.document())
            .chain(self.descendants_iter(self.document()))
            .filter(|&node| self.is_focusable(node));

        let mut pos_tabindex: Vec<(i32, usize, NodeId)> = Vec::new();
        let mut zero_tabindex: Vec<NodeId> = Vec::new();

        for (idx, node) in all_focusable.enumerate() {
            let tab_val = self.tabindex(node);
            match tab_val {
                Some(val) => {
                    if val > 0 {
                        pos_tabindex.push((val, idx, node));
                    } else if val == 0 {
                        zero_tabindex.push(node);
                    }
                    // if val < 0, it's omitted from sequential navigation order
                }
                None => {
                    // Focusable by default (effectively tabindex = 0)
                    zero_tabindex.push(node);
                }
            }
        }

        // Sort positive tabindex elements by tabindex value, then by document order index (idx)
        pos_tabindex.sort_by(|a, b| match a.0.cmp(&b.0) {
            std::cmp::Ordering::Equal => a.1.cmp(&b.1),
            other => other,
        });

        let mut order = Vec::with_capacity(pos_tabindex.len() + zero_tabindex.len());
        for (_, _, node) in pos_tabindex {
            order.push(node);
        }
        order.extend(zero_tabindex);
        order
    }

    /// Returns the next node in the sequential focus navigation order after `current`.
    /// If `current` is `None` or not in the list, returns the first focusable node.
    /// Wraps around to the first node if `current` is the last focusable node.
    pub fn next_focusable_node(&self, current: Option<NodeId>) -> Option<NodeId> {
        let order = self.sequential_focus_navigation_order();
        if order.is_empty() {
            return None;
        }
        let current_node = match current {
            Some(node) => node,
            None => return Some(order[0]),
        };
        if let Some(pos) = order.iter().position(|&node| node == current_node) {
            let next_pos = (pos + 1) % order.len();
            Some(order[next_pos])
        } else {
            Some(order[0])
        }
    }

    /// Returns the previous node in the sequential focus navigation order before `current`.
    /// If `current` is `None` or not in the list, returns the last focusable node.
    /// Wraps around to the last node if `current` is the first focusable node.
    pub fn prev_focusable_node(&self, current: Option<NodeId>) -> Option<NodeId> {
        let order = self.sequential_focus_navigation_order();
        if order.is_empty() {
            return None;
        }
        let current_node = match current {
            Some(node) => node,
            None => return order.last().copied(),
        };
        if let Some(pos) = order.iter().position(|&node| node == current_node) {
            let prev_pos = if pos == 0 { order.len() - 1 } else { pos - 1 };
            Some(order[prev_pos])
        } else {
            order.last().copied()
        }
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
        // Verify if node is focusable.
        if !self.is_focusable(node) {
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
        let target = self.focused_node.get().unwrap_or(self.document);
        registry.dispatch_to(self, event, target)
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

    #[test]
    fn test_document_purity_during_dispatch() {
        let mut dom = Dom::new();
        let doc_id = dom.document();
        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, input_id);
        dom.focus(input_id);

        let mut registry = EventRegistry::new();
        let purity_verified = Rc::new(RefCell::new(false));
        let purity_verified_clone = purity_verified.clone();

        let doc_id_outside = dom.document();

        registry.add_listener(
            input_id,
            false,
            Box::new(move |_event| {
                *purity_verified_clone.borrow_mut() = true;
            }),
        );

        let event = DomEvent::key_down("a".to_string());
        dom.dispatch_keyboard_event(&mut registry, &event);

        assert!(*purity_verified.borrow());
        assert_eq!(dom.document(), doc_id_outside);
        assert_eq!(dom.document(), doc_id);
    }

    #[test]
    fn test_is_focusable() {
        let mut dom = Dom::new();
        let doc_id = dom.document();

        // 1. Unconnected input is not focusable
        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        assert!(!dom.is_focusable(input_id));

        // Connected input is focusable
        dom.append_child(doc_id, input_id);
        assert!(dom.is_focusable(input_id));

        // 2. Disabled input is not focusable
        let disabled_input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("disabled".to_string(), "".to_string())],
        });
        dom.append_child(doc_id, disabled_input_id);
        assert!(!dom.is_focusable(disabled_input_id));

        // 3. Hidden input is not focusable
        let hidden_input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("hidden".to_string(), "".to_string())],
        });
        dom.append_child(doc_id, hidden_input_id);
        assert!(!dom.is_focusable(hidden_input_id));

        // 4. Anchor without href is not focusable
        let anchor_no_href = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, anchor_no_href);
        assert!(!dom.is_focusable(anchor_no_href));

        // Anchor with href is focusable
        let anchor_with_href = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![("href".to_string(), "#foo".to_string())],
        });
        dom.append_child(doc_id, anchor_with_href);
        assert!(dom.is_focusable(anchor_with_href));

        // 5. Div without tabindex is not focusable
        let div_no_tabindex = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, div_no_tabindex);
        assert!(!dom.is_focusable(div_no_tabindex));

        // Div with tabindex is focusable
        let div_with_tabindex = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("tabindex".to_string(), "0".to_string())],
        });
        dom.append_child(doc_id, div_with_tabindex);
        assert!(dom.is_focusable(div_with_tabindex));

        // Div with negative tabindex is focusable
        let div_with_neg_tabindex = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("tabindex".to_string(), "-1".to_string())],
        });
        dom.append_child(doc_id, div_with_neg_tabindex);
        assert!(dom.is_focusable(div_with_neg_tabindex));

        // 6. Contenteditable elements are focusable
        let div_contenteditable = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("contenteditable".to_string(), "true".to_string())],
        });
        dom.append_child(doc_id, div_contenteditable);
        assert!(dom.is_focusable(div_contenteditable));

        // Contenteditable="false" is not focusable if no tabindex
        let div_contenteditable_false = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("contenteditable".to_string(), "false".to_string())],
        });
        dom.append_child(doc_id, div_contenteditable_false);
        assert!(!dom.is_focusable(div_contenteditable_false));
    }

    #[test]
    fn test_focus_ignored_on_non_focusable() {
        let mut dom = Dom::new();
        let doc_id = dom.document();

        let div_no_tabindex = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, div_no_tabindex);

        // Attempting to focus a non-focusable div should be ignored
        dom.focus(div_no_tabindex);
        assert_eq!(dom.focused_node(), None);
        assert!(!get_node_state(div_no_tabindex).focus);
    }

    #[test]
    fn test_sequential_focus_navigation_order() {
        let mut dom = Dom::new();
        let doc_id = dom.document();

        // Let's create nodes in document order:
        // 1. input1 (default focusable, tabindex = none, i.e. 0)
        let input1 = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("id".to_string(), "input1".to_string())],
        });
        dom.append_child(doc_id, input1);

        // 2. div_t2 (tabindex = 2)
        let div_t2 = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("id".to_string(), "div_t2".to_string()),
                ("tabindex".to_string(), "2".to_string()),
            ],
        });
        dom.append_child(doc_id, div_t2);

        // 3. div_t1 (tabindex = 1)
        let div_t1 = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("id".to_string(), "div_t1".to_string()),
                ("tabindex".to_string(), "1".to_string()),
            ],
        });
        dom.append_child(doc_id, div_t1);

        // 4. div_t2_second (tabindex = 2)
        let div_t2_second = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("id".to_string(), "div_t2_second".to_string()),
                ("tabindex".to_string(), "2".to_string()),
            ],
        });
        dom.append_child(doc_id, div_t2_second);

        // 5. div_neg (tabindex = -1) -> focusable but excluded from sequential order
        let div_neg = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("id".to_string(), "div_neg".to_string()),
                ("tabindex".to_string(), "-1".to_string()),
            ],
        });
        dom.append_child(doc_id, div_neg);

        // Expected sequential order:
        // Category 1: positive tabindex -> [div_t1, div_t2, div_t2_second] (div_t1 is tabindex=1; div_t2 and div_t2_second are tabindex=2 sorted by document order)
        // Category 2: tabindex 0 or default -> [input1]
        // div_neg is excluded.
        // Total expected: [div_t1, div_t2, div_t2_second, input1]

        let order = dom.sequential_focus_navigation_order();
        assert_eq!(order, vec![div_t1, div_t2, div_t2_second, input1]);
    }

    #[test]
    fn test_next_prev_focusable_node() {
        let mut dom = Dom::new();
        let doc_id = dom.document();

        let btn1 = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, btn1);

        let btn2 = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, btn2);

        let btn3 = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, btn3);

        // Sequential order: [btn1, btn2, btn3]

        // 1. Next navigation
        assert_eq!(dom.next_focusable_node(None), Some(btn1));
        assert_eq!(dom.next_focusable_node(Some(btn1)), Some(btn2));
        assert_eq!(dom.next_focusable_node(Some(btn2)), Some(btn3));
        assert_eq!(dom.next_focusable_node(Some(btn3)), Some(btn1)); // wrap around

        // 2. Prev navigation
        assert_eq!(dom.prev_focusable_node(None), Some(btn3));
        assert_eq!(dom.prev_focusable_node(Some(btn3)), Some(btn2));
        assert_eq!(dom.prev_focusable_node(Some(btn2)), Some(btn1));
        assert_eq!(dom.prev_focusable_node(Some(btn1)), Some(btn3)); // wrap around
    }

    #[test]
    fn test_is_disabled_and_fieldset_rules() {
        let mut dom = Dom::new();
        let doc_id = dom.document();

        // 1. Anchor with href and disabled attribute should STILL be focusable
        // because <a> is not a disabled-capable element.
        let a_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![
                ("href".to_string(), "https://google.com".to_string()),
                ("disabled".to_string(), "".to_string()),
            ],
        });
        dom.append_child(doc_id, a_id);
        assert!(!dom.is_disabled(a_id));
        assert!(dom.is_focusable(a_id));

        // 2. Button with disabled attribute is NOT focusable
        let btn_id = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![("disabled".to_string(), "".to_string())],
        });
        dom.append_child(doc_id, btn_id);
        assert!(dom.is_disabled(btn_id));
        assert!(!dom.is_focusable(btn_id));

        // 3. Fieldset disabled hierarchy rules
        let fieldset_id = dom.create_node(NodeData::Element {
            name: "fieldset".to_string(),
            attrs: vec![("disabled".to_string(), "".to_string())],
        });
        dom.append_child(doc_id, fieldset_id);

        // a. First legend child of the fieldset
        let first_legend_id = dom.create_node(NodeData::Element {
            name: "legend".to_string(),
            attrs: vec![],
        });
        dom.append_child(fieldset_id, first_legend_id);

        let input_inside_first_legend_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        dom.append_child(first_legend_id, input_inside_first_legend_id);

        // This input is inside the first legend of a disabled fieldset, so it is NOT disabled!
        assert!(!dom.is_disabled(input_inside_first_legend_id));
        assert!(dom.is_focusable(input_inside_first_legend_id));

        // b. Second legend child of the fieldset
        let second_legend_id = dom.create_node(NodeData::Element {
            name: "legend".to_string(),
            attrs: vec![],
        });
        dom.append_child(fieldset_id, second_legend_id);

        let input_inside_second_legend_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        dom.append_child(second_legend_id, input_inside_second_legend_id);

        // This input is inside a second legend of a disabled fieldset, so it IS disabled!
        assert!(dom.is_disabled(input_inside_second_legend_id));
        assert!(!dom.is_focusable(input_inside_second_legend_id));

        // c. Plain input descendant of the fieldset (not in a legend)
        let plain_fieldset_input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        dom.append_child(fieldset_id, plain_fieldset_input_id);

        assert!(dom.is_disabled(plain_fieldset_input_id));
        assert!(!dom.is_focusable(plain_fieldset_input_id));
    }

    #[test]
    fn test_active_element_and_has_focus() {
        let mut dom = Dom::new();
        let doc_id = dom.document();

        // 1. Initial document with nothing connected -> active_element() is None
        assert_eq!(dom.active_element(), None);
        assert!(!dom.has_focus());

        // 2. Add html (document element)
        let html_id = dom.create_node(NodeData::Element {
            name: "html".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, html_id);

        // Active element should fall back to html_id because body is absent
        assert_eq!(dom.active_element(), Some(html_id));
        assert!(!dom.has_focus()); // has_focus is still false because no specific element is focused

        // 3. Add body element
        let body_id = dom.create_node(NodeData::Element {
            name: "body".to_string(),
            attrs: vec![],
        });
        dom.append_child(html_id, body_id);

        // Active element should fall back to body_id because it exists
        assert_eq!(dom.active_element(), Some(body_id));
        assert!(!dom.has_focus());

        // 4. Add focusable element and focus it
        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        dom.append_child(body_id, input_id);

        dom.focus(input_id);
        assert_eq!(dom.active_element(), Some(input_id));
        assert!(dom.has_focus());

        // 5. Blur the element -> reverts to body_id
        dom.blur();
        assert_eq!(dom.active_element(), Some(body_id));
        assert!(!dom.has_focus());
    }

    #[test]
    fn test_area_and_summary_focusable_defaults() {
        let mut dom = Dom::new();
        let doc_id = dom.document();

        // 1. Area tag without href is not focusable
        let area_no_href = dom.create_node(NodeData::Element {
            name: "area".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, area_no_href);
        assert!(!dom.is_focusable(area_no_href));

        // Area tag with href is focusable by default
        let area_with_href = dom.create_node(NodeData::Element {
            name: "area".to_string(),
            attrs: vec![("href".to_string(), "#test".to_string())],
        });
        dom.append_child(doc_id, area_with_href);
        assert!(dom.is_focusable(area_with_href));

        // 2. Summary tag outside details is not focusable by default
        let summary_outside = dom.create_node(NodeData::Element {
            name: "summary".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, summary_outside);
        assert!(!dom.is_focusable(summary_outside));

        // Details tag containing summary
        let details_id = dom.create_node(NodeData::Element {
            name: "details".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, details_id);

        let summary_inside = dom.create_node(NodeData::Element {
            name: "summary".to_string(),
            attrs: vec![],
        });
        dom.append_child(details_id, summary_inside);
        assert!(dom.is_focusable(summary_inside));
    }
}
