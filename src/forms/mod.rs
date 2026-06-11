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
    /// For POST, the content type of the body (e.g., "application/x-www-form-urlencoded").
    pub content_type: Option<String>,
}

/// Live values of form controls (what the user has typed / toggled), keyed by
/// the control's `NodeId`. Overrides the element's `value`/`checked` attributes.
#[derive(Debug, Default, Clone)]
pub struct FormState {
    values: HashMap<NodeId, String>,
    checked: HashMap<NodeId, bool>,
    selected: HashMap<NodeId, bool>,
    pub submitter: Option<NodeId>,
    pub current_url: Option<String>,
}

impl FormState {
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the submitter button that initiated the submission.
    pub fn set_submitter(&mut self, node: NodeId) {
        self.submitter = Some(node);
    }
    /// Gets the submitter button.
    pub fn submitter(&self) -> Option<NodeId> {
        self.submitter
    }
    /// Sets the current document URL.
    pub fn set_current_url(&mut self, url: &str) {
        self.current_url = Some(url.to_string());
    }
    /// Gets the current document URL.
    pub fn current_url(&self) -> Option<&str> {
        self.current_url.as_deref()
    }
    /// Sets the current value of a text-like control.
    pub fn set_value(&mut self, node: NodeId, value: &str) {
        self.values.insert(node, value.to_string());
    }
    /// Sets the checked state of a checkbox/radio.
    pub fn set_checked(&mut self, node: NodeId, checked: bool) {
        self.checked.insert(node, checked);
    }
    /// Sets the selected state of an option.
    pub fn set_selected(&mut self, node: NodeId, selected: bool) {
        self.selected.insert(node, selected);
    }
    /// Selects a specific option within a `<select>` element and deselects all other options.
    // spec: S-76
    pub fn select_option(&mut self, dom: &Dom, select_node: NodeId, option_node: NodeId) {
        for desc in dom.descendants(select_node) {
            let Some(t) = tag(dom, desc) else {
                continue;
            };
            if t.eq_ignore_ascii_case("option") {
                self.set_selected(desc, desc == option_node);
            }
        }
    }
    /// Checks a radio button and unchecks all other radio buttons with the same name under the given root (e.g. the form).
    // spec: S-76
    pub fn check_radio(&mut self, dom: &Dom, root: NodeId, radio_node: NodeId) {
        let Some(t) = tag(dom, radio_node) else {
            return;
        };
        if !t.eq_ignore_ascii_case("input") {
            return;
        }
        let is_radio = attr(dom, radio_node, "type")
            .map(|typ| typ.eq_ignore_ascii_case("radio"))
            .unwrap_or(false);
        if !is_radio {
            return;
        }

        let radio_name = attr(dom, radio_node, "name");

        if let Some(name) = radio_name {
            for desc in dom.descendants(root) {
                if desc == radio_node {
                    continue;
                }
                let Some(t_other) = tag(dom, desc) else {
                    continue;
                };
                if t_other.eq_ignore_ascii_case("input") {
                    let is_other_radio = attr(dom, desc, "type")
                        .map(|typ| typ.eq_ignore_ascii_case("radio"))
                        .unwrap_or(false);
                    let other_name = attr(dom, desc, "name");
                    if is_other_radio && other_name == Some(name) {
                        self.set_checked(desc, false);
                    }
                }
            }
        }

        self.set_checked(radio_node, true);
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

fn is_radio_effectively_checked(dom: &Dom, form: NodeId, node: NodeId, values: &FormState) -> bool {
    let checked = values
        .checked
        .get(&node)
        .copied()
        .unwrap_or_else(|| attr(dom, node, "checked").is_some());
    if !checked {
        return false;
    }

    let Some(name) = attr(dom, node, "name") else {
        return true;
    };

    let mut found_self = false;
    for desc in dom.descendants(form) {
        if desc == node {
            found_self = true;
            continue;
        }
        if !found_self {
            continue;
        }
        let Some(t) = tag(dom, desc) else {
            continue;
        };
        if t.eq_ignore_ascii_case("input") {
            let is_radio = attr(dom, desc, "type")
                .map(|typ| typ.eq_ignore_ascii_case("radio"))
                .unwrap_or(false);
            if is_radio {
                let other_name = attr(dom, desc, "name");
                if other_name == Some(name) {
                    let other_checked = values
                        .checked
                        .get(&desc)
                        .copied()
                        .unwrap_or_else(|| attr(dom, desc, "checked").is_some());
                    if other_checked {
                        return false;
                    }
                }
            }
        }
    }

    true
}

/// Submits `form`, producing a [`NavigationRequest`], or `None` if `form` is not
/// an element. Walks the form's descendants for successful controls (those with
/// a `name`), serializing them as `application/x-www-form-urlencoded`.
// spec: https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-submission-algorithm
pub fn submit(dom: &Dom, form: NodeId, values: &FormState) -> Option<NavigationRequest> {
    let fallback_url = values.current_url.as_deref().unwrap_or("");
    submit_with_current_url(dom, form, values, fallback_url)
}

/// Resolves a click on `button` into a form submission, if applicable.
///
/// Returns `Some(NavigationRequest)` when `button` is a submit button
/// (`is_submit_button`) that has an owning form (`find_form_for_button`);
/// otherwise returns `None`. The clicked button is recorded as the form's
/// submitter before submitting, so any named/`<input type=image>` submitter
/// participates in the serialized controls exactly as `submit` already handles it.
pub fn submit_from_button(
    dom: &Dom,
    button: NodeId,
    values: &FormState,
) -> Option<NavigationRequest> {
    if !is_submit_button(dom, button) {
        return None;
    }
    let form = find_form_for_button(dom, button)?;
    let mut local_values = values.clone();
    local_values.set_submitter(button);
    submit(dom, form, &local_values)
}

/// Submits `form` with an optional current URL context.
/// Correctly collects successful controls, filters out disabled controls,
/// and overrides any existing query string in the action URL for GET requests.
pub fn submit_with_current_url(
    dom: &Dom,
    form: NodeId,
    values: &FormState,
    current_url: &str,
) -> Option<NavigationRequest> {
    tag(dom, form)?; // must be an element

    let method = match attr(dom, form, "method") {
        Some(m) if m.eq_ignore_ascii_case("post") => Method::Post,
        _ => Method::Get,
    };

    let action = attr(dom, form, "action").unwrap_or("");
    let action_url = if action.is_empty() {
        if !current_url.is_empty() {
            current_url.to_string()
        } else {
            "".to_string()
        }
    } else {
        action.to_string()
    };

    let mut pairs: Vec<String> = Vec::new();
    for node in dom.descendants(form) {
        let Some(t) = tag(dom, node) else { continue };
        let t = t.to_ascii_lowercase();
        if !matches!(t.as_str(), "input" | "textarea" | "select" | "button") {
            continue;
        }

        // Skip disabled controls.
        if attr(dom, node, "disabled").is_some() {
            continue;
        }

        let Some(name) = attr(dom, node, "name") else {
            continue;
        };

        let input_type = if t.as_str() == "button" {
            attr(dom, node, "type")
                .unwrap_or("submit")
                .to_ascii_lowercase()
        } else {
            attr(dom, node, "type")
                .unwrap_or("text")
                .to_ascii_lowercase()
        };

        // Buttons and submit/reset/file controls are not successful here.
        // Files and images are simplified and skipped with a comment.
        // Submit buttons are only successful if they are the submitter.
        if matches!(
            input_type.as_str(),
            "submit" | "reset" | "button" | "file" | "image"
        ) {
            if input_type.as_str() == "submit" {
                if values.submitter != Some(node) {
                    continue;
                }
            } else {
                continue;
            }
        }

        if t.as_str() == "select" {
            // Check if there is an overridden value for the select itself
            if let Some(overridden_val) = values.values.get(&node) {
                pairs.push(format!(
                    "{}={}",
                    form_encode(name),
                    form_encode(overridden_val)
                ));
            } else {
                let mut options = Vec::new();
                let mut selected_options = Vec::new();
                for desc in dom.descendants(node) {
                    let Some(t_desc) = tag(dom, desc) else {
                        continue;
                    };
                    if t_desc.eq_ignore_ascii_case("option") {
                        options.push(desc);
                        let is_selected = values
                            .selected
                            .get(&desc)
                            .copied()
                            .unwrap_or_else(|| dom.get_attribute(desc, "selected").is_some());
                        if is_selected {
                            selected_options.push(desc);
                        }
                    }
                }

                if !options.is_empty() {
                    let is_multiple = dom.get_attribute(node, "multiple").is_some();
                    if is_multiple {
                        for opt in selected_options {
                            let val = dom
                                .get_attribute(opt, "value")
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| dom.text_content(opt));
                            pairs.push(format!("{}={}", form_encode(name), form_encode(&val)));
                        }
                    } else {
                        let chosen_opt = if !selected_options.is_empty() {
                            selected_options.last().copied()
                        } else {
                            options.first().copied()
                        };
                        if let Some(opt) = chosen_opt {
                            let val = dom
                                .get_attribute(opt, "value")
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| dom.text_content(opt));
                            pairs.push(format!("{}={}", form_encode(name), form_encode(&val)));
                        }
                    }
                }
            }
            continue;
        }

        // Checkbox/radio: only successful when checked.
        if matches!(input_type.as_str(), "checkbox" | "radio") {
            let is_radio = input_type.as_str() == "radio";
            let checked = if is_radio {
                is_radio_effectively_checked(dom, form, node, values)
            } else {
                values
                    .checked
                    .get(&node)
                    .copied()
                    .unwrap_or_else(|| attr(dom, node, "checked").is_some())
            };
            if !checked {
                continue;
            }
        }

        let value_fallback = if matches!(input_type.as_str(), "checkbox" | "radio") {
            "on"
        } else {
            ""
        };

        let value = values
            .values
            .get(&node)
            .map(|s| s.as_str())
            .or_else(|| attr(dom, node, "value"))
            .unwrap_or(value_fallback);

        pairs.push(format!("{}={}", form_encode(name), form_encode(value)));
    }

