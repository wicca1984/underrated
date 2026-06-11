//! Scripting module providing JavaScript execution via the Boa engine.
//!
//! This module implements the `ScriptHost` port, allowing the browser engine
//! to execute scripts. The current implementation uses the `boa_engine` crate.

pub mod storage;
pub mod timer;
pub mod xhr;

use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;
use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsString, JsValue, NativeFunction, Source};
use std::cell::RefCell;
use std::collections::HashMap;

pub mod event;
pub mod navigator;

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
    document_url: Option<String>,
}

thread_local! {
    static CURRENT_DOM: RefCell<Option<Dom>> = const { RefCell::new(None) };
    static KEY_TO_NODE: RefCell<HashMap<String, NodeId>> = RefCell::new(HashMap::new());
    static CURRENT_STYLES: RefCell<Option<HashMap<NodeId, crate::style::ComputedStyle>>> = const { RefCell::new(None) };
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

        // Setup Web Storage bindings (localStorage / sessionStorage)
        storage::setup_storage(&mut context);

        // Setup timer built-ins (setTimeout, clearTimeout, setInterval, clearInterval)
        let _ = timer::register_timer_builtins(&mut context);

        // Register XMLHttpRequest stub (t0242)
        if let Err(e) = xhr::register_xhr(&mut context) {
            eprintln!("Failed to register XMLHttpRequest: {:?}", e);
        }

        Self {
            context,
            document_url: None,
        }
    }

    fn setup_experimental_dom(context: &mut Context) {
        let _ = context.register_global_class::<event::EventTarget>();
        let _ = context.register_global_class::<event::Event>();

        let bridge = ObjectInitializer::new(context)
            .function(
                NativeFunction::from_fn_ptr(bridge_create_element),
                JsString::from("createElement"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_create_text_node),
                JsString::from("createTextNode"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_has_attribute),
                JsString::from("hasAttribute"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_replace_child),
                JsString::from("replaceChild"),
                3,
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
                NativeFunction::from_fn_ptr(bridge_matches),
                JsString::from("matches"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_closest),
                JsString::from("closest"),
                2,
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
                NativeFunction::from_fn_ptr(bridge_remove_attribute),
                JsString::from("removeAttribute"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_toggle_attribute),
                JsString::from("toggleAttribute"),
                3,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_get_attribute_names),
                JsString::from("getAttributeNames"),
                1,
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
                NativeFunction::from_fn_ptr(bridge_get_inner_html),
                JsString::from("getInnerHTML"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_set_inner_html),
                JsString::from("setInnerHTML"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_get_outer_html),
                JsString::from("getOuterHTML"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_set_outer_html),
                JsString::from("setOuterHTML"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_parent_node),
                JsString::from("parentNode"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_contains),
                JsString::from("contains"),
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
                NativeFunction::from_fn_ptr(bridge_first_element_child),
                JsString::from("firstElementChild"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_last_element_child),
                JsString::from("lastElementChild"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_next_element_sibling),
                JsString::from("nextElementSibling"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_previous_element_sibling),
                JsString::from("previousElementSibling"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_children),
                JsString::from("children"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_child_element_count),
                JsString::from("childElementCount"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_parent_element),
                JsString::from("parentElement"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(event::add_event_listener),
                JsString::from("addEventListener"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(event::remove_event_listener),
                JsString::from("removeEventListener"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(event::dispatch_event),
                JsString::from("dispatchEvent"),
                1,
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
            .function(
                NativeFunction::from_fn_ptr(bridge_clone_node),
                JsString::from("cloneNode"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_get_computed_style_value),
                JsString::from("getComputedStyleValue"),
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

        let navigator = navigator::create_navigator(context);
        let _ = context.register_global_property(
            JsString::from("navigator"),
            navigator,
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

                window.__document_location__ = {
                    href: "",
                    protocol: "",
                    host: "",
                    hostname: "",
                    port: "",
                    pathname: "",
                    search: "",
                    hash: "",
                    origin: ""
                };

                const locationObj = {
                    get href() { return window.__document_location__.href; },
                    get protocol() { return window.__document_location__.protocol; },
                    get host() { return window.__document_location__.host; },
                    get hostname() { return window.__document_location__.hostname; },
                    get port() { return window.__document_location__.port; },
                    get pathname() { return window.__document_location__.pathname; },
                    get search() { return window.__document_location__.search; },
                    get hash() { return window.__document_location__.hash; },
                    get origin() { return window.__document_location__.origin; },

                    set href(val) {
                        // TODO(spec): wire location assignment to navigation pipeline (follow-up)
                    },
                    assign(url) {
                        // TODO(spec): wire location assignment to navigation pipeline (follow-up)
                    },
                    replace(url) {
                        // TODO(spec): wire location assignment to navigation pipeline (follow-up)
                    },
                    reload() {
                        // TODO(spec): wire location assignment to navigation pipeline (follow-up)
                    },

                    toString() {
                        return this.href;
                    }
                };

                window.location = locationObj;
                document.location = locationObj;

                class DOMException extends Error {
                    constructor(message, name) {
                        super(message);
                        this.name = name || "DOMException";
                    }
                }
                window.DOMException = DOMException;

                function getTokens(element) {
                    const value = element.getAttribute('class');
                    if (!value) return [];
                    const rawTokens = value.split(/[\t\n\r\f ]+/);
                    const tokens = [];
                    const seen = new Set();
                    for (const t of rawTokens) {
                        if (t !== "" && !seen.has(t)) {
                            seen.add(t);
                            tokens.push(t);
                        }
                    }
                    return tokens;
                }

                function setTokens(element, tokens) {
                    element.setAttribute('class', tokens.join(' '));
                }

                function validateToken(token) {
                    if (token === undefined || token === null) {
                        token = String(token);
                    }
                    if (token === "") {
                        throw new DOMException("The token provided must not be empty", "SyntaxError");
                    }
                    if (/[\t\n\r\f ]/.test(token)) {
                        throw new DOMException("The token provided contains whitespace", "InvalidCharacterError");
                    }
                }

                class DOMTokenList {
                    constructor(element) {
                        this.__element__ = element;
                        return new Proxy(this, {
                            get(target, prop, receiver) {
                                if (typeof prop === 'string' && /^\d+$/.test(prop)) {
                                    const index = parseInt(prop, 10);
                                    const tokens = getTokens(target.__element__);
                                    return index < tokens.length ? tokens[index] : undefined;
                                }
                                const value = Reflect.get(target, prop, receiver);
                                if (typeof value === 'function') {
                                    return value.bind(target);
                                }
                                return value;
                            }
                        });
                    }

                    get length() {
                        return getTokens(this.__element__).length;
                    }

                    get value() {
                        return this.__element__.getAttribute('class') || '';
                    }

                    set value(val) {
                        this.__element__.setAttribute('class', String(val));
                    }

                    item(index) {
                        const idx = Number(index) >>> 0;
                        const tokens = getTokens(this.__element__);
                        if (idx >= tokens.length) {
                            return null;
                        }
                        return tokens[idx];
                    }

                    contains(token) {
                        validateToken(token);
                        const tokens = getTokens(this.__element__);
                        return tokens.includes(String(token));
                    }

                    add(...tokens) {
                        for (const t of tokens) {
                            validateToken(t);
                        }
                        const current = getTokens(this.__element__);
                        let changed = false;
                        for (const t of tokens) {
                            const s = String(t);
                            if (!current.includes(s)) {
                                current.push(s);
                                changed = true;
                            }
                        }
                        if (changed) {
                            setTokens(this.__element__, current);
                        }
                    }

                    remove(...tokens) {
                        for (const t of tokens) {
                            validateToken(t);
                        }
                        const current = getTokens(this.__element__);
                        let changed = false;
                        for (const t of tokens) {
                            const s = String(t);
                            const idx = current.indexOf(s);
                            if (idx !== -1) {
                                current.splice(idx, 1);
                                changed = true;
                            }
                        }
                        if (changed) {
                            setTokens(this.__element__, current);
                        }
                    }

                    toggle(token, force) {
                        validateToken(token);
                        const s = String(token);
                        const current = getTokens(this.__element__);
                        const idx = current.indexOf(s);
                        const present = idx !== -1;

                        if (arguments.length >= 2) {
                            if (force) {
                                if (!present) {
                                    current.push(s);
                                    setTokens(this.__element__, current);
                                }
                                return true;
                            } else {
                                if (present) {
                                    current.splice(idx, 1);
                                    setTokens(this.__element__, current);
                                }
                                return false;
                            }
                        } else {
                            if (present) {
                                current.splice(idx, 1);
                                setTokens(this.__element__, current);
                                return false;
                            } else {
                                current.push(s);
                                setTokens(this.__element__, current);
                                return true;
                            }
                        }
                    }

                    replace(oldToken, newToken) {
                        validateToken(oldToken);
                        validateToken(newToken);
                        const current = getTokens(this.__element__);
                        const index = current.indexOf(String(oldToken));
                        if (index === -1) {
                            return false;
                        }
                        const replaceIndex = current.indexOf(String(newToken));
                        if (replaceIndex !== -1) {
                            if (replaceIndex > index) {
                                current[index] = String(newToken);
                                current.splice(replaceIndex, 1);
                            } else {
                                current.splice(index, 1);
                            }
                        } else {
                            current[index] = String(newToken);
                        }
                        setTokens(this.__element__, current);
                        return true;
                    }

                    toString() {
                        return this.__element__.getAttribute('class') || '';
                    }
                }
                window.DOMTokenList = DOMTokenList;

                function camelToKebab(str) {
                    return str.replace(/[A-Z]/g, match => '-' + match.toLowerCase());
                }

                function parseStyleString(styleStr) {
                    const decls = [];
                    if (!styleStr) return decls;
                    const parts = [];
                    let currentPart = "";
                    let inDoubleQuotes = false;
                    let inSingleQuotes = false;
                    let parenDepth = 0;
                    for (let i = 0; i < styleStr.length; i++) {
                        const char = styleStr[i];
                        if (char === '"' && !inSingleQuotes) {
                            inDoubleQuotes = !inDoubleQuotes;
                            currentPart += char;
                        } else if (char === "'" && !inDoubleQuotes) {
                            inSingleQuotes = !inSingleQuotes;
                            currentPart += char;
                        } else if (char === '(' && !inDoubleQuotes && !inSingleQuotes) {
                            parenDepth++;
                            currentPart += char;
                        } else if (char === ')' && !inDoubleQuotes && !inSingleQuotes) {
                            if (parenDepth > 0) parenDepth--;
                            currentPart += char;
                        } else if (char === ';' && !inDoubleQuotes && !inSingleQuotes && parenDepth === 0) {
                            parts.push(currentPart);
                            currentPart = "";
                        } else {
                            currentPart += char;
                        }
                    }
                    if (currentPart.trim() !== "") {
                        parts.push(currentPart);
                    }

                    for (const part of parts) {
                        const trimmed = part.trim();
                        if (trimmed === "") continue;
                        const colonIndex = trimmed.indexOf(":");
                        if (colonIndex === -1) continue;
                        const name = trimmed.substring(0, colonIndex).trim().toLowerCase();
                        const value = trimmed.substring(colonIndex + 1).trim();
                        if (name !== "") {
                            decls.push({ name, value });
                        }
                    }
                    return decls;
                }

                function serializeStyleDecls(decls) {
                    return decls.map(d => `${d.name}: ${d.value}`).join('; ') + (decls.length > 0 ? ';' : '');
                }

                class CSSStyleDeclaration {
                    constructor(element) {
                        this.__element__ = element;
                        return new Proxy(this, {
                            get(target, prop, receiver) {
                                if (prop === 'cssText') {
                                    return target.cssText;
                                }
                                if (prop in target || typeof prop === 'symbol') {
                                    const value = Reflect.get(target, prop, receiver);
                                    if (typeof value === 'function') {
                                        return value.bind(target);
                                    }
                                    return value;
                                }
                                if (typeof prop === 'string') {
                                    // // TODO(spec): index getter is skipped in this minimal version
                                    const kebab = camelToKebab(prop);
                                    return target.getPropertyValue(kebab);
                                }
                                return undefined;
                            },
                            set(target, prop, value, receiver) {
                                if (prop === 'cssText') {
                                    target.cssText = value;
                                    return true;
                                }
                                if (typeof prop === 'string') {
                                    const kebab = camelToKebab(prop);
                                    target.setProperty(kebab, value);
                                    return true;
                                }
                                return Reflect.set(target, prop, value, receiver);
                            }
                        });
                    }

                    get cssText() {
                        return this.__element__.getAttribute('style') || '';
                    }

                    set cssText(val) {
                        // // TODO(spec): Shorthand expansion (margin -> margin-top/...), !important priority parsing, computed styles, and units normalization are skipped in CSSOM-lite.
                        if (val === undefined || val === null) {
                            val = "";
                        }
                        const strVal = String(val).trim();
                        if (strVal === "") {
                            this.__element__.removeAttribute('style');
                        } else {
                            this.__element__.setAttribute('style', strVal);
                        }
                    }

                    setProperty(name, value) {
                        // // TODO(spec): priority/!important parsing and shorthand expansion are skipped in CSSOM-lite.
                        if (value === undefined || value === null) {
                            value = "";
                        }
                        const strVal = String(value).trim();
                        const lowerName = String(name).trim().toLowerCase();
                        if (lowerName === "") return;

                        const styleStr = this.__element__.getAttribute('style') || '';
                        const decls = parseStyleString(styleStr);

                        if (strVal === "") {
                            const filtered = decls.filter(d => d.name !== lowerName);
                            const newStyle = serializeStyleDecls(filtered);
                            if (newStyle === "") {
                                this.__element__.removeAttribute('style');
                            } else {
                                this.__element__.setAttribute('style', newStyle);
                            }
                        } else {
                            let found = false;
                            for (const d of decls) {
                                if (d.name === lowerName) {
                                    d.value = strVal;
                                    found = true;
                                    break;
                                }
                            }
                            if (!found) {
                                decls.push({ name: lowerName, value: strVal });
                            }
                            this.__element__.setAttribute('style', serializeStyleDecls(decls));
                        }
                    }

                    getPropertyValue(name) {
                        // // TODO(spec): Computed/used style resolution and units normalization are skipped in CSSOM-lite.
                        const lowerName = String(name).trim().toLowerCase();
                        if (lowerName === "") return "";
                        const styleStr = this.__element__.getAttribute('style') || '';
                        const decls = parseStyleString(styleStr);
                        for (const d of decls) {
                            if (d.name === lowerName) {
                                return d.value;
                            }
                        }
                        return "";
                    }

                    removeProperty(name) {
                        // // TODO(spec): Shorthand expansion and computed style resolution are skipped in CSSOM-lite.
                        const lowerName = String(name).trim().toLowerCase();
                        if (lowerName === "") return "";
                        const styleStr = this.__element__.getAttribute('style') || '';
                        const decls = parseStyleString(styleStr);
                        let removedValue = "";
                        const filtered = [];
                        for (const d of decls) {
                            if (d.name === lowerName) {
                                removedValue = d.value;
                            } else {
                                filtered.push(d);
                            }
                        }
                        if (removedValue !== "") {
                            const newStyle = serializeStyleDecls(filtered);
                            if (newStyle === "") {
                                this.__element__.removeAttribute('style');
                            } else {
                                this.__element__.setAttribute('style', newStyle);
                            }
                        }
                        return removedValue;
                    }
                }
                window.CSSStyleDeclaration = CSSStyleDeclaration;

                function kebabToCamel(str) {
                    return str.replace(/-([a-z])/g, (match, char) => char.toUpperCase());
                }

                class DOMStringMap {
                    constructor(element) {
                        this.__element__ = element;
                        return new Proxy(this, {
                            get(target, prop, receiver) {
                                if (typeof prop !== 'string') {
                                    return Reflect.get(target, prop, receiver);
                                }
                                if (prop === '__element__') {
                                    return target.__element__;
                                }
                                const attrName = 'data-' + camelToKebab(prop);
                                const hasAttr = target.__element__.hasAttribute(attrName);
                                if (!hasAttr && (prop in target)) {
                                    const val = Reflect.get(target, prop, receiver);
                                    if (typeof val === 'function') {
                                        return val.bind(target);
                                    }
                                    return val;
                                }
                                const attrVal = target.__element__.getAttribute(attrName);
                                return attrVal === null ? undefined : attrVal;
                            },
                            set(target, prop, value, receiver) {
                                if (typeof prop !== 'string') {
                                    return Reflect.set(target, prop, value, receiver);
                                }
                                if (prop === '__element__') {
                                    return Reflect.set(target, prop, value, receiver);
                                }
                                if (/-[a-z]/.test(prop)) {
                                    throw new DOMException("Property name must not contain a hyphen followed by a lowercase letter", "SyntaxError");
                                }
                                // TODO(spec): Validate that prop is a valid XML name, throwing "InvalidCharacterError" if not.
                                const attrName = 'data-' + camelToKebab(prop);
                                target.__element__.setAttribute(attrName, String(value));
                                return true;
                            },
                            has(target, prop) {
                                if (typeof prop !== 'string') {
                                    return Reflect.has(target, prop);
                                }
                                if (prop === '__element__') {
                                    return true;
                                }
                                const attrName = 'data-' + camelToKebab(prop);
                                if (target.__element__.hasAttribute(attrName)) {
                                    return true;
                                }
                                return prop in target;
                            },
                            deleteProperty(target, prop) {
                                if (typeof prop !== 'string') {
                                    return Reflect.deleteProperty(target, prop);
                                }
                                if (prop === '__element__') {
                                    return false;
                                }
                                if (/-[a-z]/.test(prop)) {
                                    throw new DOMException("Property name must not contain a hyphen followed by a lowercase letter", "SyntaxError");
                                }
                                const attrName = 'data-' + camelToKebab(prop);
                                target.__element__.removeAttribute(attrName);
                                return true;
                            },
                            ownKeys(target) {
                                const names = target.__element__.getAttributeNames();
                                const keys = [];
                                const seen = new Set();
                                for (const name of names) {
                                    if (name.startsWith('data-')) {
                                        const remainder = name.slice(5);
                                        const camel = kebabToCamel(remainder);
                                        if (!seen.has(camel)) {
                                            seen.add(camel);
                                            keys.push(camel);
                                        }
                                    }
                                }
                                return keys;
                            },
                            getOwnPropertyDescriptor(target, prop) {
                                if (typeof prop !== 'string') {
                                    return Reflect.getOwnPropertyDescriptor(target, prop);
                                }
                                if (prop === '__element__') {
                                    return undefined;
                                }
                                const attrName = 'data-' + camelToKebab(prop);
                                if (target.__element__.hasAttribute(attrName)) {
                                    const value = target.__element__.getAttribute(attrName);
                                    return {
                                        enumerable: true,
                                        configurable: true,
                                        value: value,
                                        writable: true
                                    };
                                }
                                return undefined;
                            }
                        });
                    }
                }
                window.DOMStringMap = DOMStringMap;

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
                        replaceChild(newChild, oldChild) {
                            if (!newChild || !newChild.__key__) {
                                throw new TypeError("newChild must be a Node");
                            }
                            if (!oldChild || !oldChild.__key__) {
                                throw new TypeError("oldChild must be a Node");
                            }
                            bridge.replaceChild(this.__key__, newChild.__key__, oldChild.__key__);
                            return oldChild;
                        },
                        cloneNode(deep) {
                            const isDeep = deep !== undefined ? Boolean(deep) : false;
                            const clonedKey = bridge.cloneNode(this.__key__, isDeep);
                            return getOrCreateNode(clonedKey);
                        },
                        setAttribute(name, value) {
                            bridge.setAttribute(this.__key__, String(name), String(value));
                        },
                        getAttribute(name) {
                            return bridge.getAttribute(this.__key__, String(name));
                        },
                        hasAttribute(name) {
                            return bridge.hasAttribute(this.__key__, String(name));
                        },
                        removeAttribute(name) {
                            bridge.removeAttribute(this.__key__, String(name));
                        },
                        toggleAttribute(name, force) {
                            if (arguments.length >= 2) {
                                return bridge.toggleAttribute(this.__key__, String(name), Boolean(force));
                            } else {
                                return bridge.toggleAttribute(this.__key__, String(name));
                            }
                        },
                        getAttributeNames() {
                            return bridge.getAttributeNames(this.__key__);
                        },
                        matches(selector) {
                            if (this.nodeType !== 1) return false;
                            return bridge.matches(this.__key__, String(selector));
                        },
                        closest(selector) {
                            if (this.nodeType !== 1) return null;
                            const key = bridge.closest(this.__key__, String(selector));
                            return getOrCreateNode(key);
                        }
                    };

                    Object.setPrototypeOf(node, EventTarget.prototype);
                    node.addEventListener = bridge.addEventListener;
                    node.removeEventListener = bridge.removeEventListener;
                    node.dispatchEvent = bridge.dispatchEvent;

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

                    Object.defineProperty(node, 'innerHTML', {
                        get() {
                            if (this.nodeType !== 1) return undefined;
                            return bridge.getInnerHTML(this.__key__);
                        },
                        set(val) {
                            if (this.nodeType !== 1) return;
                            bridge.setInnerHTML(this.__key__, String(val));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'outerHTML', {
                        get() {
                            if (this.nodeType !== 1) return undefined;
                            return bridge.getOuterHTML(this.__key__);
                        },
                        set(val) {
                            if (this.nodeType !== 1) return;
                            bridge.setOuterHTML(this.__key__, String(val));
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

                    // TODO(spec): input.value dirty-value-flag / live IDL value vs content attribute (see HTML spec)
                    Object.defineProperty(node, 'value', {
                        get() {
                            return this.getAttribute('value') || '';
                        },
                        set(val) {
                            this.setAttribute('value', String(val));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'classList', {
                        get() {
                            if (!this.__classList__) {
                                this.__classList__ = new DOMTokenList(this);
                            }
                            return this.__classList__;
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'style', {
                        get() {
                            if (!this.__style__) {
                                this.__style__ = new CSSStyleDeclaration(this);
                            }
                            return this.__style__;
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'dataset', {
                        get() {
                            if (this.nodeType !== 1) return undefined;
                            if (!this.__dataset__) {
                                this.__dataset__ = new DOMStringMap(this);
                            }
                            return this.__dataset__;
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

                    Object.defineProperty(node, 'firstElementChild', {
                        get() {
                            return getOrCreateNode(bridge.firstElementChild(this.__key__));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'lastElementChild', {
                        get() {
                            return getOrCreateNode(bridge.lastElementChild(this.__key__));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'nextElementSibling', {
                        get() {
                            return getOrCreateNode(bridge.nextElementSibling(this.__key__));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'previousElementSibling', {
                        get() {
                            return getOrCreateNode(bridge.previousElementSibling(this.__key__));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'children', {
                        get() {
                            const keys = bridge.children(this.__key__);
                            if (!keys) return [];
                            return keys.map(key => getOrCreateNode(key));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'childElementCount', {
                        get() {
                            return bridge.childElementCount(this.__key__);
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'parentElement', {
                        get() {
                            return getOrCreateNode(bridge.parentElement(this.__key__));
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

                    Object.defineProperty(node, 'before', {
                        value: function(newNode) {
                            // TODO(spec): ChildNode.before/after — DOMString args and variadic nodes not yet supported
                            if (!this.parentNode) return;
                            if (!newNode || !newNode.__key__) {
                                throw new TypeError("newNode must be a Node");
                            }
                            this.parentNode.insertBefore(newNode, this);
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    Object.defineProperty(node, 'after', {
                        value: function(newNode) {
                            // TODO(spec): ChildNode.before/after — DOMString args and variadic nodes not yet supported
                            if (!this.parentNode) return;
                            if (!newNode || !newNode.__key__) {
                                throw new TypeError("newNode must be a Node");
                            }
                            this.parentNode.insertBefore(newNode, this.nextSibling);
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    // TODO(spec): ParentNode.append()/prepend() v1 — Node and string (->Text) args only; DocumentFragment expansion and other edge cases out of scope.
                    Object.defineProperty(node, 'append', {
                        value: function(...args) {
                            for (let i = 0; i < args.length; i++) {
                                let arg = args[i];
                                let n;
                                if (typeof arg === 'string') {
                                    n = document.createTextNode(arg);
                                } else if (arg && arg.__key__) {
                                    n = arg;
                                } else {
                                    throw new TypeError("Argument must be a Node or a string");
                                }
                                this.appendChild(n);
                            }
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    Object.defineProperty(node, 'prepend', {
                        value: function(...args) {
                            const refNode = this.firstChild;
                            for (let i = 0; i < args.length; i++) {
                                let arg = args[i];
                                let n;
                                if (typeof arg === 'string') {
                                    n = document.createTextNode(arg);
                                } else if (arg && arg.__key__) {
                                    n = arg;
                                } else {
                                    throw new TypeError("Argument must be a Node or a string");
                                }
                                this.insertBefore(n, refNode);
                            }
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    Object.defineProperty(node, 'replaceChildren', {
                        value: function(...args) {
                            // TODO(spec): ParentNode.replaceChildren() v1 — Node/DOMString args; DocumentFragment expansion out of scope.
                            const validatedNodes = [];
                            for (let i = 0; i < args.length; i++) {
                                let arg = args[i];
                                if (typeof arg === 'string') {
                                    validatedNodes.push(document.createTextNode(arg));
                                } else if (arg && arg.__key__) {
                                    validatedNodes.push(arg);
                                } else {
                                    throw new TypeError("Argument must be a Node or a string");
                                }
                            }

                            while (this.firstChild) {
                                this.removeChild(this.firstChild);
                            }

                            for (let i = 0; i < validatedNodes.length; i++) {
                                this.appendChild(validatedNodes[i]);
                            }
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    // TODO(spec): Node.contains() v1 — ancestor-or-self walk via parentNode; cross-document / shadow-tree edge cases out of scope.
                    Object.defineProperty(node, 'contains', {
                        value: function(other) {
                            if (!other || !other.__key__) {
                                return false;
                            }
                            let cur = other;
                            while (cur) {
                                if (cur.__key__ === this.__key__) {
                                    return true;
                                }
                                cur = cur.parentNode;
                            }
                            return false;
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    // TODO(spec): ChildNode.remove() v1 — single-node removal only; DocumentFragment / cross-document host edge cases out of scope.
                    Object.defineProperty(node, 'remove', {
                        value: function() {
                            if (!this.parentNode) return;
                            this.parentNode.removeChild(this);
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    Object.defineProperty(node, 'replaceWith', {
                        value: function(newNode) {
                            // TODO(spec): ChildNode.replaceWith() v1 — single Node arg only; variadic nodes and DOMString
                            // arguments are out of scope (same limitation as before()/after()).
                            if (!this.parentNode) return;
                            if (!newNode || !newNode.__key__) {
                                throw new TypeError("newNode must be a Node");
                            }
                            this.parentNode.insertBefore(newNode, this);
                            this.parentNode.removeChild(this);
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    registry[key] = node;
                    return node;
                }

                window.__getOrCreateNode = getOrCreateNode;

                document.createElement = function(tagName) {
                    const key = bridge.createElement(String(tagName));
                    return getOrCreateNode(key);
                };

                document.createTextNode = function(data) {
                    const key = bridge.createTextNode(data !== undefined ? String(data) : "");
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

                document.replaceChild = function(newChild, oldChild) {
                    if (!newChild || !newChild.__key__) {
                        throw new TypeError("newChild must be a Node");
                    }
                    if (!oldChild || !oldChild.__key__) {
                        throw new TypeError("oldChild must be a Node");
                    }
                    bridge.replaceChild(this.__key__, newChild.__key__, oldChild.__key__);
                    return oldChild;
                };

                document.cloneNode = function(deep) {
                    const isDeep = deep !== undefined ? Boolean(deep) : false;
                    const clonedKey = bridge.cloneNode(this.__key__, isDeep);
                    return getOrCreateNode(clonedKey);
                };

                Object.setPrototypeOf(document, EventTarget.prototype);
                document.addEventListener = bridge.addEventListener;
                document.removeEventListener = bridge.removeEventListener;
                document.dispatchEvent = bridge.dispatchEvent;

                window.addEventListener = bridge.addEventListener;
                window.removeEventListener = bridge.removeEventListener;
                window.dispatchEvent = bridge.dispatchEvent;

                document.__readyState__ = 'loading';
                Object.defineProperty(document, 'readyState', {
                    get() {
                        return this.__readyState__ || 'loading';
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'parentNode', {
                    get() {
                        return getOrCreateNode(bridge.parentNode(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'contains', {
                    value: function(other) {
                        if (!other || !other.__key__) {
                            return false;
                        }
                        let cur = other;
                        while (cur) {
                            if (cur.__key__ === this.__key__) {
                                return true;
                            }
                            cur = cur.parentNode;
                        }
                        return false;
                    },
                    enumerable: false,
                    configurable: true,
                    writable: true
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

                // spec: https://dom.spec.whatwg.org/#dom-document-documentelement
                // TODO(spec): getElementsByTagName-based lookup does not enforce that the root <html> is the document element or a direct child of document.
                Object.defineProperty(document, 'documentElement', {
                    get() {
                        return document.getElementsByTagName("html")[0] || null;
                    },
                    enumerable: true,
                    configurable: true
                });

                // spec: https://dom.spec.whatwg.org/#dom-document-body
                // TODO(spec): getElementsByTagName-based lookup does not enforce the "must be a child of documentElement" / frameset rules.
                Object.defineProperty(document, 'body', {
                    get() {
                        return document.getElementsByTagName("body")[0] || null;
                    },
                    enumerable: true,
                    configurable: true
                });

                // spec: https://dom.spec.whatwg.org/#dom-document-head
                // TODO(spec): getElementsByTagName-based lookup does not enforce the "must be a child of documentElement" / head rules.
                Object.defineProperty(document, 'head', {
                    get() {
                        return document.getElementsByTagName("head")[0] || null;
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

                Object.defineProperty(document, 'firstElementChild', {
                    get() {
                        return getOrCreateNode(bridge.firstElementChild(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'lastElementChild', {
                    get() {
                        return getOrCreateNode(bridge.lastElementChild(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'nextElementSibling', {
                    get() {
                        return getOrCreateNode(bridge.nextElementSibling(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'previousElementSibling', {
                    get() {
                        return getOrCreateNode(bridge.previousElementSibling(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'children', {
                    get() {
                        const keys = bridge.children(this.__key__);
                        if (!keys) return [];
                        return keys.map(key => getOrCreateNode(key));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'childElementCount', {
                    get() {
                        return bridge.childElementCount(this.__key__);
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(document, 'parentElement', {
                    get() {
                        return getOrCreateNode(bridge.parentElement(this.__key__));
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

                window.getComputedStyle = function(element) {
                    if (!element || !element.__key__) {
                        throw new TypeError("getComputedStyle requires an element");
                    }
                    return {
                        getPropertyValue(propertyName) {
                            return bridge.getComputedStyleValue(element.__key__, String(propertyName));
                        }
                    };
                };
            })();
        "#;

        let source = Source::from_bytes(setup_code.as_bytes());
        if let Err(e) = context.eval(source) {
            eprintln!("Failed to initialize DOM bindings: {:?}", e);
        }
    }

    /// Sets the current document's URL.
    pub fn set_document_url(&mut self, url: &str) {
        self.document_url = Some(url.to_string());

        let parsed_url = crate::url::Url::parse(url);

        let mut href = String::new();
        let mut protocol = String::new();
        let mut host = String::new();
        let mut hostname = String::new();
        let mut port = String::new();
        let mut pathname = String::new();
        let mut search = String::new();
        let mut hash = String::new();
        let mut origin = String::new();

        if let Ok(u) = parsed_url {
            href = u.serialize();
            protocol = format!("{}:", u.scheme);
            if let Some(h) = &u.host {
                hostname = h.clone();
                if let Some(p) = u.port {
                    port = p.to_string();
                    host = format!("{}:{}", h, p);
                } else {
                    host = h.clone();
                }
            }
            pathname = u.path.clone();
            if let Some(q) = &u.query
                && !q.is_empty()
            {
                search = format!("?{}", q);
            }
            if let Some(f) = &u.fragment
                && !f.is_empty()
            {
                hash = format!("#{}", f);
            }
            if u.scheme == "file" {
                origin = "file://".to_string();
            } else if let Some(h) = &u.host {
                let mut orig = format!("{}://{}", u.scheme, h);
                if let Some(p) = u.port {
                    orig.push_str(&format!(":{}", p));
                }
                origin = orig;
            } else {
                origin = "null".to_string();
            }
        }

        // Get window.__document_location__ and update its fields
        let global = self.context.global_object().clone();
        if let Ok(doc_loc_val) =
            global.get(JsString::from("__document_location__"), &mut self.context)
            && let Some(doc_loc_obj) = doc_loc_val.as_object()
        {
            let _ = doc_loc_obj.set(
                JsString::from("href"),
                JsValue::from(JsString::from(href)),
                false,
                &mut self.context,
            );
            let _ = doc_loc_obj.set(
                JsString::from("protocol"),
                JsValue::from(JsString::from(protocol)),
                false,
                &mut self.context,
            );
            let _ = doc_loc_obj.set(
                JsString::from("host"),
                JsValue::from(JsString::from(host)),
                false,
                &mut self.context,
            );
            let _ = doc_loc_obj.set(
                JsString::from("hostname"),
                JsValue::from(JsString::from(hostname)),
                false,
                &mut self.context,
            );
            let _ = doc_loc_obj.set(
                JsString::from("port"),
                JsValue::from(JsString::from(port)),
                false,
                &mut self.context,
            );
            let _ = doc_loc_obj.set(
                JsString::from("pathname"),
                JsValue::from(JsString::from(pathname)),
                false,
                &mut self.context,
            );
            let _ = doc_loc_obj.set(
                JsString::from("search"),
                JsValue::from(JsString::from(search)),
                false,
                &mut self.context,
            );
            let _ = doc_loc_obj.set(
                JsString::from("hash"),
                JsValue::from(JsString::from(hash)),
                false,
                &mut self.context,
            );
            let _ = doc_loc_obj.set(
                JsString::from("origin"),
                JsValue::from(JsString::from(origin)),
                false,
                &mut self.context,
            );
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

    /// Evaluates the given script with the provided DOM context and computed styles.
    ///
    /// The computed styles are exposed via window.getComputedStyle().
    pub fn eval_with_dom_and_styles(
        &mut self,
        src: &str,
        dom: &mut Dom,
        styles: &HashMap<NodeId, crate::style::ComputedStyle>,
    ) -> Result<String, ScriptError> {
        CURRENT_STYLES.with(|cell| {
            *cell.borrow_mut() = Some(styles.clone());
        });

        let res = self.eval_with_dom(src, dom);

        CURRENT_STYLES.with(|cell| {
            *cell.borrow_mut() = None;
        });

        res
    }

    fn dispatch_single_lifecycle_event(
        &mut self,
        target: &JsValue,
        event_type: &str,
    ) -> Result<(), ScriptError> {
        let global = self.context.global_object().clone();
        let event_constructor = global
            .get(JsString::from("Event"), &mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let event_constructor_obj = event_constructor
            .as_object()
            .ok_or_else(|| ScriptError::Runtime("Event constructor not found".to_string()))?;
        let event_obj = event_constructor_obj
            .construct(
                &[JsValue::from(JsString::from(event_type))],
                None,
                &mut self.context,
            )
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;

        let event_js_val = JsValue::from(event_obj.clone());

        let mut listeners_to_call = Vec::new();
        if let Some(target_obj) = target.as_object() {
            let events_prop = JsString::from("__events__");
            if let Ok(events_val) = target_obj.get(events_prop, &mut self.context)
                && let Some(events_obj) = events_val.as_object()
            {
                let type_prop = JsString::from(event_type);
                if let Ok(handlers_val) = events_obj.get(type_prop, &mut self.context)
                    && let Some(handlers_obj) = handlers_val.as_object()
                    && let Ok(length_val) =
                        handlers_obj.get(JsString::from("length"), &mut self.context)
                {
                    let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);
                    for i in 0..length {
                        if let Ok(handler) = handlers_obj.get(i, &mut self.context) {
                            listeners_to_call.push(handler);
                        }
                    }
                }
            }
        }

        for listener in listeners_to_call {
            if let Some(event) = event_obj.downcast_ref::<event::Event>() {
                *event.target.borrow_mut() = Some(target.clone());
                *event.current_target.borrow_mut() = Some(target.clone());
            }

            if let Some(callable) = listener.as_object() {
                let res = if callable.is_callable() {
                    callable.call(
                        target,
                        std::slice::from_ref(&event_js_val),
                        &mut self.context,
                    )
                } else if let Ok(handle_event_val) =
                    callable.get(JsString::from("handleEvent"), &mut self.context)
                    && let Some(handle_event_callable) = handle_event_val.as_object()
                    && handle_event_callable.is_callable()
                {
                    handle_event_callable.call(
                        &listener,
                        std::slice::from_ref(&event_js_val),
                        &mut self.context,
                    )
                } else {
                    Ok(JsValue::undefined())
                };

                if let Err(err) = res {
                    eprintln!(
                        "Error in lifecycle listener for event {}: {:?}",
                        event_type, err
                    );
                }
            }
        }

        if let Some(event) = event_obj.downcast_ref::<event::Event>() {
            *event.current_target.borrow_mut() = None;
        }

        Ok(())
    }

    /// Transition readyState to 'complete' and dispatch DOMContentLoaded and load events.
    pub fn dispatch_lifecycle_events(
        &mut self,
        dom: &mut Dom,
        styles: &HashMap<NodeId, crate::style::ComputedStyle>,
    ) -> Result<(), ScriptError> {
        // 1. Swap DOM out of `dom` to place in thread-safe RefCell, set styles
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

        CURRENT_STYLES.with(|cell| {
            *cell.borrow_mut() = Some(styles.clone());
        });

        // 2. Set readyState to 'complete'. This is best-effort: any failure must NOT
        // early-return, because `dom` has been taken out via `mem::take` above and the
        // caller would otherwise receive an empty `Dom` (control must always reach the
        // step-5 restore below).
        let global = self.context.global_object().clone();
        let document_val = match global.get(JsString::from("document"), &mut self.context) {
            Ok(val) => val,
            Err(_) => JsValue::undefined(),
        };
        if let Some(document_obj) = document_val.as_object() {
            let _ = document_obj.set(
                JsString::from("__readyState__"),
                JsValue::from(JsString::from("complete")),
                false,
                &mut self.context,
            );
        }

        // 3. Dispatch DOMContentLoaded at document then window
        let _ = self.dispatch_single_lifecycle_event(&document_val, "DOMContentLoaded");
        let window_val = JsValue::from(global.clone());
        let _ = self.dispatch_single_lifecycle_event(&window_val, "DOMContentLoaded");

        // 4. Dispatch load at document then window
        let _ = self.dispatch_single_lifecycle_event(&document_val, "load");
        let _ = self.dispatch_single_lifecycle_event(&window_val, "load");

        // 5. Restore DOM and clear styles
        let restored_dom = CURRENT_DOM.with(|cell| cell.borrow_mut().take());
        if let Some(final_dom) = restored_dom {
            *dom = final_dom;
        }

        KEY_TO_NODE.with(|cell| cell.borrow_mut().clear());

        CURRENT_STYLES.with(|cell| {
            *cell.borrow_mut() = None;
        });

        Ok(())
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
                // Construct a real Event object
                let event_ctor_val = self
                    .context
                    .global_object()
                    .get(JsString::from("Event"), &mut self.context)
                    .map_err(|e| ScriptError::Runtime(e.to_string()))?;
                let event_ctor = event_ctor_val.as_object().ok_or_else(|| {
                    ScriptError::Runtime("Event constructor not found".to_string())
                })?;
                let event_val = event_ctor
                    .construct(
                        &[JsValue::from(JsString::from(event_type))],
                        None,
                        &mut self.context,
                    )
                    .map_err(|e| ScriptError::Runtime(e.to_string()))?;

                if let Some(event) = event_val.downcast_ref::<event::Event>() {
                    *event.target.borrow_mut() = Some(JsValue::from(elem_obj.clone()));
                    *event.current_target.borrow_mut() = Some(JsValue::from(elem_obj.clone()));
                }

                handler_obj
                    .call(
                        &JsValue::from(elem_obj.clone()),
                        &[JsValue::from(event_val)],
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

fn bridge_matches(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(false));
    };

    let selector_val = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(false));
    };

    let is_match = with_dom(|dom, key_to_node| {
        let node_id = match key_to_node.get(&node_key).copied() {
            Some(id) => id,
            None => return false,
        };

        let selector_list = match crate::selector::parse_selector_list(&selector_val) {
            Ok(list) => list,
            Err(_) => return false,
        };

        crate::selector::matches(&selector_list, dom, node_id)
    })?;

    Ok(JsValue::from(is_match))
}

fn bridge_closest(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let selector_val = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let key_opt = with_dom(|dom, key_to_node| {
        let mut curr_node_id = match key_to_node.get(&node_key).copied() {
            Some(id) => id,
            None => return None,
        };

        let selector_list = match crate::selector::parse_selector_list(&selector_val) {
            Ok(list) => list,
            Err(_) => return None,
        };

        loop {
            if crate::selector::matches(&selector_list, dom, curr_node_id) {
                let k = format!("{:?}", curr_node_id);
                key_to_node.insert(k.clone(), curr_node_id);
                return Some(k);
            }
            if let Some(parent_id) = dom.parent(curr_node_id) {
                curr_node_id = parent_id;
            } else {
                break;
            }
        }
        None
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

fn bridge_create_text_node(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let data = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        String::new()
    };

    let key = with_dom(|dom, key_to_node| {
        let node_id = dom.create_node(NodeData::Text(data));
        let k = format!("{:?}", node_id);
        key_to_node.insert(k.clone(), node_id);
        k
    })?;

    Ok(JsValue::from(JsString::from(key)))
}

fn bridge_has_attribute(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(false));
    };

    let attr_name = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(false));
    };

    let has_attr = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            dom.get_attribute(n_id, &attr_name).is_some()
        } else {
            false
        }
    })?;

    Ok(JsValue::from(has_attr))
}

fn bridge_replace_child(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let parent_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let new_child_key = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let old_child_key = if let Some(arg) = args.get(2) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    with_dom(|dom, key_to_node| {
        let parent_id = key_to_node.get(&parent_key).copied();
        let new_child_id = key_to_node.get(&new_child_key).copied();
        let old_child_id = key_to_node.get(&old_child_key).copied();

        if let (Some(p_id), Some(new_cid), Some(old_cid)) = (parent_id, new_child_id, old_child_id)
        {
            dom.insert_before(p_id, new_cid, Some(old_cid));
            dom.remove_child(p_id, old_cid);
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

fn bridge_remove_attribute(
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

    with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            dom.remove_attribute(n_id, &attr_name);
            // TODO(spec): Re-layout on mutation
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_toggle_attribute(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(false));
    };

    let attr_name = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(false));
    };

    // If force argument is provided:
    let force = args.get(2).map(|arg| arg.to_boolean());

    let result = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            let present = dom.get_attribute(n_id, &attr_name).is_some();
            let should_be_present = match force {
                Some(f) => f,
                None => !present,
            };

            if should_be_present {
                if !present {
                    dom.set_attribute(n_id, &attr_name, "");
                }
                true
            } else {
                if present {
                    dom.remove_attribute(n_id, &attr_name);
                }
                false
            }
        } else {
            false
        }
    })?;

    Ok(JsValue::from(result))
}

fn bridge_get_attribute_names(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let attr_names = with_dom(|dom, key_to_node| {
        let mut names = Vec::new();
        if let Some(n_id) = key_to_node.get(&node_key).copied()
            && let Some(NodeData::Element { attrs, .. }) = dom.data(n_id)
        {
            for (name, _) in attrs {
                names.push(name.clone());
            }
        }
        names
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
        for name in attr_names {
            push_fn.call(
                &JsValue::from(array_val.clone()),
                &[JsValue::from(JsString::from(name))],
                context,
            )?;
        }
    }

    Ok(JsValue::from(array_val))
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

fn copy_node_to_dom_recursive(src_dom: &Dom, src_node_id: NodeId, dest_dom: &mut Dom) -> NodeId {
    let node_data = if let Some(data) = src_dom.data(src_node_id) {
        data.clone()
    } else {
        NodeData::Comment(String::new())
    };

    let dest_node_id = dest_dom.create_node(node_data);

    let children: Vec<NodeId> = src_dom.children(src_node_id).to_vec();
    for child_id in children {
        let cloned_child_id = copy_node_to_dom_recursive(src_dom, child_id, dest_dom);
        dest_dom.append_child(dest_node_id, cloned_child_id);
    }

    dest_node_id
}

fn bridge_get_inner_html(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let inner_html = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            let mut result = String::new();
            for &child_id in dom.children(n_id) {
                result.push_str(&dom.serialize(child_id));
            }
            result
        } else {
            String::new()
        }
    })?;

    Ok(JsValue::from(JsString::from(inner_html)))
}

fn bridge_set_inner_html(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let html_val = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            // Remove existing children
            let children: Vec<NodeId> = dom.children(n_id).to_vec();
            for child in children {
                dom.remove_child(n_id, child);
            }

            // Parse the HTML fragment (using wrapped body).
            // TODO(spec): We emulate fragment parsing by wrapping in a body tag and parsing the document.
            let wrapped_html = format!("<body>{}</body>", html_val);
            let temp_dom = crate::html::parse_document(crate::encoding::InputStream::from_utf8(
                wrapped_html.as_bytes(),
            ));

            // Find the <body> element in temp_dom.
            let body_id_opt =
                temp_dom
                    .descendants(temp_dom.document())
                    .into_iter()
                    .find(|&node_id| {
                        if let Some(crate::dom::NodeData::Element { name, .. }) =
                            temp_dom.data(node_id)
                        {
                            name == "body"
                        } else {
                            false
                        }
                    });

            if let Some(body_id) = body_id_opt {
                let temp_children = temp_dom.children(body_id).to_vec();
                for temp_child_id in temp_children {
                    let dest_child_id = copy_node_to_dom_recursive(&temp_dom, temp_child_id, dom);
                    dom.append_child(n_id, dest_child_id);
                }
            }
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_get_outer_html(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let outer_html = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            dom.serialize(n_id)
        } else {
            String::new()
        }
    })?;

    Ok(JsValue::from(JsString::from(outer_html)))
}

fn bridge_set_outer_html(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let html_val = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            let parent = match dom.parent(n_id) {
                Some(p) => p,
                None => {
                    // TODO(spec): outerHTML on a parentless element is a no-op here; the HTML spec requires throwing a NoModificationAllowedError DOMException. Fragment parsing is emulated via a wrapped <body> like setInnerHTML.
                    return;
                }
            };

            // Parse the HTML fragment (using wrapped body).
            let wrapped_html = format!("<body>{}</body>", html_val);
            let temp_dom = crate::html::parse_document(crate::encoding::InputStream::from_utf8(
                wrapped_html.as_bytes(),
            ));

            // Find the <body> element in temp_dom.
            let body_id_opt =
                temp_dom
                    .descendants(temp_dom.document())
                    .into_iter()
                    .find(|&node_id| {
                        if let Some(crate::dom::NodeData::Element { name, .. }) =
                            temp_dom.data(node_id)
                        {
                            name == "body"
                        } else {
                            false
                        }
                    });

            if let Some(body_id) = body_id_opt {
                let temp_children = temp_dom.children(body_id).to_vec();
                for temp_child_id in temp_children {
                    let dest_child_id = copy_node_to_dom_recursive(&temp_dom, temp_child_id, dom);
                    dom.insert_before(parent, dest_child_id, Some(n_id));
                }
            }

            dom.remove_child(parent, n_id);
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

fn bridge_contains(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    // TODO(spec): Node.contains v1 handles inclusive-descendant containment within a single document tree; cross-document, shadow-root, and DocumentFragment host edge cases are out of scope.
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(false));
    };

    let other_key = if let Some(arg) = args.get(1) {
        if arg.is_null() || arg.is_undefined() {
            None
        } else {
            let key_str = arg.to_string(context)?.to_std_string().unwrap_or_default();
            if key_str.is_empty() || key_str == "null" || key_str == "undefined" {
                None
            } else {
                Some(key_str)
            }
        }
    } else {
        None
    };

    let other_key = match other_key {
        Some(k) => k,
        None => return Ok(JsValue::from(false)),
    };

    let contains = with_dom(|dom, key_to_node| {
        let this_id = match key_to_node.get(&node_key).copied() {
            Some(id) => id,
            None => return false,
        };
        let other_id = match key_to_node.get(&other_key).copied() {
            Some(id) => id,
            None => return false,
        };

        if this_id == other_id {
            return true;
        }

        let mut curr = other_id;
        while let Some(parent_id) = dom.parent(curr) {
            if parent_id == this_id {
                return true;
            }
            curr = parent_id;
        }
        false
    })?;

    Ok(JsValue::from(contains))
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

fn bridge_first_element_child(
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
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            for &child_id in dom.children(n_id) {
                if matches!(dom.data(child_id), Some(NodeData::Element { .. })) {
                    let k = format!("{:?}", child_id);
                    key_to_node.insert(k.clone(), child_id);
                    return Some(k);
                }
            }
        }
        None
    })?;

    if let Some(k) = first_child_key_opt {
        Ok(JsValue::from(JsString::from(k)))
    } else {
        Ok(JsValue::null())
    }
}

fn bridge_last_element_child(
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
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            for &child_id in dom.children(n_id).iter().rev() {
                if matches!(dom.data(child_id), Some(NodeData::Element { .. })) {
                    let k = format!("{:?}", child_id);
                    key_to_node.insert(k.clone(), child_id);
                    return Some(k);
                }
            }
        }
        None
    })?;

    if let Some(k) = last_child_key_opt {
        Ok(JsValue::from(JsString::from(k)))
    } else {
        Ok(JsValue::null())
    }
}

fn bridge_next_element_sibling(
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
            if let Some(pos) = children.iter().position(|&id| id == n_id) {
                for &sibling_id in &children[(pos + 1)..] {
                    if matches!(dom.data(sibling_id), Some(NodeData::Element { .. })) {
                        let k = format!("{:?}", sibling_id);
                        key_to_node.insert(k.clone(), sibling_id);
                        return Some(k);
                    }
                }
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

fn bridge_previous_element_sibling(
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
            if let Some(pos) = children.iter().position(|&id| id == n_id) {
                for &sibling_id in children[..pos].iter().rev() {
                    if matches!(dom.data(sibling_id), Some(NodeData::Element { .. })) {
                        let k = format!("{:?}", sibling_id);
                        key_to_node.insert(k.clone(), sibling_id);
                        return Some(k);
                    }
                }
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

fn bridge_children(
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
                if matches!(dom.data(c_id), Some(NodeData::Element { .. })) {
                    let k = format!("{:?}", c_id);
                    key_to_node.insert(k.clone(), c_id);
                    keys.push(k);
                }
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

fn bridge_child_element_count(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(0));
    };

    let count = with_dom(|dom, key_to_node| {
        let mut cnt = 0;
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            for &c_id in dom.children(n_id) {
                if matches!(dom.data(c_id), Some(NodeData::Element { .. })) {
                    cnt += 1;
                }
            }
        }
        cnt
    })?;

    Ok(JsValue::from(count))
}

fn bridge_parent_element(
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
            && matches!(dom.data(p_id), Some(NodeData::Element { .. }))
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

fn clone_node_recursive(dom: &mut Dom, node_id: NodeId, deep: bool) -> NodeId {
    let node_data = if let Some(data) = dom.data(node_id) {
        data.clone()
    } else {
        NodeData::Comment(String::new())
    };

    let cloned_node_id = dom.create_node(node_data);

    if deep {
        let children: Vec<NodeId> = dom.children(node_id).to_vec();
        for child_id in children {
            let cloned_child_id = clone_node_recursive(dom, child_id, true);
            dom.append_child(cloned_node_id, cloned_child_id);
        }
    }

    cloned_node_id
}

fn bridge_clone_node(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let deep = if let Some(arg) = args.get(1) {
        arg.to_boolean()
    } else {
        false
    };

    // TODO(spec): Cloning does not copy event listeners registered with addEventListener or JS expando properties.
    // TODO(spec): Doctype / Document cloning special cases beyond returning a detached copy.
    // TODO(spec): id deduplication.
    // TODO(spec): live-collection effects.

    let cloned_key_opt = with_dom(|dom, key_to_node| {
        if let Some(&node_id) = key_to_node.get(&node_key) {
            let cloned_node_id = clone_node_recursive(dom, node_id, deep);
            let k = format!("{:?}", cloned_node_id);
            key_to_node.insert(k.clone(), cloned_node_id);
            Some(k)
        } else {
            None
        }
    })?;

    if let Some(cloned_key) = cloned_key_opt {
        Ok(JsValue::from(JsString::from(cloned_key)))
    } else {
        Ok(JsValue::null())
    }
}

fn camel_to_kebab(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            result.push('-');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

fn css_value_to_string(val: &crate::css::values::CssValue) -> String {
    use crate::css::values::{
        AlignItemsValue, BoxSizingValue, Color, CssValue, DisplayValue, FlexDirectionValue,
        JustifyContentValue, LengthUnit, OverflowValue, PositionValue,
    };
    match val {
        CssValue::Keyword(s) => s.clone(),
        CssValue::Length(v, unit) => {
            let unit_str = match unit {
                LengthUnit::Px => "px",
                LengthUnit::Em => "em",
                LengthUnit::Rem => "rem",
                LengthUnit::Pt => "pt",
                LengthUnit::Percent => "%",
                LengthUnit::Vw => "vw",
                LengthUnit::Vh => "vh",
            };
            format!("{}{}", v, unit_str)
        }
        CssValue::Number(v) => format!("{}", v),
        CssValue::Color(Color::Rgba(r, g, b, a)) => {
            if *a == 255 {
                format!("rgb({}, {}, {})", r, g, b)
            } else {
                format!("rgba({}, {}, {}, {})", r, g, b, *a as f32 / 255.0)
            }
        }
        CssValue::Multiple(vec) => vec
            .iter()
            .map(css_value_to_string)
            .collect::<Vec<_>>()
            .join(" "),
        CssValue::Position(pv) => match pv {
            PositionValue::Static => "static".to_string(),
            PositionValue::Relative => "relative".to_string(),
            PositionValue::Absolute => "absolute".to_string(),
            PositionValue::Fixed => "fixed".to_string(),
        },
        CssValue::Overflow(ov) => match ov {
            OverflowValue::Visible => "visible".to_string(),
            OverflowValue::Hidden => "hidden".to_string(),
            OverflowValue::Scroll => "scroll".to_string(),
            OverflowValue::Auto => "auto".to_string(),
        },
        CssValue::BoxSizing(bs) => match bs {
            BoxSizingValue::ContentBox => "content-box".to_string(),
            BoxSizingValue::BorderBox => "border-box".to_string(),
        },
        CssValue::Display(dv) => match dv {
            DisplayValue::Block => "block".to_string(),
            DisplayValue::Inline => "inline".to_string(),
            DisplayValue::InlineBlock => "inline-block".to_string(),
            DisplayValue::None => "none".to_string(),
            DisplayValue::Flex => "flex".to_string(),
            DisplayValue::Table => "table".to_string(),
            DisplayValue::TableRow => "table-row".to_string(),
            DisplayValue::TableCell => "table-cell".to_string(),
        },
        CssValue::FlexDirection(fd) => match fd {
            FlexDirectionValue::Row => "row".to_string(),
            FlexDirectionValue::RowReverse => "row-reverse".to_string(),
            FlexDirectionValue::Column => "column".to_string(),
            FlexDirectionValue::ColumnReverse => "column-reverse".to_string(),
        },
        CssValue::JustifyContent(jc) => match jc {
            JustifyContentValue::FlexStart => "flex-start".to_string(),
            JustifyContentValue::FlexEnd => "flex-end".to_string(),
            JustifyContentValue::Center => "center".to_string(),
            JustifyContentValue::SpaceBetween => "space-between".to_string(),
            JustifyContentValue::SpaceAround => "space-around".to_string(),
            JustifyContentValue::SpaceEvenly => "space-evenly".to_string(),
        },
        CssValue::AlignItems(ai) => match ai {
            AlignItemsValue::Stretch => "stretch".to_string(),
            AlignItemsValue::FlexStart => "flex-start".to_string(),
            AlignItemsValue::FlexEnd => "flex-end".to_string(),
            AlignItemsValue::Center => "center".to_string(),
            AlignItemsValue::Baseline => "baseline".to_string(),
        },
    }
}

fn bridge_get_computed_style_value(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let element_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(JsString::from("")));
    };

    let property_name = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(JsString::from("")));
    };

    let mut resolved_value = String::new();
    with_dom(|_dom, key_to_node| {
        if let Some(&node_id) = key_to_node.get(&element_key) {
            CURRENT_STYLES.with(|styles_cell| {
                if let Some(styles) = styles_cell.borrow().as_ref()
                    && let Some(computed_style) = styles.get(&node_id)
                {
                    let kebab = camel_to_kebab(&property_name);
                    if let Some(css_val) = computed_style.get(&kebab) {
                        resolved_value = css_value_to_string(css_val);
                    }
                }
            });
        }
    })?;

    Ok(JsValue::from(JsString::from(resolved_value)))
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
pub fn run_inline_scripts(
    mut dom: Dom,
    styles: &std::collections::HashMap<crate::infra::NodeId, crate::style::ComputedStyle>,
) -> Dom {
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
        let _ = host.eval_with_dom_and_styles(&src, &mut dom, styles);
    }

    // Fire DOM lifecycle events (DOMContentLoaded, load) and expose document.readyState after inline scripts run.
    let _ = host.dispatch_lifecycle_events(&mut dom, styles);

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
    fn test_navigator_properties() {
        let mut host = BoaHost::new();
        assert!(
            host.eval("if (navigator.userAgent !== 'underrated/1.0') throw 'userAgent mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (navigator.platform !== 'Rust') throw 'platform mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (navigator.language !== 'en-US') throw 'language mismatch';")
                .is_ok()
        );
        assert!(host.eval("if (window.navigator.userAgent !== 'underrated/1.0') throw 'window userAgent mismatch';").is_ok());
    }

    #[test]
    fn test_get_computed_style() {
        let mut dom = Dom::new();
        let document = dom.document();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "my-div".to_string())],
        });
        dom.append_child(document, div_id);

        let css = "div { color: red; display: flex; }";
        let stylesheet = crate::css::parser::parse_stylesheet(css);
        let styles = crate::style::compute_styles(&dom, &stylesheet);

        let mut host = BoaHost::new();

        let script = r#"
            const el = document.getElementById('my-div');
            const style = window.getComputedStyle(el);
            const color = style.getPropertyValue('color');
            const display = style.getPropertyValue('display');
            const margin = style.getPropertyValue('margin');
            
            if (color !== 'rgb(255, 0, 0)') {
                throw new Error('Expected rgb(255, 0, 0), got ' + color);
            }
            if (display !== 'flex') {
                throw new Error('Expected flex, got ' + display);
            }
            if (margin !== '') {
                throw new Error('Expected empty string, got ' + margin);
            }
            'SUCCESS'
        "#;

        let res = host.eval_with_dom_and_styles(script, &mut dom, &styles);
        assert_eq!(res, Ok("SUCCESS".to_string()));
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
    fn test_clone_node_behavior() {
        let mut dom = Dom::new();
        let document = dom.document();

        // Build:
        // <div id="parent" class="test-class">
        //   <span>Hello</span>
        // </div>
        let parent_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("id".to_string(), "parent".to_string()),
                ("class".to_string(), "test-class".to_string()),
            ],
        });
        let span_id = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![],
        });
        let text_id = dom.create_node(NodeData::Text("Hello".to_string()));
        dom.append_child(span_id, text_id);
        dom.append_child(parent_id, span_id);
        dom.append_child(document, parent_id);

        let mut host = BoaHost::new();

        let res = host.eval_with_dom(
            r#"
            const parent = document.getElementById('parent');
            
            // 1. Shallow clone
            const cloneShallow = parent.cloneNode(false);
            const tagShallowOk = cloneShallow.tagName === 'DIV';
            const idShallowOk = cloneShallow.getAttribute('id') === 'parent';
            const classShallowOk = cloneShallow.getAttribute('class') === 'test-class';
            const childShallowLenOk = cloneShallow.childNodes.length === 0;
            const parentShallowNodeOk = cloneShallow.parentNode === null;
            const shallowOk = tagShallowOk && idShallowOk && classShallowOk && childShallowLenOk && parentShallowNodeOk;

            // 2. Deep clone
            const cloneDeep = parent.cloneNode(true);
            const tagDeepOk = cloneDeep.tagName === 'DIV';
            const idDeepOk = cloneDeep.getAttribute('id') === 'parent';
            const hasChildrenDeepOk = cloneDeep.childNodes.length === 1;
            const firstChildSpanDeepOk = cloneDeep.firstChild.tagName === 'SPAN';
            const nestedTextDeepOk = cloneDeep.firstChild.firstChild.textContent === 'Hello';
            const parentDeepNodeOk = cloneDeep.parentNode === null;
            const deepOk = tagDeepOk && idDeepOk && hasChildrenDeepOk && firstChildSpanDeepOk && nestedTextDeepOk && parentDeepNodeOk;

            // 3. Text node clone
            const span = parent.firstChild;
            const textNode = span.firstChild;
            const textClone = textNode.cloneNode(true);
            const textCloneOk = textClone.textContent === 'Hello' && textClone.parentNode === null;

            // 4. Original subtree unchanged
            const origChildrenLenOk = parent.childNodes.length === 1;
            const origSpanTagOk = parent.firstChild.tagName === 'SPAN';
            const origTextOk = parent.firstChild.firstChild.textContent === 'Hello';
            const originalOk = origChildrenLenOk && origSpanTagOk && origTextOk;

            shallowOk && deepOk && textCloneOk && originalOk;
            "#,
            &mut dom
        );
        assert_eq!(res, Ok("true".to_string()));
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
    fn test_eval_with_dom_document_elements() {
        let mut dom = Dom::new();
        let document = dom.document();
        let html = dom.create_node(NodeData::Element {
            name: "html".to_string(),
            attrs: vec![],
        });
        let head = dom.create_node(NodeData::Element {
            name: "head".to_string(),
            attrs: vec![],
        });
        let body = dom.create_node(NodeData::Element {
            name: "body".to_string(),
            attrs: vec![],
        });
        dom.append_child(html, head);
        dom.append_child(html, body);
        dom.append_child(document, html);
        let mut host = BoaHost::new();

        let res = host.eval_with_dom(
            r#"
            (
                document.documentElement.tagName === "HTML" &&
                document.body.tagName === "BODY" &&
                document.head.tagName === "HEAD" &&
                document.body === document.getElementsByTagName("body")[0]
            ) ? "true" : "false"
            "#,
            &mut dom,
        );
        assert_eq!(res, Ok("true".to_string()));

        let mut empty_dom = Dom::new();
        let res_empty = host.eval_with_dom(
            r#"
            (
                document.body === null &&
                document.documentElement === null &&
                document.head === null
            ) ? "true" : "false"
            "#,
            &mut empty_dom,
        );
        assert_eq!(res_empty, Ok("true".to_string()));
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
    fn test_node_contains() {
        let mut dom = Dom::new();
        let document = dom.document();

        // Build a DOM tree:
        // document -> parent_div (div) -> child_span (span) -> grandchild_text
        // also sibling_div (div)
        let parent_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "parent".to_string())],
        });
        let child_id = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![("id".to_string(), "child".to_string())],
        });
        let grandchild_id = dom.create_node(NodeData::Text("grandchild".to_string()));
        let sibling_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "sibling".to_string())],
        });

        dom.append_child(child_id, grandchild_id);
        dom.append_child(parent_id, child_id);
        dom.append_child(parent_id, sibling_id);
        dom.append_child(document, parent_id);

        let mut host = BoaHost::new();

        // 1. A node contains itself (inclusive-descendant)
        let res_self_parent = host.eval_with_dom(
            "document.getElementById('parent').contains(document.getElementById('parent'))",
            &mut dom,
        );
        assert_eq!(res_self_parent, Ok("true".to_string()));

        let res_self_child = host.eval_with_dom(
            "document.getElementById('child').contains(document.getElementById('child'))",
            &mut dom,
        );
        assert_eq!(res_self_child, Ok("true".to_string()));

        // 2. A parent element contains its direct child
        let res_parent_contains_child = host.eval_with_dom(
            "document.getElementById('parent').contains(document.getElementById('child'))",
            &mut dom,
        );
        assert_eq!(res_parent_contains_child, Ok("true".to_string()));

        // 3. A node contains a deeper (grand-child) descendant
        // Note: grandchild is a text node, which we don't directly have getElementById for,
        // but we can access it via firstChild of child.
        let res_parent_contains_grandchild = host.eval_with_dom(
            "document.getElementById('parent').contains(document.getElementById('child').firstChild)",
            &mut dom,
        );
        assert_eq!(res_parent_contains_grandchild, Ok("true".to_string()));

        // 4. A node does NOT contain an unrelated sibling or ancestor
        let res_child_contains_parent = host.eval_with_dom(
            "document.getElementById('child').contains(document.getElementById('parent'))",
            &mut dom,
        );
        assert_eq!(res_child_contains_parent, Ok("false".to_string()));

        let res_sibling_contains_child = host.eval_with_dom(
            "document.getElementById('sibling').contains(document.getElementById('child'))",
            &mut dom,
        );
        assert_eq!(res_sibling_contains_child, Ok("false".to_string()));

        // 5. Node.contains(null) and Node.contains(undefined) return false (and do not throw)
        let res_contains_null =
            host.eval_with_dom("document.getElementById('parent').contains(null)", &mut dom);
        assert_eq!(res_contains_null, Ok("false".to_string()));

        let res_contains_undefined = host.eval_with_dom(
            "document.getElementById('parent').contains(undefined)",
            &mut dom,
        );
        assert_eq!(res_contains_undefined, Ok("false".to_string()));

        // 6. document contains nodes under it
        let res_document_contains_parent = host.eval_with_dom(
            "document.contains(document.getElementById('parent'))",
            &mut dom,
        );
        assert_eq!(res_document_contains_parent, Ok("true".to_string()));

        let res_document_contains_child = host.eval_with_dom(
            "document.contains(document.getElementById('child'))",
            &mut dom,
        );
        assert_eq!(res_document_contains_child, Ok("true".to_string()));

        let res_document_contains_self =
            host.eval_with_dom("document.contains(document)", &mut dom);
        assert_eq!(res_document_contains_self, Ok("true".to_string()));

        // 7. document does NOT contain nodes outside it (not appended, or null)
        let res_document_contains_null = host.eval_with_dom("document.contains(null)", &mut dom);
        assert_eq!(res_document_contains_null, Ok("false".to_string()));
    }

    #[test]
    fn test_node_contains_t0297() {
        let mut dom = Dom::new();
        let document = dom.document();

        let parent_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "p".to_string())],
        });
        let child_id = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![("id".to_string(), "c".to_string())],
        });
        let grandchild_id = dom.create_node(NodeData::Element {
            name: "strong".to_string(),
            attrs: vec![("id".to_string(), "g".to_string())],
        });
        let sibling_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "s".to_string())],
        });

        dom.append_child(child_id, grandchild_id);
        dom.append_child(parent_id, child_id);
        dom.append_child(parent_id, sibling_id);
        dom.append_child(document, parent_id);

        let mut host = BoaHost::new();

        // Self
        let res_self = host.eval_with_dom(
            "document.getElementById('p').contains(document.getElementById('p'))",
            &mut dom,
        );
        assert_eq!(res_self, Ok("true".to_string()));

        // Descendants (child and grandchild)
        let res_child = host.eval_with_dom(
            "document.getElementById('p').contains(document.getElementById('c'))",
            &mut dom,
        );
        assert_eq!(res_child, Ok("true".to_string()));

        let res_grandchild = host.eval_with_dom(
            "document.getElementById('p').contains(document.getElementById('g'))",
            &mut dom,
        );
        assert_eq!(res_grandchild, Ok("true".to_string()));

        // Non-descendant (sibling contains child is false)
        let res_sibling = host.eval_with_dom(
            "document.getElementById('s').contains(document.getElementById('c'))",
            &mut dom,
        );
        assert_eq!(res_sibling, Ok("false".to_string()));

        // Null / undefined
        let res_null = host.eval_with_dom("document.getElementById('p').contains(null)", &mut dom);
        assert_eq!(res_null, Ok("false".to_string()));

        let res_undefined =
            host.eval_with_dom("document.getElementById('p').contains(undefined)", &mut dom);
        assert_eq!(res_undefined, Ok("false".to_string()));
    }

    #[test]
    fn test_element_only_dom_traversal_accessors() {
        let mut dom = Dom::new();
        let document = dom.document();

        // Build:
        // document -> parent_div (div) -> (text1, span_a, text2, b_b, text3)
        let parent_div = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "parent".to_string())],
        });
        let text1 = dom.create_node(NodeData::Text("text1".to_string()));
        let span_a = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![("id".to_string(), "a".to_string())],
        });
        let text2 = dom.create_node(NodeData::Text("text2".to_string()));
        let b_b = dom.create_node(NodeData::Element {
            name: "b".to_string(),
            attrs: vec![("id".to_string(), "b".to_string())],
        });
        let text3 = dom.create_node(NodeData::Text("text3".to_string()));

        dom.append_child(parent_div, text1);
        dom.append_child(parent_div, span_a);
        dom.append_child(parent_div, text2);
        dom.append_child(parent_div, b_b);
        dom.append_child(parent_div, text3);
        dom.append_child(document, parent_div);

        let mut host = BoaHost::new();

        // Let's assert the expected properties of the parent div:
        // firstElementChild is span#a
        assert_eq!(
            host.eval_with_dom("document.getElementById('parent').firstElementChild === document.getElementById('a')", &mut dom),
            Ok("true".to_string())
        );

        // lastElementChild is b#b
        assert_eq!(
            host.eval_with_dom("document.getElementById('parent').lastElementChild === document.getElementById('b')", &mut dom),
            Ok("true".to_string())
        );

        // childElementCount is 2
        assert_eq!(
            host.eval_with_dom(
                "document.getElementById('parent').childElementCount",
                &mut dom
            ),
            Ok("2".to_string())
        );

        // children has length 2 and consists of [span_a, b_b] in order
        assert_eq!(
            host.eval_with_dom(
                "document.getElementById('parent').children.length",
                &mut dom
            ),
            Ok("2".to_string())
        );
        assert_eq!(
            host.eval_with_dom(
                "document.getElementById('parent').children[0] === document.getElementById('a')",
                &mut dom
            ),
            Ok("true".to_string())
        );
        assert_eq!(
            host.eval_with_dom(
                "document.getElementById('parent').children[1] === document.getElementById('b')",
                &mut dom
            ),
            Ok("true".to_string())
        );

        // From the span:
        // nextElementSibling is the <b> (skipping the text node)
        assert_eq!(
            host.eval_with_dom(
                "document.getElementById('a').nextElementSibling === document.getElementById('b')",
                &mut dom
            ),
            Ok("true".to_string())
        );
        // previousElementSibling is null
        assert_eq!(
            host.eval_with_dom(
                "document.getElementById('a').previousElementSibling === null",
                &mut dom
            ),
            Ok("true".to_string())
        );

        // parentElement of the span is the div
        assert_eq!(
            host.eval_with_dom(
                "document.getElementById('a').parentElement === document.getElementById('parent')",
                &mut dom
            ),
            Ok("true".to_string())
        );

        // parentElement of an element whose parent is the document node is null
        assert_eq!(
            host.eval_with_dom(
                "document.getElementById('parent').parentElement === null",
                &mut dom
            ),
            Ok("true".to_string())
        );

        // from b_b:
        // previousElementSibling is span_a
        assert_eq!(
            host.eval_with_dom("document.getElementById('b').previousElementSibling === document.getElementById('a')", &mut dom),
            Ok("true".to_string())
        );
        // nextElementSibling is null
        assert_eq!(
            host.eval_with_dom(
                "document.getElementById('b').nextElementSibling === null",
                &mut dom
            ),
            Ok("true".to_string())
        );
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
    fn test_eval_with_dom_matches_and_closest() {
        let mut dom = Dom::new();
        let document = dom.document();

        // Build structure: <div id="a"><span class="b highlight" id="b-span">hi</span></div>
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "a".to_string())],
        });
        let span_id = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![
                ("class".to_string(), "b highlight".to_string()),
                ("id".to_string(), "b-span".to_string()),
            ],
        });
        let text_id = dom.create_node(NodeData::Text("hi".to_string()));

        dom.append_child(span_id, text_id);
        dom.append_child(div_id, span_id);
        dom.append_child(document, div_id);

        let mut host = BoaHost::new();

        // Assertions requested by t0281:
        // - getElementById('b-span').matches('span.highlight') => "true"
        let res1 = host.eval_with_dom(
            "document.getElementById('b-span').matches('span.highlight')",
            &mut dom,
        );
        assert_eq!(res1, Ok("true".to_string()));

        // - getElementById('b-span').matches('div') => "false"
        let res2 = host.eval_with_dom("document.getElementById('b-span').matches('div')", &mut dom);
        assert_eq!(res2, Ok("false".to_string()));

        // - getElementById('b-span').matches('#b-span') => "true"
        let res3 = host.eval_with_dom(
            "document.getElementById('b-span').matches('#b-span')",
            &mut dom,
        );
        assert_eq!(res3, Ok("true".to_string()));

        // - getElementById('b-span').closest('div') resolves to the ancestor div (e.g. compare its .id to "a") => the expected id string
        let res4 = host.eval_with_dom(
            "document.getElementById('b-span').closest('div').id",
            &mut dom,
        );
        assert_eq!(res4, Ok("a".to_string()));

        // - getElementById('b-span').closest('.no-such-class') => "null" (assert the JS stringifies to "null")
        let res5 = host.eval_with_dom(
            "document.getElementById('b-span').closest('.no-such-class')",
            &mut dom,
        );
        assert_eq!(res5, Ok("null".to_string()));

        // - closest returns self when self matches: getElementById('a').closest('#a').id => "a"
        let res6 = host.eval_with_dom("document.getElementById('a').closest('#a').id", &mut dom);
        assert_eq!(res6, Ok("a".to_string()));
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
    fn test_element_value() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            // 1. An <input> that has a `value` attribute exposes it via `.value`
            let input = document.createElement('input');
            input.setAttribute('value', 'initial-val');
            let res1 = input.value;

            // 2. An element with no `value` attribute returns the empty string `''` from `.value`
            let select = document.createElement('select');
            let res2 = select.value;

            // 3. Setting el.value = 'hello' makes el.getAttribute('value') === 'hello' and el.value === 'hello'
            let textarea = document.createElement('textarea');
            textarea.value = 'hello';
            let res3_attr = textarea.getAttribute('value');
            let res3_val = textarea.value;

            // 4. Setting a non-string (e.g. a number) coerces via String() — el.value = 42; el.value === '42'
            let button = document.createElement('button');
            button.value = 42;
            let res4_attr = button.getAttribute('value');
            let res4_val = button.value;

            [res1, res2, res3_attr, res3_val, res4_attr, res4_val].join('|');
        ";
        assert_eq!(
            host.eval_with_dom(script, &mut dom),
            Ok("initial-val||hello|hello|42|42".to_string())
        );
    }

    #[test]
    fn test_element_classlist() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            const div = document.createElement('div');
            
            // 1. Initial state
            const r1 = div.classList ? div.classList.length : 'no-classlist';
            const r2 = div.classList ? div.classList.value : 'no-classlist';
            
            // 2. Add
            if (div.classList) div.classList.add('a', 'b', 'a');
            const r3 = div.classList ? div.classList.length : 0;
            const r4 = div.classList ? div.classList.value : '';
            const r5 = div.getAttribute('class') || '';
            const r6 = div.className;
            
            // 3. Contains & item & bracket indexing
            const r7 = div.classList ? (div.classList.contains('a') && !div.classList.contains('c')) : false;
            const r8 = div.classList ? div.classList.item(0) : null;
            const r9 = div.classList ? div.classList.item(1) : null;
            const r10 = div.classList ? div.classList.item(2) : null;
            const r11 = div.classList ? div.classList[0] : undefined;
            const r12 = div.classList ? div.classList[2] : undefined;
            
            // 4. Remove
            if (div.classList) div.classList.remove('b', 'c');
            const r13 = div.classList ? div.classList.value : '';
            
            // 5. Toggle without force
            const r14 = div.classList ? div.classList.toggle('a') : false;
            const r15 = div.classList ? div.classList.value : '';
            const r16 = div.classList ? div.classList.toggle('b') : false;
            const r17 = div.classList ? div.classList.value : '';
            
            // 6. Toggle with force
            const r18 = div.classList ? div.classList.toggle('b', true) : false;
            const r19 = div.classList ? div.classList.value : '';
            const r20 = div.classList ? div.classList.toggle('b', false) : false;
            const r21 = div.classList ? div.classList.value : '';
            
            // 7. Replace
            if (div.classList) div.classList.add('x', 'y');
            const r22 = div.classList ? div.classList.replace('x', 'z') : false;
            const r23 = div.classList ? div.classList.value : '';
            const r24 = div.classList ? div.classList.replace('w', 'z') : false;
            
            // 8. Deduplication in parsing
            div.setAttribute('class', '  p  q   p  ');
            const r25 = div.classList ? div.classList.length : 0;
            const r26 = div.classList ? div.classList.item(0) : null;
            const r27 = div.classList ? div.classList.item(1) : null;
            
            // 9. Value assignment consistency
            if (div.classList) div.classList.value = 'hello world';
            const r28 = div.className;
            const r29 = div.classList ? div.classList.length : 0;

            // 10. Identity
            const r30 = div.classList ? (div.classList === div.classList) : false;

            // 11. Exception throwing behavior
            let r31 = 'no-error';
            try {
                if (div.classList) div.classList.add('');
            } catch (e) {
                r31 = e.name;
            }
            let r32 = 'no-error';
            try {
                if (div.classList) div.classList.add('a b');
            } catch (e) {
                r32 = e.name;
            }

            [
                r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14, r15, r16, r17, r18, r19, r20, r21, r22, r23, r24, r25, r26, r27, r28, r29, r30, r31, r32
            ].map(String).join('|');
        ";
        assert_eq!(
            host.eval_with_dom(script, &mut dom),
            Ok("0||2|a b|a b|a b|true|a|b|null|a|undefined|a|false||true|b|true|b|false||true|z y|false|2|p|q|hello world|2|true|SyntaxError|InvalidCharacterError".to_string())
        );
    }

    #[test]
    fn test_element_dataset() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            const div = document.createElement('div');

            // 1. Read: an element with attribute data-user-id=\"42\" exposes el.dataset.userId === '42'
            div.setAttribute('data-user-id', '42');
            const r1 = div.dataset.userId;

            // 2. Missing key returns undefined (check typeof === 'undefined' or standard === undefined)
            const r2 = typeof div.dataset.nope;

            // 3. Write: el.dataset.fooBar = 'x' then el.getAttribute('data-foo-bar') === 'x'
            div.dataset.fooBar = 'x';
            const r3_attr = div.getAttribute('data-foo-bar');
            const r3_val = div.dataset.fooBar;

            // 4. Delete: delete el.dataset.userId then el.hasAttribute('data-user-id') === false
            const r4_has_before = div.hasAttribute('data-user-id');
            const r4_delete_ret = delete div.dataset.userId;
            const r4_has_after = div.hasAttribute('data-user-id');

            // 5. Enumerate: after setting multiple data-* attrs, Object.keys(el.dataset).sort().join(',') equals expected
            div.dataset.userId = '99'; // resets data-user-id
            div.dataset.anotherKey = 'hello';
            const keys = Object.keys(div.dataset).sort().join(',');

            // 6. Test syntax error on invalid prop names in setting or deleting
            let r6_set_err = 'no-error';
            try {
                div.dataset['user-id'] = 'fail';
            } catch (e) {
                r6_set_err = e.name;
            }

            let r6_delete_err = 'no-error';
            try {
                delete div.dataset['user-id'];
            } catch (e) {
                r6_delete_err = e.name;
            }

            // 7. Identity/Caching: el.dataset === el.dataset
            const r7 = (div.dataset === div.dataset);

            [
                r1, r2, r3_attr, r3_val, r4_has_before, r4_delete_ret, r4_has_after, keys, r6_set_err, r6_delete_err, r7
            ].map(String).join('|');
        ";
        assert_eq!(
            host.eval_with_dom(script, &mut dom),
            Ok("42|undefined|x|x|true|true|false|anotherKey,fooBar,userId|SyntaxError|SyntaxError|true".to_string())
        );
    }

    #[test]
    fn test_dom_write_attribute_management() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            const div = document.createElement('div');
            div.setAttribute('id', 'test-id');
            div.setAttribute('class', 'foo bar');
            div.setAttribute('data-custom', 'val');

            // 1. Check getAttributeNames
            const names1 = div.getAttributeNames().join(',');

            // 2. removeAttribute('class')
            div.removeAttribute('class');
            const has_class = div.hasAttribute('class');
            const class_val = div.getAttribute('class');
            const names2 = div.getAttributeNames().join(',');

            // 3. removeAttribute for non-existent is no-op
            div.removeAttribute('non-existent');

            // 4. toggleAttribute(name) (no force)
            // If absent: adds it with empty value
            const t1 = div.toggleAttribute('class'); // should return true
            const has_class_now = div.hasAttribute('class');
            const class_val_now = div.getAttribute('class');

            // If present: removes it
            const t2 = div.toggleAttribute('class'); // should return false
            const has_class_now2 = div.hasAttribute('class');

            // 5. toggleAttribute(name, force)
            // force = true, absent -> adds empty value, returns true
            const t3 = div.toggleAttribute('class', true); // should return true
            const class_val_t3 = div.getAttribute('class');

            // force = true, present -> no-op, returns true
            const t4 = div.toggleAttribute('class', true); // should return true
            
            // force = false, present -> removes, returns false
            const t5 = div.toggleAttribute('class', false); // should return false
            const has_class_t5 = div.hasAttribute('class');

            // force = false, absent -> no-op, returns false
            const t6 = div.toggleAttribute('class', false); // should return false

            [
                names1,
                has_class,
                class_val,
                names2,
                t1,
                has_class_now,
                class_val_now,
                t2,
                has_class_now2,
                t3,
                class_val_t3,
                t4,
                t5,
                has_class_t5,
                t6
            ].map(String).join('|');
        ";

        let expected = "id,class,data-custom|false|null|id,data-custom|true|true||false|false|true||true|false|false|false";
        assert_eq!(
            host.eval_with_dom(script, &mut dom),
            Ok(expected.to_string())
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
    fn test_dom_childnode_before_after() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let parent = document.createElement('div');
            document.appendChild(parent);

            let refNode = document.createElement('span');
            refNode.textContent = 'ref';
            parent.appendChild(refNode);

            // 1. before: insert a node before refNode
            let beforeNode = document.createElement('span');
            beforeNode.textContent = 'before';
            refNode.before(beforeNode);

            // 2. after (with next sibling): insert a node after refNode but before any subsequent node
            let afterNode = document.createElement('span');
            afterNode.textContent = 'after';
            refNode.after(afterNode);

            // 3. after (when refNode's next sibling is now 'afterNode', and we target 'afterNode' which is the last child)
            let lastNode = document.createElement('span');
            lastNode.textContent = 'last';
            afterNode.after(lastNode);

            // 4. before / after on a node with null parentNode is a no-op (should not throw)
            let detached = document.createElement('span');
            let dummy = document.createElement('span');
            detached.before(dummy);
            detached.after(dummy);
        ";
        assert!(host.eval_with_dom(script, &mut dom).is_ok());

        // Verify the DOM structure from the Rust side
        let doc_children = dom.children(dom.document());
        assert_eq!(doc_children.len(), 1);
        let parent_id = doc_children[0];
        let parent_children = dom.children(parent_id);

        // Order should be: before, ref, after, last
        assert_eq!(parent_children.len(), 4);
        assert_eq!(dom.text_content(parent_children[0]), "before");
        assert_eq!(dom.text_content(parent_children[1]), "ref");
        assert_eq!(dom.text_content(parent_children[2]), "after");
        assert_eq!(dom.text_content(parent_children[3]), "last");
    }

    #[test]
    fn test_dom_parentnode_append_prepend() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let parent = document.createElement('div');
            document.appendChild(parent);

            let a = document.createElement('span');
            a.textContent = 'a';
            let b = document.createElement('span');
            b.textContent = 'b';
            parent.append(a, b);

            let c = document.createElement('span');
            c.textContent = 'c';
            parent.append(c);

            let z = document.createElement('span');
            z.textContent = 'z';
            parent.prepend(z);

            parent.append('hi');

            let x = document.createElement('span');
            x.textContent = 'x';
            let y = document.createElement('span');
            y.textContent = 'y';
            parent.prepend(x, y);
        ";
        assert!(host.eval_with_dom(script, &mut dom).is_ok());

        // Verify the DOM structure from the Rust side
        let doc_children = dom.children(dom.document());
        assert_eq!(doc_children.len(), 1);
        let parent_id = doc_children[0];
        let parent_children = dom.children(parent_id);

        assert_eq!(parent_children.len(), 7);
        assert_eq!(dom.text_content(parent_children[0]), "x");
        assert_eq!(dom.text_content(parent_children[1]), "y");
        assert_eq!(dom.text_content(parent_children[2]), "z");
        assert_eq!(dom.text_content(parent_children[3]), "a");
        assert_eq!(dom.text_content(parent_children[4]), "b");
        assert_eq!(dom.text_content(parent_children[5]), "c");
        assert_eq!(dom.text_content(parent_children[6]), "hi");
    }

    #[test]
    fn test_dom_parentnode_replace_children() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let parent = document.createElement('div');
            document.appendChild(parent);

            let initial1 = document.createElement('span');
            initial1.textContent = 'old1';
            let initial2 = document.createElement('span');
            initial2.textContent = 'old2';
            parent.appendChild(initial1);
            parent.appendChild(initial2);

            let a = document.createElement('span');
            a.textContent = 'newA';
            let b = document.createElement('span');
            b.textContent = 'newB';

            parent.replaceChildren(a, 'txt', b);
        ";
        assert!(host.eval_with_dom(script, &mut dom).is_ok());

        // Verify the DOM structure from the Rust side
        let doc_children = dom.children(dom.document());
        assert_eq!(doc_children.len(), 1);
        let parent_id = doc_children[0];
        let parent_children = dom.children(parent_id);

        assert_eq!(parent_children.len(), 3);
        assert_eq!(dom.text_content(parent_children[0]), "newA");
        assert_eq!(dom.text_content(parent_children[1]), "txt");
        assert_eq!(dom.text_content(parent_children[2]), "newB");

        // Now clear with no arguments
        let script_clear = "
            parent.replaceChildren();
        ";
        assert!(host.eval_with_dom(script_clear, &mut dom).is_ok());

        let parent_children_cleared = dom.children(parent_id);
        assert_eq!(parent_children_cleared.len(), 0);
    }

    #[test]
    fn test_dom_childnode_remove() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let parent = document.createElement('div');
            document.appendChild(parent);

            let a = document.createElement('span');
            a.textContent = 'a';
            parent.appendChild(a);

            let b = document.createElement('span');
            b.textContent = 'b';
            parent.appendChild(b);

            let c = document.createElement('span');
            c.textContent = 'c';
            parent.appendChild(c);

            // Removing a middle child detaches it:
            b.remove();

            // Calling remove() on a node with no parent is a silent no-op:
            let detached = document.createElement('div');
            detached.remove();
        ";
        assert!(host.eval_with_dom(script, &mut dom).is_ok());

        // Verify the DOM structure from the Rust side
        let doc_children = dom.children(dom.document());
        assert_eq!(doc_children.len(), 1);
        let parent_id = doc_children[0];
        let parent_children = dom.children(parent_id);

        // Order should be exactly: a, c
        assert_eq!(parent_children.len(), 2);
        assert_eq!(dom.text_content(parent_children[0]), "a");
        assert_eq!(dom.text_content(parent_children[1]), "c");
    }

    #[test]
    fn test_dom_childnode_replacewith() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let parent = document.createElement('div');
            document.appendChild(parent);

            let a = document.createElement('span');
            a.textContent = 'a';
            parent.appendChild(a);

            let b = document.createElement('span');
            b.textContent = 'b';
            parent.appendChild(b);

            let c = document.createElement('span');
            c.textContent = 'c';
            parent.appendChild(c);

            let x = document.createElement('span');
            x.textContent = 'x';

            // Replaces b with x:
            b.replaceWith(x);

            // Calling replaceWith() on a node with no parent is a silent no-op:
            let detached = document.createElement('span');
            let detached_replacer = document.createElement('span');
            detached.replaceWith(detached_replacer);
        ";
        assert!(host.eval_with_dom(script, &mut dom).is_ok());

        // Verify the DOM structure from the Rust side
        let doc_children = dom.children(dom.document());
        assert_eq!(doc_children.len(), 1);
        let parent_id = doc_children[0];
        let parent_children = dom.children(parent_id);

        // Order should be exactly: a, x, c
        assert_eq!(parent_children.len(), 3);
        assert_eq!(dom.text_content(parent_children[0]), "a");
        assert_eq!(dom.text_content(parent_children[1]), "x");
        assert_eq!(dom.text_content(parent_children[2]), "c");
    }

    #[test]
    fn test_dom_write_create_text_node() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let div = document.createElement('div');
            let text = document.createTextNode('hi');
            div.appendChild(text);
            document.appendChild(div);
            div.textContent;
        ";
        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(res, Ok("hi".to_string()));

        // Verify backing Node type and content
        let root_children = dom.children(dom.document());
        assert_eq!(root_children.len(), 1);
        let div_id = root_children[0];
        let div_children = dom.children(div_id);
        assert_eq!(div_children.len(), 1);
        let text_id = div_children[0];
        match dom.data(text_id) {
            Some(NodeData::Text(content)) => assert_eq!(content, "hi"),
            _ => panic!("Expected Text node"),
        }
    }

    #[test]
    fn test_dom_write_has_attribute() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let div = document.createElement('div');
            div.setAttribute('class', 'active');
            let r1 = div.hasAttribute('class');
            let r2 = div.hasAttribute('id');
            [r1, r2].join(',');
        ";
        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(res, Ok("true,false".to_string()));
    }

    #[test]
    fn test_dom_write_replace_child() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let parent = document.createElement('div');
            document.appendChild(parent);

            let a = document.createElement('span');
            a.textContent = 'A';
            parent.appendChild(a);

            let b = document.createElement('span');
            b.textContent = 'B';
            parent.appendChild(b);

            let c = document.createElement('span');
            c.textContent = 'C';
            parent.appendChild(c);

            // Replace b with a new node
            let newNode = document.createElement('span');
            newNode.textContent = 'NEW';
            let oldNode = parent.replaceChild(newNode, b);

            // Return values and current nodes order
            let oldText = oldNode.textContent;
            let currentOrder = parent.childNodes.map(node => node.textContent).join(',');
            [oldText, currentOrder].join('|');
        ";
        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(res, Ok("B|A,NEW,C".to_string()));

        // Also check DOM state from Rust side
        let root_children = dom.children(dom.document());
        let parent_id = root_children[0];
        let parent_children = dom.children(parent_id);
        assert_eq!(parent_children.len(), 3);
        assert_eq!(dom.text_content(parent_children[0]), "A");
        assert_eq!(dom.text_content(parent_children[1]), "NEW");
        assert_eq!(dom.text_content(parent_children[2]), "C");
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

        let mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());
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
        let mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());
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
        let mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());
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
        let mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());
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
        let _mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());

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

        let mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());
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

        let mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());
        assert_eq!(mutated_dom.text_content(element_id), "changed");

        // Restore defaults
        set_limits_enabled(true);
        set_max_script_length(5000);
    }

    #[test]
    fn test_event_target_and_event_classes() {
        let mut host = BoaHost::new();

        // 1. Basic properties and prototype of Event and EventTarget
        let script = r#"
            const target = new EventTarget();
            const event = new Event('click');

            if (event.type !== 'click') throw new Error('Expected click type');
            if (event.target !== null) throw new Error('Expected target to be null initially');
            if (event.currentTarget !== null) throw new Error('Expected currentTarget to be null initially');
            if (event.defaultPrevented !== false) throw new Error('Expected defaultPrevented to be false');

            event.preventDefault();
            if (event.defaultPrevented !== true) throw new Error('Expected defaultPrevented to be true after preventDefault()');

            let callbackCalled = false;
            target.addEventListener('click', (e) => {
                callbackCalled = true;
                if (e.type !== 'click') throw new Error('Expected event click inside handler');
                if (e.target !== target) throw new Error('Expected target to be the event target');
                if (e.currentTarget !== target) throw new Error('Expected currentTarget to be the event target');
            });

            const dispatchResult = target.dispatchEvent(event);
            if (!callbackCalled) throw new Error('Expected callback to be called');
            if (dispatchResult !== false) throw new Error('Expected dispatchResult to be false since defaultPrevented is true');
            if (event.currentTarget !== null) throw new Error('Expected currentTarget to be null after dispatching');
        "#;
        assert!(host.eval(script).is_ok());
    }

    #[test]
    fn test_event_target_remove_event_listener() {
        let mut host = BoaHost::new();

        let script = r#"
            const target = new EventTarget();
            let count = 0;
            const listener = () => { count++; };

            target.addEventListener('custom', listener);
            target.dispatchEvent(new Event('custom'));
            if (count !== 1) throw new Error('Expected count to be 1');

            target.removeEventListener('custom', listener);
            target.dispatchEvent(new Event('custom'));
            if (count !== 1) throw new Error('Expected count to be 1 after removing');
        "#;
        assert!(host.eval(script).is_ok());
    }

    #[test]
    fn test_element_style() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = r#"
            const el = document.createElement('div');

            // 1. Initially style.cssText and properties are empty
            const r1 = el.style.cssText;
            const r2 = el.style.color;

            // 2. setProperty color
            el.style.setProperty('color', 'red');
            const r3 = el.style.color;
            const r4 = el.style.getPropertyValue('color');
            const r5 = el.getAttribute('style') || '';

            // 3. camelCase mapping (backgroundColor) and setting directly
            el.style.backgroundColor = 'blue';
            const r6 = el.style.backgroundColor;
            const r7 = el.style.getPropertyValue('background-color');
            const r8 = el.getAttribute('style') || '';

            // 4. cssText setting and getting
            el.style.cssText = 'display: none; border-radius: 5px';
            const r9 = el.style.cssText;
            const r10 = el.style.display;

            // 5. removeProperty
            const removed = el.style.removeProperty('display');
            const r11 = el.style.cssText;
            const r12 = el.style.display;

            // 6. setting property to empty string removes it
            el.style.backgroundColor = '';
            const r13 = el.style.cssText;

            [r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, removed, r11, r12, r13].join('|');
        "#;

        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(
            res,
            Ok("||red|red|color: red;|blue|blue|color: red; background-color: blue;|display: none; border-radius: 5px|none|none|border-radius: 5px;||border-radius: 5px;".to_string())
        );

        // Ensure robust error/garbage safety: setting null or invalid empty values shouldn't panic
        let script_invalid = r#"
            const el_inv = document.createElement('div');
            el_inv.style.setProperty('', 'red');
            el_inv.style.setProperty(null, 'blue');
            el_inv.style.setProperty('color', null);
            el_inv.style.cssText = null;
            el_inv.style.cssText;
        "#;
        assert_eq!(
            host.eval_with_dom(script_invalid, &mut dom),
            Ok("".to_string())
        );
    }

    #[test]
    fn test_element_inner_html_getter_setter() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        // 1. Basic get/set round trip with mixed element and text nodes
        let script1 = "
            let div = document.createElement('div');
            document.appendChild(div);
            div.innerHTML = '<span class=\"a\">hi</span><b>x</b>';
            div.innerHTML;
        ";
        let res1 = host.eval_with_dom(script1, &mut dom);
        // Note: attribute orders and formatting may differ, but our serialize implementation
        // produces `<span class="a">hi</span><b>x</b>`
        assert_eq!(res1, Ok("<span class=\"a\">hi</span><b>x</b>".to_string()));

        // 2. Inspect the real DOM tree from the Rust side to ensure it was properly parsed and transplanted
        let root_children = dom.children(dom.document());
        let div_id = root_children[0];
        let div_children = dom.children(div_id);
        assert_eq!(div_children.len(), 2);

        // First child: span with class="a" and text "hi"
        let span_id = div_children[0];
        assert!(
            matches!(dom.data(span_id), Some(NodeData::Element { name, .. }) if name == "span")
        );
        assert_eq!(dom.get_attribute(span_id, "class"), Some("a"));
        let span_children = dom.children(span_id);
        assert_eq!(span_children.len(), 1);
        assert_eq!(dom.text_content(span_id), "hi");

        // Second child: b with text "x"
        let b_id = div_children[1];
        assert!(matches!(dom.data(b_id), Some(NodeData::Element { name, .. }) if name == "b"));
        assert_eq!(dom.text_content(b_id), "x");

        // 3. Clear using empty string setter
        let script2 = "
            div.innerHTML = '';
            div.innerHTML;
        ";
        let res2 = host.eval_with_dom(script2, &mut dom);
        assert_eq!(res2, Ok("".to_string()));
        assert_eq!(dom.children(div_id).len(), 0);

        // 4. Getter/setter with void element and text nodes
        let script3 = "
            div.innerHTML = 'hello<br>world<img>';
            div.innerHTML;
        ";
        let res3 = host.eval_with_dom(script3, &mut dom);
        assert_eq!(res3, Ok("hello<br>world<img>".to_string()));

        // 5. Query and set innerHTML on a non-element (Text node) - should do nothing (not crash or mutate) and return undefined
        let script4 = "
            let textNode = document.createTextNode('sample');
            let res_get = String(textNode.innerHTML);
            textNode.innerHTML = '<b>failed</b>';
            let res_text = textNode.textContent;
            [res_get, res_text].join('|');
        ";
        let res4 = host.eval_with_dom(script4, &mut dom);
        assert_eq!(res4, Ok("undefined|sample".to_string()));

        // 6. Check escaping of text/attributes
        let script6 = "
            div.innerHTML = '<span title=\"a &quot; b &amp; c\">&lt;test&gt;</span>';
            div.innerHTML;
        ";
        let res6 = host.eval_with_dom(script6, &mut dom);
        assert_eq!(
            res6,
            Ok("<span title=\"a &quot; b &amp; c\">&lt;test&gt;</span>".to_string())
        );
    }

    #[test]
    fn test_element_outer_html_getter_setter() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        // 1. Basic outerHTML GET on an element with known children vs innerHTML
        let script1 = "
            let div1 = document.createElement('div');
            div1.setAttribute('id', 'x');
            document.appendChild(div1);
            div1.innerHTML = '<span>hi</span>';
            let outer1 = div1.outerHTML;
            let inner1 = div1.innerHTML;
            [outer1, inner1].join('|');
        ";
        let res1 = host.eval_with_dom(script1, &mut dom);
        assert_eq!(
            res1,
            Ok("<div id=\"x\"><span>hi</span></div>|<span>hi</span>".to_string())
        );

        // 2. SET: replacing an element via el.outerHTML = '<p>new</p>'
        // Old element is removed, new <p>new</p> is in its place.
        let script2 = "
            let parent2 = document.createElement('div');
            document.appendChild(parent2);
            let child2 = document.createElement('span');
            parent2.appendChild(child2);
            child2.outerHTML = '<p>new</p>';
            parent2.innerHTML;
        ";
        let res2 = host.eval_with_dom(script2, &mut dom);
        assert_eq!(res2, Ok("<p>new</p>".to_string()));

        // 3. SET multiple nodes: el.outerHTML = '<a>1</a><b>2</b>'
        // Inserts both replacement nodes in correct order where the old element was.
        let script3 = "
            let parent3 = document.createElement('div');
            document.appendChild(parent3);
            
            // Add some context nodes around the target element to verify relative positioning/ordering
            let pre3 = document.createElement('pre');
            parent3.appendChild(pre3);
            
            let child3 = document.createElement('span');
            parent3.appendChild(child3);
            
            let post3 = document.createElement('code');
            parent3.appendChild(post3);
            
            child3.outerHTML = '<a>1</a><b>2</b>';
            parent3.innerHTML;
        ";
        let res3 = host.eval_with_dom(script3, &mut dom);
        assert_eq!(
            res3,
            Ok("<pre></pre><a>1</a><b>2</b><code></code>".to_string())
        );

        // 4. Non-element (Text node) and/or parentless element:
        // Setting outerHTML on text node is a no-op, get returns undefined.
        // Setting outerHTML on parentless element is a no-op, does not crash.
        let script4 = "
            let textNode4 = document.createTextNode('sample');
            let res_text_get4 = String(textNode4.outerHTML);
            textNode4.outerHTML = '<b>failed</b>';
            let res_text_content4 = textNode4.textContent;
            
            let parentless4 = document.createElement('div');
            parentless4.outerHTML = '<a>replaced</a>';
            let res_parentless_get4 = parentless4.outerHTML;
            
            [res_text_get4, res_text_content4, res_parentless_get4].join('|');
        ";
        let res4 = host.eval_with_dom(script4, &mut dom);
        assert_eq!(res4, Ok("undefined|sample|<div></div>".to_string()));
    }

    #[test]
    fn test_location_uninitialized() {
        let mut host = BoaHost::new();
        assert!(
            host.eval("if (window.location.href !== '') throw 'href mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.protocol !== '') throw 'protocol mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.host !== '') throw 'host mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.hostname !== '') throw 'hostname mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.port !== '') throw 'port mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.pathname !== '') throw 'pathname mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.search !== '') throw 'search mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.hash !== '') throw 'hash mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.origin !== '') throw 'origin mismatch';")
                .is_ok()
        );
    }

    #[test]
    fn test_location_initialized() {
        let mut host = BoaHost::new();
        host.set_document_url("https://example.com:8080/path/to/page?q=foo#frag");
        assert!(host.eval("if (window.location.href !== 'https://example.com:8080/path/to/page?q=foo#frag') throw 'href mismatch';").is_ok());
        assert!(
            host.eval("if (window.location.protocol !== 'https:') throw 'protocol mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.hostname !== 'example.com') throw 'hostname mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.port !== '8080') throw 'port mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.host !== 'example.com:8080') throw 'host mismatch';")
                .is_ok()
        );
        assert!(
            host.eval(
                "if (window.location.pathname !== '/path/to/page') throw 'pathname mismatch';"
            )
            .is_ok()
        );
        assert!(
            host.eval("if (window.location.search !== '?q=foo') throw 'search mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.hash !== '#frag') throw 'hash mismatch';")
                .is_ok()
        );
        assert!(host.eval("if (window.location.origin !== 'https://example.com:8080') throw 'origin mismatch';").is_ok());
        assert!(host.eval("if (document.location !== window.location) throw 'document.location !== window.location';").is_ok());
        assert!(host.eval("if (window.location.toString() !== 'https://example.com:8080/path/to/page?q=foo#frag') throw 'toString mismatch';").is_ok());
    }

    #[test]
    fn test_location_no_port_or_search_or_hash() {
        let mut host = BoaHost::new();
        host.set_document_url("http://example.org/home");
        assert!(
            host.eval(
                "if (window.location.href !== 'http://example.org/home') throw 'href mismatch';"
            )
            .is_ok()
        );
        assert!(
            host.eval("if (window.location.protocol !== 'http:') throw 'protocol mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.hostname !== 'example.org') throw 'hostname mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.port !== '') throw 'port mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.host !== 'example.org') throw 'host mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.pathname !== '/home') throw 'pathname mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.search !== '') throw 'search mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.location.hash !== '') throw 'hash mismatch';")
                .is_ok()
        );
        assert!(
            host.eval(
                "if (window.location.origin !== 'http://example.org') throw 'origin mismatch';"
            )
            .is_ok()
        );
    }

    #[test]
    fn test_lifecycle_readystate_loading_and_complete() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("original".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        let script_text = dom.create_node(NodeData::Text(
            "document.getElementById('target').textContent = document.readyState;".to_string(),
        ));
        dom.append_child(script_id, script_text);
        dom.append_child(document, script_id);

        let mut host = BoaHost::new();
        let script_ids = vec![script_id];
        for id in script_ids {
            let src = dom.text_content(id);
            let _ =
                host.eval_with_dom_and_styles(&src, &mut dom, &std::collections::HashMap::new());
        }

        // readyState should be 'loading' during the execution of the script
        assert_eq!(dom.text_content(element_id), "loading");

        // After all inline scripts, we run lifecycle events which transitions state to complete
        let _ = host.dispatch_lifecycle_events(&mut dom, &std::collections::HashMap::new());

        // We can query the state via host directly
        let state_res = host.eval_with_dom("document.readyState", &mut dom).unwrap();
        assert_eq!(state_res, "complete");
    }

    #[test]
    fn test_lifecycle_domcontentloaded_listener() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("original".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        // Register a DOMContentLoaded listener during script execution
        let script_text = dom.create_node(NodeData::Text(
            r#"
            document.addEventListener('DOMContentLoaded', () => {
                document.getElementById('target').textContent = 'domloaded';
            });
            "#
            .to_string(),
        ));
        dom.append_child(script_id, script_text);
        dom.append_child(document, script_id);

        let mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());
        assert_eq!(mutated_dom.text_content(element_id), "domloaded");
    }

    #[test]
    fn test_lifecycle_window_load_listener() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("original".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        // Register a window load listener during script execution
        let script_text = dom.create_node(NodeData::Text(
            r#"
            window.addEventListener('load', () => {
                document.getElementById('target').textContent = 'windowloaded';
            });
            "#
            .to_string(),
        ));
        dom.append_child(script_id, script_text);
        dom.append_child(document, script_id);

        let mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());
        assert_eq!(mutated_dom.text_content(element_id), "windowloaded");
    }

    #[test]
    fn test_lifecycle_readystate_idiom() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("0".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        // The readyState idiom that registers DOMContentLoaded listener during script execution
        // and only calls init once.
        let script_text = dom.create_node(NodeData::Text(
            r#"
            let count = 0;
            function init() {
                count += 1;
                document.getElementById('target').textContent = String(count);
            }
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', init);
            } else {
                init();
            }
            "#
            .to_string(),
        ));
        dom.append_child(script_id, script_text);
        dom.append_child(document, script_id);

        let mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());
        assert_eq!(mutated_dom.text_content(element_id), "1");
    }

    #[test]
    fn test_lifecycle_throwing_listener_safety() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("original".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        // Register multiple listeners where first one throws but second must still run
        let script_text = dom.create_node(NodeData::Text(
            r#"
            document.addEventListener('DOMContentLoaded', () => {
                throw new Error('This should be caught safely');
            });
            document.addEventListener('DOMContentLoaded', () => {
                document.getElementById('target').textContent = 'second_ran';
            });
            "#
            .to_string(),
        ));
        dom.append_child(script_id, script_text);
        dom.append_child(document, script_id);

        let mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());
        assert_eq!(mutated_dom.text_content(element_id), "second_ran");
    }
}
