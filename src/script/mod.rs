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

thread_local! {
    static LIMITS_ENABLED: RefCell<bool> = const { RefCell::new(true) };
    static MAX_SCRIPT_LENGTH: RefCell<usize> = const { RefCell::new(5000) };
    static MAX_YIELDS: RefCell<usize> = const { RefCell::new(100) };
    static BUDGET_PER_YIELD: RefCell<usize> = const { RefCell::new(1000) };
}

/// Configures whether script execution budgets/limits are enabled.
///
/// By default, limits are enabled to prevent infinite loops and freezing.
/// Disabling limits (i.e. opting-in to complete execution) can be done by
/// calling `set_limits_enabled(false)`.
pub fn set_limits_enabled(enabled: bool) {
    LIMITS_ENABLED.with(|cell| *cell.borrow_mut() = enabled);
}

/// Checks whether script execution budgets/limits are enabled.
pub fn is_limits_enabled() -> bool {
    LIMITS_ENABLED.with(|cell| *cell.borrow())
}

/// Sets the maximum character length for inline scripts when limits are enabled.
/// Scripts exceeding this limit will be truncated or aborted safely.
pub fn set_max_script_length(len: usize) {
    MAX_SCRIPT_LENGTH.with(|cell| *cell.borrow_mut() = len);
}

/// Sets the budget per yield (VM instruction cost threshold).
pub fn set_budget_per_yield(budget: usize) {
    BUDGET_PER_YIELD.with(|cell| *cell.borrow_mut() = budget);
}

/// Sets the maximum number of yields/polls allowed before a script is aborted.
pub fn set_max_yields(yields: usize) {
    MAX_YIELDS.with(|cell| *cell.borrow_mut() = yields);
}

/// Sane recursion limit well under the native stack budget to prevent stack overflow on hostile deep JS (I-6 compliance).
const JS_RECURSION_LIMIT: usize = 256;