    let query = pairs.join("&");

    match method {
        Method::Get => {
            let base_action_url = if let Some(pos) = action_url.find('?') {
                &action_url[..pos]
            } else {
                &action_url
            };
            let url = if query.is_empty() {
                base_action_url.to_string()
            } else {
                format!("{}?{}", base_action_url, query)
            };
            Some(NavigationRequest {
                url,
                method,
                body: String::new(),
                content_type: None,
            })
        }
        Method::Post => Some(NavigationRequest {
            url: action_url,
            method,
            body: query,
            content_type: Some("application/x-www-form-urlencoded".to_string()),
        }),
    }
}

fn is_radio_effectively_checked_with_values(
    dom: &Dom,
    form: NodeId,
    node: NodeId,
    edited_values: &HashMap<String, String>,
) -> bool {
    let Some(name) = attr(dom, node, "name") else {
        return false;
    };
    let checked = if let Some(ev) = edited_values.get(name) {
        let ctrl_val = attr(dom, node, "value").unwrap_or("on");
        ev == ctrl_val || ev == "on" || ev == "true"
    } else {
        attr(dom, node, "checked").is_some()
    };
    if !checked {
        return false;
    }

    let mut found_self = false;
    for desc in dom.descendants(form) {
        if desc == node {
            found_self = true;
            continue;
        }
        if !found_self {
            continue;
        }
        let Some(t) = tag(dom, desc) else {
            continue;
        };
        if t.eq_ignore_ascii_case("input") {
            let is_radio = attr(dom, desc, "type")
                .map(|typ| typ.eq_ignore_ascii_case("radio"))
                .unwrap_or(false);
            if is_radio {
                let other_name = attr(dom, desc, "name");
                if other_name == Some(name) {
                    let other_checked = if let Some(ev) = edited_values.get(name) {
                        let ctrl_val = attr(dom, desc, "value").unwrap_or("on");
                        ev == ctrl_val || ev == "on" || ev == "true"
                    } else {
                        attr(dom, desc, "checked").is_some()
                    };
                    if other_checked {
                        return false;
                    }
                }
            }
        }
    }

    true
}

