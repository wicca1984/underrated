//! Scripting module providing JavaScript execution via the Boa engine.
//!
//! This module implements the `ScriptHost` port, allowing the browser engine
//! to execute scripts. The current implementation uses the `boa_engine` crate.

use crate::dom::{Dom, NodeData};
use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsString, JsValue, NativeFunction, Source};

/// Errors that can occur during script execution.
#[derive(Debug, PartialEq)]
pub enum ScriptError {
    /// A syntax error in the script.
    Syntax(String),
    /// A runtime error during script execution.
    Runtime(String),
}

impl core::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Syntax(msg) => write!(f, "Syntax Error: {}", msg),
            Self::Runtime(msg) => write!(f, "Runtime Error: {}", msg),
        }
    }
}

impl std::error::Error for ScriptError {}

/// Trait for a host that can execute scripts.
pub trait ScriptHost {
    /// Evaluates the given script source.
    fn eval(&mut self, src: &str) -> Result<(), ScriptError>;
}

/// A `ScriptHost` implementation using the Boa JavaScript engine.
pub struct BoaHost {
    context: Context,
}

impl BoaHost {
    /// Creates a new `BoaHost` with an empty context.
    pub fn new() -> Self {
        let mut context = Context::default();

        // EXPERIMENTAL: Minimal DOM binding (document.title)
        // TODO(spec): Full DOM bindings are a large separate effort.
        Self::setup_experimental_dom(&mut context);

        Self { context }
    }

    fn setup_experimental_dom(context: &mut Context) {
        let get_element_by_id = NativeFunction::from_fn_ptr(|_this, args, context| {
            let id_val = if let Some(arg) = args.first() {
                arg.to_string(context)?.to_std_string().unwrap_or_default()
            } else {
                return Ok(JsValue::null());
            };

            let global = context.global_object();
            let document = global.get(JsString::from("document"), context)?;
            if let Some(document_obj) = document.as_object() {
                let elements_val = document_obj.get(JsString::from("__elements__"), context)?;
                if let Some(elements_obj) = elements_val.as_object() {
                    let elem = elements_obj.get(JsString::from(id_val), context)?;
                    if !elem.is_undefined() {
                        return Ok(elem);
                    }
                }
            }
            Ok(JsValue::null())
        });

        let document = ObjectInitializer::new(context)
            .property(
                JsString::from("title"),
                JsString::from("Underrated"),
                Attribute::all(),
            )
            .function(get_element_by_id, JsString::from("getElementById"), 1)
            .build();

        let _ = context.register_global_property(
            JsString::from("document"),
            document,
            Attribute::all(),
        );

        let global = context.global_object().clone();
        let _ =
            context.register_global_property(JsString::from("window"), global, Attribute::all());
    }

