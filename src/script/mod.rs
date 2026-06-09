//! Scripting module providing JavaScript execution via the Boa engine.
//!
//! This module implements the `ScriptHost` port, allowing the browser engine
//! to execute scripts. The current implementation uses the `boa_engine` crate.

use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;
use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsString, JsValue, NativeFunction, Source};
use std::cell::RefCell;
use std::collections::HashMap;

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

thread_local! {
    static CURRENT_DOM: RefCell<Option<Dom>> = const { RefCell::new(None) };
    static KEY_TO_NODE: RefCell<HashMap<String, NodeId>> = RefCell::new(HashMap::new());
}

impl BoaHost {
    /// Creates a new `BoaHost` with an empty context.
    pub fn new() -> Self {
        let mut context = Context::default();

        // Setup DOM bindings including the write APIs
        Self::setup_experimental_dom(&mut context);

        Self { context }
    }

    fn setup_experimental_dom(context: &mut Context) {
        let bridge = ObjectInitializer::new(context)
            .function(
                NativeFunction::from_fn_ptr(bridge_create_element),
                JsString::from("createElement"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_get_element_by_id),
                JsString::from("getElementById"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_append_child),
                JsString::from("appendChild"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_set_attribute),
                JsString::from("setAttribute"),
                3,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_get_attribute),
                JsString::from("getAttribute"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_get_text_content),
                JsString::from("getTextContent"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_set_text_content),
                JsString::from("setTextContent"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(add_event_listener),
                JsString::from("addEventListener"),
                2,
            )
            .build();

        let _ = context.register_global_property(
            JsString::from("__dom_bridge__"),
            bridge,
            Attribute::all(),
        );

        let document = ObjectInitializer::new(context)
            .property(
                JsString::from("title"),
                JsString::from("Underrated"),
                Attribute::all(),
            )
            .build();

        let _ = context.register_global_property(
            JsString::from("document"),
            document,
            Attribute::all(),
        );

        let global = context.global_object().clone();
        let _ =
            context.register_global_property(JsString::from("window"), global, Attribute::all());

        // Evaluate the JS wrapper code to build the dynamic DOM API.
        // spec: https://dom.spec.whatwg.org/#dom-document-createelement
        // spec: https://dom.spec.whatwg.org/#dom-node-appendchild
        // spec: https://dom.spec.whatwg.org/#dom-element-setattribute
        // spec: https://dom.spec.whatwg.org/#dom-element-getattribute
        // spec: https://dom.spec.whatwg.org/#dom-node-textcontent
        let setup_code = r#"
            (function() {
                const bridge = window.__dom_bridge__;
                const registry = {};
                document.__node_registry__ = registry;

                function getOrCreateNode(key) {
                    if (!key) return null;
                    if (registry[key]) {
                        return registry[key];
                    }

                    const node = {
                        __key__: key,
                        appendChild(child) {
                            if (!child || !child.__key__) {
                                throw new TypeError("child must be a Node");
                            }
                            bridge.appendChild(this.__key__, child.__key__);
                            return child;
                        },
                        setAttribute(name, value) {
                            bridge.setAttribute(this.__key__, String(name), String(value));
                        },
                        getAttribute(name) {
                            return bridge.getAttribute(this.__key__, String(name));
                        }
                    };

                    node.addEventListener = bridge.addEventListener;

                    Object.defineProperty(node, 'textContent', {
                        get() {
                            return bridge.getTextContent(this.__key__);
                        },
                        set(val) {
                            bridge.setTextContent(this.__key__, String(val));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'id', {
                        get() {
                            return this.getAttribute('id') || '';
                        },
                        set(val) {
                            this.setAttribute('id', val);
                        },
                        enumerable: true,
                        configurable: true
                    });

                    registry[key] = node;
                    return node;
                }

                window.__getOrCreateNode = getOrCreateNode;

                document.createElement = function(tagName) {
                    const key = bridge.createElement(String(tagName));
                    return getOrCreateNode(key);
                };

                document.getElementById = function(id) {
                    const key = bridge.getElementById(String(id));
                    return getOrCreateNode(key);
                };

                document.appendChild = function(child) {
                    if (!child || !child.__key__) {
                        throw new TypeError("child must be a Node");
                    }
                    bridge.appendChild(this.__key__, child.__key__);
                    return child;
                };

                document.addEventListener = bridge.addEventListener;

                document.__elements__ = new Proxy({}, {
                    get(target, prop) {
                        if (typeof prop === 'string') {
                            return document.getElementById(prop);
                        }
                        return undefined;
                    }
                });
            })();
        "#;

        let source = Source::from_bytes(setup_code.as_bytes());
        if let Err(e) = context.eval(source) {
            eprintln!("Failed to initialize DOM bindings: {:?}", e);
        }
    }

    /// Evaluates the given script with the provided DOM context.
    ///
    /// Exposes a read-write `document` object to the script enabling DOM mutations.
    pub fn eval_with_dom(&mut self, src: &str, dom: &mut Dom) -> Result<String, ScriptError> {
        // 1. Swap DOM out of `dom` to place in thread-safe RefCell
        let temp_dom = std::mem::take(dom);

        CURRENT_DOM.with(|cell| {
            let mut opt = cell.borrow_mut();
            *opt = Some(temp_dom);
            if let Some(d) = opt.as_ref() {
                KEY_TO_NODE.with(|key_cell| {
                    let mut map = key_cell.borrow_mut();
                    map.clear();
                    index_dom_nodes(d, &mut map);
                });
            }
        });

        // 2. Bind the document's key and register the document object in `__node_registry__`
        let root_key = CURRENT_DOM.with(|cell| {
            if let Some(d) = cell.borrow().as_ref() {
                format!("{:?}", d.document())
            } else {
                String::new()
            }
        });

        let global = self.context.global_object();
        let document_val = global
            .get(JsString::from("document"), &mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        if let Some(document_obj) = document_val.as_object() {
            document_obj
                .set(
                    JsString::from("__key__"),
                    JsValue::from(JsString::from(root_key.clone())),
                    false,
                    &mut self.context,
                )
                .map_err(|e| ScriptError::Runtime(e.to_string()))?;

            let registry_val = document_obj
                .get(JsString::from("__node_registry__"), &mut self.context)
                .map_err(|e| ScriptError::Runtime(e.to_string()))?;
            if let Some(registry_obj) = registry_val.as_object() {
                registry_obj
                    .set(
                        JsString::from(root_key),
                        JsValue::from(document_obj.clone()),
                        false,
                        &mut self.context,
                    )
                    .map_err(|e| ScriptError::Runtime(e.to_string()))?;
            }
        }

        // 3. Evaluate the source code.
        let source = Source::from_bytes(src.as_bytes());
        let res_val = self.context.eval(source);

        // 4. Restore DOM
        let restored_dom = CURRENT_DOM.with(|cell| cell.borrow_mut().take());
        if let Some(final_dom) = restored_dom {
            *dom = final_dom;
        }

        KEY_TO_NODE.with(|cell| cell.borrow_mut().clear());

        // 5. Handle evaluation result
        let res_val = res_val.map_err(map_boa_error)?;
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
    pub fn dispatch_event(
        &mut self,
        id: &str,
        event_type: &str,
        dom: &mut Dom,
    ) -> Result<(), ScriptError> {
        // 1. Swap DOM out of `dom` to place in thread-safe RefCell
        let temp_dom = std::mem::take(dom);

        CURRENT_DOM.with(|cell| {
            let mut opt = cell.borrow_mut();
            *opt = Some(temp_dom);
            if let Some(d) = opt.as_ref() {
                KEY_TO_NODE.with(|key_cell| {
                    let mut map = key_cell.borrow_mut();
                    map.clear();
                    index_dom_nodes(d, &mut map);
                });
            }
        });

        // 2. Do the dispatching
        let dispatch_result = self.perform_dispatch_event(id, event_type);

        // 3. Restore DOM
        let restored_dom = CURRENT_DOM.with(|cell| cell.borrow_mut().take());
        if let Some(final_dom) = restored_dom {
            *dom = final_dom;
        }

        KEY_TO_NODE.with(|cell| cell.borrow_mut().clear());

        dispatch_result
    }

    fn perform_dispatch_event(&mut self, id: &str, event_type: &str) -> Result<(), ScriptError> {
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

fn index_dom_nodes(dom: &Dom, key_to_node: &mut HashMap<String, NodeId>) {
    let root = dom.document();
    let mut stack = vec![root];
    while let Some(node_id) = stack.pop() {
        let key = format!("{:?}", node_id);
        key_to_node.insert(key, node_id);
        stack.extend(dom.children(node_id).iter().rev().copied());
    }
}

fn find_element_by_id(dom: &Dom, id: &str) -> Option<NodeId> {
    let root = dom.document();
    let mut stack = vec![root];
    while let Some(node_id) = stack.pop() {
        if let Some(NodeData::Element { attrs, .. }) = dom.data(node_id)
            && attrs.iter().any(|(k, v)| k == "id" && v == id)
        {
            return Some(node_id);
        }
        stack.extend(dom.children(node_id).iter().rev().copied());
    }
    None
}

fn with_dom<F, R>(f: F) -> Result<R, JsError>
where
    F: FnOnce(&mut Dom, &mut HashMap<String, NodeId>) -> R,
{
    CURRENT_DOM.with(|dom_cell| {
        let mut dom_opt = dom_cell.borrow_mut();
        if let Some(dom) = dom_opt.as_mut() {
            KEY_TO_NODE.with(|key_cell| {
                let mut key_map = key_cell.borrow_mut();
                Ok(f(dom, &mut key_map))
            })
        } else {
            Err(JsError::from_opaque(JsValue::from(JsString::from(
                "No active DOM context found",
            ))))
        }
    })
}

fn bridge_create_element(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let tag_name = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let key = with_dom(|dom, key_to_node| {
        let node_id = dom.create_node(NodeData::Element {
            name: tag_name,
            attrs: Vec::new(),
        });

        // TODO(spec): Re-layout on mutation

        let k = format!("{:?}", node_id);
        key_to_node.insert(k.clone(), node_id);
        k
    })?;

    Ok(JsValue::from(JsString::from(key)))
}

fn bridge_get_element_by_id(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let id_val = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let key_opt = with_dom(|dom, key_to_node| {
        if let Some(node_id) = find_element_by_id(dom, &id_val) {
            let k = format!("{:?}", node_id);
            key_to_node.insert(k.clone(), node_id);
            Some(k)
        } else {
            None
        }
    })?;

    if let Some(key) = key_opt {
        Ok(JsValue::from(JsString::from(key)))
    } else {
        Ok(JsValue::null())
    }
}

fn bridge_append_child(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let parent_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let child_key = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    with_dom(|dom, key_to_node| {
        let parent_id = key_to_node.get(&parent_key).copied();
        let child_id = key_to_node.get(&child_key).copied();
        if let (Some(p_id), Some(c_id)) = (parent_id, child_id) {
            dom.append_child(p_id, c_id);
            // TODO(spec): Re-layout on mutation
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_set_attribute(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let attr_name = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let attr_val = if let Some(arg) = args.get(2) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            dom.set_attribute(n_id, &attr_name, &attr_val);
            // TODO(spec): Re-layout on mutation
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_get_attribute(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let attr_name = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let attr_val = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            dom.get_attribute(n_id, &attr_name).map(String::from)
        } else {
            None
        }
    })?;

    if let Some(val) = attr_val {
        Ok(JsValue::from(JsString::from(val)))
    } else {
        Ok(JsValue::null())
    }
}

fn bridge_get_text_content(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let text_val = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            dom.text_content(n_id)
        } else {
            String::new()
        }
    })?;

    Ok(JsValue::from(JsString::from(text_val)))
}

fn bridge_set_text_content(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let text_val = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            let children: Vec<NodeId> = dom.children(n_id).to_vec();
            for child in children {
                dom.remove_child(n_id, child);
            }
            if !text_val.is_empty() {
                let text_id = dom.create_node(NodeData::Text(text_val));
                dom.append_child(n_id, text_id);
            }
            // TODO(spec): Re-layout on mutation
        }
    })?;

    Ok(JsValue::undefined())
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
        let result = host.eval("console.log(");
        assert!(result.is_err());
        assert!(matches!(result, Err(ScriptError::Syntax(_))));
    }

    #[test]
    fn test_experimental_dom_binding() {
        let mut host = BoaHost::new();
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
        let res = host.eval_with_dom("document.getElementById('greeting').textContent", &mut dom);
        assert_eq!(res, Ok("Hello".to_string()));
    }

    #[test]
    fn test_eval_with_dom_missing_id() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();
        let res = host.eval_with_dom("document.getElementById('nonexistent')", &mut dom);
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
        let register_script = "
            window.clicked = false;
            document.getElementById('btn').addEventListener('click', () => {
                window.clicked = true;
            });
        ";
        assert!(host.eval_with_dom(register_script, &mut dom).is_ok());

        assert_eq!(host.dispatch_event("btn", "click", &mut dom), Ok(()));

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
        assert!(host.eval_with_dom(register_script, &mut dom).is_ok());

        assert_eq!(host.dispatch_event("btn", "click", &mut dom), Ok(()));

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
        assert!(host.eval_with_dom("1", &mut dom).is_ok());

        assert_eq!(host.dispatch_event("btn", "click", &mut dom), Ok(()));
        assert_eq!(
            host.dispatch_event("nonexistent", "click", &mut dom),
            Ok(())
        );
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
        assert!(host.eval_with_dom(register_script, &mut dom).is_ok());

        let res = host.dispatch_event("btn", "click", &mut dom);
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

    // TDD tests for DOM Write Bindings (S-48)
    #[test]
    fn test_dom_write_create_element_and_append_child() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        // 1. Initially, root has 0 children
        let root_children_count_before = dom.children(dom.document()).len();
        assert_eq!(root_children_count_before, 0);

        // 2. createElement('div') and appendChild to document
        let script = "
            let div = document.createElement('div');
            document.appendChild(div);
        ";
        assert!(host.eval_with_dom(script, &mut dom).is_ok());

        // 3. Children count of document should be 1
        let root_children = dom.children(dom.document());
        assert_eq!(root_children.len(), 1);

        // 4. Verify node name is "div"
        let child_id = root_children[0];
        match dom.data(child_id) {
            Some(NodeData::Element { name, .. }) => assert_eq!(name, "div"),
            _ => panic!("Expected Element"),
        }
    }

    #[test]
    fn test_dom_write_set_get_attribute() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let div = document.createElement('div');
            div.setAttribute('class', 'main-box');
            div.setAttribute('id', 'content');
            document.appendChild(div);
            
            // Get attributes back
            let c1 = div.getAttribute('class');
            let c2 = div.getAttribute('id');
            [c1, c2].join(',');
        ";
        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(res, Ok("main-box,content".to_string()));

        // Also check attributes from Rust side
        let root_children = dom.children(dom.document());
        let child_id = root_children[0];
        assert_eq!(dom.get_attribute(child_id, "class"), Some("main-box"));
        assert_eq!(dom.get_attribute(child_id, "id"), Some("content"));
    }

    #[test]
    fn test_dom_write_text_content_setter() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let div = document.createElement('div');
            div.textContent = 'Updated content!';
            document.appendChild(div);
            div.textContent;
        ";
        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(res, Ok("Updated content!".to_string()));

        // Check text content from Rust side
        let root_children = dom.children(dom.document());
        let child_id = root_children[0];
        assert_eq!(dom.text_content(child_id), "Updated content!");
    }
}