/// Submits `form` using a map/assoc of field name to current edited value,
/// producing a [`NavigationRequest`], or `None` if `form` is not an element.
/// GET requests build query parameters in the URL, while POST requests
/// place them in the urlencoded body and set the content type.
// spec: S-62
pub fn submit_with_values(
    dom: &Dom,
    form: NodeId,
    edited_values: &HashMap<String, String>,
) -> Option<NavigationRequest> {
    submit_with_values_and_current_url(dom, form, edited_values, "")
}

/// Submits `form` using a map/assoc of field name to current edited value
/// with an optional current URL context.
pub fn submit_with_values_and_current_url(
    dom: &Dom,
    form: NodeId,
    edited_values: &HashMap<String, String>,
    current_url: &str,
) -> Option<NavigationRequest> {
    tag(dom, form)?; // must be an element

    let method = match attr(dom, form, "method") {
        Some(m) if m.eq_ignore_ascii_case("post") => Method::Post,
        _ => Method::Get,
    };

    let action = attr(dom, form, "action").unwrap_or("");
    let action_url = if action.is_empty() {
        if !current_url.is_empty() {
            current_url.to_string()
        } else {
            "".to_string()
        }
    } else {
        action.to_string()
    };

    let mut pairs: Vec<String> = Vec::new();
    for node in dom.descendants(form) {
        let Some(t) = tag(dom, node) else { continue };
        let t = t.to_ascii_lowercase();
        if !matches!(t.as_str(), "input" | "textarea" | "select" | "button") {
            continue;
        }

        // Skip disabled controls.
        if attr(dom, node, "disabled").is_some() {
            continue;
        }

        let Some(name) = attr(dom, node, "name") else {
            continue;
        };

        let input_type = if t.as_str() == "button" {
            attr(dom, node, "type")
                .unwrap_or("submit")
                .to_ascii_lowercase()
        } else {
            attr(dom, node, "type")
                .unwrap_or("text")
                .to_ascii_lowercase()
        };

        // Buttons and submit/reset/file controls are not successful here.
        // Submit buttons are only successful if they are the submitter (in edited_values).
        if matches!(
            input_type.as_str(),
            "submit" | "reset" | "button" | "file" | "image"
        ) {
            if input_type.as_str() == "submit" {
                if !edited_values.contains_key(name) {
                    continue;
                }
            } else {
                continue;
            }
        }

        if t.as_str() == "select" {
            // Check if there is an overridden value for the select itself in edited_values
            if let Some(overridden_val) = edited_values.get(name) {
                pairs.push(format!(
                    "{}={}",
                    form_encode(name),
                    form_encode(overridden_val)
                ));
            } else {
                let mut options = Vec::new();
                let mut selected_options = Vec::new();
                for desc in dom.descendants(node) {
                    let Some(t_desc) = tag(dom, desc) else {
                        continue;
                    };
                    if t_desc.eq_ignore_ascii_case("option") {
                        options.push(desc);
                        let is_selected = dom.get_attribute(desc, "selected").is_some();
                        if is_selected {
                            selected_options.push(desc);
                        }
                    }
                }

                if !options.is_empty() {
                    let is_multiple = dom.get_attribute(node, "multiple").is_some();
                    if is_multiple {
                        for opt in selected_options {
                            let val = dom
                                .get_attribute(opt, "value")
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| dom.text_content(opt));
                            pairs.push(format!("{}={}", form_encode(name), form_encode(&val)));
                        }
                    } else {
                        let chosen_opt = if !selected_options.is_empty() {
                            selected_options.last().copied()
                        } else {
                            options.first().copied()
                        };
                        if let Some(opt) = chosen_opt {
                            let val = dom
                                .get_attribute(opt, "value")
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| dom.text_content(opt));
                            pairs.push(format!("{}={}", form_encode(name), form_encode(&val)));
                        }
                    }
                }
            }
            continue;
        }

        // Checkbox/radio: only successful when checked.
        if matches!(input_type.as_str(), "checkbox" | "radio") {
            let is_radio = input_type.as_str() == "radio";
            let checked = if is_radio {
                is_radio_effectively_checked_with_values(dom, form, node, edited_values)
            } else {
                if let Some(ev) = edited_values.get(name) {
                    let ctrl_val = attr(dom, node, "value").unwrap_or("on");
                    ev == ctrl_val || ev == "on" || ev == "true"
                } else {
                    attr(dom, node, "checked").is_some()
                }
            };
            if !checked {
                continue;
            }
        }

        let value_fallback = if matches!(input_type.as_str(), "checkbox" | "radio") {
            "on"
        } else {
            ""
        };

        let value = edited_values
            .get(name)
            .map(|s| s.as_str())
            .or_else(|| attr(dom, node, "value"))
            .unwrap_or(value_fallback);

        pairs.push(format!("{}={}", form_encode(name), form_encode(value)));
    }

    let query = pairs.join("&");

    match method {
        Method::Get => {
            let base_action_url = if let Some(pos) = action_url.find('?') {
                &action_url[..pos]
            } else {
                &action_url
            };
            let url = if query.is_empty() {
                base_action_url.to_string()
            } else {
                format!("{}?{}", base_action_url, query)
            };
            Some(NavigationRequest {
                url,
                method,
                body: String::new(),
                content_type: None,
            })
        }
        Method::Post => Some(NavigationRequest {
            url: action_url,
            method,
            body: query,
            content_type: Some("application/x-www-form-urlencoded".to_string()),
        }),
    }
}

