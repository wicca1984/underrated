//! Forms: control state and form submission.
//!
//! Collects the successful controls of a `<form>` and produces a
//! [`NavigationRequest`] — the URL (with an `application/x-www-form-urlencoded`
//! query for GET) the browser should navigate to. This is the core of a working
//! search box: text input + submit → `action?q=...`.

use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;
use std::collections::HashMap;

/// HTTP method of a form submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

/// The navigation a form submission resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationRequest {
    pub url: String,
    pub method: Method,
    /// For POST, the urlencoded body (empty for GET, which carries the query in `url`).
    pub body: String,
}

/// Live values of form controls (what the user has typed / toggled), keyed by
/// the control's `NodeId`. Overrides the element's `value`/`checked` attributes.
#[derive(Debug, Default, Clone)]
pub struct FormState {
    values: HashMap<NodeId, String>,
    checked: HashMap<NodeId, bool>,
}

impl FormState {
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the current value of a text-like control.
    pub fn set_value(&mut self, node: NodeId, value: &str) {
        self.values.insert(node, value.to_string());
    }
    /// Sets the checked state of a checkbox/radio.
    pub fn set_checked(&mut self, node: NodeId, checked: bool) {
        self.checked.insert(node, checked);
    }
}

/// `application/x-www-form-urlencoded` byte serialization: alnum and `-_.*`
/// pass through, space becomes `+`, everything else becomes `%XX`.
// spec: https://url.spec.whatwg.org/#urlencoded-serializing
fn form_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'*' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push(hex(b >> 4));
                out.push(hex(b & 0x0f));
            }
        }
    }
    out
}

fn hex(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'A' + (nibble - 10)) as char,
    }
}

fn attr<'a>(dom: &'a Dom, node: NodeId, name: &str) -> Option<&'a str> {
    if let Some(NodeData::Element { attrs, .. }) = dom.data(node) {
        attrs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    } else {
        None
    }
}

fn tag(dom: &Dom, node: NodeId) -> Option<&str> {
    if let Some(NodeData::Element { name, .. }) = dom.data(node) {
        Some(name.as_str())
    } else {
        None
    }
}

/// Submits `form`, producing a [`NavigationRequest`], or `None` if `form` is not
/// an element. Walks the form's descendants for successful controls (those with
/// a `name`), serializing them as `application/x-www-form-urlencoded`.
// spec: https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-submission-algorithm
pub fn submit(dom: &Dom, form: NodeId, values: &FormState) -> Option<NavigationRequest> {
    tag(dom, form)?; // must be an element

    let method = match attr(dom, form, "method") {
        Some(m) if m.eq_ignore_ascii_case("post") => Method::Post,
        _ => Method::Get,
    };
    let action = attr(dom, form, "action").unwrap_or("").to_string();

    let mut pairs: Vec<String> = Vec::new();
    for node in dom.descendants(form) {
        let Some(t) = tag(dom, node) else { continue };
        let t = t.to_ascii_lowercase();
        if !matches!(t.as_str(), "input" | "textarea" | "select") {
            continue;
        }
        let Some(name) = attr(dom, node, "name") else {
            continue;
        };
        let input_type = attr(dom, node, "type").unwrap_or("text").to_ascii_lowercase();

        // Buttons and submit/reset/file controls are not successful here.
        if matches!(input_type.as_str(), "submit" | "reset" | "button" | "file" | "image") {
            continue;
        }

        // Checkbox/radio: only successful when checked.
        if matches!(input_type.as_str(), "checkbox" | "radio") {
            let checked = values
                .checked
                .get(&node)
                .copied()
                .unwrap_or_else(|| attr(dom, node, "checked").is_some());
            if !checked {
                continue;
            }
        }

        let value = values
            .values
            .get(&node)
            .map(|s| s.as_str())
            .or_else(|| attr(dom, node, "value"))
            .unwrap_or("");

        pairs.push(format!("{}={}", form_encode(name), form_encode(value)));
    }

    let query = pairs.join("&");

    match method {
        Method::Get => {
            let url = if query.is_empty() {
                action
            } else if action.contains('?') {
                format!("{action}&{query}")
            } else {
                format!("{action}?{query}")
            };
            Some(NavigationRequest {
                url,
                method,
                body: String::new(),
            })
        }
        Method::Post => Some(NavigationRequest {
            url: action,
            method,
            body: query,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{Dom, NodeData};

    fn el(dom: &mut Dom, name: &str, attrs: &[(&str, &str)]) -> NodeId {
        dom.create_node(NodeData::Element {
            name: name.to_string(),
            attrs: attrs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        })
    }

    #[test]
    fn get_search_builds_query() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/search"), ("method", "get")]);
        let input = el(&mut dom, "input", &[("name", "q")]);
        dom.append_child(form, input);

        let mut state = FormState::new();
        state.set_value(input, "hello rust");

        let req = submit(&dom, form, &state).unwrap();
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.url, "/search?q=hello+rust");
        assert_eq!(req.body, "");
    }

    #[test]
    fn value_attribute_used_when_no_override() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let input = el(&mut dom, "input", &[("name", "x"), ("value", "1 2")]);
        dom.append_child(form, input);
        let req = submit(&dom, form, &FormState::new()).unwrap();
        assert_eq!(req.url, "/a?x=1+2");
    }

    #[test]
    fn unchecked_checkbox_is_skipped() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let cb = el(&mut dom, "input", &[("name", "c"), ("type", "checkbox")]);
        let txt = el(&mut dom, "input", &[("name", "t"), ("value", "v")]);
        dom.append_child(form, cb);
        dom.append_child(form, txt);
        let req = submit(&dom, form, &FormState::new()).unwrap();
        assert_eq!(req.url, "/a?t=v"); // checkbox not checked -> omitted
    }

    #[test]
    fn submit_button_not_serialized() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let btn = el(&mut dom, "input", &[("name", "go"), ("type", "submit"), ("value", "Go")]);
        dom.append_child(form, btn);
        let req = submit(&dom, form, &FormState::new()).unwrap();
        assert_eq!(req.url, "/a"); // empty query
    }

    #[test]
    fn special_chars_are_encoded() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/s")]);
        let input = el(&mut dom, "input", &[("name", "q")]);
        dom.append_child(form, input);
        let mut state = FormState::new();
        state.set_value(input, "a&b=c");
        let req = submit(&dom, form, &state).unwrap();
        assert_eq!(req.url, "/s?q=a%26b%3Dc");
    }

    #[test]
    fn non_element_form_is_none() {
        let mut dom = Dom::new();
        let t = dom.create_node(NodeData::Text("x".into()));
        assert!(submit(&dom, t, &FormState::new()).is_none());
    }
}