/// Generous but finite loop iteration limit to prevent pathological infinite loops from hanging.
const JS_LOOP_ITERATION_LIMIT: u64 = 100_000_000;

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

        // Set safe recursion and loop iteration limits to prevent native stack overflow and infinite loops.
        context
            .runtime_limits_mut()
            .set_recursion_limit(JS_RECURSION_LIMIT);
        context
            .runtime_limits_mut()
            .set_loop_iteration_limit(JS_LOOP_ITERATION_LIMIT);

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
                NativeFunction::from_fn_ptr(bridge_query_selector),
                JsString::from("querySelector"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_query_selector_all),
                JsString::from("querySelectorAll"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_get_elements_by_tag_name),
                JsString::from("getElementsByTagName"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_get_elements_by_class_name),
                JsString::from("getElementsByClassName"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_append_child),
                JsString::from("appendChild"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_remove_child),
                JsString::from("removeChild"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_insert_before),
                JsString::from("insertBefore"),
                3,
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
                NativeFunction::from_fn_ptr(bridge_parent_node),
                JsString::from("parentNode"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_child_nodes),
                JsString::from("childNodes"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_first_child),
                JsString::from("firstChild"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_next_sibling),
                JsString::from("nextSibling"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_last_child),
                JsString::from("lastChild"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_previous_sibling),
                JsString::from("previousSibling"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(add_event_listener),
                JsString::from("addEventListener"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_tag_name),
                JsString::from("tagName"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_node_name),
                JsString::from("nodeName"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_node_type),
                JsString::from("nodeType"),
                1,
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
                        removeChild(child) {
                            if (!child || !child.__key__) {
                                throw new TypeError("child must be a Node");
                            }
                            bridge.removeChild(this.__key__, child.__key__);
                            return child;
                        },
                        insertBefore(newNode, refNode) {
                            if (!newNode || !newNode.__key__) {
                                throw new TypeError("newNode must be a Node");
                            }
                            const refKey = (refNode && refNode.__key__) ? refNode.__key__ : null;
                            bridge.insertBefore(this.__key__, newNode.__key__, refKey);
                            return newNode;
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

                    Object.defineProperty(node, 'className', {
                        get() {
                            return this.getAttribute('class') || '';
                        },
                        set(val) {
                            this.setAttribute('class', String(val));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'parentNode', {
                        get() {
                            return getOrCreateNode(bridge.parentNode(this.__key__));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'childNodes', {
                        get() {
                            const keys = bridge.childNodes(this.__key__);
                            if (!keys) return [];
                            return keys.map(key => getOrCreateNode(key));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'firstChild', {
                        get() {
                            return getOrCreateNode(bridge.firstChild(this.__key__));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'lastChild', {
                        get() {
                            return getOrCreateNode(bridge.lastChild(this.__key__));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'nextSibling', {
                        get() {
                            return getOrCreateNode(bridge.nextSibling(this.__key__));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'previousSibling', {
                        get() {
                            return getOrCreateNode(bridge.previousSibling(this.__key__));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'tagName', {
                        get() {
                            return bridge.tagName(this.__key__);
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'nodeName', {
                        get() {
                            return bridge.nodeName(this.__key__);
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'nodeType', {
                        get() {
                            return bridge.nodeType(this.__key__);
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

                document.querySelector = function(selector) {
                    const key = bridge.querySelector(String(selector));
                    return getOrCreateNode(key);
                };

                document.querySelectorAll = function(selector) {
                    const keys = bridge.querySelectorAll(String(selector));
                    if (!keys) return [];
                    return keys.map(key => getOrCreateNode(key));
                };

                document.getElementsByTagName = function(tagName) {
                    const keys = bridge.getElementsByTagName(String(tagName));
                    if (!keys) return [];
                    return keys.map(key => getOrCreateNode(key));
                };

                document.getElementsByClassName = function(className) {
                    const keys = bridge.getElementsByClassName(String(className));
                    if (!keys) return [];
                    return keys.map(key => getOrCreateNode(key));
                };

                document.appendChild = function(child) {
                    if (!child || !child.__key__) {
                        throw new TypeError("child must be a Node");
                    }
                    bridge.appendChild(this.__key__, child.__key__);
                    return child;
                };

                document.removeChild = function(child) {
                    if (!child || !child.__key__) {
                        throw new TypeError("child must be a Node");
                    }
                    bridge.removeChild(this.__key__, child.__key__);
                    return child;
                };

                document.insertBefore = function(newNode, refNode) {
                    if (!newNode || !newNode.__key__) {
                        throw new TypeError("newNode must be a Node");
                    }
                    const refKey = (refNode && refNode.__key__) ? refNode.__key__ : null;
                    bridge.insertBefore(this.__key__, newNode.__key__, refKey);
                    return newNode;
                };

                document.addEventListener = bridge.addEventListener;

                Object.defineProperty(document, 'parentNode', {
                    get() {
                        return getOrCreateNode(bridge.parentNode(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'childNodes', {
                    get() {
                        const keys = bridge.childNodes(this.__key__);
                        if (!keys) return [];
                        return keys.map(key => getOrCreateNode(key));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'firstChild', {
                    get() {
                        return getOrCreateNode(bridge.firstChild(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'lastChild', {
                    get() {
                        return getOrCreateNode(bridge.lastChild(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'nextSibling', {
                    get() {
                        return getOrCreateNode(bridge.nextSibling(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'previousSibling', {
                    get() {
                        return getOrCreateNode(bridge.previousSibling(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'tagName', {
                    get() {
                        return bridge.tagName(this.__key__);
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'nodeName', {
                    get() {
                        return bridge.nodeName(this.__key__);
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'nodeType', {
                    get() {
                        return bridge.nodeType(this.__key__);
                    },
                    enumerable: true,
                    configurable: true
                });

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
        let is_limit = is_limits_enabled();
        let final_src = if is_limit {
            let max_len = MAX_SCRIPT_LENGTH.with(|cell| *cell.borrow());
            if src.chars().count() > max_len {
                // Safely truncate to the maximum number of characters
                src.chars().take(max_len).collect::<String>()
            } else {
                src.to_string()
            }
        } else {
            src.to_string()
        };

        let res_val = if is_limit {
            let source = Source::from_bytes(final_src.as_bytes());
            match boa_engine::script::Script::parse(source, None, &mut self.context) {
                Ok(script) => {
                    let budget = BUDGET_PER_YIELD.with(|cell| *cell.borrow()) as u32;
                    let max_yields = MAX_YIELDS.with(|cell| *cell.borrow());
                    let mut future =
                        Box::pin(script.evaluate_async_with_budget(&mut self.context, budget));
                    let waker = std::task::Waker::noop();
                    let mut cx = std::task::Context::from_waker(waker);
                    let mut yield_count = 0;

                    loop {
                        match future.as_mut().poll(&mut cx) {
                            std::task::Poll::Ready(res) => {
                                break res;
                            }
                            std::task::Poll::Pending => {
                                yield_count += 1;
                                if yield_count > max_yields {
                                    // Budget exceeded! Abort execution.
                                    break Err(JsError::from_opaque(JsValue::from(
                                        JsString::from("Execution budget exceeded"),
                                    )));
                                }
                            }
                        }
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            let source = Source::from_bytes(final_src.as_bytes());
            self.context.eval(source)
        };

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

fn bridge_query_selector(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let selector_val = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let key_opt = with_dom(|dom, key_to_node| {
        if let Some(node_id) = dom.query_selector(&selector_val) {
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

fn execute_dom_query_to_js_array(
    selector: &str,
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let keys = with_dom(|dom, key_to_node| {
        let mut keys_list = Vec::new();
        for node_id in dom.query_selector_all(selector) {
            let k = format!("{:?}", node_id);
            key_to_node.insert(k.clone(), node_id);
            keys_list.push(k);
        }
        keys_list
    })?;

    let array_constructor = context
        .global_object()
        .get(JsString::from("Array"), context)?;
    let array_obj = array_constructor.as_object().ok_or_else(|| {
        JsError::from_opaque(JsValue::from(JsString::from("Array constructor not found")))
    })?;
    let array_val = array_obj.construct(&[], None, context)?;

    let push_val = array_val.get(JsString::from("push"), context)?;
    if let Some(push_fn) = push_val.as_object() {
        for key in keys {
            push_fn.call(
                &JsValue::from(array_val.clone()),
                &[JsValue::from(JsString::from(key))],
                context,
            )?;
        }
    }

    Ok(JsValue::from(array_val))
}

fn bridge_query_selector_all(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let selector_val = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        String::new()
    };
    execute_dom_query_to_js_array(&selector_val, context)
}

fn bridge_get_elements_by_tag_name(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let tag_name = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        String::new()
    };

    // Case-insensitivity: getElementsByTagName tag matching in HTML is ASCII-case-insensitive.
    // HTML tag names are conventionally lowercase in the parsed DOM.
    let selector = if tag_name == "*" {
        // Special-case the wildcard "*" by passing "*" through to query_selector_all.
        // TODO(spec): Check if query_selector_all supports "*" as a universal selector.
        "*".to_string()
    } else {
        tag_name.to_ascii_lowercase()
    };

    execute_dom_query_to_js_array(&selector, context)
}

fn bridge_get_elements_by_class_name(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let cls = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        String::new()
    };

    let tokens: Vec<&str> = cls
        .split_ascii_whitespace()
        .filter(|t| !t.is_empty())
        .collect();

    if tokens.is_empty() {
        // If there are no class tokens, pass an empty selector which will fail to parse
        // and safely return an empty array.
        execute_dom_query_to_js_array("", context)
    } else {
        // Map ["a", "b"] to ".a.b"
        let selector = tokens
            .iter()
            .map(|t| format!(".{}", t))
            .collect::<Vec<String>>()
            .join("");
        execute_dom_query_to_js_array(&selector, context)
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

fn bridge_remove_child(
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
            dom.remove_child(p_id, c_id);
            // TODO(spec): Re-layout on mutation
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_insert_before(
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

    let reference_key = if let Some(arg) = args.get(2) {
        if arg.is_null() || arg.is_undefined() {
            None
        } else {
            let key_str = arg.to_string(context)?.to_std_string().unwrap_or_default();
            if key_str.is_empty() {
                None
            } else {
                Some(key_str)
            }
        }
    } else {
        None
    };

    with_dom(|dom, key_to_node| {
        let parent_id = key_to_node.get(&parent_key).copied();
        let child_id = key_to_node.get(&child_key).copied();
        let reference_id = match reference_key {
            Some(ref r_key) => key_to_node.get(r_key).copied(),
            None => None,
        };

        if let (Some(p_id), Some(c_id)) = (parent_id, child_id) {
            dom.insert_before(p_id, c_id, reference_id);
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

fn bridge_parent_node(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let parent_key_opt = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied()
            && let Some(p_id) = dom.parent(n_id)
        {
            let k = format!("{:?}", p_id);
            key_to_node.insert(k.clone(), p_id);
            Some(k)
        } else {
            None
        }
    })?;

    if let Some(parent_key) = parent_key_opt {
        Ok(JsValue::from(JsString::from(parent_key)))
    } else {
        Ok(JsValue::null())
    }
}

fn bridge_child_nodes(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let child_keys = with_dom(|dom, key_to_node| {
        let mut keys = Vec::new();
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            for &c_id in dom.children(n_id) {
                let k = format!("{:?}", c_id);
                key_to_node.insert(k.clone(), c_id);
                keys.push(k);
            }
        }
        keys
    })?;

    let array_constructor = context
        .global_object()
        .get(JsString::from("Array"), context)?;
    let array_obj = array_constructor.as_object().ok_or_else(|| {
        JsError::from_opaque(JsValue::from(JsString::from("Array constructor not found")))
    })?;
    let array_val = array_obj.construct(&[], None, context)?;

    let push_val = array_val.get(JsString::from("push"), context)?;
    if let Some(push_fn) = push_val.as_object() {
        for key in child_keys {
            push_fn.call(
                &JsValue::from(array_val.clone()),
                &[JsValue::from(JsString::from(key))],
                context,
            )?;
        }
    }

    Ok(JsValue::from(array_val))
}

fn bridge_first_child(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let first_child_key_opt = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied()
            && let Some(&first_id) = dom.children(n_id).first()
        {
            let k = format!("{:?}", first_id);
            key_to_node.insert(k.clone(), first_id);
            Some(k)
        } else {
            None
        }
    })?;

    if let Some(k) = first_child_key_opt {
        Ok(JsValue::from(JsString::from(k)))
    } else {
        Ok(JsValue::null())
    }
}

fn bridge_next_sibling(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let next_sibling_key_opt = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied()
            && let Some(p_id) = dom.parent(n_id)
        {
            let children = dom.children(p_id);
            if let Some(pos) = children.iter().position(|&id| id == n_id)
                && let Some(&next_id) = children.get(pos + 1)
            {
                let k = format!("{:?}", next_id);
                key_to_node.insert(k.clone(), next_id);
                return Some(k);
            }
        }
        None
    })?;

    if let Some(k) = next_sibling_key_opt {
        Ok(JsValue::from(JsString::from(k)))
    } else {
        Ok(JsValue::null())
    }
}

fn bridge_last_child(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let last_child_key_opt = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied()
            && let Some(&last_id) = dom.children(n_id).last()
        {
            let k = format!("{:?}", last_id);
            key_to_node.insert(k.clone(), last_id);
            Some(k)
        } else {
            None
        }
    })?;

    if let Some(k) = last_child_key_opt {
        Ok(JsValue::from(JsString::from(k)))
    } else {
        Ok(JsValue::null())
    }
}

fn bridge_previous_sibling(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let previous_sibling_key_opt = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied()
            && let Some(p_id) = dom.parent(n_id)
        {
            let children = dom.children(p_id);
            if let Some(pos) = children.iter().position(|&id| id == n_id)
                && let Some(prev_pos) = pos.checked_sub(1)
                && let Some(&prev_id) = children.get(prev_pos)
            {
                let k = format!("{:?}", prev_id);
                key_to_node.insert(k.clone(), prev_id);
                return Some(k);
            }
        }
        None
    })?;

    if let Some(k) = previous_sibling_key_opt {
        Ok(JsValue::from(JsString::from(k)))
    } else {
        Ok(JsValue::null())
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

fn bridge_tag_name(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let tag_name_opt = with_dom(|dom, key_to_node| {
        if let Some(&node_id) = key_to_node.get(&node_key) {
            if let Some(NodeData::Element { name, .. }) = dom.data(node_id) {
                Some(name.to_ascii_uppercase())
            } else {
                None
            }
        } else {
            None
        }
    })?;

    if let Some(tag_name) = tag_name_opt {
        Ok(JsValue::from(JsString::from(tag_name)))
    } else {
        Ok(JsValue::undefined())
    }
}

fn bridge_node_name(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let node_name_opt = with_dom(|dom, key_to_node| {
        if let Some(&node_id) = key_to_node.get(&node_key) {
            match dom.data(node_id) {
                Some(NodeData::Element { name, .. }) => Some(name.to_ascii_uppercase()),
                Some(NodeData::Text(_)) => Some("#text".to_string()),
                Some(NodeData::Comment(_)) => Some("#comment".to_string()),
                Some(NodeData::Document) => Some("#document".to_string()),
                Some(NodeData::Doctype { name, .. }) => Some(name.clone()),
                None => None,
            }
        } else {
            None
        }
    })?;

    if let Some(node_name) = node_name_opt {
        Ok(JsValue::from(JsString::from(node_name)))
    } else {
        Ok(JsValue::undefined())
    }
}

fn bridge_node_type(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let node_type_opt = with_dom(|dom, key_to_node| {
        if let Some(&node_id) = key_to_node.get(&node_key) {
            match dom.data(node_id) {
                Some(NodeData::Element { .. }) => Some(1),
                Some(NodeData::Text(_)) => Some(3),
                Some(NodeData::Comment(_)) => Some(8),
                Some(NodeData::Document) => Some(9),
                Some(NodeData::Doctype { .. }) => Some(10),
                None => None,
            }
        } else {
            None
        }
    })?;

    if let Some(node_type) = node_type_opt {
        Ok(JsValue::from(node_type))
    } else {
        Ok(JsValue::undefined())
    }
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

/// Finds inline `<script>` elements in document order and runs them.
///
/// If a script throws an error, it is caught per-script and does not abort
/// the overall run (I-6 safety). External `src`, `defer`, or `async` scripts
/// are skipped and marked with a spec TODO.
pub fn run_inline_scripts(mut dom: Dom) -> Dom {
    // Collect inline script node IDs in document order (pre-order traversal)
    let mut script_ids = Vec::new();
    for id in dom.descendants(dom.document()) {
        if let Some(NodeData::Element { name, .. }) = dom.data(id)
            && name.eq_ignore_ascii_case("script")
        {
            // TODO(spec): Support external src, defer, or async execution modes.
            let has_src = dom.get_attribute(id, "src").is_some();
            let has_defer = dom.get_attribute(id, "defer").is_some();
            let has_async = dom.get_attribute(id, "async").is_some();

            if has_src || has_defer || has_async {
                continue;
            }

            script_ids.push(id);
        }
    }

    let mut host = BoaHost::new();
    for id in script_ids {
        let src = dom.text_content(id);
        // Execute the script with the current DOM context
        // spec: S-61 Any exception from a throwing script must be caught per-script and not abort the entire run.
        let _ = host.eval_with_dom(&src, &mut dom);
    }

    dom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_recursive_js_does_not_overflow() {
        let mut host = BoaHost::new();
        let recursive_script = "function f() { return f(); } f();";
        let res = host.eval(recursive_script);
        assert!(res.is_err(), "Recursive script did not return an error");
        match res {
            Err(ScriptError::Runtime(msg)) => {
                assert!(
                    msg.contains("RangeError")
                        || msg.contains("recursion")
                        || msg.contains("RuntimeLimit"),
                    "Error message should mention RangeError, RuntimeLimit, or recursion, got: {}",
                    msg
                );
            }
            other => panic!("Expected a Runtime error, got {:?}", other),
        }
    }

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
    fn test_eval_with_dom_tree_navigation() {
        let mut dom = Dom::new();
        let document = dom.document();

        // Let's build a small tree:
        // document -> parent_div -> (child_p1, child_p2)
        let parent_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "parent".to_string())],
        });
        let child1_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![("id".to_string(), "child1".to_string())],
        });
        let child2_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![("id".to_string(), "child2".to_string())],
        });
        dom.append_child(parent_id, child1_id);
        dom.append_child(parent_id, child2_id);
        dom.append_child(document, parent_id);

        let mut host = BoaHost::new();

        // 1. For a parent with two element children, parent.childNodes.length === 2 and parent.firstChild.__key__ equals the first child.
        let res_len = host.eval_with_dom(
            "document.getElementById('parent').childNodes.length",
            &mut dom,
        );
        assert_eq!(res_len, Ok("2".to_string()));

        let res_first_child_matches = host.eval_with_dom(
            "document.getElementById('parent').firstChild === document.getElementById('child1')",
            &mut dom,
        );
        assert_eq!(res_first_child_matches, Ok("true".to_string()));

        // 2. child.parentNode.__key__ equals the parent's key (round-trips back).
        let res_parent_round_trip = host.eval_with_dom(
            "document.getElementById('child1').parentNode === document.getElementById('parent')",
            &mut dom,
        );
        assert_eq!(res_parent_round_trip, Ok("true".to_string()));

        // 3. firstChild.nextSibling.__key__ equals the second child; the last child's nextSibling === null.
        let res_next_sibling = host.eval_with_dom(
            "document.getElementById('child1').nextSibling === document.getElementById('child2')",
            &mut dom,
        );
        assert_eq!(res_next_sibling, Ok("true".to_string()));

        let res_last_child_next_sibling = host.eval_with_dom(
            "document.getElementById('child2').nextSibling === null",
            &mut dom,
        );
        assert_eq!(res_last_child_next_sibling, Ok("true".to_string()));

        // 4. A node with no children has firstChild === null and childNodes.length === 0; the root/document's relevant case behaves sanely.
        let res_no_child_first = host.eval_with_dom(
            "document.getElementById('child1').firstChild === null",
            &mut dom,
        );
        assert_eq!(res_no_child_first, Ok("true".to_string()));

        let res_no_child_len = host.eval_with_dom(
            "document.getElementById('child1').childNodes.length",
            &mut dom,
        );
        assert_eq!(res_no_child_len, Ok("0".to_string()));

        // Document's child is parent, document has no parentNode, nextSibling is null
        let res_doc_parent = host.eval_with_dom("document.parentNode === null", &mut dom);
        assert_eq!(res_doc_parent, Ok("true".to_string()));

        let res_doc_next_sibling = host.eval_with_dom("document.nextSibling === null", &mut dom);
        assert_eq!(res_doc_next_sibling, Ok("true".to_string()));

        let res_doc_first_child = host.eval_with_dom(
            "document.firstChild === document.getElementById('parent')",
            &mut dom,
        );
        assert_eq!(res_doc_first_child, Ok("true".to_string()));

        // 5. Symmetric accessors: lastChild and previousSibling
        let res_last_child = host.eval_with_dom(
            "document.getElementById('parent').lastChild === document.getElementById('child2')",
            &mut dom,
        );
        assert_eq!(res_last_child, Ok("true".to_string()));

        let res_prev_sibling = host.eval_with_dom(
            "document.getElementById('child2').previousSibling === document.getElementById('child1')",
            &mut dom,
        );
        assert_eq!(res_prev_sibling, Ok("true".to_string()));

        let res_first_child_prev = host.eval_with_dom(
            "document.getElementById('child1').previousSibling === null",
            &mut dom,
        );
        assert_eq!(res_first_child_prev, Ok("true".to_string()));

        let res_no_child_last = host.eval_with_dom(
            "document.getElementById('child1').lastChild === null",
            &mut dom,
        );
        assert_eq!(res_no_child_last, Ok("true".to_string()));

        let res_doc_last_child = host.eval_with_dom(
            "document.lastChild === document.getElementById('parent')",
            &mut dom,
        );
        assert_eq!(res_doc_last_child, Ok("true".to_string()));

        let res_doc_prev_sibling =
            host.eval_with_dom("document.previousSibling === null", &mut dom);
        assert_eq!(res_doc_prev_sibling, Ok("true".to_string()));
    }

    #[test]
    fn test_eval_with_dom_js_created_tree_navigation() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        // Build tree dynamically using script
        let script = r#"
            const container = document.createElement('div');
            container.setAttribute('id', 'js-container');
            const item1 = document.createElement('span');
            item1.setAttribute('id', 'js-item1');
            const item2 = document.createElement('span');
            item2.setAttribute('id', 'js-item2');

            container.appendChild(item1);
            container.appendChild(item2);
            document.appendChild(container);

            // Verify tree navigation on newly created nodes in JS
            const verification = {
                containerParent: container.parentNode === document,
                item1Parent: item1.parentNode === container,
                item2Parent: item2.parentNode === container,
                firstChild: container.firstChild === item1,
                lastChild: container.lastChild === item2,
                nextSibling: item1.nextSibling === item2,
                previousSibling: item2.previousSibling === item1,
                lastChildNextSibling: item2.nextSibling === null,
                firstChildPreviousSibling: item1.previousSibling === null,
                childNodesLength: container.childNodes.length,
                documentLastChild: document.lastChild === container,
                documentPreviousSibling: document.previousSibling === null
            };
            JSON.stringify(verification);
        "#;

        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(
            res,
            Ok(
                r#"{"containerParent":true,"item1Parent":true,"item2Parent":true,"firstChild":true,"lastChild":true,"nextSibling":true,"previousSibling":true,"lastChildNextSibling":true,"firstChildPreviousSibling":true,"childNodesLength":2,"documentLastChild":true,"documentPreviousSibling":true}"#
                    .to_string()
            )
        );
    }

    #[test]
    fn test_eval_with_dom_descriptors() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = r#"
            const div = document.createElement('div');
            const desc = Object.getOwnPropertyDescriptor(div, 'parentNode');
            const descChildren = Object.getOwnPropertyDescriptor(div, 'childNodes');
            const verification = {
                enumerable: desc.enumerable === true && descChildren.enumerable === true,
                configurable: desc.configurable === true && descChildren.configurable === true,
                readOnly: desc.set === undefined && descChildren.set === undefined
            };
            JSON.stringify(verification);
        "#;

        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(
            res,
            Ok(r#"{"enumerable":true,"configurable":true,"readOnly":true}"#.to_string())
        );
    }

    #[test]
    fn test_dom_node_identity_accessors() {
        let mut dom = Dom::new();
        let document = dom.document();

        // Let's manually add some comment and doctype nodes as well to verify them.
        let doctype_id = dom.create_node(NodeData::Doctype {
            name: "html".to_string(),
            public_id: "".to_string(),
            system_id: "".to_string(),
        });
        dom.append_child(document, doctype_id);

        let parent_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "parent-div".to_string())],
        });
        dom.append_child(document, parent_id);

        let comment_id = dom.create_node(NodeData::Comment("this is a comment".to_string()));
        dom.append_child(parent_id, comment_id);

        let mut host = BoaHost::new();

        // 1. Check tagName, nodeName, nodeType of document and doctype
        let script_doc_doctype = r#"
            (function() {
                const docTypeNode = document.firstChild; // doctype is first child
                const verification = {
                    documentNodeName: document.nodeName,
                    documentNodeType: document.nodeType,
                    documentTagName: document.tagName,
                    doctypeNodeName: docTypeNode.nodeName,
                    doctypeNodeType: docTypeNode.nodeType,
                    doctypeTagName: docTypeNode.tagName,
                };
                return JSON.stringify(verification);
            })()
        "#;
        let res1 = host.eval_with_dom(script_doc_doctype, &mut dom);
        assert_eq!(
            res1,
            Ok(r##"{"documentNodeName":"#document","documentNodeType":9,"doctypeNodeName":"html","doctypeNodeType":10}"##.to_string())
        );

        // 2. Check createElement elements and querySelector fetched elements
        let script_elements = r#"
            (function() {
                const div = document.createElement('div');
                const p = document.createElement('p');
                p.textContent = 'hello';
                const textNode = p.firstChild;
                
                const parentDiv = document.getElementById('parent-div');
                const commentNode = parentDiv.firstChild;

                const verification = {
                    divTagName: div.tagName,
                    divNodeName: div.nodeName,
                    divNodeType: div.nodeType,
                    pTagName: p.tagName,
                    pNodeName: p.nodeName,
                    pNodeType: p.nodeType,
                    textNodeName: textNode.nodeName,
                    textNodeType: textNode.nodeType,
                    textTagName: textNode.tagName,
                    parentDivTagName: parentDiv.tagName,
                    parentDivNodeName: parentDiv.nodeName,
                    parentDivNodeType: parentDiv.nodeType,
                    commentNodeName: commentNode.nodeName,
                    commentNodeType: commentNode.nodeType,
                    commentTagName: commentNode.tagName,
                };
                return JSON.stringify(verification);
            })()
        "#;
        let res2 = host.eval_with_dom(script_elements, &mut dom);
        assert_eq!(
            res2,
            Ok(r##"{"divTagName":"DIV","divNodeName":"DIV","divNodeType":1,"pTagName":"P","pNodeName":"P","pNodeType":1,"textNodeName":"#text","textNodeType":3,"parentDivTagName":"DIV","parentDivNodeName":"DIV","parentDivNodeType":1,"commentNodeName":"#comment","commentNodeType":8}"##.to_string())
        );
    }

    #[test]
    fn test_eval_with_dom_missing_id() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();
        let res = host.eval_with_dom("document.getElementById('nonexistent')", &mut dom);
        assert_eq!(res, Ok("null".to_string()));
    }

    #[test]
    fn test_eval_with_dom_query_selector() {
        let mut dom = Dom::new();
        let document = dom.document();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("class".to_string(), "x".to_string())],
        });
        let p_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![],
        });
        let text_id = dom.create_node(NodeData::Text("Target Paragraph".to_string()));
        dom.append_child(p_id, text_id);
        dom.append_child(div_id, p_id);
        dom.append_child(document, div_id);

        let mut host = BoaHost::new();
        let res = host.eval_with_dom("document.querySelector('div.x > p').textContent", &mut dom);
        assert_eq!(res, Ok("Target Paragraph".to_string()));
    }

    #[test]
    fn test_eval_with_dom_query_selector_all() {
        let mut dom = Dom::new();
        let document = dom.document();

        let a1_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![("href".to_string(), "https://foo".to_string())],
        });
        let a2_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![("href".to_string(), "https://bar".to_string())],
        });
        dom.append_child(document, a1_id);
        dom.append_child(document, a2_id);

        let mut host = BoaHost::new();
        let res_len = host.eval_with_dom("document.querySelectorAll('a').length", &mut dom);
        assert_eq!(res_len, Ok("2".to_string()));

        let res_content = host.eval_with_dom(
            "[document.querySelectorAll('a')[0].getAttribute('href'), document.querySelectorAll('a')[1].getAttribute('href')].join(',')",
            &mut dom,
        );
        assert_eq!(res_content, Ok("https://foo,https://bar".to_string()));
    }

    #[test]
    fn test_eval_with_dom_query_selector_non_matching_and_invalid() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        // Non-matching selector
        let res_null = host.eval_with_dom("document.querySelector('.nonexistent')", &mut dom);
        assert_eq!(res_null, Ok("null".to_string()));

        let res_empty_arr =
            host.eval_with_dom("document.querySelectorAll('.nonexistent').length", &mut dom);
        assert_eq!(res_empty_arr, Ok("0".to_string()));

        // Invalid selector should return null and empty array respectively without panicking
        let res_invalid_qs = host.eval_with_dom("document.querySelector('div > > p')", &mut dom);
        assert_eq!(res_invalid_qs, Ok("null".to_string()));

        let res_invalid_qsa =
            host.eval_with_dom("document.querySelectorAll('div > > p').length", &mut dom);
        assert_eq!(res_invalid_qsa, Ok("0".to_string()));
    }

    #[test]
    fn test_eval_with_dom_get_elements_by_tag_name() {
        let mut dom = Dom::new();
        let document = dom.document();

        let p1_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![("id".to_string(), "p1".to_string())],
        });
        let p2_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![("id".to_string(), "p2".to_string())],
        });
        let text1 = dom.create_node(NodeData::Text("First".to_string()));
        let text2 = dom.create_node(NodeData::Text("Second".to_string()));
        dom.append_child(p1_id, text1);
        dom.append_child(p2_id, text2);
        dom.append_child(document, p1_id);
        dom.append_child(document, p2_id);

        let mut host = BoaHost::new();

        // Check normal case
        let res_len = host.eval_with_dom("document.getElementsByTagName('p').length", &mut dom);
        assert_eq!(res_len, Ok("2".to_string()));

        let res_content = host.eval_with_dom(
            "[document.getElementsByTagName('p')[0].textContent, document.getElementsByTagName('p')[1].textContent].join(',')",
            &mut dom,
        );
        assert_eq!(res_content, Ok("First,Second".to_string()));

        // Case-insensitivity test (HTML tag name matching is ASCII case-insensitive)
        let res_case = host.eval_with_dom("document.getElementsByTagName('P').length", &mut dom);
        assert_eq!(res_case, Ok("2".to_string()));

        // Wildcard tag name "*" test
        let res_wildcard =
            host.eval_with_dom("document.getElementsByTagName('*').length", &mut dom);
        assert_eq!(res_wildcard, Ok("2".to_string()));

        // Non-existent tags should return an empty array (length 0), not null/undefined
        let res_nonexistent =
            host.eval_with_dom("document.getElementsByTagName('div').length", &mut dom);
        assert_eq!(res_nonexistent, Ok("0".to_string()));
    }

    #[test]
    fn test_eval_with_dom_get_elements_by_class_name() {
        let mut dom = Dom::new();
        let document = dom.document();

        let div1_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("class".to_string(), "foo bar".to_string())],
        });
        let div2_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("class".to_string(), "foo baz".to_string())],
        });
        dom.append_child(document, div1_id);
        dom.append_child(document, div2_id);

        let mut host = BoaHost::new();

        // Single class matching
        let res_single =
            host.eval_with_dom("document.getElementsByClassName('foo').length", &mut dom);
        assert_eq!(res_single, Ok("2".to_string()));

        let res_baz = host.eval_with_dom("document.getElementsByClassName('baz').length", &mut dom);
        assert_eq!(res_baz, Ok("1".to_string()));

        // Multiple classes (multi-token compound class selector)
        let res_multi = host.eval_with_dom(
            "document.getElementsByClassName('foo bar').length",
            &mut dom,
        );
        assert_eq!(res_multi, Ok("1".to_string()));

        // Class list with extra spaces and different token order
        let res_spaces = host.eval_with_dom(
            "document.getElementsByClassName('  bar   foo  ').length",
            &mut dom,
        );
        assert_eq!(res_spaces, Ok("1".to_string()));

        // Non-matching class returns empty array (length 0)
        let res_nonexistent =
            host.eval_with_dom("document.getElementsByClassName('qux').length", &mut dom);
        assert_eq!(res_nonexistent, Ok("0".to_string()));

        // Empty class name returns empty array (length 0)
        let res_empty = host.eval_with_dom("document.getElementsByClassName('').length", &mut dom);
        assert_eq!(res_empty, Ok("0".to_string()));
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

        // Check attributes from Rust side
        let root_children = dom.children(dom.document());
        let child_id = root_children[0];
        assert_eq!(dom.get_attribute(child_id, "class"), Some("main-box"));
        assert_eq!(dom.get_attribute(child_id, "id"), Some("content"));
    }

    #[test]
    fn test_element_classname() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            // 1. A freshly created element has className === '' by default.
            let div1 = document.createElement('div');
            let res1 = div1.className;

            // 2. Setting el.className = 'foo bar' makes el.getAttribute('class') === 'foo bar'.
            let div2 = document.createElement('div');
            div2.className = 'foo bar';
            let res2 = div2.getAttribute('class');

            // 3. After el.setAttribute('class', 'x y'), reading el.className === 'x y'.
            let div3 = document.createElement('div');
            div3.setAttribute('class', 'x y');
            let res3 = div3.className;

            // 4. Round-trip via document.getElementById/querySelector also reflects className.
            let div4 = document.createElement('div');
            div4.setAttribute('id', 'mydiv');
            div4.className = 'test-class';
            document.appendChild(div4);

            let retrieved = document.getElementById('mydiv');
            let qsa = document.querySelector('.test-class');
            let res4 = [retrieved.className, qsa.className].join(',');

            [res1, res2, res3, res4].join('|');
        ";
        assert_eq!(
            host.eval_with_dom(script, &mut dom),
            Ok("|foo bar|x y|test-class,test-class".to_string())
        );
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

    #[test]
    fn test_dom_write_insert_before_and_remove_child() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let parent = document.createElement('div');
            document.appendChild(parent);
            
            let c1 = document.createElement('span');
            c1.textContent = 'one';
            parent.appendChild(c1);
            
            let c2 = document.createElement('span');
            c2.textContent = 'two';
            parent.appendChild(c2);
            
            // 1. Insert 'inserted' before c2
            let new_child = document.createElement('span');
            new_child.textContent = 'inserted';
            parent.insertBefore(new_child, c2);
            
            // 2. Insert 'last' before null
            let last_child = document.createElement('span');
            last_child.textContent = 'last';
            parent.insertBefore(last_child, null);
            
            // 3. Remove c1
            parent.removeChild(c1);
        ";
        assert!(host.eval_with_dom(script, &mut dom).is_ok());

        // Check DOM state from Rust side
        let doc_children = dom.children(dom.document());
        assert_eq!(doc_children.len(), 1);
        let parent_id = doc_children[0];
        let parent_children = dom.children(parent_id);
        assert_eq!(parent_children.len(), 3);

        assert_eq!(dom.text_content(parent_children[0]), "inserted");
        assert_eq!(dom.text_content(parent_children[1]), "two");
        assert_eq!(dom.text_content(parent_children[2]), "last");
    }

    #[test]
    fn test_run_inline_scripts_basic() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "x".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("initial".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        let script_text = dom.create_node(NodeData::Text(
            "document.getElementById('x').textContent='hi'".to_string(),
        ));
        dom.append_child(script_id, script_text);
        dom.append_child(document, script_id);

        let mutated_dom = run_inline_scripts(dom);
        assert_eq!(mutated_dom.text_content(element_id), "hi");
    }

    #[test]
    fn test_run_inline_scripts_throwing_ignored() {
        let mut dom = Dom::new();
        let document = dom.document();

        // 1. First script element throws an error
        let script_id1 = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        let script_text1 = dom.create_node(NodeData::Text(
            "throw new Error('Some panic code');".to_string(),
        ));
        dom.append_child(script_id1, script_text1);
        dom.append_child(document, script_id1);

        // 2. Second script element works fine and modifies an element
        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("original".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        let script_id2 = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        let script_text2 = dom.create_node(NodeData::Text(
            "document.getElementById('target').textContent='recovered'".to_string(),
        ));
        dom.append_child(script_id2, script_text2);
        dom.append_child(document, script_id2);

        // This must run successfully without panic and execute the second script!
        let mutated_dom = run_inline_scripts(dom);
        assert_eq!(mutated_dom.text_content(element_id), "recovered");
    }

    #[test]
    fn test_run_inline_scripts_skipped_modes() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("original".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        // Script 1: has src attribute (external script)
        let script_src = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![("src".to_string(), "app.js".to_string())],
        });
        let text_src = dom.create_node(NodeData::Text(
            "document.getElementById('target').textContent='src_run'".to_string(),
        ));
        dom.append_child(script_src, text_src);
        dom.append_child(document, script_src);

        // Script 2: has defer attribute
        let script_defer = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![("defer".to_string(), "".to_string())],
        });
        let text_defer = dom.create_node(NodeData::Text(
            "document.getElementById('target').textContent='defer_run'".to_string(),
        ));
        dom.append_child(script_defer, text_defer);
        dom.append_child(document, script_defer);

        // Script 3: has async attribute
        let script_async = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![("async".to_string(), "".to_string())],
        });
        let text_async = dom.create_node(NodeData::Text(
            "document.getElementById('target').textContent='async_run'".to_string(),
        ));
        dom.append_child(script_async, text_async);
        dom.append_child(document, script_async);

        // Run scripts: all three must be skipped and target must remain "original"
        let mutated_dom = run_inline_scripts(dom);
        assert_eq!(mutated_dom.text_content(element_id), "original");
    }

    #[test]
    fn test_run_inline_scripts_ordering() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        // Script 1: sets it to "A"
        let script_id1 = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        let script_text1 = dom.create_node(NodeData::Text(
            "document.getElementById('target').textContent += 'A';".to_string(),
        ));
        dom.append_child(script_id1, script_text1);
        dom.append_child(document, script_id1);

        // Script 2: appends "B"
        let script_id2 = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        let script_text2 = dom.create_node(NodeData::Text(
            "document.getElementById('target').textContent += 'B';".to_string(),
        ));
        dom.append_child(script_id2, script_text2);
        dom.append_child(document, script_id2);

        // Run: output should be "AB"
        let mutated_dom = run_inline_scripts(dom);
        assert_eq!(mutated_dom.text_content(element_id), "AB");
    }

    #[test]
    fn test_script_budget_infinite_loop() {
        // Enforce default limits
        set_limits_enabled(true);
        set_max_yields(5); // low limit for quick test
        set_budget_per_yield(100);

        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("original".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        // Infinite loop script.
        // It tries to set textContent to 'looping' inside the loop,
        // but it should hit the budget and get aborted.
        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        let script_text = dom.create_node(NodeData::Text(
            "while(true) { document.getElementById('target').textContent='looping'; }".to_string(),
        ));
        dom.append_child(script_id, script_text);
        dom.append_child(document, script_id);

        // Run scripts: must NOT hang or panic!
        let _mutated_dom = run_inline_scripts(dom);

        // Restore defaults
        set_limits_enabled(true);
        set_max_yields(100);
        set_budget_per_yield(1000);
        set_max_script_length(5000);
    }

    #[test]
    fn test_script_budget_length_limit() {
        set_limits_enabled(true);
        set_max_script_length(20);

        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("original".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        // This script is 54 characters. With limit of 20, it is truncated to:
        // "document.getElementB" which has a syntax error.
        // It must NOT panic!
        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        let script_text = dom.create_node(NodeData::Text(
            "document.getElementById('target').textContent='changed';".to_string(),
        ));
        dom.append_child(script_id, script_text);
        dom.append_child(document, script_id);

        let mutated_dom = run_inline_scripts(dom);
        assert_eq!(mutated_dom.text_content(element_id), "original");

        // Restore defaults
        set_max_script_length(5000);
    }

    #[test]
    fn test_script_budget_opt_out() {
        // Turn off limits
        set_limits_enabled(false);

        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });

        let text_id = dom.create_node(NodeData::Text("original".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        // A long script that would have been truncated if limits were on
        set_max_script_length(20);

        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        let script_text = dom.create_node(NodeData::Text(
            "document.getElementById('target').textContent='changed';".to_string(),
        ));
        dom.append_child(script_id, script_text);
        dom.append_child(document, script_id);

        let mutated_dom = run_inline_scripts(dom);
        assert_eq!(mutated_dom.text_content(element_id), "changed");

        // Restore defaults
        set_limits_enabled(true);
        set_max_script_length(5000);
    }
}