/// Helper to determine if a node is a submit button.
///
/// True if the node is:
/// - `<button>` with no type attribute, or type="submit" (case-insensitive)
/// - `<input type="submit">` (case-insensitive)
/// - `<input type="image">` (case-insensitive)
// spec: https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-submit-button
pub fn is_submit_button(dom: &Dom, node: NodeId) -> bool {
    let Some(t) = tag(dom, node) else {
        return false;
    };
    if t.eq_ignore_ascii_case("button") {
        match attr(dom, node, "type") {
            Some(typ) => typ.eq_ignore_ascii_case("submit"),
            None => true,
        }
    } else if t.eq_ignore_ascii_case("input") {
        if let Some(typ) = attr(dom, node, "type") {
            typ.eq_ignore_ascii_case("submit") || typ.eq_ignore_ascii_case("image")
        } else {
            false
        }
    } else {
        false
    }
}

/// Helper to find the form associated with a button (or other form control).
///
/// If the button has a `form` attribute, returns the `<form>` element in the
/// document with the matching `id`. Otherwise, walks up the ancestor chain and
/// returns the nearest `<form>` ancestor. Returns `None` if no form is associated.
// spec: https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-owner
pub fn find_form_for_button(dom: &Dom, button: NodeId) -> Option<NodeId> {
    if let Some(form_id) = attr(dom, button, "form") {
        // Explicit association: search the document for a <form> element with id == form_id
        let doc_root = dom.document();
        for desc in dom.descendants(doc_root) {
            if let Some(t) = tag(dom, desc)
                && t.eq_ignore_ascii_case("form")
                && let Some(id) = attr(dom, desc, "id")
                && id == form_id
            {
                return Some(desc);
            }
        }
        None
    } else {
        // Implicit association: walk up ancestor chain to find the nearest `<form>`
        let mut current = button;
        while let Some(parent) = dom.parent(current) {
            if let Some(t) = tag(dom, parent)
                && t.eq_ignore_ascii_case("form")
            {
                return Some(parent);
            }
            current = parent;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{Dom, NodeData};

    fn el(dom: &mut Dom, name: &str, attrs: &[(&str, &str)]) -> NodeId {
        dom.create_node(NodeData::Element {
            name: name.to_string(),
            attrs: attrs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        })
    }

    #[test]
    fn get_search_builds_query() {
        let mut dom = Dom::new();
        let form = el(
            &mut dom,
            "form",
            &[("action", "/search"), ("method", "get")],
        );
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
        let btn = el(
            &mut dom,
            "input",
            &[("name", "go"), ("type", "submit"), ("value", "Go")],
        );
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

    #[test]
    fn test_editing_model_basic() {
        let mut state = EditingState::new(String::new());
        assert_eq!(state.value(), "");
        assert_eq!(state.cursor(), 0);

        state = insert_char(state, 'a');
        state = insert_char(state, 'b');
        state = insert_char(state, 'c');
        assert_eq!(state.value(), "abc");
        assert_eq!(state.cursor(), 3);

        state = backspace(state);
        assert_eq!(state.value(), "ab");
        assert_eq!(state.cursor(), 2);
    }

    #[test]
    fn test_editing_model_move_cursor_and_clamping() {
        let mut state = EditingState::new("hello".to_string());
        assert_eq!(state.value(), "hello");
        assert_eq!(state.cursor(), 5);

        // Move past right boundary
        state = move_cursor(state, 5);
        assert_eq!(state.cursor(), 5);

        // Move left
        state = move_cursor(state, -2);
        assert_eq!(state.cursor(), 3);

        // Insert at cursor
        state = insert_char(state, 'x');
        assert_eq!(state.value(), "helxlo");
        assert_eq!(state.cursor(), 4);

        // Backspace at cursor
        state = backspace(state);
        assert_eq!(state.value(), "hello");
        assert_eq!(state.cursor(), 3);

        // Move past left boundary
        state = move_cursor(state, -10);
        assert_eq!(state.cursor(), 0);

        // Backspace at boundary 0 (noop)
        state = backspace(state);
        assert_eq!(state.value(), "hello");
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn test_editing_model_utf8() {
        // Multi-byte Unicode character editing (e.g., CJK)
        let mut state = EditingState::new("こんにちは".to_string());
        assert_eq!(state.value(), "こんにちは");
        assert_eq!(state.cursor(), 5);

        state = move_cursor(state, -2);
        assert_eq!(state.cursor(), 3);

        // Backspace 'に' which is at character index 2 (preceding cursor 3)
        state = backspace(state);
        assert_eq!(state.value(), "こんちは");
        assert_eq!(state.cursor(), 2);

        state = insert_char(state, 'に');
        assert_eq!(state.value(), "こんにちは");
        assert_eq!(state.cursor(), 3);
    }

    #[test]
    fn test_submit_with_values_get() {
        let mut dom = Dom::new();
        let form = el(
            &mut dom,
            "form",
            &[("action", "/search"), ("method", "get")],
        );
        let input = el(&mut dom, "input", &[("name", "q")]);
        dom.append_child(form, input);

        let mut edited = HashMap::new();
        edited.insert("q".to_string(), "a b".to_string());

        let req = submit_with_values(&dom, form, &edited).unwrap();
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.url, "/search?q=a+b");
        assert_eq!(req.body, "");
        assert_eq!(req.content_type, None);
    }

    #[test]
    fn test_submit_with_values_post() {
        let mut dom = Dom::new();
        let form = el(
            &mut dom,
            "form",
            &[("action", "/submit"), ("method", "post")],
        );
        let input = el(&mut dom, "input", &[("name", "data")]);
        dom.append_child(form, input);

        let mut edited = HashMap::new();
        edited.insert("data".to_string(), "a b".to_string());

        let req = submit_with_values(&dom, form, &edited).unwrap();
        assert_eq!(req.method, Method::Post);
        assert_eq!(req.url, "/submit");
        assert_eq!(req.body, "data=a+b");
        assert_eq!(
            req.content_type,
            Some("application/x-www-form-urlencoded".to_string())
        );
    }

    #[test]
    fn test_submit_with_values_fallback() {
        let mut dom = Dom::new();
        let form = el(
            &mut dom,
            "form",
            &[("action", "/submit"), ("method", "post")],
        );
        let input1 = el(&mut dom, "input", &[("name", "f1"), ("value", "default1")]);
        let input2 = el(&mut dom, "input", &[("name", "f2"), ("value", "default2")]);
        dom.append_child(form, input1);
        dom.append_child(form, input2);

        let mut edited = HashMap::new();
        edited.insert("f1".to_string(), "overridden1".to_string());

        let req = submit_with_values(&dom, form, &edited).unwrap();
        assert_eq!(req.method, Method::Post);
        assert_eq!(req.url, "/submit");
        assert_eq!(req.body, "f1=overridden1&f2=default2");
        assert_eq!(
            req.content_type,
            Some("application/x-www-form-urlencoded".to_string())
        );
    }

    #[test]
    fn test_select_single_first_option_default() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let select = el(&mut dom, "select", &[("name", "color")]);
        let opt1 = el(&mut dom, "option", &[("value", "red")]);
        let opt2 = el(&mut dom, "option", &[("value", "blue")]);
        dom.append_child(select, opt1);
        dom.append_child(select, opt2);
        dom.append_child(form, select);

        let req = submit(&dom, form, &FormState::new()).unwrap();
        assert_eq!(req.url, "/a?color=red");
    }

    #[test]
    fn test_select_single_attribute_selected() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let select = el(&mut dom, "select", &[("name", "color")]);
        let opt1 = el(&mut dom, "option", &[("value", "red")]);
        let opt2 = el(&mut dom, "option", &[("value", "blue"), ("selected", "")]);
        dom.append_child(select, opt1);
        dom.append_child(select, opt2);
        dom.append_child(form, select);

        let req = submit(&dom, form, &FormState::new()).unwrap();
        assert_eq!(req.url, "/a?color=blue");
    }

    #[test]
    fn test_select_option_text_content_fallback() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let select = el(&mut dom, "select", &[("name", "color")]);
        let opt1 = el(&mut dom, "option", &[]);
        let t1 = dom.create_node(NodeData::Text("  Green  ".to_string()));
        dom.append_child(opt1, t1);
        dom.append_child(select, opt1);
        dom.append_child(form, select);

        let req = submit(&dom, form, &FormState::new()).unwrap();
        assert_eq!(req.url, "/a?color=++Green++");
    }

    #[test]
    fn test_select_form_state_override() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let select = el(&mut dom, "select", &[("name", "color")]);
        let opt1 = el(&mut dom, "option", &[("value", "red")]);
        let opt2 = el(&mut dom, "option", &[("value", "blue"), ("selected", "")]);
        dom.append_child(select, opt1);
        dom.append_child(select, opt2);
        dom.append_child(form, select);

        let mut state = FormState::new();
        state.set_selected(opt1, true);
        state.set_selected(opt2, false);

        let req = submit(&dom, form, &state).unwrap();
        assert_eq!(req.url, "/a?color=red");
    }

    #[test]
    fn test_select_option_helper() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let select = el(&mut dom, "select", &[("name", "color")]);
        let opt1 = el(&mut dom, "option", &[("value", "red"), ("selected", "")]);
        let opt2 = el(&mut dom, "option", &[("value", "blue")]);
        dom.append_child(select, opt1);
        dom.append_child(select, opt2);
        dom.append_child(form, select);

        let mut state = FormState::new();
        state.select_option(&dom, select, opt2);

        let req = submit(&dom, form, &state).unwrap();
        assert_eq!(req.url, "/a?color=blue");
    }

    #[test]
    fn test_select_multiple() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let select = el(&mut dom, "select", &[("name", "color"), ("multiple", "")]);
        let opt1 = el(&mut dom, "option", &[("value", "red"), ("selected", "")]);
        let opt2 = el(&mut dom, "option", &[("value", "blue"), ("selected", "")]);
        dom.append_child(select, opt1);
        dom.append_child(select, opt2);
        dom.append_child(form, select);

        let req = submit(&dom, form, &FormState::new()).unwrap();
        assert_eq!(req.url, "/a?color=red&color=blue");
    }

    #[test]
    fn test_radio_exclusivity_on_submit() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let r1 = el(
            &mut dom,
            "input",
            &[
                ("type", "radio"),
                ("name", "gender"),
                ("value", "male"),
                ("checked", ""),
            ],
        );
        let r2 = el(
            &mut dom,
            "input",
            &[
                ("type", "radio"),
                ("name", "gender"),
                ("value", "female"),
                ("checked", ""),
            ],
        );
        dom.append_child(form, r1);
        dom.append_child(form, r2);

        let req = submit(&dom, form, &FormState::new()).unwrap();
        assert_eq!(req.url, "/a?gender=female");
    }

    #[test]
    fn test_radio_exclusivity_helper() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let r1 = el(
            &mut dom,
            "input",
            &[("type", "radio"), ("name", "gender"), ("value", "male")],
        );
        let r2 = el(
            &mut dom,
            "input",
            &[("type", "radio"), ("name", "gender"), ("value", "female")],
        );
        dom.append_child(form, r1);
        dom.append_child(form, r2);

        let mut state = FormState::new();
        state.check_radio(&dom, form, r1);
        state.check_radio(&dom, form, r2);

        let req = submit(&dom, form, &state).unwrap();
        assert_eq!(req.url, "/a?gender=female");

        state.check_radio(&dom, form, r1);
        let req2 = submit(&dom, form, &state).unwrap();
        assert_eq!(req2.url, "/a?gender=male");
    }

    #[test]
    fn test_checkbox_default_value() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let cb = el(
            &mut dom,
            "input",
            &[("type", "checkbox"), ("name", "agree"), ("checked", "")],
        );
        dom.append_child(form, cb);

        let req = submit(&dom, form, &FormState::new()).unwrap();
        assert_eq!(req.url, "/a?agree=on");
    }

    #[test]
    fn test_submit_with_values_select_and_radio() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/a")]);
        let select = el(&mut dom, "select", &[("name", "color")]);
        let opt1 = el(&mut dom, "option", &[("value", "red")]);
        let opt2 = el(&mut dom, "option", &[("value", "blue")]);
        dom.append_child(select, opt1);
        dom.append_child(select, opt2);
        dom.append_child(form, select);

        let r1 = el(
            &mut dom,
            "input",
            &[("type", "radio"), ("name", "gender"), ("value", "male")],
        );
        let r2 = el(
            &mut dom,
            "input",
            &[("type", "radio"), ("name", "gender"), ("value", "female")],
        );
        dom.append_child(form, r1);
        dom.append_child(form, r2);

        let mut edited = HashMap::new();
        edited.insert("color".to_string(), "blue".to_string());
        edited.insert("gender".to_string(), "male".to_string());

        let req = submit_with_values(&dom, form, &edited).unwrap();
        assert_eq!(req.url, "/a?color=blue&gender=male");
    }

    #[test]
    fn test_is_submit_button_helper() {
        let mut dom = Dom::new();

        let b_no_type = el(&mut dom, "button", &[]);
        let b_submit = el(&mut dom, "button", &[("type", "submit")]);
        let b_button = el(&mut dom, "button", &[("type", "button")]);
        let b_reset = el(&mut dom, "button", &[("type", "reset")]);

        let i_submit = el(&mut dom, "input", &[("type", "submit")]);
        let i_image = el(&mut dom, "input", &[("type", "image")]);
        let i_text = el(&mut dom, "input", &[("type", "text")]);
        let i_no_type = el(&mut dom, "input", &[]);

        let other = el(&mut dom, "div", &[]);

        assert!(is_submit_button(&dom, b_no_type));
        assert!(is_submit_button(&dom, b_submit));
        assert!(!is_submit_button(&dom, b_button));
        assert!(!is_submit_button(&dom, b_reset));

        assert!(is_submit_button(&dom, i_submit));
        assert!(is_submit_button(&dom, i_image));
        assert!(!is_submit_button(&dom, i_text));
        assert!(!is_submit_button(&dom, i_no_type));

        assert!(!is_submit_button(&dom, other));

        // Case insensitivity checks
        let b_submit_caps = el(&mut dom, "BUTTON", &[("TYPE", "SUBMIT")]);
        let i_image_caps = el(&mut dom, "INPUT", &[("TYPE", "IMAGE")]);

        assert!(is_submit_button(&dom, b_submit_caps));
        assert!(is_submit_button(&dom, i_image_caps));
    }

    #[test]
    fn test_find_form_for_button_helper() {
        let mut dom = Dom::new();
        let doc_root = dom.document();

        let form1 = el(&mut dom, "form", &[("id", "f1")]);
        let form2 = el(&mut dom, "form", &[("id", "f2")]);
        dom.append_child(doc_root, form1);
        dom.append_child(doc_root, form2);

        // Implicit ancestry matching: nested in form1
        let btn_implicit1 = el(&mut dom, "button", &[]);
        dom.append_child(form1, btn_implicit1);

        assert_eq!(find_form_for_button(&dom, btn_implicit1), Some(form1));

        // Deeply nested implicit matching
        let div = el(&mut dom, "div", &[]);
        dom.append_child(form1, div);
        let btn_deep_implicit = el(&mut dom, "button", &[]);
        dom.append_child(div, btn_deep_implicit);

        assert_eq!(find_form_for_button(&dom, btn_deep_implicit), Some(form1));

        // Multiple nested forms - should pick nearest ancestor
        let form_nested = el(&mut dom, "form", &[("id", "fnested")]);
        dom.append_child(form1, form_nested);
        let btn_nested = el(&mut dom, "button", &[]);
        dom.append_child(form_nested, btn_nested);

        assert_eq!(find_form_for_button(&dom, btn_nested), Some(form_nested));

        // Explicit association via "form" attribute (form1 is outside)
        let btn_explicit = el(&mut dom, "button", &[("form", "f1")]);
        dom.append_child(doc_root, btn_explicit);

        assert_eq!(find_form_for_button(&dom, btn_explicit), Some(form1));

        // Explicit association to non-form element with matching ID should return None
        let div_id_only = el(&mut dom, "div", &[("id", "not-a-form")]);
        dom.append_child(doc_root, div_id_only);
        let btn_explicit_not_a_form = el(&mut dom, "button", &[("form", "not-a-form")]);
        dom.append_child(doc_root, btn_explicit_not_a_form);

        assert_eq!(find_form_for_button(&dom, btn_explicit_not_a_form), None);

        // Button with no form association
        let btn_no_form = el(&mut dom, "button", &[]);
        dom.append_child(doc_root, btn_no_form);

        assert_eq!(find_form_for_button(&dom, btn_no_form), None);
    }

    #[test]
    fn test_submit_from_button_success() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/s"), ("method", "get")]);
        let input = el(&mut dom, "input", &[("name", "q")]);
        let btn = el(&mut dom, "button", &[("type", "submit")]);
        dom.append_child(form, input);
        dom.append_child(form, btn);

        let mut state = FormState::new();
        state.set_value(input, "hello");

        let req = submit_from_button(&dom, btn, &state).unwrap();
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.url, "/s?q=hello");
    }

    #[test]
    fn test_submit_from_button_non_submit_node() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/s"), ("method", "get")]);
        let div = el(&mut dom, "div", &[]);
        let btn_type_button = el(&mut dom, "button", &[("type", "button")]);
        dom.append_child(form, div);
        dom.append_child(form, btn_type_button);

        let state = FormState::new();

        assert!(submit_from_button(&dom, div, &state).is_none());
        assert!(submit_from_button(&dom, btn_type_button, &state).is_none());
    }

    #[test]
    fn test_submit_from_button_no_owning_form() {
        let mut dom = Dom::new();
        let doc_root = dom.document();
        let btn = el(&mut dom, "button", &[("type", "submit")]);
        dom.append_child(doc_root, btn);

        let state = FormState::new();

        assert!(submit_from_button(&dom, btn, &state).is_none());
    }

    #[test]
    fn test_submit_from_button_named_submitter_included() {
        let mut dom = Dom::new();
        let form = el(&mut dom, "form", &[("action", "/s"), ("method", "get")]);
        let input = el(&mut dom, "input", &[("name", "q")]);
        let btn = el(
            &mut dom,
            "button",
            &[("type", "submit"), ("name", "op"), ("value", "go")],
        );
        dom.append_child(form, input);
        dom.append_child(form, btn);

        let mut state = FormState::new();
        state.set_value(input, "hello");

        let req = submit_from_button(&dom, btn, &state).unwrap();
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.url, "/s?q=hello&op=go");
    }
}