    /// Evaluates the given script with the provided DOM context.
    ///
    /// Exposes a read-only `document` object to the script enabling `document.getElementById`.
    pub fn eval_with_dom(&mut self, src: &str, dom: &Dom) -> Result<String, ScriptError> {
        // 1. Gather all element nodes in `dom` with an `id`.
        let mut elements_with_id = Vec::new();
        let root = dom.document();
        let mut nodes_to_check = vec![root];
        while let Some(node_id) = nodes_to_check.pop() {
            if let Some(NodeData::Element { attrs, .. }) = dom.data(node_id) {
                let id_attr = attrs.iter().find(|(n, _)| n == "id");
                if let Some((_, id_val)) = id_attr {
                    elements_with_id.push((id_val.clone(), dom.text_content(node_id)));
                }
            }
            nodes_to_check.extend(dom.children(node_id).iter().rev().copied());
        }

        // Get the existing `__elements__` object if it exists to reuse element JS objects
        // and preserve their event registrations/state.
        let global = self.context.global_object();
        let document_val = global
            .get(JsString::from("document"), &mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let document_obj = document_val
            .as_object()
            .ok_or_else(|| ScriptError::Runtime("global document is not an object".to_string()))?;

        let existing_elements_val = document_obj
            .get(JsString::from("__elements__"), &mut self.context)
            .unwrap_or(JsValue::undefined());
        let existing_elements_obj = existing_elements_val.as_object();

        // 2. Build or update the element JS objects.
        let mut element_objs = Vec::new();
        for (id_val, text_content_val) in elements_with_id {
            let mut element_obj = None;
            if let Some(ee_obj) = &existing_elements_obj {
                let existing_elem_val = ee_obj
                    .get(JsString::from(id_val.clone()), &mut self.context)
                    .unwrap_or(JsValue::undefined());
                if let Some(existing_elem_obj) = existing_elem_val.as_object() {
                    // Update textContent in place to match the new DOM content
                    existing_elem_obj
                        .set(
                            JsString::from("textContent"),
                            JsValue::from(JsString::from(text_content_val.clone())),
                            false,
                            &mut self.context,
                        )
                        .map_err(|e| ScriptError::Runtime(e.to_string()))?;
                    element_obj = Some(existing_elem_obj.clone());
                }
            }

            let element_obj = match element_obj {
                Some(obj) => obj,
                None => ObjectInitializer::new(&mut self.context)
                    .property(
                        JsString::from("textContent"),
                        JsString::from(text_content_val),
                        Attribute::all(),
                    )
                    .property(
                        JsString::from("id"),
                        JsString::from(id_val.clone()),
                        Attribute::all(),
                    )
                    .function(
                        NativeFunction::from_fn_ptr(add_event_listener),
                        JsString::from("addEventListener"),
                        2,
                    )
                    .build(),
            };
            element_objs.push((JsString::from(id_val), element_obj));
        }

        // Build the `__elements__` registry JS object.
        let mut registry_builder = ObjectInitializer::new(&mut self.context);
        for (id_js, element_obj) in element_objs {
            registry_builder.property(id_js, element_obj, Attribute::all());
        }
        let registry_obj = registry_builder.build();

        // 4. Attach `__elements__` to `document`.
        document_obj
            .set(
                JsString::from("__elements__"),
                JsValue::from(registry_obj),
                false,
                &mut self.context,
            )
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;

        // 5. Evaluate the source code.
        let source = Source::from_bytes(src.as_bytes());
        let res_val = self.context.eval(source).map_err(map_boa_error)?;

        // 6. Coerce the JS result to String.
        let res_str = res_val
            .to_string(&mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?
            .to_std_string()
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;

        Ok(res_str)
    }

    /// Dispatches an event of type `event_type` to the element with `id`.
    ///
    /// Invokes any handlers registered via `addEventListener` on that element.
    /// The handlers are executed in the same Boa context, with their `this`
    /// bound to the target element.
    ///
    /// If a handler throws an exception, the dispatch is aborted and the error is returned.
    pub fn dispatch_event(&mut self, id: &str, event_type: &str) -> Result<(), ScriptError> {
        // TODO(spec): Support full Event interface, event capturing/bubbling propagation phases,
        // stopPropagation, preventDefault, and DOM write propagation during handler execution.
        let global = self.context.global_object();
        let document_val = global
            .get(JsString::from("document"), &mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let document_obj = match document_val.as_object() {
            Some(obj) => obj,
            None => return Ok(()), // Document not found, no-op
        };

        let elements_val = document_obj
            .get(JsString::from("__elements__"), &mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let elements_obj = match elements_val.as_object() {
            Some(obj) => obj,
            None => return Ok(()), // Registry not found, no-op
        };

        let elem_val = elements_obj
            .get(JsString::from(id), &mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let elem_obj = match elem_val.as_object() {
            Some(obj) => obj,
            None => return Ok(()), // Element not found, no-op
        };

        let events_val = elem_obj
            .get(JsString::from("__events__"), &mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let events_obj = match events_val.as_object() {
            Some(obj) => obj,
            None => return Ok(()), // No events registered, no-op
        };

        let handlers_val = events_obj
            .get(JsString::from(event_type), &mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let handlers_obj = match handlers_val.as_object() {
            Some(obj) => obj,
            None => return Ok(()), // No handlers for this event type, no-op
        };

        // Handlers is a JS Array. Let's get its length.
        let length_val = handlers_obj
            .get(JsString::from("length"), &mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);

        for i in 0..length {
            let handler_val = handlers_obj
                .get(i, &mut self.context)
                .map_err(|e| ScriptError::Runtime(e.to_string()))?;

            if let Some(handler_obj) = handler_val.as_object() {
                // Construct a mock Event object: { target: elem_obj, type: event_type }
                let event_obj = ObjectInitializer::new(&mut self.context)
                    .property(
                        JsString::from("target"),
                        JsValue::from(elem_obj.clone()),
                        Attribute::all(),
                    )
                    .property(
                        JsString::from("type"),
                        JsString::from(event_type),
                        Attribute::all(),
                    )
                    .build();

                handler_obj
                    .call(
                        &JsValue::from(elem_obj.clone()),
                        &[JsValue::from(event_obj)],
                        &mut self.context,
                    )
                    .map_err(map_boa_error)?;
            }
        }

        Ok(())
    }
}

fn add_event_listener(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    // TODO(spec): Support optional third argument (options: { capture: bool, once: bool, passive: bool }).
    let event_type = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let handler = if let Some(arg) = args.get(1) {
        arg.clone()
    } else {
        return Ok(JsValue::undefined());
    };

    if let Some(this_obj) = this.as_object() {
        let events_prop = JsString::from("__events__");
        let mut events_val = this_obj.get(events_prop.clone(), context)?;
        if events_val.is_undefined() || events_val.is_null() {
            let new_events_obj = ObjectInitializer::new(context).build();
            this_obj.set(
                events_prop.clone(),
                JsValue::from(new_events_obj.clone()),
                false,
                context,
            )?;
            events_val = JsValue::from(new_events_obj);
        }

        if let Some(events_obj) = events_val.as_object() {
            let type_prop = JsString::from(event_type.as_str());
            let mut handlers_val = events_obj.get(type_prop.clone(), context)?;
            if handlers_val.is_undefined() || handlers_val.is_null() {
                let array_constructor = context
                    .global_object()
                    .get(JsString::from("Array"), context)?;
                let array_obj = array_constructor.as_object().ok_or_else(|| {
                    JsError::from_opaque(JsValue::from(JsString::from(
                        "Array constructor not found",
                    )))
                })?;
                let array_val = array_obj.construct(&[], None, context)?;
                events_obj.set(
                    type_prop.clone(),
                    JsValue::from(array_val.clone()),
                    false,
                    context,
                )?;
                handlers_val = JsValue::from(array_val);
            }

            if let Some(handlers_obj) = handlers_val.as_object() {
                let push_val = handlers_obj.get(JsString::from("push"), context)?;
                if let Some(push_fn) = push_val.as_object() {
                    push_fn.call(&handlers_val, &[handler], context)?;
                }
            }
        }
    }

    Ok(JsValue::undefined())
}

impl Default for BoaHost {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptHost for BoaHost {
    fn eval(&mut self, src: &str) -> Result<(), ScriptError> {
        let source = Source::from_bytes(src.as_bytes());
        match self.context.eval(source) {
            Ok(_) => Ok(()),
            Err(err) => Err(map_boa_error(err)),
        }
    }
}

fn map_boa_error(err: JsError) -> ScriptError {
    let msg = err.to_string();
    if msg.contains("SyntaxError") {
        ScriptError::Syntax(msg)
    } else {
        ScriptError::Runtime(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boa_eval_basic() {
        let mut host = BoaHost::new();
        assert!(host.eval("1 + 1").is_ok());
    }

    #[test]
    fn test_boa_eval_syntax_error() {
        let mut host = BoaHost::new();
        // Invalid syntax: missing closing parenthesis
        let result = host.eval("console.log(");
        assert!(result.is_err());
        assert!(matches!(result, Err(ScriptError::Syntax(_))));
    }

    #[test]
    fn test_experimental_dom_binding() {
        let mut host = BoaHost::new();
        // Check if document.title is accessible.
        // We can't easily get the return value from eval(()) so we might need a way to check state.
        // But we can check if it doesn't throw.
        assert!(
            host.eval("if (document.title !== 'Underrated') throw 'Wrong title';")
                .is_ok()
        );
        assert!(host.eval("document.title = 'New Title';").is_ok());
        assert!(
            host.eval("if (document.title !== 'New Title') throw 'Title not updated';")
                .is_ok()
        );
    }

    #[test]
    fn test_eval_with_dom_basic() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "greeting".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("Hello".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        let mut host = BoaHost::new();
        let res = host.eval_with_dom("document.getElementById('greeting').textContent", &dom);
        assert_eq!(res, Ok("Hello".to_string()));
    }

    #[test]
    fn test_eval_with_dom_missing_id() {
        let dom = Dom::new();
        let mut host = BoaHost::new();
        let res = host.eval_with_dom("document.getElementById('nonexistent')", &dom);
        assert_eq!(res, Ok("null".to_string()));
    }

    #[test]
    fn test_add_event_listener_and_dispatch_event_basic() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![("id".to_string(), "btn".to_string())],
        });
        dom.append_child(document, element_id);

        let mut host = BoaHost::new();
        // Register an event listener on "btn" for "click".
        // It sets window.clicked to true.
        let register_script = "
            window.clicked = false;
            document.getElementById('btn').addEventListener('click', () => {
                window.clicked = true;
            });
        ";
        assert!(host.eval_with_dom(register_script, &dom).is_ok());

        // Dispatch click on "btn"
        assert_eq!(host.dispatch_event("btn", "click"), Ok(()));

        // Evaluate window.clicked to verify it became true
        let verify_res = host.eval("window.clicked");
        assert!(verify_res.is_ok());
        // In Boa, evaluating a statement or global value might not return it as a result of eval()
        // since we didn't specify return, but wait, Boa's eval returns the last evaluated statement.
        // Let's verify by throwing an error or checking state.
        let check_script = "if (!window.clicked) throw 'Not clicked';";
        assert!(host.eval(check_script).is_ok());
    }

    #[test]
    fn test_add_event_listener_and_dispatch_event_multiple_handlers() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![("id".to_string(), "btn".to_string())],
        });
        dom.append_child(document, element_id);

        let mut host = BoaHost::new();
        let register_script = "
            window.counter = 0;
            let btn = document.getElementById('btn');
            btn.addEventListener('click', () => {
                window.counter += 1;
            });
            btn.addEventListener('click', () => {
                window.counter += 10;
            });
        ";
        assert!(host.eval_with_dom(register_script, &dom).is_ok());

        assert_eq!(host.dispatch_event("btn", "click"), Ok(()));

        let check_script = "if (window.counter !== 11) throw 'Counter is ' + window.counter;";
        assert!(host.eval(check_script).is_ok());
    }

    #[test]
    fn test_add_event_listener_and_dispatch_event_no_handler() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![("id".to_string(), "btn".to_string())],
        });
        dom.append_child(document, element_id);

        let mut host = BoaHost::new();
        assert!(host.eval_with_dom("1", &dom).is_ok());

        // Dispatch click on "btn" which has no handlers registered
        assert_eq!(host.dispatch_event("btn", "click"), Ok(()));

        // Dispatch on non-existent element
        assert_eq!(host.dispatch_event("nonexistent", "click"), Ok(()));
    }

    #[test]
    fn test_add_event_listener_and_dispatch_event_error_handling() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![("id".to_string(), "btn".to_string())],
        });
        dom.append_child(document, element_id);

        let mut host = BoaHost::new();
        let register_script = "
            document.getElementById('btn').addEventListener('click', () => {
                throw new Error('Test throw error from handler');
            });
        ";
        assert!(host.eval_with_dom(register_script, &dom).is_ok());

        let res = host.dispatch_event("btn", "click");
        assert!(res.is_err());
        match res {
            Err(ScriptError::Runtime(msg)) => {
                assert!(
                    msg.contains("Test throw error from handler"),
                    "Got message: {}",
                    msg
                );
            }
            other => panic!("Expected ScriptError::Runtime, got {:?}", other),
        }
    }
}