/// Editing state for `<input type=text>` and `<textarea>`.
/// Holds the current text value and the cursor position (as a character index).
// spec: https://html.spec.whatwg.org/multipage/form-control-infrastructure.html
// spec: S-51
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditingState {
    value: String,
    cursor: usize,
}

impl EditingState {
    /// Creates a new editing state with the given value.
    /// The cursor is initialized at the end of the value.
    pub fn new(value: String) -> Self {
        let char_count = value.chars().count();
        Self {
            value,
            cursor: char_count,
        }
    }

    /// Returns the current text value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the current cursor index as a character position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Inserts a character at the current cursor position, modifying the state in-place.
    ///
    // spec: S-51
    pub fn insert_char_mut(&mut self, ch: char) {
        // TODO(spec): IME support is not implemented yet.
        // TODO(spec): Selection support is not implemented yet.
        let mut chars: Vec<char> = self.value.chars().collect();
        if self.cursor > chars.len() {
            self.cursor = chars.len();
        }
        chars.insert(self.cursor, ch);
        self.value = chars.into_iter().collect();
        self.cursor += 1;
    }

    /// Deletes the character immediately preceding the current cursor position, modifying the state in-place.
    ///
    // spec: S-51
    pub fn backspace_mut(&mut self) {
        // TODO(spec): IME support is not implemented yet.
        // TODO(spec): Selection support is not implemented yet.
        let mut chars: Vec<char> = self.value.chars().collect();
        if self.cursor > chars.len() {
            self.cursor = chars.len();
        }
        if self.cursor > 0 {
            chars.remove(self.cursor - 1);
            self.value = chars.into_iter().collect();
            self.cursor -= 1;
        }
    }

    /// Moves the cursor by delta (positive or negative), clamping to the bounds of the value, modifying the state in-place.
    ///
    // spec: S-51
    pub fn move_cursor_mut(&mut self, delta: isize) {
        let char_count = self.value.chars().count();
        if self.cursor > char_count {
            self.cursor = char_count;
        }
        let new_cursor = (self.cursor as isize).saturating_add(delta);
        self.cursor = if new_cursor < 0 {
            0
        } else if new_cursor > char_count as isize {
            char_count
        } else {
            new_cursor as usize
        };
    }
}

/// Inserts a character at the current cursor position.
///
// spec: S-51
pub fn insert_char(mut state: EditingState, ch: char) -> EditingState {
    state.insert_char_mut(ch);
    state
}

/// Deletes the character immediately preceding the current cursor position.
///
// spec: S-51
pub fn backspace(mut state: EditingState) -> EditingState {
    state.backspace_mut();
    state
}

/// Moves the cursor by delta (positive or negative), clamping it to the bounds of the value.
///
// spec: S-51
pub fn move_cursor(mut state: EditingState, delta: isize) -> EditingState {
    state.move_cursor_mut(delta);
    state
}
