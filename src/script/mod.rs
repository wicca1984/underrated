//! Scripting module providing JavaScript execution via the Boa engine.
//!
//! This module implements the `ScriptHost` port, allowing the browser engine
//! to execute scripts. The current implementation uses the `boa_engine` crate.

pub mod encoding;
pub mod storage;
pub mod timer;
pub mod xhr;

use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;
use boa_engine::class::{Class, ClassBuilder};
use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsString, JsValue, NativeFunction, Source};
use boa_engine::{JsData, JsNativeError, JsResult};
use boa_gc::{Finalize, GcRefCell, Trace};
use std::cell::RefCell;
use std::collections::HashMap;

pub mod crypto;
pub mod event;
pub mod formdata;
pub mod navigator;
pub mod performance;

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
/// Scripts exceeding this limit will be skipped entirely.
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
    static CURRENT_STYLES: RefCell<Option<HashMap<NodeId, crate::style::CategorizedComputedStyle>>> = const { RefCell::new(None) };
    static PENDING_NAVIGATION: RefCell<Option<String>> = const { RefCell::new(None) };
    static ELEMENT_SCROLL_TOP: RefCell<HashMap<NodeId, f64>> = RefCell::new(HashMap::new());
    static ELEMENT_SCROLL_LEFT: RefCell<HashMap<NodeId, f64>> = RefCell::new(HashMap::new());
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

        // Setup Encoding API built-ins (t0505)
        let _ = encoding::register_encoding_builtins(&mut context);
        let _ = encoding::register_base64_builtins(&mut context);

        // Setup structuredClone global (t0514)
        let _ = context.register_global_builtin_callable(
            JsString::from("structuredClone"),
            1,
            NativeFunction::from_fn_ptr(structured_clone_fn),
        );

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
        let _ = context.register_global_class::<CustomEvent>();
        let _ = context.register_global_class::<URLSearchParams>();
        let _ = context.register_global_class::<formdata::FormData>();
        let _ = context.register_global_class::<AbortSignal>();
        let _ = context.register_global_class::<AbortController>();
        let _ = context.register_global_class::<DOMParser>();
        let _ = context.register_global_class::<MutationObserver>();
        let _ = context.register_global_class::<MutationRecord>();
        let _ = context.register_global_class::<Blob>();

        let bridge = ObjectInitializer::new(context)
            .function(
                NativeFunction::from_fn_ptr(bridge_active_element),
                JsString::from("activeElement"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_focus),
                JsString::from("focus"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_blur),
                JsString::from("blur"),
                0,
            )
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
                NativeFunction::from_fn_ptr(bridge_create_comment),
                JsString::from("createComment"),
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
                NativeFunction::from_fn_ptr(bridge_get_elements_by_name),
                JsString::from("getElementsByName"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_append_child),
                JsString::from("appendChild"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_append),
                JsString::from("append"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_prepend),
                JsString::from("prepend"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_before),
                JsString::from("before"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_after),
                JsString::from("after"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_replace_with),
                JsString::from("replaceWith"),
                0,
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
                NativeFunction::from_fn_ptr(bridge_insert_adjacent_element),
                JsString::from("insertAdjacentElement"),
                3,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_insert_adjacent_html),
                JsString::from("insertAdjacentHTML"),
                3,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_insert_adjacent_text),
                JsString::from("insertAdjacentText"),
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
                NativeFunction::from_fn_ptr(bridge_normalize),
                JsString::from("normalize"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_is_connected),
                JsString::from("isConnected"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_contains),
                JsString::from("contains"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_has_child_nodes),
                JsString::from("hasChildNodes"),
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
                NativeFunction::from_fn_ptr(bridge_get_node_value),
                JsString::from("getNodeValue"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_set_node_value),
                JsString::from("setNodeValue"),
                2,
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
            .function(
                NativeFunction::from_fn_ptr(bridge_get_bounding_client_rect),
                JsString::from("getBoundingClientRect"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_get_scroll_top),
                JsString::from("getScrollTop"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_set_scroll_top),
                JsString::from("setScrollTop"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_get_scroll_left),
                JsString::from("getScrollLeft"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_set_scroll_left),
                JsString::from("setScrollLeft"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(bridge_scroll_into_view),
                JsString::from("scrollIntoView"),
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

        let performance = performance::create_performance(context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        let crypto = crypto::create_crypto(context);
        let _ =
            context.register_global_property(JsString::from("crypto"), crypto, Attribute::all());

        let global = context.global_object().clone();
        let _ =
            context.register_global_property(JsString::from("window"), global, Attribute::all());

        let _ = context.register_global_builtin_callable(
            JsString::from("__request_navigation__"),
            1,
            NativeFunction::from_fn_ptr(request_navigation),
        );

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
                        __request_navigation__(String(val));
                    },
                    assign(url) {
                        __request_navigation__(String(url));
                    },
                    replace(url) {
                        __request_navigation__(String(url));
                    },
                    reload() {
                        __request_navigation__(window.__document_location__.href);
                    },

                    toString() {
                        return this.href;
                    }
                };

                window.location = locationObj;
                document.location = locationObj;

                // --- HTML5 History API implementation (t0497) ---
                (function() {
                    function parseAbsoluteUrl(urlStr) {
                        const match = urlStr.match(/^(([a-zA-Z][a-zA-Z0-9.+-]*):)?\/\/([^/?#]*)([^?#]*)(\?[^#]*)?(#.*)?$/);
                        if (match) {
                            const protocol = match[2] ? match[2] + ":" : "";
                            const host = match[3] || "";
                            const pathname = match[4] || "/";
                            const search = match[5] || "";
                            const hash = match[6] || "";
                            let hostname = host;
                            let port = "";
                            const portIndex = host.lastIndexOf(':');
                            if (portIndex !== -1 && portIndex > host.lastIndexOf(']')) {
                                hostname = host.substring(0, portIndex);
                                port = host.substring(portIndex + 1);
                            }
                            const origin = protocol ? protocol + "//" + host : "";
                            return {
                                href: urlStr,
                                protocol,
                                host,
                                hostname,
                                port,
                                pathname,
                                search,
                                hash,
                                origin
                            };
                        }
                        return {
                            href: urlStr,
                            protocol: "",
                            host: "",
                            hostname: "",
                            port: "",
                            pathname: urlStr,
                            search: "",
                            hash: "",
                            origin: ""
                        };
                    }

                    function resolveAndParse(url) {
                        if (/^[a-zA-Z][a-zA-Z0-9.+-]*:/.test(url)) {
                            return parseAbsoluteUrl(url);
                        }
                        const loc = window.__document_location__;
                        if (url.startsWith('//')) {
                            return parseAbsoluteUrl(loc.protocol + url);
                        }
                        const urlMatch = url.match(/^([^?#]*)(\?[^#]*)?(#.*)?$/);
                        const rawPath = urlMatch[1] || "";
                        const rawSearch = urlMatch[2] || "";
                        const rawHash = urlMatch[3] || "";

                        let newPathname = loc.pathname;
                        let newSearch = loc.search;
                        let newHash = loc.hash;

                        if (url.startsWith('/')) {
                            newPathname = rawPath;
                            newSearch = rawSearch;
                            newHash = rawHash;
                        } else if (url.startsWith('?')) {
                            newSearch = rawSearch;
                            newHash = rawHash;
                        } else if (url.startsWith('#')) {
                            newHash = rawHash;
                        } else {
                            const lastSlash = loc.pathname.lastIndexOf('/');
                            let basePath = "/";
                            if (lastSlash !== -1) {
                                basePath = loc.pathname.substring(0, lastSlash + 1);
                            }
                            newPathname = basePath + rawPath;
                            newSearch = rawSearch;
                            newHash = rawHash;
                        }

                        let href = "";
                        if (loc.protocol || loc.host) {
                            href = (loc.protocol || "") + "//" + (loc.host || "") + newPathname + newSearch + newHash;
                        } else {
                            href = newPathname + newSearch + newHash;
                        }

                        return {
                            href,
                            protocol: loc.protocol,
                            host: loc.host,
                            hostname: loc.hostname,
                            port: loc.port,
                            pathname: newPathname,
                            search: newSearch,
                            hash: newHash,
                            origin: loc.origin
                        };
                    }

                    function updateDocumentLocation(parsed) {
                        window.__document_location__.href = parsed.href;
                        window.__document_location__.protocol = parsed.protocol;
                        window.__document_location__.host = parsed.host;
                        window.__document_location__.hostname = parsed.hostname;
                        window.__document_location__.port = parsed.port;
                        window.__document_location__.pathname = parsed.pathname;
                        window.__document_location__.search = parsed.search;
                        window.__document_location__.hash = parsed.hash;
                        window.__document_location__.origin = parsed.origin;
                    }

                    function cloneState(state) {
                        if (state === undefined) return null;
                        try {
                            return JSON.parse(JSON.stringify(state));
                        } catch (e) {
                            return state;
                        }
                    }

                    function getEntryUrl(entry) {
                        if (!entry) return "";
                        if (entry._url !== undefined && entry._url !== null && entry._url !== "") {
                            return entry._url;
                        }
                        return window.__document_location__.href || "";
                    }

                    const entries = [
                        {
                            state: null,
                            title: "",
                            _url: undefined
                        }
                    ];
                    let currentIndex = 0;

                    const historyObj = {
                        get state() {
                            return entries[currentIndex] ? entries[currentIndex].state : null;
                        },
                        get length() {
                            return entries.length;
                        },
                        pushState(state, title, url) {
                            // If the first entry's _url is still undefined, capture the current href
                            if (entries[0] && entries[0]._url === undefined) {
                                entries[0]._url = window.__document_location__.href;
                            }

                            // Truncate any forward entries after current index
                            entries.splice(currentIndex + 1);

                            let resolvedUrl = window.__document_location__.href;
                            if (url !== undefined && url !== null && url !== "") {
                                const parsed = resolveAndParse(String(url));
                                updateDocumentLocation(parsed);
                                resolvedUrl = parsed.href;
                            }

                            const cloned = cloneState(state);
                            entries.push({
                                state: cloned,
                                title: title || "",
                                _url: resolvedUrl
                            });
                            currentIndex = entries.length - 1;
                        },
                        replaceState(state, title, url) {
                            if (entries[0] && entries[0]._url === undefined) {
                                entries[0]._url = window.__document_location__.href;
                            }

                            let resolvedUrl = window.__document_location__.href;
                            if (url !== undefined && url !== null && url !== "") {
                                const parsed = resolveAndParse(String(url));
                                updateDocumentLocation(parsed);
                                resolvedUrl = parsed.href;
                            }

                            const cloned = cloneState(state);
                            entries[currentIndex] = {
                                state: cloned,
                                title: title || "",
                                _url: resolvedUrl
                            };
                        },
                        go(delta) {
                            if (typeof delta !== 'number') {
                                delta = parseInt(delta, 10);
                                if (isNaN(delta)) {
                                    return;
                                }
                            }
                            let targetIndex = currentIndex + delta;
                            if (targetIndex < 0) {
                                targetIndex = 0;
                            }
                            if (targetIndex >= entries.length) {
                                targetIndex = entries.length - 1;
                            }
                            if (targetIndex !== currentIndex) {
                                currentIndex = targetIndex;
                                const entry = entries[currentIndex];
                                if (entry) {
                                    const parsed = resolveAndParse(getEntryUrl(entry));
                                    updateDocumentLocation(parsed);

                                    const event = new Event('popstate');
                                    try {
                                        event.state = entry.state;
                                    } catch (e) {}
                                    try {
                                        Object.defineProperty(event, 'state', {
                                            value: entry.state,
                                            writable: true,
                                            configurable: true,
                                            enumerable: true
                                        });
                                    } catch (e) {}
                                    window.dispatchEvent(event);
                                }
                            }
                        },
                        back() {
                            this.go(-1);
                        },
                        forward() {
                            this.go(1);
                        }
                    };

                    window.history = historyObj;
                })();

                class DOMException extends Error {
                    constructor(message, name) {
                        super(message);
                        this.name = name || "DOMException";
                    }
                }
                window.DOMException = DOMException;

                class Node extends EventTarget {}
                class Element extends Node {}
                class Document extends Node {
                    constructor(key) {
                        super();
                        this.__key__ = key;
                        document.__node_registry__[key] = this;
                        this.__readyState__ = 'complete';
                    }
                }
                window.Node = Node;
                window.Element = Element;
                window.Document = Document;

                Element.prototype.matches = function(selector) {
                    if (this.nodeType !== 1) return false;
                    return bridge.matches(this.__key__, String(selector));
                };

                Element.prototype.closest = function(selector) {
                    if (this.nodeType !== 1) return null;
                    const key = bridge.closest(this.__key__, String(selector));
                    return getOrCreateNode(key);
                };

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

                function dispatchFocusEvent(target, type, bubbles) {
                    const event = new Event(type);
                    let currentTargetVal = target;
                    Object.defineProperty(event, 'target', {
                        get() { return target; },
                        configurable: true
                    });
                    Object.defineProperty(event, 'currentTarget', {
                        get() { return currentTargetVal; },
                        configurable: true
                    });
                    let propagationStopped = false;
                    const originalStopPropagation = event.stopPropagation;
                    event.stopPropagation = function() {
                        propagationStopped = true;
                        if (originalStopPropagation) {
                            originalStopPropagation.call(this);
                        }
                    };
                    let curr = target;
                    while (curr) {
                        currentTargetVal = curr;
                        curr.dispatchEvent(event);
                        if (propagationStopped || !bubbles) {
                            break;
                        }
                        if (curr === document) {
                            currentTargetVal = window;
                            window.dispatchEvent(event);
                            break;
                        }
                        curr = curr.parentNode;
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

                function isEqualNodeHelper(node, other) {
                    if (!other) return false;
                    if (node === other) return true;
                    if (node.nodeType !== other.nodeType) return false;
                    if (node.nodeName !== other.nodeName) return false;

                    if (node.nodeType === 1) { // ELEMENT_NODE
                        const thisAttrs = node.getAttributeNames();
                        const otherAttrs = other.getAttributeNames();
                        if (thisAttrs.length !== otherAttrs.length) return false;
                        for (let i = 0; i < thisAttrs.length; i++) {
                            const attr = thisAttrs[i];
                            if (node.getAttribute(attr) !== other.getAttribute(attr)) {
                                return false;
                            }
                        }
                    }

                    if (node.nodeType === 3 || node.nodeType === 8) { // Text or Comment
                        // TODO(spec): comment text is currently not fully exposed in Rust text_content.
                        if (node.textContent !== other.textContent) {
                            return false;
                        }
                    }

                    const thisChildren = node.childNodes;
                    const otherChildren = other.childNodes;
                    if (thisChildren.length !== otherChildren.length) return false;
                    for (let i = 0; i < thisChildren.length; i++) {
                        if (!isEqualNodeHelper(thisChildren[i], otherChildren[i])) {
                            return false;
                        }
                    }

                    return true;
                }

                function compareDocumentPositionHelper(node, other) {
                    // TODO(spec): Attribute nodes and shadow roots are not yet supported or exposed, so some advanced edge cases are simplified.
                    const DOCUMENT_POSITION_DISCONNECTED = 1;
                    const DOCUMENT_POSITION_PRECEDING = 2;
                    const DOCUMENT_POSITION_FOLLOWING = 4;
                    const DOCUMENT_POSITION_CONTAINS = 8;
                    const DOCUMENT_POSITION_CONTAINED_BY = 16;
                    const DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC = 32;

                    if (!other || !other.__key__) {
                        return DOCUMENT_POSITION_DISCONNECTED | DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC | DOCUMENT_POSITION_PRECEDING;
                    }

                    if (node === other) {
                        return 0;
                    }

                    // Check containment first
                    if (other.contains(node)) {
                        return DOCUMENT_POSITION_CONTAINS | DOCUMENT_POSITION_PRECEDING;
                    }
                    if (node.contains(other)) {
                        return DOCUMENT_POSITION_CONTAINED_BY | DOCUMENT_POSITION_FOLLOWING;
                    }

                    // Traverse up to find common ancestor
                    const thisAncestors = [];
                    let curr = node;
                    while (curr) {
                        thisAncestors.push(curr);
                        curr = curr.parentNode;
                    }

                    const otherAncestors = [];
                    let currOther = other;
                    while (currOther) {
                        otherAncestors.push(currOther);
                        currOther = currOther.parentNode;
                    }

                    const thisRoot = thisAncestors[thisAncestors.length - 1];
                    const otherRoot = otherAncestors[otherAncestors.length - 1];

                    if (thisRoot !== otherRoot) {
                        // Different trees
                        return DOCUMENT_POSITION_DISCONNECTED | DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC | DOCUMENT_POSITION_PRECEDING;
                    }

                    // Find lowest common ancestor
                    let i = thisAncestors.length - 1;
                    let j = otherAncestors.length - 1;
                    let lca = null;
                    while (i >= 0 && j >= 0 && thisAncestors[i] === otherAncestors[j]) {
                        lca = thisAncestors[i];
                        i--;
                        j--;
                    }

                    if (!lca) {
                        return DOCUMENT_POSITION_DISCONNECTED | DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC | DOCUMENT_POSITION_PRECEDING;
                    }

                    // Determine document order of the children of the LCA
                    const children = lca.childNodes;
                    const indexThis = children.indexOf(thisAncestors[i]);
                    const indexOther = children.indexOf(otherAncestors[j]);

                    if (indexThis === -1 || indexOther === -1) {
                        return DOCUMENT_POSITION_DISCONNECTED | DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC | DOCUMENT_POSITION_PRECEDING;
                    }

                    if (indexThis < indexOther) {
                        return DOCUMENT_POSITION_FOLLOWING;
                    } else {
                        return DOCUMENT_POSITION_PRECEDING;
                    }
                }

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
                        hasAttributes() {
                            return this.getAttributeNames().length > 0;
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
                        insertAdjacentElement(position, element) {
                            if (this.nodeType !== 1) return null;
                            if (!element || !element.__key__) {
                                throw new TypeError("element must be a Node");
                            }
                            const resKey = bridge.insertAdjacentElement(this.__key__, String(position), element.__key__);
                            return getOrCreateNode(resKey);
                        },
                        insertAdjacentHTML(position, html) {
                            if (this.nodeType !== 1) return;
                            bridge.insertAdjacentHTML(this.__key__, String(position), String(html));
                        },
                        insertAdjacentText(position, data) {
                            if (this.nodeType !== 1) return;
                            const pos = String(position).trim().toLowerCase();
                            if (pos !== "beforebegin" && pos !== "afterbegin" && pos !== "beforeend" && pos !== "afterend") {
                                throw new DOMException("SyntaxError: The position provided is not one of the allowed values.", "SyntaxError");
                            }
                            bridge.insertAdjacentText(this.__key__, pos, String(data));
                        },
                        normalize() {
                            bridge.normalize(this.__key__);
                        },
                        click() {
                            if (this.nodeType !== 1) return;
                            const event = new Event('click');
                            this.dispatchEvent(event);
                        },
                        focus() {
                            if (this.nodeType !== 1) return;
                            const currentKey = bridge.activeElement();
                            if (currentKey === this.__key__) return;

                            const prev = currentKey ? getOrCreateNode(currentKey) : null;
                            bridge.focus(this.__key__);

                            if (prev) {
                                dispatchFocusEvent(prev, 'focusout', true);
                                dispatchFocusEvent(prev, 'blur', false);
                            }
                            dispatchFocusEvent(this, 'focus', false);
                            dispatchFocusEvent(this, 'focusin', true);
                        },
                        blur() {
                            if (this.nodeType !== 1) return;
                            const currentKey = bridge.activeElement();
                            if (currentKey !== this.__key__) return;

                            bridge.blur();

                            dispatchFocusEvent(this, 'focusout', true);
                            dispatchFocusEvent(this, 'blur', false);
                        }
                    };

                    const isElement = bridge.nodeType(key) === 1;
                    if (isElement) {
                        Object.setPrototypeOf(node, Element.prototype);
                    } else {
                        Object.setPrototypeOf(node, Node.prototype);
                    }
                    node.addEventListener = bridge.addEventListener;
                    node.removeEventListener = bridge.removeEventListener;
                    node.dispatchEvent = bridge.dispatchEvent;

                    Object.defineProperty(node, 'textContent', {
                        get() {
                            const type = this.nodeType;
                            if (type === 3 || type === 8) {
                                return this.nodeValue;
                            }
                            return bridge.getTextContent(this.__key__);
                        },
                        set(val) {
                            const type = this.nodeType;
                            if (type === 3 || type === 8) {
                                this.nodeValue = val;
                            } else {
                                bridge.setTextContent(this.__key__, String(val));
                            }
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'innerText', {
                        get() {
                            // TODO(spec): innerText is layout-aware in real browsers; this is a whitespace-collapsing textContent approximation
                            if (this.nodeType !== 1) return '';
                            const text = bridge.getTextContent(this.__key__) || '';
                            return text.replace(/^[ \t\n\r\f]+|[ \t\n\r\f]+$/g, '').replace(/[ \t\n\r\f]+/g, ' ');
                        },
                        set(val) {
                            if (this.nodeType !== 1) return;
                            bridge.setTextContent(this.__key__, String(val));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'outerText', {
                        get() {
                            return this.innerText;
                        },
                        // TODO(spec): outerText setter (destructive element replacement) not implemented
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

                    Object.defineProperty(node, 'isConnected', {
                        get() {
                            return bridge.isConnected(this.__key__);
                        },
                        enumerable: true,
                        configurable: true
                    });

                    node.contains = function(otherNode) {
                        return bridge.contains(this.__key__, (otherNode && otherNode.__key__) || null);
                    };

                    node.hasChildNodes = function() {
                        return bridge.hasChildNodes(this.__key__);
                    };

                    node.isSameNode = function(otherNode) {
                        return this === otherNode;
                    };

                    node.getRootNode = function(options) {
                        let curr = this;
                        while (curr.parentNode) {
                            curr = curr.parentNode;
                        }
                        return curr;
                    };

                    node.isEqualNode = function(otherNode) {
                        return isEqualNodeHelper(this, otherNode);
                    };

                    node.compareDocumentPosition = function(otherNode) {
                        return compareDocumentPositionHelper(this, otherNode);
                    };

                    Object.defineProperty(node, 'DOCUMENT_POSITION_DISCONNECTED', { value: 1, enumerable: true });
                    Object.defineProperty(node, 'DOCUMENT_POSITION_PRECEDING', { value: 2, enumerable: true });
                    Object.defineProperty(node, 'DOCUMENT_POSITION_FOLLOWING', { value: 4, enumerable: true });
                    Object.defineProperty(node, 'DOCUMENT_POSITION_CONTAINS', { value: 8, enumerable: true });
                    Object.defineProperty(node, 'DOCUMENT_POSITION_CONTAINED_BY', { value: 16, enumerable: true });
                    Object.defineProperty(node, 'DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC', { value: 32, enumerable: true });

                    node.normalize = function() {
                        bridge.normalize(this.__key__);
                    };

                    node.getBoundingClientRect = function() {
                        if (this.nodeType !== 1) return null;
                        return bridge.getBoundingClientRect(this.__key__);
                    };

                    node.scrollIntoView = function(arg) {
                        if (this.nodeType !== 1) return;
                        // TODO(spec): record smooth-vs-auto behavior (e.g. from arg) rather than guessing

                        // Locate appropriate scroll container
                        let container = this.parentElement;
                        while (container) {
                            if (container === document.body || container === document.documentElement) {
                                break;
                            }
                            const style = typeof window !== 'undefined' && window.getComputedStyle ? window.getComputedStyle(container) : null;
                            if (style) {
                                const overflow = style.getPropertyValue('overflow');
                                const overflowY = style.getPropertyValue('overflow-y') || overflow;
                                const overflowX = style.getPropertyValue('overflow-x') || overflow;
                                if (overflowY === 'scroll' || overflowY === 'auto' || overflowX === 'scroll' || overflowX === 'auto') {
                                    break;
                                }
                            }
                            container = container.parentElement;
                        }
                        if (!container) {
                            container = document.documentElement || document.body;
                        }

                        if (container) {
                            const element_rect = this.getBoundingClientRect();
                            if (element_rect) {
                                let new_scrollTop = container.scrollTop + element_rect.top;
                                let new_scrollLeft = container.scrollLeft + element_rect.left;
                                if (container !== document.documentElement && container !== document.body) {
                                    const container_rect = container.getBoundingClientRect();
                                    if (container_rect) {
                                        new_scrollTop = container.scrollTop + (element_rect.top - container_rect.top);
                                        new_scrollLeft = container.scrollLeft + (element_rect.left - container_rect.left);
                                    }
                                }
                                container.scrollTop = new_scrollTop;
                                container.scrollLeft = new_scrollLeft;
                            }
                        }

                        bridge.scrollIntoView(this.__key__, arg);
                    };

                    node.getClientRects = function() {
                        if (this.nodeType !== 1) return null;
                        const rect = this.getBoundingClientRect();
                        const rects = rect ? [rect] : [];
                        Object.defineProperty(rects, 'item', {
                            value: function(index) {
                                const idx = Number(index) >>> 0;
                                if (idx >= this.length) return null;
                                return this[idx];
                            },
                            enumerable: false,
                            configurable: true,
                            writable: true
                        });
                        // TODO(spec): real DOM may return multiple rects for fragmented inline content; we approximate with the single bounding rect.
                        return rects;
                    };

                    Object.defineProperty(node, 'offsetWidth', {
                        get() {
                            if (this.nodeType !== 1) return 0;
                            const rect = this.getBoundingClientRect();
                            return rect ? Math.round(rect.width) : 0;
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'offsetHeight', {
                        get() {
                            if (this.nodeType !== 1) return 0;
                            const rect = this.getBoundingClientRect();
                            return rect ? Math.round(rect.height) : 0;
                        },
                        enumerable: true,
                        configurable: true
                    });

                    // TODO(spec): offsetTop/offsetLeft should be relative to offsetParent
                    Object.defineProperty(node, 'offsetTop', {
                        get() {
                            if (this.nodeType !== 1) return 0;
                            const rect = this.getBoundingClientRect();
                            return (rect && rect.top !== undefined) ? Math.round(rect.top) : 0;
                        },
                        enumerable: true,
                        configurable: true
                    });

                    // TODO(spec): offsetTop/offsetLeft should be relative to offsetParent
                    Object.defineProperty(node, 'offsetLeft', {
                        get() {
                            if (this.nodeType !== 1) return 0;
                            const rect = this.getBoundingClientRect();
                            return (rect && rect.left !== undefined) ? Math.round(rect.left) : 0;
                        },
                        enumerable: true,
                        configurable: true
                    });

                    // TODO(spec): clientWidth approximates border-box width from getBoundingClientRect
                    // because scrollbars and overflow scrolling are not currently modeled in this engine.
                    Object.defineProperty(node, 'clientWidth', {
                        get() {
                            if (this.nodeType !== 1) return 0;
                            const rect = this.getBoundingClientRect();
                            return rect ? Math.round(rect.width) : 0;
                        },
                        enumerable: true,
                        configurable: true
                    });

                    // TODO(spec): clientHeight approximates border-box height from getBoundingClientRect
                    // because scrollbars and overflow scrolling are not currently modeled in this engine.
                    Object.defineProperty(node, 'clientHeight', {
                        get() {
                            if (this.nodeType !== 1) return 0;
                            const rect = this.getBoundingClientRect();
                            return rect ? Math.round(rect.height) : 0;
                        },
                        enumerable: true,
                        configurable: true
                    });

                    // TODO(spec): scrollWidth approximates border-box width from getBoundingClientRect
                    // because scrollbars and overflow scrolling are not currently modeled in this engine.
                    Object.defineProperty(node, 'scrollWidth', {
                        get() {
                            if (this.nodeType !== 1) return 0;
                            const rect = this.getBoundingClientRect();
                            return rect ? Math.round(rect.width) : 0;
                        },
                        enumerable: true,
                        configurable: true
                    });

                    // TODO(spec): scrollHeight approximates border-box height from getBoundingClientRect
                    // because scrollbars and overflow scrolling are not currently modeled in this engine.
                    Object.defineProperty(node, 'scrollHeight', {
                        get() {
                            if (this.nodeType !== 1) return 0;
                            const rect = this.getBoundingClientRect();
                            return rect ? Math.round(rect.height) : 0;
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'scrollTop', {
                        get() {
                            if (this.nodeType !== 1) return 0;
                            return bridge.getScrollTop(this.__key__);
                        },
                        set(val) {
                            if (this.nodeType !== 1) return;
                            bridge.setScrollTop(this.__key__, Number(val));
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'scrollLeft', {
                        get() {
                            if (this.nodeType !== 1) return 0;
                            return bridge.getScrollLeft(this.__key__);
                        },
                        set(val) {
                            if (this.nodeType !== 1) return;
                            bridge.setScrollLeft(this.__key__, Number(val));
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

                    Object.defineProperty(node, 'nodeValue', {
                        get() {
                            return bridge.getNodeValue(this.__key__);
                        },
                        set(val) {
                            if (this.nodeType === 3 || this.nodeType === 8) {
                                bridge.setNodeValue(this.__key__, val === null ? "" : String(val));
                            }
                        },
                        enumerable: true,
                        configurable: true
                    });

                    Object.defineProperty(node, 'before', {
                        value: function(...args) {
                            if (!this.parentNode) return;
                            const bridgeArgs = [this.__key__];
                            for (let i = 0; i < args.length; i++) {
                                let arg = args[i];
                                if (arg && arg.__key__) {
                                    bridgeArgs.push("node", arg.__key__);
                                } else {
                                    bridgeArgs.push("text", String(arg));
                                }
                            }
                            bridge.before(...bridgeArgs);
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    Object.defineProperty(node, 'after', {
                        value: function(...args) {
                            if (!this.parentNode) return;
                            const bridgeArgs = [this.__key__];
                            for (let i = 0; i < args.length; i++) {
                                let arg = args[i];
                                if (arg && arg.__key__) {
                                    bridgeArgs.push("node", arg.__key__);
                                } else {
                                    bridgeArgs.push("text", String(arg));
                                }
                            }
                            bridge.after(...bridgeArgs);
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    // TODO(spec): ParentNode.append()/prepend() v1 — Node and string (->Text) args only; DocumentFragment expansion and other edge cases out of scope.
                    Object.defineProperty(node, 'append', {
                        value: function(...args) {
                            const bridgeArgs = [this.__key__];
                            for (let i = 0; i < args.length; i++) {
                                let arg = args[i];
                                if (arg && arg.__key__) {
                                    bridgeArgs.push("node", arg.__key__);
                                } else {
                                    bridgeArgs.push("text", String(arg));
                                }
                            }
                            bridge.append(...bridgeArgs);
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    Object.defineProperty(node, 'prepend', {
                        value: function(...args) {
                            const bridgeArgs = [this.__key__];
                            for (let i = 0; i < args.length; i++) {
                                let arg = args[i];
                                if (arg && arg.__key__) {
                                    bridgeArgs.push("node", arg.__key__);
                                } else {
                                    bridgeArgs.push("text", String(arg));
                                }
                            }
                            bridge.prepend(...bridgeArgs);
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
                        value: function(...args) {
                            if (!this.parentNode) return;
                            const bridgeArgs = [this.__key__];
                            for (let i = 0; i < args.length; i++) {
                                let arg = args[i];
                                if (arg && arg.__key__) {
                                    bridgeArgs.push("node", arg.__key__);
                                } else {
                                    bridgeArgs.push("text", String(arg));
                                }
                            }
                            bridge.replaceWith(...bridgeArgs);
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });

                    registry[key] = node;
                    return node;
                }

                window.__getOrCreateNode = getOrCreateNode;

                function decorateCollection(arr) {
                    Object.defineProperty(arr, 'item', {
                        value: function(index) {
                            const i = Number(index) | 0;
                            if (i < 0 || i >= this.length) return null;
                            return this[i];
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });
                    Object.defineProperty(arr, 'namedItem', {
                        value: function(name) {
                            const strName = String(name);
                            if (strName === "") return null;
                            for (let i = 0; i < this.length; i++) {
                                const el = this[i];
                                if (el && el.id === strName) {
                                    return el;
                                }
                            }
                            for (let i = 0; i < this.length; i++) {
                                const el = this[i];
                                if (el && typeof el.getAttribute === 'function' && el.getAttribute('name') === strName) {
                                    return el;
                                }
                            }
                            return null;
                        },
                        enumerable: false,
                        configurable: true,
                        writable: true
                    });
                    return arr;
                }

                Document.prototype.createElement = function(tagName) {
                    const key = bridge.createElement(String(tagName));
                    return getOrCreateNode(key);
                };

                Document.prototype.createTextNode = function(data) {
                    const key = bridge.createTextNode(data !== undefined ? String(data) : "");
                    return getOrCreateNode(key);
                };

                Document.prototype.createComment = function(data) {
                    const key = bridge.createComment(data !== undefined ? String(data) : "");
                    return getOrCreateNode(key);
                };

                Document.prototype.getElementById = function(id) {
                    const key = bridge.getElementById(String(id), this.__key__);
                    return getOrCreateNode(key);
                };

                Document.prototype.querySelector = function(selector) {
                    const key = bridge.querySelector(String(selector), this.__key__);
                    return getOrCreateNode(key);
                };

                Document.prototype.querySelectorAll = function(selector) {
                    const keys = bridge.querySelectorAll(String(selector), this.__key__);
                    if (!keys) return [];
                    return keys.map(key => getOrCreateNode(key));
                };

                Document.prototype.getElementsByTagName = function(tagName) {
                    const keys = bridge.getElementsByTagName(String(tagName), this.__key__);
                    if (!keys) return decorateCollection([]);
                    return decorateCollection(keys.map(key => getOrCreateNode(key)));
                };

                Document.prototype.getElementsByClassName = function(className) {
                    const keys = bridge.getElementsByClassName(String(className), this.__key__);
                    if (!keys) return decorateCollection([]);
                    return decorateCollection(keys.map(key => getOrCreateNode(key)));
                };

                Document.prototype.getElementsByName = function(name) {
                    const keys = bridge.getElementsByName(String(name), this.__key__);
                    if (!keys) return [];
                    return keys.map(key => getOrCreateNode(key));
                };

                Document.prototype.appendChild = function(child) {
                    if (!child || !child.__key__) {
                        throw new TypeError("child must be a Node");
                    }
                    bridge.appendChild(this.__key__, child.__key__);
                    return child;
                };

                Document.prototype.removeChild = function(child) {
                    if (!child || !child.__key__) {
                        throw new TypeError("child must be a Node");
                    }
                    bridge.removeChild(this.__key__, child.__key__);
                    return child;
                };

                Document.prototype.insertBefore = function(newNode, refNode) {
                    if (!newNode || !newNode.__key__) {
                        throw new TypeError("newNode must be a Node");
                    }
                    const refKey = (refNode && refNode.__key__) ? refNode.__key__ : null;
                    bridge.insertBefore(this.__key__, newNode.__key__, refKey);
                    return newNode;
                };

                Document.prototype.replaceChild = function(newChild, oldChild) {
                    if (!newChild || !newChild.__key__) {
                        throw new TypeError("newChild must be a Node");
                    }
                    if (!oldChild || !oldChild.__key__) {
                        throw new TypeError("oldChild must be a Node");
                    }
                    bridge.replaceChild(this.__key__, newChild.__key__, oldChild.__key__);
                    return oldChild;
                };

                Document.prototype.cloneNode = function(deep) {
                    const isDeep = deep !== undefined ? Boolean(deep) : false;
                    const clonedKey = bridge.cloneNode(this.__key__, isDeep);
                    return getOrCreateNode(clonedKey);
                };

                Object.setPrototypeOf(document, Document.prototype);
                document.addEventListener = bridge.addEventListener;
                document.removeEventListener = bridge.removeEventListener;
                document.dispatchEvent = bridge.dispatchEvent;

                window.addEventListener = bridge.addEventListener;
                window.removeEventListener = bridge.removeEventListener;
                window.dispatchEvent = bridge.dispatchEvent;

                document.__readyState__ = 'loading';
                Object.defineProperty(Document.prototype, 'readyState', {
                    get() {
                        return this.__readyState__ || 'loading';
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'activeElement', {
                    get() {
                        const key = bridge.activeElement();
                        if (key) {
                            return getOrCreateNode(key);
                        }
                        return this.body;
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'parentNode', {
                    get() {
                        return getOrCreateNode(bridge.parentNode(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'isConnected', {
                    get() {
                        return bridge.isConnected(this.__key__);
                    },
                    enumerable: true,
                    configurable: true
                });

                Document.prototype.contains = function(otherNode) {
                    return bridge.contains(this.__key__, (otherNode && otherNode.__key__) || null);
                };

                Document.prototype.hasChildNodes = function() {
                    return bridge.hasChildNodes(this.__key__);
                };

                Document.prototype.isSameNode = function(otherNode) {
                    return this === otherNode;
                };

                Document.prototype.getRootNode = function(options) {
                    let curr = this;
                    while (curr.parentNode) {
                        curr = curr.parentNode;
                    }
                    return curr;
                };

                Document.prototype.isEqualNode = function(otherNode) {
                    return isEqualNodeHelper(this, otherNode);
                };

                Document.prototype.compareDocumentPosition = function(otherNode) {
                    return compareDocumentPositionHelper(this, otherNode);
                };

                Object.defineProperty(Document.prototype, 'DOCUMENT_POSITION_DISCONNECTED', { value: 1, enumerable: true });
                Object.defineProperty(Document.prototype, 'DOCUMENT_POSITION_PRECEDING', { value: 2, enumerable: true });
                Object.defineProperty(Document.prototype, 'DOCUMENT_POSITION_FOLLOWING', { value: 4, enumerable: true });
                Object.defineProperty(Document.prototype, 'DOCUMENT_POSITION_CONTAINS', { value: 8, enumerable: true });
                Object.defineProperty(Document.prototype, 'DOCUMENT_POSITION_CONTAINED_BY', { value: 16, enumerable: true });
                Object.defineProperty(Document.prototype, 'DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC', { value: 32, enumerable: true });

                Document.prototype.normalize = function() {
                    bridge.normalize(this.__key__);
                };

                Object.defineProperty(Document.prototype, 'childNodes', {
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
                Object.defineProperty(Document.prototype, 'documentElement', {
                    get() {
                        return this.getElementsByTagName("html")[0] || null;
                    },
                    enumerable: true,
                    configurable: true
                });

                // spec: https://dom.spec.whatwg.org/#dom-document-body
                // TODO(spec): getElementsByTagName-based lookup does not enforce the "must be a child of documentElement" / frameset rules.
                Object.defineProperty(Document.prototype, 'body', {
                    get() {
                        return this.getElementsByTagName("body")[0] || null;
                    },
                    enumerable: true,
                    configurable: true
                });

                // spec: https://dom.spec.whatwg.org/#dom-document-head
                // TODO(spec): getElementsByTagName-based lookup does not enforce the "must be a child of documentElement" / head rules.
                Object.defineProperty(Document.prototype, 'head', {
                    get() {
                        return this.getElementsByTagName("head")[0] || null;
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'firstChild', {
                    get() {
                        return getOrCreateNode(bridge.firstChild(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'lastChild', {
                    get() {
                        return getOrCreateNode(bridge.lastChild(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'nextSibling', {
                    get() {
                        return getOrCreateNode(bridge.nextSibling(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'previousSibling', {
                    get() {
                        return getOrCreateNode(bridge.previousSibling(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'firstElementChild', {
                    get() {
                        return getOrCreateNode(bridge.firstElementChild(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'lastElementChild', {
                    get() {
                        return getOrCreateNode(bridge.lastElementChild(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'nextElementSibling', {
                    get() {
                        return getOrCreateNode(bridge.nextElementSibling(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'previousElementSibling', {
                    get() {
                        return getOrCreateNode(bridge.previousElementSibling(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'children', {
                    get() {
                        const keys = bridge.children(this.__key__);
                        if (!keys) return [];
                        return keys.map(key => getOrCreateNode(key));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'childElementCount', {
                    get() {
                        return bridge.childElementCount(this.__key__);
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'parentElement', {
                    get() {
                        return getOrCreateNode(bridge.parentElement(this.__key__));
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'tagName', {
                    get() {
                        return bridge.tagName(this.__key__);
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'nodeName', {
                    get() {
                        return bridge.nodeName(this.__key__);
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'nodeType', {
                    get() {
                        return bridge.nodeType(this.__key__);
                    },
                    enumerable: true,
                    configurable: true
                });

                Object.defineProperty(Document.prototype, 'nodeValue', {
                    get() {
                        return bridge.getNodeValue(this.__key__);
                    },
                    set(val) {
                        // no-op for Document
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

    /// Returns the most recently requested pending navigation URL string, if any,
    /// and clears the slot so it won't be returned again.
    ///
    /// The engine post-eval will consume this pending navigation to drive navigation.
    ///
    /// // TODO(spec): noting the engine-side wiring (engine::navigate) is a follow-up task and that relative URLs are resolved engine-side, not here.
    pub fn take_pending_navigation(&mut self) -> Option<String> {
        PENDING_NAVIGATION.with(|cell| cell.borrow_mut().take())
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
        if is_limit {
            let max_len = MAX_SCRIPT_LENGTH.with(|cell| *cell.borrow());
            if src.chars().count() > max_len {
                // Skip oversized script entirely. Restore DOM and return cleanly.
                let restored_dom = CURRENT_DOM.with(|cell| cell.borrow_mut().take());
                if let Some(final_dom) = restored_dom {
                    *dom = final_dom;
                }
                clear_bridge_state();
                return Ok(String::new());
            }
        }

        let final_src = src.to_string();

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

        clear_bridge_state();

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
        styles: &HashMap<NodeId, crate::style::CategorizedComputedStyle>,
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

            // Also support on<event_type> property handlers (e.g., window.onload)
            let on_prop_name = format!("on{}", event_type);
            if let Ok(on_handler_val) =
                target_obj.get(JsString::from(on_prop_name), &mut self.context)
                && let Some(on_handler_obj) = on_handler_val.as_object()
                && on_handler_obj.is_callable()
            {
                listeners_to_call.push(on_handler_val);
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
        styles: &HashMap<NodeId, crate::style::CategorizedComputedStyle>,
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

        clear_bridge_state();

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

        clear_bridge_state();

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

fn find_element_by_id(dom: &Dom, root: NodeId, id: &str) -> Option<NodeId> {
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

fn clear_bridge_state() {
    KEY_TO_NODE.with(|cell| cell.borrow_mut().clear());
    ELEMENT_SCROLL_TOP.with(|cell| cell.borrow_mut().clear());
    ELEMENT_SCROLL_LEFT.with(|cell| cell.borrow_mut().clear());
}

fn request_navigation(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let url = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        String::new()
    };
    PENDING_NAVIGATION.with(|cell| {
        *cell.borrow_mut() = Some(url);
    });
    Ok(JsValue::undefined())
}

fn structured_clone_value(value: &JsValue, context: &mut Context) -> JsResult<JsValue> {
    if value.is_symbol() {
        // TODO(spec): should be a DOMException DataCloneError
        return Err(JsError::from(
            JsNativeError::typ().with_message("Symbol is not cloneable"),
        ));
    }

    let Some(obj) = value.as_object() else {
        // It's a primitive (undefined, null, boolean, number, string, bigint)
        return Ok(value.clone());
    };

    if obj.is_callable() {
        // TODO(spec): should be a DOMException DataCloneError
        return Err(JsError::from(
            JsNativeError::typ().with_message("Function is not cloneable"),
        ));
    }

    // Get [[Class]] using Object.prototype.toString.call(value) to identify internal type
    let object_constructor = context
        .global_object()
        .get(JsString::from("Object"), context)?;
    let object_constructor_obj = object_constructor.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Object constructor not found"))
    })?;
    let prototype = object_constructor_obj.get(JsString::from("prototype"), context)?;
    let prototype_obj = prototype.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Object.prototype not found"))
    })?;
    let to_string_fn_val = prototype_obj.get(JsString::from("toString"), context)?;
    let to_string_fn = to_string_fn_val.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Object.prototype.toString not callable"))
    })?;
    let class_str_val = to_string_fn.call(value, &[], context)?;
    let class_str = class_str_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    if class_str == "[object Date]" {
        let get_time_val = obj.get(JsString::from("getTime"), context)?;
        let get_time_fn = get_time_val.as_object().ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Date.prototype.getTime not callable"))
        })?;
        let time_ms_val = get_time_fn.call(value, &[], context)?;
        let date_constructor = context
            .global_object()
            .get(JsString::from("Date"), context)?;
        let date_constructor_obj = date_constructor.as_object().ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Date constructor not found"))
        })?;
        let new_date = date_constructor_obj.construct(&[time_ms_val], None, context)?;
        return Ok(JsValue::from(new_date));
    }

    if class_str == "[object Array]" {
        let array_constructor = context
            .global_object()
            .get(JsString::from("Array"), context)?;
        let array_constructor_obj = array_constructor.as_object().ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Array constructor not found"))
        })?;

        let length_val = obj.get(JsString::from("length"), context)?;
        let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);

        let array_val = array_constructor_obj.construct(&[JsValue::from(length)], None, context)?;

        // TODO(spec): circular references not yet handled

        for i in 0..length {
            let item_val = obj.get(i, context)?;
            let cloned_item = structured_clone_value(&item_val, context)?;
            array_val.set(i, cloned_item, true, context)?;
        }
        return Ok(JsValue::from(array_val));
    }

    if class_str == "[object Map]" {
        let map_constructor = context
            .global_object()
            .get(JsString::from("Map"), context)?;
        let map_constructor_obj = map_constructor.as_object().ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Map constructor not found"))
        })?;
        let new_map_val = map_constructor_obj.construct(&[], None, context)?;

        // TODO(spec): circular references not yet handled

        let map_set_fn = new_map_val
            .get(JsString::from("set"), context)?
            .as_object()
            .ok_or_else(|| {
                JsError::from(JsNativeError::typ().with_message("Map.prototype.set not callable"))
            })?;

        let array_constructor = context
            .global_object()
            .get(JsString::from("Array"), context)?;
        let array_constructor_obj = array_constructor.as_object().ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Array constructor not found"))
        })?;
        let array_from_fn = array_constructor_obj
            .get(JsString::from("from"), context)?
            .as_object()
            .ok_or_else(|| {
                JsError::from(JsNativeError::typ().with_message("Array.from not callable"))
            })?;

        let entries_iterator = obj
            .get(JsString::from("entries"), context)?
            .as_object()
            .ok_or_else(|| {
                JsError::from(
                    JsNativeError::typ().with_message("Map.prototype.entries not callable"),
                )
            })?;
        let entries_iter_val = entries_iterator.call(value, &[], context)?;
        let entries_array_val =
            array_from_fn.call(&JsValue::undefined(), &[entries_iter_val], context)?;

        let length_val = entries_array_val
            .as_object()
            .ok_or_else(|| {
                JsError::from(JsNativeError::typ().with_message("Entries array is not an object"))
            })?
            .get(JsString::from("length"), context)?;
        let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);

        for i in 0..length {
            let pair_val = entries_array_val
                .as_object()
                .ok_or_else(|| {
                    JsError::from(
                        JsNativeError::typ().with_message("Entries array element is not an object"),
                    )
                })?
                .get(i, context)?;
            let pair_obj = pair_val.as_object().ok_or_else(|| {
                JsError::from(JsNativeError::typ().with_message("Map entry is not an object"))
            })?;
            let entry_key = pair_obj.get(0, context)?;
            let entry_val = pair_obj.get(1, context)?;
            let cloned_key = structured_clone_value(&entry_key, context)?;
            let cloned_val = structured_clone_value(&entry_val, context)?;
            map_set_fn.call(
                &JsValue::from(new_map_val.clone()),
                &[cloned_key, cloned_val],
                context,
            )?;
        }
        return Ok(JsValue::from(new_map_val));
    }

    if class_str == "[object Set]" {
        let set_constructor = context
            .global_object()
            .get(JsString::from("Set"), context)?;
        let set_constructor_obj = set_constructor.as_object().ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Set constructor not found"))
        })?;
        let new_set_val = set_constructor_obj.construct(&[], None, context)?;

        // TODO(spec): circular references not yet handled

        let set_add_fn = new_set_val
            .get(JsString::from("add"), context)?
            .as_object()
            .ok_or_else(|| {
                JsError::from(JsNativeError::typ().with_message("Set.prototype.add not callable"))
            })?;

        let array_constructor = context
            .global_object()
            .get(JsString::from("Array"), context)?;
        let array_constructor_obj = array_constructor.as_object().ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Array constructor not found"))
        })?;
        let array_from_fn = array_constructor_obj
            .get(JsString::from("from"), context)?
            .as_object()
            .ok_or_else(|| {
                JsError::from(JsNativeError::typ().with_message("Array.from not callable"))
            })?;

        let values_iterator = obj
            .get(JsString::from("values"), context)?
            .as_object()
            .ok_or_else(|| {
                JsError::from(
                    JsNativeError::typ().with_message("Set.prototype.values not callable"),
                )
            })?;
        let values_iter_val = values_iterator.call(value, &[], context)?;
        let values_array_val =
            array_from_fn.call(&JsValue::undefined(), &[values_iter_val], context)?;

        let length_val = values_array_val
            .as_object()
            .ok_or_else(|| {
                JsError::from(JsNativeError::typ().with_message("Values array is not an object"))
            })?
            .get(JsString::from("length"), context)?;
        let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);

        for i in 0..length {
            let elem_val = values_array_val
                .as_object()
                .ok_or_else(|| {
                    JsError::from(
                        JsNativeError::typ().with_message("Values array element is not an object"),
                    )
                })?
                .get(i, context)?;
            let cloned_elem = structured_clone_value(&elem_val, context)?;
            set_add_fn.call(&JsValue::from(new_set_val.clone()), &[cloned_elem], context)?;
        }
        return Ok(JsValue::from(new_set_val));
    }

    // Fallback: Plain Object
    let new_obj_val = object_constructor_obj.construct(&[], None, context)?;

    // TODO(spec): circular references not yet handled

    let keys_val = object_constructor_obj.get(JsString::from("keys"), context)?;
    let keys_fn = keys_val.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Object.keys not callable"))
    })?;

    let keys_array_val =
        keys_fn.call(&JsValue::undefined(), std::slice::from_ref(value), context)?;
    let keys_array_obj = keys_array_val.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Keys array is not an object"))
    })?;

    let length_val = keys_array_obj.get(JsString::from("length"), context)?;
    let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);

    for i in 0..length {
        let key_val = keys_array_obj.get(i, context)?;
        let key_str = key_val.to_string(context)?;
        let prop_val = obj.get(key_str.clone(), context)?;
        let cloned_prop = structured_clone_value(&prop_val, context)?;
        new_obj_val.set(key_str, cloned_prop, true, context)?;
    }

    Ok(JsValue::from(new_obj_val))
}

fn structured_clone_fn(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let value = args.first().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("structuredClone requires at least 1 argument"),
        )
    })?;
    structured_clone_value(value, context)
}

fn bridge_active_element(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    let key_opt = with_dom(|dom, key_to_node| {
        if let Some(node_id) = dom.focused_node() {
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

fn bridge_focus(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    with_dom(|dom, key_to_node| {
        if let Some(node_id) = key_to_node.get(&key).copied() {
            dom.focus(node_id);
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_blur(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    with_dom(|dom, _key_to_node| {
        dom.blur();
    })?;

    Ok(JsValue::undefined())
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

    let root_key_opt = if let Some(arg) = args.get(1) {
        if !arg.is_undefined() && !arg.is_null() {
            Some(arg.to_string(context)?.to_std_string().unwrap_or_default())
        } else {
            None
        }
    } else {
        None
    };

    let key_opt = with_dom(|dom, key_to_node| {
        let root_node = if let Some(ref r_key) = root_key_opt {
            key_to_node
                .get(r_key)
                .copied()
                .unwrap_or_else(|| dom.document())
        } else {
            dom.document()
        };
        if let Some(node_id) = find_element_by_id(dom, root_node, &id_val) {
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

    let root_key_opt = if let Some(arg) = args.get(1) {
        if !arg.is_undefined() && !arg.is_null() {
            Some(arg.to_string(context)?.to_std_string().unwrap_or_default())
        } else {
            None
        }
    } else {
        None
    };

    let key_opt = with_dom(|dom, key_to_node| {
        let root_node = if let Some(ref r_key) = root_key_opt {
            key_to_node
                .get(r_key)
                .copied()
                .unwrap_or_else(|| dom.document())
        } else {
            dom.document()
        };
        if let Some(node_id) = dom.query_selector_from(root_node, &selector_val) {
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
    root_key_opt: Option<String>,
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let keys = with_dom(|dom, key_to_node| {
        let root_node = if let Some(ref r_key) = root_key_opt {
            key_to_node
                .get(r_key)
                .copied()
                .unwrap_or_else(|| dom.document())
        } else {
            dom.document()
        };
        let mut keys_list = Vec::new();
        for node_id in dom.query_selector_all_from(root_node, selector) {
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
    let root_key_opt = if let Some(arg) = args.get(1) {
        if !arg.is_undefined() && !arg.is_null() {
            Some(arg.to_string(context)?.to_std_string().unwrap_or_default())
        } else {
            None
        }
    } else {
        None
    };
    execute_dom_query_to_js_array(&selector_val, root_key_opt, context)
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
    let root_key_opt = if let Some(arg) = args.get(1) {
        if !arg.is_undefined() && !arg.is_null() {
            Some(arg.to_string(context)?.to_std_string().unwrap_or_default())
        } else {
            None
        }
    } else {
        None
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

    execute_dom_query_to_js_array(&selector, root_key_opt, context)
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
    let root_key_opt = if let Some(arg) = args.get(1) {
        if !arg.is_undefined() && !arg.is_null() {
            Some(arg.to_string(context)?.to_std_string().unwrap_or_default())
        } else {
            None
        }
    } else {
        None
    };

    let tokens: Vec<&str> = cls
        .split_ascii_whitespace()
        .filter(|t| !t.is_empty())
        .collect();

    if tokens.is_empty() {
        // If there are no class tokens, pass an empty selector which will fail to parse
        // and safely return an empty array.
        execute_dom_query_to_js_array("", root_key_opt, context)
    } else {
        // Map ["a", "b"] to ".a.b"
        let selector = tokens
            .iter()
            .map(|t| format!(".{}", t))
            .collect::<Vec<String>>()
            .join("");
        execute_dom_query_to_js_array(&selector, root_key_opt, context)
    }
}

fn bridge_get_elements_by_name(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let name = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        String::new()
    };
    let root_key_opt = if let Some(arg) = args.get(1) {
        if !arg.is_undefined() && !arg.is_null() {
            Some(arg.to_string(context)?.to_std_string().unwrap_or_default())
        } else {
            None
        }
    } else {
        None
    };

    if name.is_empty() {
        execute_dom_query_to_js_array("", root_key_opt, context)
    } else {
        let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
        let selector = format!("[name=\"{}\"]", escaped);
        execute_dom_query_to_js_array(&selector, root_key_opt, context)
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

fn bridge_append(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let parent_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let mut args_parsed = Vec::new();
    let mut i = 1;
    while i < args.len() {
        if let Some(type_arg) = args.get(i) {
            let type_str = type_arg
                .to_string(context)?
                .to_std_string()
                .unwrap_or_default();
            if let Some(val_arg) = args.get(i + 1) {
                let val_str = val_arg
                    .to_string(context)?
                    .to_std_string()
                    .unwrap_or_default();
                args_parsed.push((type_str, val_str));
            }
        }
        i += 2;
    }

    with_dom(|dom, key_to_node| {
        let parent_id = key_to_node.get(&parent_key).copied();
        if let Some(p_id) = parent_id {
            let mut nodes_to_append = Vec::new();
            for (type_str, val_str) in args_parsed {
                if type_str == "text" {
                    let text_node_id = dom.create_node(NodeData::Text(val_str));
                    let k = format!("{:?}", text_node_id);
                    key_to_node.insert(k, text_node_id);
                    nodes_to_append.push(text_node_id);
                } else if type_str == "node"
                    && let Some(&c_id) = key_to_node.get(&val_str)
                {
                    nodes_to_append.push(c_id);
                }
            }

            for c_id in nodes_to_append {
                dom.append_child(p_id, c_id);
                // TODO(spec): Re-layout on mutation
            }
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_prepend(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let parent_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let mut args_parsed = Vec::new();
    let mut i = 1;
    while i < args.len() {
        if let Some(type_arg) = args.get(i) {
            let type_str = type_arg
                .to_string(context)?
                .to_std_string()
                .unwrap_or_default();
            if let Some(val_arg) = args.get(i + 1) {
                let val_str = val_arg
                    .to_string(context)?
                    .to_std_string()
                    .unwrap_or_default();
                args_parsed.push((type_str, val_str));
            }
        }
        i += 2;
    }

    with_dom(|dom, key_to_node| {
        let parent_id = key_to_node.get(&parent_key).copied();
        if let Some(p_id) = parent_id {
            let mut nodes_to_prepend = Vec::new();
            for (type_str, val_str) in args_parsed {
                if type_str == "text" {
                    let text_node_id = dom.create_node(NodeData::Text(val_str));
                    let k = format!("{:?}", text_node_id);
                    key_to_node.insert(k, text_node_id);
                    nodes_to_prepend.push(text_node_id);
                } else if type_str == "node"
                    && let Some(&c_id) = key_to_node.get(&val_str)
                {
                    nodes_to_prepend.push(c_id);
                }
            }

            let original_first_child = dom.children(p_id).first().copied();

            for c_id in nodes_to_prepend {
                dom.insert_before(p_id, c_id, original_first_child);
                // TODO(spec): Re-layout on mutation
            }
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_before(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let child_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let mut args_parsed = Vec::new();
    let mut i = 1;
    while i < args.len() {
        if let Some(type_arg) = args.get(i) {
            let type_str = type_arg
                .to_string(context)?
                .to_std_string()
                .unwrap_or_default();
            if let Some(val_arg) = args.get(i + 1) {
                let val_str = val_arg
                    .to_string(context)?
                    .to_std_string()
                    .unwrap_or_default();
                args_parsed.push((type_str, val_str));
            }
        }
        i += 2;
    }

    with_dom(|dom, key_to_node| {
        let child_id = key_to_node.get(&child_key).copied();
        if let Some(c_id) = child_id
            && let Some(p_id) = dom.parent(c_id)
        {
            let mut nodes_to_insert = Vec::new();
            for (type_str, val_str) in args_parsed {
                if type_str == "text" {
                    let text_node_id = dom.create_node(NodeData::Text(val_str));
                    let k = format!("{:?}", text_node_id);
                    key_to_node.insert(k, text_node_id);
                    nodes_to_insert.push(text_node_id);
                } else if type_str == "node"
                    && let Some(&node_id) = key_to_node.get(&val_str)
                {
                    nodes_to_insert.push(node_id);
                }
            }

            for node_id in nodes_to_insert {
                dom.insert_before(p_id, node_id, Some(c_id));
                // TODO(spec): Re-layout on mutation
            }
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_after(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let child_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let mut args_parsed = Vec::new();
    let mut i = 1;
    while i < args.len() {
        if let Some(type_arg) = args.get(i) {
            let type_str = type_arg
                .to_string(context)?
                .to_std_string()
                .unwrap_or_default();
            if let Some(val_arg) = args.get(i + 1) {
                let val_str = val_arg
                    .to_string(context)?
                    .to_std_string()
                    .unwrap_or_default();
                args_parsed.push((type_str, val_str));
            }
        }
        i += 2;
    }

    with_dom(|dom, key_to_node| {
        let child_id = key_to_node.get(&child_key).copied();
        if let Some(c_id) = child_id
            && let Some(p_id) = dom.parent(c_id)
        {
            let mut nodes_to_insert = Vec::new();
            for (type_str, val_str) in args_parsed {
                if type_str == "text" {
                    let text_node_id = dom.create_node(NodeData::Text(val_str));
                    let k = format!("{:?}", text_node_id);
                    key_to_node.insert(k, text_node_id);
                    nodes_to_insert.push(text_node_id);
                } else if type_str == "node"
                    && let Some(&node_id) = key_to_node.get(&val_str)
                {
                    nodes_to_insert.push(node_id);
                }
            }

            // Find child_id's position in parent's children
            let pos = dom.children(p_id).iter().position(|&id| id == c_id);
            let next_sibling_id = pos.and_then(|idx| dom.children(p_id).get(idx + 1).copied());

            for node_id in nodes_to_insert {
                dom.insert_before(p_id, node_id, next_sibling_id);
                // TODO(spec): Re-layout on mutation
            }
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_replace_with(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let child_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let mut args_parsed = Vec::new();
    let mut i = 1;
    while i < args.len() {
        if let Some(type_arg) = args.get(i) {
            let type_str = type_arg
                .to_string(context)?
                .to_std_string()
                .unwrap_or_default();
            if let Some(val_arg) = args.get(i + 1) {
                let val_str = val_arg
                    .to_string(context)?
                    .to_std_string()
                    .unwrap_or_default();
                args_parsed.push((type_str, val_str));
            }
        }
        i += 2;
    }

    with_dom(|dom, key_to_node| {
        let child_id = key_to_node.get(&child_key).copied();
        if let Some(c_id) = child_id
            && let Some(p_id) = dom.parent(c_id)
        {
            let mut nodes_to_insert = Vec::new();
            for (type_str, val_str) in args_parsed {
                if type_str == "text" {
                    let text_node_id = dom.create_node(NodeData::Text(val_str));
                    let k = format!("{:?}", text_node_id);
                    key_to_node.insert(k, text_node_id);
                    nodes_to_insert.push(text_node_id);
                } else if type_str == "node"
                    && let Some(&node_id) = key_to_node.get(&val_str)
                {
                    nodes_to_insert.push(node_id);
                }
            }

            for node_id in nodes_to_insert {
                dom.insert_before(p_id, node_id, Some(c_id));
                // TODO(spec): Re-layout on mutation
            }

            dom.remove_child(p_id, c_id);
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

fn bridge_create_comment(
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
        let node_id = dom.create_node(NodeData::Comment(data));
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

fn bridge_insert_adjacent_element(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let ref_node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let position_str = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let element_node_key = if let Some(arg) = args.get(2) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let position = position_str.trim().to_lowercase();

    let inserted_key_opt = with_dom(|dom, key_to_node| {
        if let Some(&ref_id) = key_to_node.get(&ref_node_key) {
            if let Some(&elem_id) = key_to_node.get(&element_node_key) {
                match position.as_str() {
                    "beforebegin" => {
                        if let Some(parent_id) = dom.parent(ref_id) {
                            dom.insert_before(parent_id, elem_id, Some(ref_id));
                            Some(element_node_key.clone())
                        } else {
                            None
                        }
                    }
                    "afterbegin" => {
                        let first_child = dom.children(ref_id).first().copied();
                        dom.insert_before(ref_id, elem_id, first_child);
                        Some(element_node_key.clone())
                    }
                    "beforeend" => {
                        dom.insert_before(ref_id, elem_id, None);
                        Some(element_node_key.clone())
                    }
                    "afterend" => {
                        if let Some(parent_id) = dom.parent(ref_id) {
                            let parent_children = dom.children(parent_id);
                            let next_sibling = if let Some(pos) =
                                parent_children.iter().position(|&c| c == ref_id)
                            {
                                parent_children.get(pos + 1).copied()
                            } else {
                                None
                            };
                            dom.insert_before(parent_id, elem_id, next_sibling);
                            Some(element_node_key.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    })?;

    if let Some(key) = inserted_key_opt {
        Ok(JsValue::from(JsString::from(key)))
    } else {
        Ok(JsValue::null())
    }
}

fn bridge_insert_adjacent_html(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let ref_node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let position_str = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let html_val = if let Some(arg) = args.get(2) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let position = position_str.trim().to_lowercase();

    with_dom(|dom, _key_to_node| {
        if let Some(&ref_id) = _key_to_node.get(&ref_node_key) {
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

                match position.as_str() {
                    "beforebegin" => {
                        if let Some(parent_id) = dom.parent(ref_id) {
                            for temp_child_id in temp_children {
                                let dest_child_id =
                                    copy_node_to_dom_recursive(&temp_dom, temp_child_id, dom);
                                dom.insert_before(parent_id, dest_child_id, Some(ref_id));
                            }
                        }
                    }
                    "afterbegin" => {
                        let original_first_child = dom.children(ref_id).first().copied();
                        for temp_child_id in temp_children {
                            let dest_child_id =
                                copy_node_to_dom_recursive(&temp_dom, temp_child_id, dom);
                            dom.insert_before(ref_id, dest_child_id, original_first_child);
                        }
                    }
                    "beforeend" => {
                        for temp_child_id in temp_children {
                            let dest_child_id =
                                copy_node_to_dom_recursive(&temp_dom, temp_child_id, dom);
                            dom.insert_before(ref_id, dest_child_id, None);
                        }
                    }
                    "afterend" => {
                        if let Some(parent_id) = dom.parent(ref_id) {
                            let parent_children = dom.children(parent_id);
                            let original_next_sibling = if let Some(pos) =
                                parent_children.iter().position(|&c| c == ref_id)
                            {
                                parent_children.get(pos + 1).copied()
                            } else {
                                None
                            };
                            for temp_child_id in temp_children {
                                let dest_child_id =
                                    copy_node_to_dom_recursive(&temp_dom, temp_child_id, dom);
                                dom.insert_before(parent_id, dest_child_id, original_next_sibling);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_insert_adjacent_text(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let ref_node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let position_str = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let text_val = if let Some(arg) = args.get(2) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let position = position_str.trim().to_lowercase();

    with_dom(|dom, key_to_node| {
        if let Some(&ref_id) = key_to_node.get(&ref_node_key) {
            let text_node_id = dom.create_node(NodeData::Text(text_val));
            let k = format!("{:?}", text_node_id);
            key_to_node.insert(k, text_node_id);

            match position.as_str() {
                "beforebegin" => {
                    if let Some(parent_id) = dom.parent(ref_id) {
                        dom.insert_before(parent_id, text_node_id, Some(ref_id));
                    }
                }
                "afterbegin" => {
                    let original_first_child = dom.children(ref_id).first().copied();
                    dom.insert_before(ref_id, text_node_id, original_first_child);
                }
                "beforeend" => {
                    dom.insert_before(ref_id, text_node_id, None);
                }
                "afterend" => {
                    if let Some(parent_id) = dom.parent(ref_id) {
                        let parent_children = dom.children(parent_id);
                        let original_next_sibling =
                            if let Some(pos) = parent_children.iter().position(|&c| c == ref_id) {
                                parent_children.get(pos + 1).copied()
                            } else {
                                None
                            };
                        dom.insert_before(parent_id, text_node_id, original_next_sibling);
                    }
                }
                _ => {}
            }
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

fn normalize_node(dom: &mut Dom, node_id: NodeId) {
    // 1. Collect children first to avoid concurrent borrow/mutation of the DOM tree.
    let children = dom.children(node_id).to_vec();
    for &child in &children {
        normalize_node(dom, child);
    }

    // 2. Coalesce adjacent Text nodes and remove empty Text nodes among the direct children of node_id.
    let mut current_children = dom.children(node_id).to_vec();
    let mut i = 0;
    while i < current_children.len() {
        let child = current_children[i];
        let is_text_opt = if let Some(NodeData::Text(text)) = dom.data(child) {
            Some(text.clone())
        } else {
            None
        };

        if let Some(text) = is_text_opt {
            if text.is_empty() {
                dom.remove_child(node_id, child);
                current_children.remove(i);
                continue;
            }

            // Look ahead to collect contiguous adjacent Text siblings
            let mut next_idx = i + 1;
            let mut merged_text = text.clone();
            let mut to_remove = Vec::new();
            while next_idx < current_children.len() {
                let next_child = current_children[next_idx];
                if let Some(NodeData::Text(next_text)) = dom.data(next_child) {
                    merged_text.push_str(next_text);
                    to_remove.push(next_child);
                    next_idx += 1;
                } else {
                    break;
                }
            }

            if !to_remove.is_empty() {
                dom.set_text(child, &merged_text);
                for rem_child in to_remove {
                    dom.remove_child(node_id, rem_child);
                }
                current_children.drain(i + 1..next_idx);
            }
        }
        i += 1;
    }
}

fn bridge_normalize(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    with_dom(|dom, key_to_node| {
        if let Some(&node_id) = key_to_node.get(&node_key) {
            normalize_node(dom, node_id);
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_is_connected(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(false));
    };

    let is_connected = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            let mut curr = n_id;
            while let Some(parent_id) = dom.parent(curr) {
                curr = parent_id;
            }
            curr == dom.document()
        } else {
            false
        }
    })?;

    Ok(JsValue::from(is_connected))
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

fn bridge_has_child_nodes(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(false));
    };

    let has_children = with_dom(|dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            !dom.children(n_id).is_empty()
        } else {
            false
        }
    })?;

    Ok(JsValue::from(has_children))
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

fn bridge_get_node_value(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let val_opt = with_dom(|dom, key_to_node| {
        if let Some(&node_id) = key_to_node.get(&node_key) {
            match dom.data(node_id) {
                Some(NodeData::Text(s)) => Some(s.clone()),
                Some(NodeData::Comment(s)) => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        }
    })?;

    if let Some(val) = val_opt {
        Ok(JsValue::from(JsString::from(val)))
    } else {
        Ok(JsValue::null())
    }
}

fn bridge_set_node_value(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let val = if let Some(arg) = args.get(1) {
        if arg.is_null() || arg.is_undefined() {
            String::new()
        } else {
            arg.to_string(context)?.to_std_string().unwrap_or_default()
        }
    } else {
        String::new()
    };

    with_dom(|dom, key_to_node| {
        if let Some(&node_id) = key_to_node.get(&node_key) {
            dom.set_text(node_id, &val);
        }
    })?;

    Ok(JsValue::undefined())
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
                    if let Some(val_str) = computed_style.get_property_as_string(&kebab) {
                        resolved_value = val_str;
                    }
                }
            });
        }
    })?;

    Ok(JsValue::from(JsString::from(resolved_value)))
}

fn bridge_get_bounding_client_rect(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let element_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let mut rect_opt = None;
    with_dom(|dom, key_to_node| {
        if let Some(&node_id) = key_to_node.get(&element_key) {
            rect_opt = Some(dom.get_bounding_client_rect(node_id));
        }
    })?;

    let rect = rect_opt.unwrap_or_else(|| crate::dom::DomRect::new(0.0, 0.0, 0.0, 0.0));

    // DOMRectReadOnly properties are enumerable + configurable but NOT writable
    // (getBoundingClientRect returns a DOMRectReadOnly per the CSSOM View spec).
    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let js_rect = ObjectInitializer::new(context)
        .property(JsString::from("x"), JsValue::from(rect.x()), ro)
        .property(JsString::from("y"), JsValue::from(rect.y()), ro)
        .property(JsString::from("width"), JsValue::from(rect.width()), ro)
        .property(JsString::from("height"), JsValue::from(rect.height()), ro)
        .property(JsString::from("top"), JsValue::from(rect.top()), ro)
        .property(JsString::from("right"), JsValue::from(rect.right()), ro)
        .property(JsString::from("bottom"), JsValue::from(rect.bottom()), ro)
        .property(JsString::from("left"), JsValue::from(rect.left()), ro)
        .build();

    Ok(JsValue::from(js_rect))
}

fn bridge_get_scroll_top(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(0.0));
    };

    let scroll_top = with_dom(|_dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            ELEMENT_SCROLL_TOP.with(|cell| *cell.borrow().get(&n_id).unwrap_or(&0.0))
        } else {
            0.0
        }
    })?;

    Ok(JsValue::from(scroll_top))
}

fn bridge_set_scroll_top(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let val = if let Some(arg) = args.get(1) {
        arg.to_number(context)?
    } else {
        0.0
    };

    let val_clamped = if val.is_nan() || val < 0.0 { 0.0 } else { val };

    with_dom(|_dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            ELEMENT_SCROLL_TOP.with(|cell| {
                cell.borrow_mut().insert(n_id, val_clamped);
            });
            // TODO(spec): wire scrollTop/scrollLeft setter to actual layout scroll (cross-module)
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_get_scroll_left(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(0.0));
    };

    let scroll_left = with_dom(|_dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            ELEMENT_SCROLL_LEFT.with(|cell| *cell.borrow().get(&n_id).unwrap_or(&0.0))
        } else {
            0.0
        }
    })?;

    Ok(JsValue::from(scroll_left))
}

fn bridge_set_scroll_left(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let node_key = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let val = if let Some(arg) = args.get(1) {
        arg.to_number(context)?
    } else {
        0.0
    };

    let val_clamped = if val.is_nan() || val < 0.0 { 0.0 } else { val };

    with_dom(|_dom, key_to_node| {
        if let Some(n_id) = key_to_node.get(&node_key).copied() {
            ELEMENT_SCROLL_LEFT.with(|cell| {
                cell.borrow_mut().insert(n_id, val_clamped);
            });
            // TODO(spec): wire scrollTop/scrollLeft setter to actual layout scroll (cross-module)
        }
    })?;

    Ok(JsValue::undefined())
}

fn bridge_scroll_into_view(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    // TODO(spec): record smooth-vs-auto behavior rather than guessing
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

/// Finds inline `<script>` elements in document order and runs them.
///
/// If a script throws an error, it is caught per-script and does not abort
/// the overall run (I-6 safety). External `src`, `defer`, or `async` scripts
/// are skipped and marked with a spec TODO.
pub fn run_inline_scripts(
    mut dom: Dom,
    styles: &std::collections::HashMap<
        crate::infra::NodeId,
        crate::style::CategorizedComputedStyle,
    >,
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

/// Finds `<script>` elements in document order and runs them.
///
/// For inline scripts, it runs their text content. For scripts with a `src` attribute,
/// it resolves and fetches the external source and executes it.
/// If any script throws an error or fails to fetch/decode, it is caught/ignored silently.
pub fn run_scripts(
    mut dom: Dom,
    styles: &std::collections::HashMap<
        crate::infra::NodeId,
        crate::style::CategorizedComputedStyle,
    >,
    base_url: &crate::url::Url,
    loader: &dyn crate::loader::ResourceLoader,
) -> Dom {
    // Collect script node IDs in document order (pre-order traversal)
    let mut script_ids = Vec::new();
    for id in dom.descendants(dom.document()) {
        if let Some(NodeData::Element { name, .. }) = dom.data(id)
            && name.eq_ignore_ascii_case("script")
        {
            let has_defer = dom.get_attribute(id, "defer").is_some();
            let has_async = dom.get_attribute(id, "async").is_some();

            if has_defer || has_async {
                // TODO(spec): Support defer or async execution modes.
                continue;
            }

            script_ids.push(id);
        }
    }

    let mut host = BoaHost::new();
    for id in script_ids {
        if let Some(src_val) = dom.get_attribute(id, "src") {
            let maybe_bytes = if src_val.starts_with("data:") {
                crate::loader::load_data_uri(src_val)
            } else if let Some(resolved_url) = crate::url::resolve(base_url, src_val) {
                if resolved_url.scheme == "data" {
                    crate::loader::load_data_uri(&resolved_url.serialize())
                } else if resolved_url.scheme == "http" || resolved_url.scheme == "https" {
                    loader.load(&resolved_url).ok()
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(bytes) = maybe_bytes
                && let Ok(script_str) = String::from_utf8(bytes)
            {
                // Execute the script with the current DOM context
                // spec: S-61 Any exception from a throwing script must be caught per-script and not abort the entire run.
                let _ = host.eval_with_dom_and_styles(&script_str, &mut dom, styles);
            }
        } else {
            let src = dom.text_content(id);
            // Execute the script with the current DOM context
            // spec: S-61 Any exception from a throwing script must be caught per-script and not abort the entire run.
            let _ = host.eval_with_dom_and_styles(&src, &mut dom, styles);
        }
    }

    // Fire DOM lifecycle events (DOMContentLoaded, load) and expose document.readyState after scripts run.
    let _ = host.dispatch_lifecycle_events(&mut dom, styles);

    dom
}

/// Implementation of WHATWG URL `URLSearchParams` interface.
/// Spec: <https://url.spec.whatwg.org/#interface-urlsearchparams>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct URLSearchParams {
    pub(crate) pairs: GcRefCell<Vec<(String, String)>>,
}

impl Class for URLSearchParams {
    const NAME: &'static str = "URLSearchParams";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        let mut pairs = Vec::new();
        if let Some(arg) = args.first()
            && !arg.is_undefined()
            && !arg.is_null()
        {
            let init_str = arg.to_string(context)?.to_std_string().unwrap_or_default();
            pairs = crate::url::parse_query(&init_str);
        }
        Ok(URLSearchParams {
            pairs: GcRefCell::new(pairs),
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class
            .method(
                JsString::from("get"),
                1,
                NativeFunction::from_fn_ptr(url_search_params_get),
            )
            .method(
                JsString::from("getAll"),
                1,
                NativeFunction::from_fn_ptr(url_search_params_get_all),
            )
            .method(
                JsString::from("has"),
                1,
                NativeFunction::from_fn_ptr(url_search_params_has),
            )
            .method(
                JsString::from("append"),
                2,
                NativeFunction::from_fn_ptr(url_search_params_append),
            )
            .method(
                JsString::from("set"),
                2,
                NativeFunction::from_fn_ptr(url_search_params_set),
            )
            .method(
                JsString::from("delete"),
                1,
                NativeFunction::from_fn_ptr(url_search_params_delete),
            )
            .method(
                JsString::from("toString"),
                0,
                NativeFunction::from_fn_ptr(url_search_params_to_string),
            );

        Ok(())
    }
}

pub fn url_search_params_get(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let params = obj.downcast_ref::<URLSearchParams>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-URLSearchParams object"),
        )
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let pairs = params.pairs.borrow();
    for (k, v) in pairs.iter() {
        if k == &name {
            return Ok(JsValue::from(JsString::from(v.as_str())));
        }
    }

    Ok(JsValue::null())
}

pub fn url_search_params_get_all(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let params = obj.downcast_ref::<URLSearchParams>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-URLSearchParams object"),
        )
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let pairs = params.pairs.borrow();
    let elements: Vec<JsValue> = pairs
        .iter()
        .filter(|(k, _)| k == &name)
        .map(|(_, v)| JsValue::from(JsString::from(v.as_str())))
        .collect();

    let array = boa_engine::object::builtins::JsArray::from_iter(elements, context);
    Ok(JsValue::from(array))
}

pub fn url_search_params_has(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let params = obj.downcast_ref::<URLSearchParams>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-URLSearchParams object"),
        )
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let pairs = params.pairs.borrow();
    let has_key = pairs.iter().any(|(k, _)| k == &name);
    Ok(JsValue::from(has_key))
}

pub fn url_search_params_append(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let params = obj.downcast_ref::<URLSearchParams>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-URLSearchParams object"),
        )
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let val = args
        .get(1)
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    params.pairs.borrow_mut().push((name, val));
    Ok(JsValue::undefined())
}

pub fn url_search_params_set(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let params = obj.downcast_ref::<URLSearchParams>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-URLSearchParams object"),
        )
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let val = args
        .get(1)
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let mut pairs = params.pairs.borrow_mut();
    let mut found = false;
    let mut i = 0;
    while i < pairs.len() {
        if pairs[i].0 == name {
            if !found {
                pairs[i].1 = val.clone();
                found = true;
                i += 1;
            } else {
                pairs.remove(i);
            }
        } else {
            i += 1;
        }
    }

    if !found {
        pairs.push((name, val));
    }

    Ok(JsValue::undefined())
}

pub fn url_search_params_delete(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let params = obj.downcast_ref::<URLSearchParams>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-URLSearchParams object"),
        )
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let mut pairs = params.pairs.borrow_mut();
    pairs.retain(|(k, _)| k != &name);

    Ok(JsValue::undefined())
}

pub fn url_search_params_to_string(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let params = obj.downcast_ref::<URLSearchParams>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-URLSearchParams object"),
        )
    })?;

    let pairs = params.pairs.borrow();
    let serialized =
        crate::url::serialize_form_urlencoded(pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    Ok(JsValue::from(JsString::from(serialized)))
}

// ==========================================
// AbortController / AbortSignal API (t0518)
// ==========================================

fn create_default_abort_error(context: &mut Context) -> JsValue {
    if let Ok(error_constructor) = context
        .global_object()
        .get(JsString::from("Error"), context)
        && let Some(error_obj) = error_constructor.as_object()
        && let Ok(error_inst) = error_obj.construct(
            &[JsValue::from(JsString::from("The user aborted a request."))],
            None,
            context,
        )
    {
        let _ = error_inst.set(
            JsString::from("name"),
            JsValue::from(JsString::from("AbortError")),
            true,
            context,
        );
        return JsValue::from(error_inst);
    }
    JsValue::undefined()
}

/// Implementation of the WHATWG `AbortSignal` interface.
/// Spec: <https://dom.spec.whatwg.org/#interface-abortsignal>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct AbortSignal {
    pub(crate) aborted: GcRefCell<bool>,
    pub(crate) reason: GcRefCell<JsValue>,
}

impl Class for AbortSignal {
    const NAME: &'static str = "AbortSignal";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        let aborted = args.first().and_then(|v| v.as_boolean()).unwrap_or(false);
        let reason = if aborted {
            args.get(1)
                .cloned()
                .unwrap_or_else(|| create_default_abort_error(context))
        } else {
            JsValue::undefined()
        };
        Ok(AbortSignal {
            aborted: GcRefCell::new(aborted),
            reason: GcRefCell::new(reason),
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let get_aborted_fn = boa_engine::object::FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(abort_signal_get_aborted),
        )
        .name("get aborted")
        .build();

        let get_reason_fn = boa_engine::object::FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(abort_signal_get_reason),
        )
        .name("get reason")
        .build();

        class
            .accessor(
                JsString::from("aborted"),
                Some(get_aborted_fn),
                None,
                Attribute::all(),
            )
            .accessor(
                JsString::from("reason"),
                Some(get_reason_fn),
                None,
                Attribute::all(),
            )
            .method(
                JsString::from("throwIfAborted"),
                0,
                NativeFunction::from_fn_ptr(abort_signal_throw_if_aborted),
            )
            .method(
                JsString::from("addEventListener"),
                2,
                NativeFunction::from_fn_ptr(event::add_event_listener),
            )
            .method(
                JsString::from("removeEventListener"),
                2,
                NativeFunction::from_fn_ptr(event::remove_event_listener),
            )
            .method(
                JsString::from("dispatchEvent"),
                1,
                NativeFunction::from_fn_ptr(event::dispatch_event),
            )
            .static_method(
                JsString::from("abort"),
                1,
                NativeFunction::from_fn_ptr(abort_signal_abort_static),
            );

        Ok(())
    }
}

pub fn abort_signal_get_aborted(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let signal = obj.downcast_ref::<AbortSignal>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-AbortSignal object"))
    })?;
    Ok(JsValue::from(*signal.aborted.borrow()))
}

pub fn abort_signal_get_reason(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let signal = obj.downcast_ref::<AbortSignal>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-AbortSignal object"))
    })?;
    Ok(signal.reason.borrow().clone())
}

pub fn abort_signal_throw_if_aborted(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let signal = obj.downcast_ref::<AbortSignal>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-AbortSignal object"))
    })?;
    if *signal.aborted.borrow() {
        return Err(JsError::from_opaque(signal.reason.borrow().clone()));
    }
    Ok(JsValue::undefined())
}

pub fn abort_signal_abort_static(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let signal_constructor = context
        .global_object()
        .get(JsString::from("AbortSignal"), context)?;
    let signal_obj = signal_constructor.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("AbortSignal constructor not found"))
    })?;
    let reason = if let Some(arg) = args.first() {
        arg.clone()
    } else {
        create_default_abort_error(context)
    };
    let signal_inst = signal_obj.construct(&[JsValue::from(true), reason], None, context)?;
    Ok(JsValue::from(signal_inst))
}

/// Implementation of the WHATWG `AbortController` interface.
/// Spec: <https://dom.spec.whatwg.org/#interface-abortcontroller>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct AbortController {
    pub(crate) signal: GcRefCell<JsValue>,
}

impl Class for AbortController {
    const NAME: &'static str = "AbortController";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        let signal_constructor = context
            .global_object()
            .get(JsString::from("AbortSignal"), context)?;
        let signal_obj = signal_constructor.as_object().ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("AbortSignal constructor not found"))
        })?;
        let signal_inst = signal_obj.construct(&[], None, context)?;
        Ok(AbortController {
            signal: GcRefCell::new(JsValue::from(signal_inst)),
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let get_signal_fn = boa_engine::object::FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(abort_controller_get_signal),
        )
        .name("get signal")
        .build();

        class
            .accessor(
                JsString::from("signal"),
                Some(get_signal_fn),
                None,
                Attribute::all(),
            )
            .method(
                JsString::from("abort"),
                1,
                NativeFunction::from_fn_ptr(abort_controller_abort),
            );

        Ok(())
    }
}

pub fn abort_controller_get_signal(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let controller = obj.downcast_ref::<AbortController>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-AbortController object"),
        )
    })?;
    Ok(controller.signal.borrow().clone())
}

pub fn abort_controller_abort(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let controller = obj.downcast_ref::<AbortController>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-AbortController object"),
        )
    })?;

    let reason = if let Some(arg) = args.first() {
        arg.clone()
    } else {
        create_default_abort_error(context)
    };

    let signal_val = controller.signal.borrow().clone();
    let signal_obj = signal_val.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Signal is not an object"))
    })?;
    let signal = signal_obj.downcast_ref::<AbortSignal>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Signal is not an AbortSignal"))
    })?;

    if *signal.aborted.borrow() {
        return Ok(JsValue::undefined());
    }

    *signal.aborted.borrow_mut() = true;
    *signal.reason.borrow_mut() = reason;

    // Fire "abort" event
    if let Ok(event_constructor) = context
        .global_object()
        .get(JsString::from("Event"), context)
        && let Some(event_obj) = event_constructor.as_object()
        && let Ok(event_inst) =
            event_obj.construct(&[JsValue::from(JsString::from("abort"))], None, context)
    {
        let _ = event::dispatch_event(&signal_val, &[JsValue::from(event_inst.clone())], context);

        if let Ok(onabort_val) = signal_obj.get(JsString::from("onabort"), context)
            && let Some(onabort_callable) = onabort_val.as_object()
            && onabort_callable.is_callable()
        {
            let _ = onabort_callable.call(&signal_val, &[JsValue::from(event_inst)], context);
        }
    }

    Ok(JsValue::undefined())
}

fn import_node_subtree(
    temp_dom: &crate::dom::Dom,
    temp_node_id: crate::infra::NodeId,
    dom: &mut crate::dom::Dom,
    key_to_node: &mut std::collections::HashMap<String, crate::infra::NodeId>,
) -> crate::infra::NodeId {
    let node_data = temp_dom
        .data(temp_node_id)
        .cloned()
        .unwrap_or(crate::dom::NodeData::Document);
    let new_node_id = dom.create_node(node_data);
    let key = format!("{:?}", new_node_id);
    key_to_node.insert(key, new_node_id);
    for child in temp_dom.children(temp_node_id) {
        let new_child_id = import_node_subtree(temp_dom, *child, dom, key_to_node);
        dom.append_child(new_node_id, new_child_id);
    }
    new_node_id
}

/// Implementation of WHATWG DOM `DOMParser` interface (t0522).
/// Spec: <https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#domparser>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct DOMParser {}

impl Class for DOMParser {
    const NAME: &'static str = "DOMParser";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Ok(DOMParser {})
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class.method(
            JsString::from("parseFromString"),
            2,
            NativeFunction::from_fn_ptr(dom_parser_parse_from_string),
        );
        Ok(())
    }
}

pub fn dom_parser_parse_from_string(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let markup = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        String::new()
    };

    let _mime_type = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        "text/html".to_string()
    };

    // // TODO(spec): XML parsing is not supported. Fallback to HTML parsing for text/xml and application/xml.

    let parsed_doc_key_opt = with_dom(|dom, key_to_node| {
        let temp_dom =
            crate::html::parse_document(crate::encoding::InputStream::from_utf8(markup.as_bytes()));

        let parsed_doc_node_id = dom.create_node(crate::dom::NodeData::Document);
        let parsed_doc_key = format!("{:?}", parsed_doc_node_id);
        key_to_node.insert(parsed_doc_key.clone(), parsed_doc_node_id);

        for child in temp_dom.children(temp_dom.document()) {
            let new_child_id = import_node_subtree(&temp_dom, *child, dom, key_to_node);
            dom.append_child(parsed_doc_node_id, new_child_id);
        }

        parsed_doc_key
    })?;

    let document_constructor = context
        .global_object()
        .get(JsString::from("Document"), context)?;
    let document_obj = document_constructor.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Document constructor not found"))
    })?;
    let document_inst = document_obj.construct(
        &[JsValue::from(JsString::from(parsed_doc_key_opt))],
        None,
        context,
    )?;
    Ok(JsValue::from(document_inst))
}

/// Options dictionary/object for `MutationObserver.observe`.
/// Spec: <https://dom.spec.whatwg.org/#dictdef-mutationobserverinit>
#[derive(Debug, Clone, Trace, Finalize)]
pub struct MutationObserverOptions {
    pub child_list: bool,
    pub attributes: Option<bool>,
    pub subtree: bool,
    pub character_data: Option<bool>,
    pub attribute_old_value: Option<bool>,
    pub character_data_old_value: Option<bool>,
    pub attribute_filter: Option<Vec<String>>,
}

/// Helper function to parse `MutationObserverInit` dictionary/options object.
fn parse_mutation_observer_options(
    options_val: &JsValue,
    context: &mut Context,
) -> JsResult<MutationObserverOptions> {
    let options_obj = options_val.as_object().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: observe() requires options parameter"),
        )
    })?;

    // childList
    let child_list_val = options_obj.get(JsString::from("childList"), context)?;
    let child_list = if child_list_val.is_undefined() {
        false
    } else {
        child_list_val.to_boolean()
    };

    // attributes
    let attributes_val = options_obj.get(JsString::from("attributes"), context)?;
    let attributes = if attributes_val.is_undefined() {
        None
    } else {
        Some(attributes_val.to_boolean())
    };

    // subtree
    let subtree_val = options_obj.get(JsString::from("subtree"), context)?;
    let subtree = if subtree_val.is_undefined() {
        false
    } else {
        subtree_val.to_boolean()
    };

    // characterData
    let character_data_val = options_obj.get(JsString::from("characterData"), context)?;
    let character_data = if character_data_val.is_undefined() {
        None
    } else {
        Some(character_data_val.to_boolean())
    };

    // attributeOldValue
    let attribute_old_value_val = options_obj.get(JsString::from("attributeOldValue"), context)?;
    let attribute_old_value = if attribute_old_value_val.is_undefined() {
        None
    } else {
        Some(attribute_old_value_val.to_boolean())
    };

    // characterDataOldValue
    let character_data_old_value_val =
        options_obj.get(JsString::from("characterDataOldValue"), context)?;
    let character_data_old_value = if character_data_old_value_val.is_undefined() {
        None
    } else {
        Some(character_data_old_value_val.to_boolean())
    };

    // attributeFilter
    let attribute_filter_val = options_obj.get(JsString::from("attributeFilter"), context)?;
    let attribute_filter = if attribute_filter_val.is_undefined() || attribute_filter_val.is_null()
    {
        None
    } else {
        let filter_obj = attribute_filter_val.as_object().ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("TypeError: attributeFilter must be a sequence of strings"),
            )
        })?;
        let len_val = filter_obj.get(JsString::from("length"), context)?;
        let len = len_val.to_number(context)? as usize;
        let mut filter = Vec::with_capacity(len);
        for i in 0..len {
            let item = filter_obj.get(i, context)?;
            filter.push(item.to_string(context)?.to_std_string().unwrap_or_default());
        }
        Some(filter)
    };

    // Apply defaults and validate options per spec:
    // 1. If options["attributeOldValue"] or options["attributeFilter"] is present, and options["attributes"] is not present, set options["attributes"] to true.
    let mut resolved_attributes = attributes;
    if (attribute_old_value.is_some() || attribute_filter.is_some()) && attributes.is_none() {
        resolved_attributes = Some(true);
    }

    // 2. If options["characterDataOldValue"] is present, and options["characterData"] is not present, set options["characterData"] to true.
    let mut resolved_character_data = character_data;
    if character_data_old_value.is_some() && character_data.is_none() {
        resolved_character_data = Some(true);
    }

    // Validation rules:
    // 1. At least one of childList, attributes, or characterData must be true.
    let final_attributes = resolved_attributes.unwrap_or(false);
    let final_character_data = resolved_character_data.unwrap_or(false);
    if !child_list && !final_attributes && !final_character_data {
        return Err(JsError::from(JsNativeError::typ().with_message(
            "TypeError: At least one of childList, attributes, or characterData must be true",
        )));
    }

    // 2. If options["attributes"] is false, and options["attributeOldValue"] is true, or options["attributeFilter"] is present, throw TypeError.
    if !final_attributes && (attribute_old_value.unwrap_or(false) || attribute_filter.is_some()) {
        return Err(JsError::from(JsNativeError::typ().with_message(
            "TypeError: attributeOldValue or attributeFilter cannot be present when attributes is false"
        )));
    }

    // 3. If options["characterData"] is false, and options["characterDataOldValue"] is true, throw TypeError.
    if !final_character_data && character_data_old_value.unwrap_or(false) {
        return Err(JsError::from(JsNativeError::typ().with_message(
            "TypeError: characterDataOldValue cannot be true when characterData is false",
        )));
    }

    Ok(MutationObserverOptions {
        child_list,
        attributes: resolved_attributes,
        subtree,
        character_data: resolved_character_data,
        attribute_old_value,
        character_data_old_value,
        attribute_filter,
    })
}

/// An active observation setup on a target DOM Node.
#[derive(Debug, Trace, Finalize, Clone)]
pub struct Observation {
    pub target: JsValue,
    pub target_key: String,
    pub options: MutationObserverOptions,
}

/// Implementation of WHATWG DOM `MutationObserver` interface.
/// Spec: <https://dom.spec.whatwg.org/#interface-mutationobserver>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct MutationObserver {
    pub(crate) callback: JsValue,
    pub(crate) active_observations: GcRefCell<Vec<Observation>>,
    pub(crate) record_queue: GcRefCell<Vec<JsValue>>,
}

impl Class for MutationObserver {
    const NAME: &'static str = "MutationObserver";
    const LENGTH: usize = 1;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        let callback = args.first().cloned().unwrap_or(JsValue::undefined());
        if !callback.is_callable() {
            return Err(JsError::from(JsNativeError::typ().with_message(
                "TypeError: MutationObserver constructor requires a callback function",
            )));
        }
        Ok(MutationObserver {
            callback,
            active_observations: GcRefCell::new(Vec::new()),
            record_queue: GcRefCell::new(Vec::new()),
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class
            .method(
                JsString::from("observe"),
                2,
                NativeFunction::from_fn_ptr(mutation_observer_observe),
            )
            .method(
                JsString::from("disconnect"),
                0,
                NativeFunction::from_fn_ptr(mutation_observer_disconnect),
            )
            .method(
                JsString::from("takeRecords"),
                0,
                NativeFunction::from_fn_ptr(mutation_observer_take_records),
            )
            .method(
                JsString::from("__queueRecord"),
                1,
                NativeFunction::from_fn_ptr(mutation_observer_queue_record_internal),
            );

        Ok(())
    }
}

pub fn mutation_observer_observe(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let observer = obj.downcast_ref::<MutationObserver>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationObserver object"),
        )
    })?;

    let target = args.first().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: observe() requires target parameter"),
        )
    })?;

    let target_obj = target.as_object().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: target must be a DOM Node (object)"),
        )
    })?;

    let key_val = target_obj.get(JsString::from("__key__"), context)?;
    if key_val.is_undefined() || key_val.is_null() {
        return Err(JsError::from(JsNativeError::typ().with_message(
            "TypeError: target must be a DOM Node with a __key__",
        )));
    }
    let target_key = key_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let options_val = args.get(1).ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: observe() requires options parameter"),
        )
    })?;
    let options = parse_mutation_observer_options(options_val, context)?;

    // Check if duplicate target is being observed
    let mut active = observer.active_observations.borrow_mut();
    if let Some(pos) = active.iter().position(|obs| obs.target_key == target_key) {
        active[pos].options = options;
    } else {
        active.push(Observation {
            target: target.clone(),
            target_key,
            options,
        });
    }

    Ok(JsValue::undefined())
}

pub fn mutation_observer_disconnect(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let observer = obj.downcast_ref::<MutationObserver>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationObserver object"),
        )
    })?;

    observer.active_observations.borrow_mut().clear();
    observer.record_queue.borrow_mut().clear();

    Ok(JsValue::undefined())
}

pub fn mutation_observer_take_records(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let observer = obj.downcast_ref::<MutationObserver>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationObserver object"),
        )
    })?;

    // Drain/empty the queue
    let mut queue = observer.record_queue.borrow_mut();
    let elements: Vec<JsValue> = std::mem::take(&mut *queue);

    let array = boa_engine::object::builtins::JsArray::from_iter(elements, context);
    Ok(JsValue::from(array))
}

pub fn mutation_observer_queue_record_internal(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let observer = obj.downcast_ref::<MutationObserver>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationObserver object"),
        )
    })?;

    let record = args.first().cloned().unwrap_or(JsValue::undefined());
    observer.record_queue.borrow_mut().push(record);

    Ok(JsValue::undefined())
}

/// Implementation of WHATWG DOM `MutationRecord` interface.
/// Spec: <https://dom.spec.whatwg.org/#interface-mutationrecord>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct MutationRecord {
    pub(crate) r_type: JsString,
    pub(crate) target: JsValue,
    pub(crate) added_nodes: JsValue,
    pub(crate) removed_nodes: JsValue,
    pub(crate) previous_sibling: JsValue,
    pub(crate) next_sibling: JsValue,
    pub(crate) attribute_name: JsValue,
    pub(crate) attribute_namespace: JsValue,
    pub(crate) old_value: JsValue,
}

impl Class for MutationRecord {
    const NAME: &'static str = "MutationRecord";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        let init_obj = args.first().and_then(|v| v.as_object());

        let (
            r_type,
            target,
            added_nodes,
            removed_nodes,
            previous_sibling,
            next_sibling,
            attribute_name,
            attribute_namespace,
            old_value,
        ) = if let Some(obj) = init_obj {
            (
                obj.get(JsString::from("type"), context)?
                    .to_string(context)?,
                obj.get(JsString::from("target"), context)?,
                obj.get(JsString::from("addedNodes"), context)?,
                obj.get(JsString::from("removedNodes"), context)?,
                obj.get(JsString::from("previousSibling"), context)?,
                obj.get(JsString::from("nextSibling"), context)?,
                obj.get(JsString::from("attributeName"), context)?,
                obj.get(JsString::from("attributeNamespace"), context)?,
                obj.get(JsString::from("oldValue"), context)?,
            )
        } else {
            (
                JsString::from(""),
                JsValue::undefined(),
                JsValue::undefined(),
                JsValue::undefined(),
                JsValue::null(),
                JsValue::null(),
                JsValue::null(),
                JsValue::null(),
                JsValue::null(),
            )
        };

        Ok(MutationRecord {
            r_type,
            target,
            added_nodes,
            removed_nodes,
            previous_sibling,
            next_sibling,
            attribute_name,
            attribute_namespace,
            old_value,
        })
    }

    #[allow(clippy::type_complexity)]
    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let getters: &[(
            &str,
            fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>,
        )] = &[
            ("type", mutation_record_get_type),
            ("target", mutation_record_get_target),
            ("addedNodes", mutation_record_get_added_nodes),
            ("removedNodes", mutation_record_get_removed_nodes),
            ("previousSibling", mutation_record_get_previous_sibling),
            ("nextSibling", mutation_record_get_next_sibling),
            ("attributeName", mutation_record_get_attribute_name),
            (
                "attributeNamespace",
                mutation_record_get_attribute_namespace,
            ),
            ("oldValue", mutation_record_get_old_value),
        ];

        for &(name, func) in getters {
            let getter_fn = boa_engine::object::FunctionObjectBuilder::new(
                &realm,
                NativeFunction::from_fn_ptr(func),
            )
            .name(format!("get {}", name))
            .build();

            class.accessor(
                JsString::from(name),
                Some(getter_fn),
                None,
                Attribute::all(),
            );
        }

        Ok(())
    }
}

pub fn mutation_record_get_type(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let record = obj.downcast_ref::<MutationRecord>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationRecord object"),
        )
    })?;
    Ok(JsValue::from(record.r_type.clone()))
}

pub fn mutation_record_get_target(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let record = obj.downcast_ref::<MutationRecord>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationRecord object"),
        )
    })?;
    Ok(record.target.clone())
}

pub fn mutation_record_get_added_nodes(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let record = obj.downcast_ref::<MutationRecord>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationRecord object"),
        )
    })?;
    Ok(record.added_nodes.clone())
}

pub fn mutation_record_get_removed_nodes(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let record = obj.downcast_ref::<MutationRecord>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationRecord object"),
        )
    })?;
    Ok(record.removed_nodes.clone())
}

pub fn mutation_record_get_previous_sibling(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let record = obj.downcast_ref::<MutationRecord>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationRecord object"),
        )
    })?;
    Ok(record.previous_sibling.clone())
}

pub fn mutation_record_get_next_sibling(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let record = obj.downcast_ref::<MutationRecord>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationRecord object"),
        )
    })?;
    Ok(record.next_sibling.clone())
}

pub fn mutation_record_get_attribute_name(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let record = obj.downcast_ref::<MutationRecord>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationRecord object"),
        )
    })?;
    Ok(record.attribute_name.clone())
}

pub fn mutation_record_get_attribute_namespace(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let record = obj.downcast_ref::<MutationRecord>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationRecord object"),
        )
    })?;
    Ok(record.attribute_namespace.clone())
}

pub fn mutation_record_get_old_value(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let record = obj.downcast_ref::<MutationRecord>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("TypeError: Method called on non-MutationRecord object"),
        )
    })?;
    Ok(record.old_value.clone())
}

/// Implementation of WHATWG DOM `CustomEvent` interface.
/// Spec: <https://dom.spec.whatwg.org/#interface-customevent>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct CustomEvent {
    pub(crate) r#type: String,
    pub(crate) bubbles: bool,
    pub(crate) cancelable: bool,
    pub(crate) detail: GcRefCell<JsValue>,
    pub(crate) target: GcRefCell<Option<JsValue>>,
    pub(crate) current_target: GcRefCell<Option<JsValue>>,
    pub(crate) default_prevented: GcRefCell<bool>,
    pub(crate) propagation_stopped: GcRefCell<bool>,
}

impl Class for CustomEvent {
    const NAME: &'static str = "CustomEvent";
    const LENGTH: usize = 1;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        let event_type = args
            .first()
            .ok_or_else(|| {
                JsError::from(
                    JsNativeError::typ()
                        .with_message("CustomEvent constructor requires at least 1 argument"),
                )
            })?
            .to_string(context)?
            .to_std_string()
            .unwrap_or_default();

        let mut detail = JsValue::null();
        let mut bubbles = false;
        let mut cancelable = false;

        if let Some(options_val) = args.get(1)
            && let Some(options_obj) = options_val.as_object()
        {
            if let Ok(detail_val) = options_obj.get(JsString::from("detail"), context)
                && !detail_val.is_undefined()
            {
                detail = detail_val;
            }
            if let Ok(bubbles_val) = options_obj.get(JsString::from("bubbles"), context) {
                bubbles = bubbles_val.as_boolean().unwrap_or(false);
            }
            if let Ok(cancelable_val) = options_obj.get(JsString::from("cancelable"), context) {
                cancelable = cancelable_val.as_boolean().unwrap_or(false);
            }
        }

        Ok(CustomEvent {
            r#type: event_type,
            bubbles,
            cancelable,
            detail: GcRefCell::new(detail),
            target: GcRefCell::new(None),
            current_target: GcRefCell::new(None),
            default_prevented: GcRefCell::new(false),
            propagation_stopped: GcRefCell::new(false),
        })
    }

    #[allow(clippy::type_complexity)]
    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let getters: &[(
            &str,
            fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>,
        )] = &[
            ("type", custom_event_get_type),
            ("bubbles", custom_event_get_bubbles),
            ("cancelable", custom_event_get_cancelable),
            ("detail", custom_event_get_detail),
            ("target", custom_event_get_target),
            ("currentTarget", custom_event_get_current_target),
            ("defaultPrevented", custom_event_get_default_prevented),
            ("propagationStopped", custom_event_get_propagation_stopped),
        ];

        for &(name, func) in getters {
            let getter_fn = boa_engine::object::FunctionObjectBuilder::new(
                &realm,
                NativeFunction::from_fn_ptr(func),
            )
            .name(format!("get {}", name))
            .build();

            // Check if there is a setter
            let setter_fn = if name == "target" {
                Some(
                    boa_engine::object::FunctionObjectBuilder::new(
                        &realm,
                        NativeFunction::from_fn_ptr(custom_event_set_target),
                    )
                    .name("set target")
                    .build(),
                )
            } else if name == "currentTarget" {
                Some(
                    boa_engine::object::FunctionObjectBuilder::new(
                        &realm,
                        NativeFunction::from_fn_ptr(custom_event_set_current_target),
                    )
                    .name("set currentTarget")
                    .build(),
                )
            } else {
                None
            };

            class.accessor(
                JsString::from(name),
                Some(getter_fn),
                setter_fn,
                Attribute::all(),
            );
        }

        class.method(
            JsString::from("preventDefault"),
            0,
            NativeFunction::from_fn_ptr(custom_event_prevent_default),
        );
        class.method(
            JsString::from("stopPropagation"),
            0,
            NativeFunction::from_fn_ptr(custom_event_stop_propagation),
        );

        Ok(())
    }
}

pub fn custom_event_get_type(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    Ok(JsValue::from(JsString::from(event.r#type.clone())))
}

pub fn custom_event_get_bubbles(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    Ok(JsValue::from(event.bubbles))
}

pub fn custom_event_get_cancelable(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    Ok(JsValue::from(event.cancelable))
}

pub fn custom_event_get_detail(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    Ok(event.detail.borrow().clone())
}

pub fn custom_event_get_target(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    Ok(event.target.borrow().clone().unwrap_or(JsValue::null()))
}

pub fn custom_event_set_target(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    let val = args.first().cloned().unwrap_or(JsValue::null());
    *event.target.borrow_mut() = if val.is_null() || val.is_undefined() {
        None
    } else {
        Some(val)
    };
    Ok(JsValue::undefined())
}

pub fn custom_event_get_current_target(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    Ok(event
        .current_target
        .borrow()
        .clone()
        .unwrap_or(JsValue::null()))
}

pub fn custom_event_set_current_target(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    let val = args.first().cloned().unwrap_or(JsValue::null());
    *event.current_target.borrow_mut() = if val.is_null() || val.is_undefined() {
        None
    } else {
        Some(val)
    };
    Ok(JsValue::undefined())
}

pub fn custom_event_get_default_prevented(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    Ok(JsValue::from(*event.default_prevented.borrow()))
}

pub fn custom_event_get_propagation_stopped(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    Ok(JsValue::from(*event.propagation_stopped.borrow()))
}

pub fn custom_event_prevent_default(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    *event.default_prevented.borrow_mut() = true;
    Ok(JsValue::undefined())
}

pub fn custom_event_stop_propagation(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<CustomEvent>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-CustomEvent object"),
        )
    })?;
    *event.propagation_stopped.borrow_mut() = true;
    Ok(JsValue::undefined())
}

/// Implementation of W3C File API `Blob` interface.
/// Spec: <https://w3c.github.io/FileAPI/#dfn-Blob>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct Blob {
    pub(crate) bytes: Vec<u8>,
    pub(crate) mime_type: String,
}

impl Class for Blob {
    const NAME: &'static str = "Blob";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        let mut bytes = Vec::new();

        // 1. Handle parts (first argument)
        if let Some(parts_val) = args.first()
            && !parts_val.is_undefined()
            && !parts_val.is_null()
        {
            if let Some(obj) = parts_val.as_object() {
                let length_val = obj.get(JsString::from("length"), context)?;
                let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);
                for i in 0..length {
                    let part_val = obj.get(i, context)?;
                    if let Some(part_obj) = part_val.as_object()
                        && let Some(other_blob) = part_obj.downcast_ref::<Blob>()
                    {
                        bytes.extend_from_slice(&other_blob.bytes);
                        continue;
                    }
                    let part_str = part_val
                        .to_string(context)?
                        .to_std_string()
                        .unwrap_or_default();
                    bytes.extend_from_slice(part_str.as_bytes());
                }
            } else {
                return Err(JsError::from(
                    JsNativeError::typ()
                        .with_message("Blob parts must be an array-like/sequence object"),
                ));
            }
        }

        // 2. Handle options (second argument)
        let mut mime_type = String::new();
        if let Some(options_val) = args.get(1)
            && !options_val.is_undefined()
            && !options_val.is_null()
            && let Some(options_obj) = options_val.as_object()
            && let Ok(type_val) = options_obj.get(JsString::from("type"), context)
            && !type_val.is_undefined()
        {
            let raw_type = type_val
                .to_string(context)?
                .to_std_string()
                .unwrap_or_default();
            if raw_type
                .chars()
                .any(|c| !(0x20..=0x7E).contains(&(c as u32)))
            {
                mime_type = String::new();
            } else {
                mime_type = raw_type.to_lowercase();
            }
        }

        Ok(Blob { bytes, mime_type })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        // Define getter/accessor for "size"
        let size_getter = boa_engine::object::FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(blob_get_size),
        )
        .name("get size")
        .build();

        class.accessor(
            JsString::from("size"),
            Some(size_getter),
            None,
            Attribute::all(),
        );

        // Define getter/accessor for "type"
        let type_getter = boa_engine::object::FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(blob_get_type),
        )
        .name("get type")
        .build();

        class.accessor(
            JsString::from("type"),
            Some(type_getter),
            None,
            Attribute::all(),
        );

        // Define method "text"
        class.method(
            JsString::from("text"),
            0,
            NativeFunction::from_fn_ptr(blob_text),
        );

        Ok(())
    }
}

pub fn blob_get_size(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let blob = obj.downcast_ref::<Blob>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-Blob object"),
        )
    })?;
    Ok(JsValue::from(blob.bytes.len()))
}

pub fn blob_get_type(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let blob = obj.downcast_ref::<Blob>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-Blob object"),
        )
    })?;
    Ok(JsValue::from(JsString::from(blob.mime_type.clone())))
}

pub fn blob_text(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("TypeError: Method called on non-object"))
    })?;
    let blob = obj.downcast_ref::<Blob>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: Method called on non-Blob object"),
        )
    })?;
    // // TODO(spec): note that the real API returns a Promise. Since our engine does not have Promise support, we return the string synchronously.
    let text = String::from_utf8_lossy(&blob.bytes).into_owned();
    Ok(JsValue::from(JsString::from(text)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_observer_t0525() {
        let mut host = BoaHost::new();
        let mut dom = crate::dom::Dom::new();

        // 1. Basic properties and constructor exist on global, and validation on constructor
        host.eval_with_dom(
            r#"{
            if (typeof MutationObserver === "undefined") throw "MutationObserver undefined";
            if (typeof MutationRecord === "undefined") throw "MutationRecord undefined";

            let thrown = false;
            try {
                new MutationObserver();
            } catch (e) {
                thrown = true;
            }
            if (!thrown) throw "Constructor without callback did not throw TypeError";

            thrown = false;
            try {
                new MutationObserver("not-a-function");
            } catch (e) {
                thrown = true;
            }
            if (!thrown) throw "Constructor with string callback did not throw TypeError";

            const observer = new MutationObserver(() => {});
            if (!observer.observe) throw "observe method missing";
            if (!observer.disconnect) throw "disconnect method missing";
            if (!observer.takeRecords) throw "takeRecords method missing";
        }"#,
            &mut dom,
        )
        .unwrap();

        // 2. Options validation for observe()
        host.eval_with_dom(
            r#"{
            const div = document.createElement("div");
            const observer = new MutationObserver(() => {});

            // Missing options or empty options must throw TypeError
            let thrown = false;
            try {
                observer.observe(div);
            } catch (e) {
                thrown = true;
            }
            if (!thrown) throw "observe without options did not throw TypeError";

            thrown = false;
            try {
                observer.observe(div, {});
            } catch (e) {
                thrown = true;
            }
            if (!thrown) throw "observe with empty options did not throw TypeError";

            thrown = false;
            try {
                observer.observe(div, { attributes: false, attributeOldValue: true });
            } catch (e) {
                thrown = true;
            }
            if (!thrown) throw "attributeOldValue with attributes: false did not throw";

            thrown = false;
            try {
                observer.observe(div, { characterData: false, characterDataOldValue: true });
            } catch (e) {
                thrown = true;
            }
            if (!thrown) throw "characterDataOldValue with characterData: false did not throw";
        }"#,
            &mut dom,
        )
        .unwrap();

        // 3. Valid options configurations and queue/drain semantics
        host.eval_with_dom(
            r#"{
            const div = document.createElement("div");
            const observer = new MutationObserver((records) => {});

            // Initial records must be empty
            const initial = observer.takeRecords();
            if (initial.length !== 0) throw "initial records not empty";

            // Observe with valid childList
            observer.observe(div, { child_list: false, childList: true });

            // Mock/Queue a mutation record
            const record = new MutationRecord({
                type: "childList",
                target: div,
                addedNodes: [div],
                removedNodes: [],
                oldValue: "old-val"
            });

            if (record.type !== "childList") throw "record type mismatch";
            if (record.target !== div) throw "record target mismatch";
            if (record.oldValue !== "old-val") throw "record oldValue mismatch";

            observer.__queueRecord(record);

            let records = observer.takeRecords();
            if (records.length !== 1) throw "queue size mismatch";
            if (records[0] !== record) throw "wrong record returned";

            // Queue is now empty
            records = observer.takeRecords();
            if (records.length !== 0) throw "queue not empty after takeRecords";
        }"#,
            &mut dom,
        )
        .unwrap();

        // 4. disconnect() clears active observations and pending records
        host.eval_with_dom(
            r#"{
            const div = document.createElement("div");
            const observer = new MutationObserver(() => {});

            observer.observe(div, { childList: true });

            const record = new MutationRecord({
                type: "attributes",
                target: div
            });
            observer.__queueRecord(record);

            observer.disconnect();

            const records = observer.takeRecords();
            if (records.length !== 0) throw "records not cleared by disconnect";
        }"#,
            &mut dom,
        )
        .unwrap();
    }

    #[test]
    fn test_abort_controller_signal_t0518() {
        let mut host = BoaHost::new();

        // 1. Basic properties and constructor exist on global
        host.eval(r#"{
            if (typeof AbortController === "undefined") throw "AbortController undefined";
            if (typeof AbortSignal === "undefined") throw "AbortSignal undefined";
            
            const controller = new AbortController();
            if (!controller.signal) throw "controller.signal not present";
            if (controller.signal.aborted !== false) throw "should not be aborted initially";
            if (controller.signal.reason !== undefined) throw "reason should be undefined initially";
        }"#).unwrap();

        // 2. abort() sets aborted and reason, and throwIfAborted throws
        host.eval(
            r#"{
            const controller = new AbortController();
            const signal = controller.signal;
            
            let thrown = false;
            try {
                signal.throwIfAborted();
            } catch (e) {
                thrown = true;
            }
            if (thrown) throw "throwIfAborted threw before aborted";

            controller.abort("custom reason");
            if (signal.aborted !== true) throw "aborted should be true";
            if (signal.reason !== "custom reason") throw "reason mismatch";

            thrown = false;
            let caughtReason = null;
            try {
                signal.throwIfAborted();
            } catch (e) {
                thrown = true;
                caughtReason = e;
            }
            if (!thrown) throw "throwIfAborted did not throw after aborted";
            if (caughtReason !== "custom reason") throw "throwIfAborted threw wrong reason";
        }"#,
        )
        .unwrap();

        // 3. Static AbortSignal.abort(reason?)
        host.eval(r#"{
            const signal1 = AbortSignal.abort();
            if (signal1.aborted !== true) throw "static abort signal not aborted";
            if (signal1.reason.name !== "AbortError") throw "static abort default reason not AbortError";

            const signal2 = AbortSignal.abort("static reason");
            if (signal2.aborted !== true) throw "static abort signal 2 not aborted";
            if (signal2.reason !== "static reason") throw "static abort reason 2 mismatch";
        }"#).unwrap();

        // 4. addEventListener("abort", cb) and onabort support
        host.eval(r#"{
            const controller = new AbortController();
            const signal = controller.signal;

            let listenerCalled = 0;
            let listenerArg = null;
            signal.addEventListener("abort", (e) => {
                listenerCalled++;
                listenerArg = e;
            });

            let onabortCalled = 0;
            let onabortArg = null;
            signal.onabort = (e) => {
                onabortCalled++;
                onabortArg = e;
            };

            controller.abort("event reason");
            
            if (listenerCalled !== 1) throw "listener should be called exactly once";
            if (listenerArg.type !== "abort") throw "listener event type mismatch";
            if (onabortCalled !== 1) throw "onabort should be called exactly once";
            if (onabortArg.type !== "abort") throw "onabort event type mismatch";

            // Subsequent aborts are no-ops
            controller.abort("another reason");
            if (listenerCalled !== 1) throw "listener called again";
            if (onabortCalled !== 1) throw "onabort called again";
            if (signal.reason !== "event reason") throw "reason should not change on subsequent aborts";
        }"#).unwrap();
    }

    #[test]
    fn test_dom_parser_t0522() {
        let mut host = BoaHost::new();

        // 1. DOMParser constructor exists on global
        host.eval(r#"{
            if (typeof DOMParser === "undefined") throw "DOMParser is not defined";
            const parser = new DOMParser();
            if (typeof parser.parseFromString !== "function") throw "parseFromString is not a function";
        }"#).unwrap();

        // 2. parseFromString works on a simple HTML string and returns a Document
        let mut dom = crate::dom::Dom::new();
        let res = host.eval_with_dom(
            r#"{
                const parser = new DOMParser();
                const doc = parser.parseFromString('<div id="x">hello DOMParser</div>', 'text/html');
                
                // Assert it behaves like a Document with the existing accessors
                if (!doc) throw "doc is null or undefined";
                const elem = doc.getElementById('x');
                if (!elem) throw "element with ID x not found in parsed document";
                elem.textContent;
            }"#,
            &mut dom,
        ).unwrap();
        assert_eq!(res, "hello DOMParser");

        // 3. Document fallback for text/xml and application/xml
        host.eval_with_dom(
            r#"{
                const parser = new DOMParser();
                const docXml = parser.parseFromString('<xml id="xml-test">xml content</xml>', 'text/xml');
                const elem = docXml.getElementById('xml-test');
                if (elem.textContent !== 'xml content') throw "XML fallback failed";
            }"#,
            &mut dom,
        ).unwrap();
    }

    #[test]
    fn test_structured_clone_t0514() {
        let mut host = BoaHost::new();

        // 1. Primitive round-trips
        host.eval(r#"{
            if (structuredClone(42) !== 42) throw "number mismatch";
            if (structuredClone("hello") !== "hello") throw "string mismatch";
            if (structuredClone(true) !== true) throw "boolean true mismatch";
            if (structuredClone(false) !== false) throw "boolean false mismatch";
            if (structuredClone(null) !== null) throw "null mismatch";
            if (structuredClone(undefined) !== undefined) throw "undefined mismatch";
            if (structuredClone(12345678901234567890n) !== 12345678901234567890n) throw "bigint mismatch";
        }"#).unwrap();

        // 2. Objects deep cloning and independence
        host.eval(
            r#"{
            const original = { a: 1, b: { c: 2 } };
            const clone = structuredClone(original);
            if (clone.a !== 1) throw "clone.a mismatch";
            if (clone.b.c !== 2) throw "clone.b.c mismatch";
            
            // Mutate clone and assert original is unchanged (deep copy)
            clone.b.c = 99;
            if (original.b.c !== 2) throw "original mutated!";
            if (clone.b.c !== 99) throw "clone not mutated";
        }"#,
        )
        .unwrap();

        // 3. Arrays and nested arrays
        host.eval(
            r#"{
            const original = [1, 2, [3, 4], { x: 5 }];
            const clone = structuredClone(original);
            if (clone[0] !== 1 || clone[1] !== 2) throw "flat array mismatch";
            if (clone[2][0] !== 3 || clone[2][1] !== 4) throw "nested array mismatch";
            if (clone[3].x !== 5) throw "nested object in array mismatch";

            // Mutate nested array
            clone[2][0] = 99;
            if (original[2][0] !== 3) throw "original array mutated!";
        }"#,
        )
        .unwrap();

        // 4. Dates
        host.eval(
            r#"{
            const d = new Date(1450000000000);
            const clone = structuredClone(d);
            if (clone.getTime() !== 1450000000000) throw "Date time mismatch";
            if (clone === d) throw "Date reference is the same";
        }"#,
        )
        .unwrap();

        // 5. Maps
        host.eval(
            r#"{
            const m = new Map();
            m.set("key", { val: 42 });
            const clone = structuredClone(m);
            if (!clone.has("key")) throw "Map clone missing key";
            if (clone.get("key").val !== 42) throw "Map clone nested value mismatch";

            // Mutate nested value in Map
            clone.get("key").val = 99;
            if (m.get("key").val !== 42) throw "original Map mutated!";
        }"#,
        )
        .unwrap();

        // 6. Sets
        host.eval(
            r#"{
            const s = new Set();
            const obj = { val: 100 };
            s.add(obj);
            const clone = structuredClone(s);
            if (clone.size !== 1) throw "Set clone size mismatch";
            
            // Retrieve the cloned object
            let clonedObj;
            clone.forEach(v => { clonedObj = v; });
            if (clonedObj.val !== 100) throw "Set clone element mismatch";
            if (clonedObj === obj) throw "Set element reference same";

            clonedObj.val = 200;
            if (obj.val !== 100) throw "original Set element mutated!";
        }"#,
        )
        .unwrap();

        // 7. Non-cloneable: Functions and Symbols should throw TypeError
        assert!(host.eval("structuredClone(() => {})").is_err());
        assert!(host.eval("structuredClone(Symbol('test'))").is_err());
    }

    #[test]
    fn test_url_search_params_basic() {
        let mut host = BoaHost::new();

        // 1. Constructor with query string parsing, get, getAll, has
        host.eval(
            r#"{
            const params = new URLSearchParams("a=1&b=2&a=3");
            if (params.get("a") !== "1") throw "get(a) mismatch";
            if (params.get("b") !== "2") throw "get(b) mismatch";
            if (params.get("z") !== null) throw "get(z) should be null";

            const allA = params.getAll("a");
            if (allA.length !== 2 || allA[0] !== "1" || allA[1] !== "3") throw "getAll(a) mismatch";

            if (params.has("b") !== true) throw "has(b) should be true";
            if (params.has("z") !== false) throw "has(z) should be false";
        }"#,
        )
        .unwrap();

        // 2. Leading "?" stripping
        host.eval(
            r#"{
            const params = new URLSearchParams("?x=y");
            if (params.get("x") !== "y") throw "leading ? strip mismatch";
        }"#,
        )
        .unwrap();

        // 3. Percent and plus decoding
        host.eval(
            r#"{
            const params = new URLSearchParams("q=hello+world%21");
            if (params.get("q") !== "hello world!") throw "percent/plus decode mismatch";
        }"#,
        )
        .unwrap();

        // 4. Mutation operations: set, append, delete, toString
        host.eval(
            r#"{
            const params = new URLSearchParams();
            params.append("a", "1");
            params.append("b", "2");
            if (params.toString() !== "a=1&b=2") throw "initial toString mismatch";

            params.append("a", "3");
            if (params.toString() !== "a=1&b=2&a=3") throw "after append toString mismatch";

            params.set("a", "4");
            if (params.get("a") !== "4") throw "after set get(a) mismatch";
            if (params.getAll("a").length !== 1) throw "set should collapse occurrences";
            if (params.toString() !== "a=4&b=2") throw "after set toString mismatch";

            params.delete("b");
            if (params.has("b") !== false) throw "after delete has(b) should be false";
            if (params.toString() !== "a=4") throw "after delete toString mismatch";
        }"#,
        )
        .unwrap();
    }

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
    fn test_performance_now() {
        let mut host = BoaHost::new();
        assert!(
            host.eval("if (typeof performance.now !== 'function') throw 'now is not a function';")
                .is_ok()
        );
        assert!(
            host.eval("if (typeof window.performance.now !== 'function') throw 'window now is not a function';")
                .is_ok()
        );
        assert!(
            host.eval("const t = performance.now(); if (typeof t !== 'number' || t < 0) throw 'invalid now value';")
                .is_ok()
        );
        assert!(
            host.eval("const t1 = performance.now(); const t2 = performance.now(); if (!(t2 >= t1)) throw 'not monotonic';")
                .is_ok()
        );
    }

    #[test]
    fn test_crypto_api() {
        let mut host = BoaHost::new();
        let script = r#"
            // Check typeof
            if (typeof crypto !== "object" || crypto === null) throw "crypto is not an object";
            if (typeof crypto.randomUUID !== "function") throw "randomUUID is not a function";
            if (typeof crypto.getRandomValues !== "function") throw "getRandomValues is not a function";

            // Check randomUUID shape
            const uuid1 = crypto.randomUUID();
            if (typeof uuid1 !== "string") throw "uuid1 is not a string";
            if (uuid1.length !== 36) throw "uuid1 length is not 36, got " + uuid1.length;
            if (uuid1[8] !== '-' || uuid1[13] !== '-' || uuid1[18] !== '-' || uuid1[23] !== '-') {
                throw "uuid1 missing dashes at correct places: " + uuid1;
            }
            if (uuid1[14] !== '4') throw "uuid1 version must be 4, got " + uuid1[14];
            const y = uuid1[19];
            if (y !== '8' && y !== '9' && y !== 'a' && y !== 'b') {
                throw "uuid1 variant must be 8, 9, a, or b, got " + y;
            }

            // Two randomUUID calls return different strings
            const uuid2 = crypto.randomUUID();
            if (uuid1 === uuid2) throw "uuid1 and uuid2 are identical: " + uuid1;

            // getRandomValues on a new Uint8Array(16) returns the same array and is not all-zero
            const arr = new Uint8Array(16);
            const result = crypto.getRandomValues(arr);
            if (result !== arr) throw "getRandomValues did not return the exact same array object";

            let allZero = true;
            for (let i = 0; i < arr.length; i++) {
                if (arr[i] !== 0) {
                    allZero = false;
                    break;
                }
            }
            if (allZero) throw "getRandomValues filled with all zeros";

            // getRandomValues on Int16Array, Int32Array, BigInt64Array
            const int16 = new Int16Array(5);
            const int16_res = crypto.getRandomValues(int16);
            if (int16_res !== int16) throw "Int16Array: getRandomValues failed";

            const int32 = new Int32Array(5);
            const int32_res = crypto.getRandomValues(int32);
            if (int32_res !== int32) throw "Int32Array: getRandomValues failed";

            const bigint64 = new BigInt64Array(5);
            const bigint64_res = crypto.getRandomValues(bigint64);
            if (bigint64_res !== bigint64) throw "BigInt64Array: getRandomValues failed";

            // getRandomValues throws when passed a non-TypedArray
            let threw = false;
            try {
                crypto.getRandomValues({});
            } catch (e) {
                threw = true;
            }
            if (!threw) throw "getRandomValues should throw for a plain object";

            threw = false;
            try {
                crypto.getRandomValues(42);
            } catch (e) {
                threw = true;
            }
            if (!threw) throw "getRandomValues should throw for a number";

            // Check byteLength > 65536 throws
            threw = false;
            try {
                const huge = new Uint8Array(65537);
                crypto.getRandomValues(huge);
            } catch (e) {
                threw = true;
            }
            if (!threw) throw "getRandomValues should throw for array larger than 65536 bytes";
        "#;
        assert!(
            host.eval(script).is_ok(),
            "Crypto API JS verification failed!"
        );
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
    fn test_node_compare_document_position() {
        let mut dom = Dom::new();
        let document = dom.document();

        // Build a DOM tree:
        // document -> parent (div) -> child1 (span), child2 (div)
        let parent_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "parent".to_string())],
        });
        let child1_id = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![("id".to_string(), "child1".to_string())],
        });
        let child2_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "child2".to_string())],
        });

        dom.append_child(parent_id, child1_id);
        dom.append_child(parent_id, child2_id);
        dom.append_child(document, parent_id);

        let mut host = BoaHost::new();

        // 1. A node compared to itself returns 0
        let res_self = host.eval_with_dom(
            "document.getElementById('parent').compareDocumentPosition(document.getElementById('parent'))",
            &mut dom,
        );
        assert_eq!(res_self, Ok("0".to_string()));

        // 2. Parent.compareDocumentPosition(child) returns 20 (CONTAINED_BY | FOLLOWING)
        let res_parent_child = host.eval_with_dom(
            "document.getElementById('parent').compareDocumentPosition(document.getElementById('child1'))",
            &mut dom,
        );
        assert_eq!(res_parent_child, Ok("20".to_string()));

        // 3. Child.compareDocumentPosition(parent) returns 10 (CONTAINS | PRECEDING)
        let res_child_parent = host.eval_with_dom(
            "document.getElementById('child1').compareDocumentPosition(document.getElementById('parent'))",
            &mut dom,
        );
        assert_eq!(res_child_parent, Ok("10".to_string()));

        // 4. Sibling comparison: child1 (earlier) and child2 (later)
        // child1.compareDocumentPosition(child2) should return 4 (DOCUMENT_POSITION_FOLLOWING)
        let res_sibling_following = host.eval_with_dom(
            "document.getElementById('child1').compareDocumentPosition(document.getElementById('child2'))",
            &mut dom,
        );
        assert_eq!(res_sibling_following, Ok("4".to_string()));

        // child2.compareDocumentPosition(child1) should return 2 (DOCUMENT_POSITION_PRECEDING)
        let res_sibling_preceding = host.eval_with_dom(
            "document.getElementById('child2').compareDocumentPosition(document.getElementById('child1'))",
            &mut dom,
        );
        assert_eq!(res_sibling_preceding, Ok("2".to_string()));

        // 5. Freshly-created, unattached node has DISCONNECTED (1) bit set (returns 35: DISCONNECTED | IMPLEMENTATION_SPECIFIC | PRECEDING)
        let res_unattached = host.eval_with_dom(
            "const unattached = document.createElement('div'); document.getElementById('parent').compareDocumentPosition(unattached)",
            &mut dom,
        );
        assert_eq!(res_unattached, Ok("35".to_string()));

        // 6. null and undefined compared to a node should return 35
        let res_null = host.eval_with_dom(
            "document.getElementById('parent').compareDocumentPosition(null)",
            &mut dom,
        );
        assert_eq!(res_null, Ok("35".to_string()));

        let res_undefined = host.eval_with_dom(
            "document.getElementById('parent').compareDocumentPosition(undefined)",
            &mut dom,
        );
        assert_eq!(res_undefined, Ok("35".to_string()));

        // 7. Verify constant properties exist on node instances and document
        let res_const_on_node = host.eval_with_dom(
            "const p = document.getElementById('parent'); p.DOCUMENT_POSITION_DISCONNECTED === 1 && p.DOCUMENT_POSITION_PRECEDING === 2 && p.DOCUMENT_POSITION_FOLLOWING === 4 && p.DOCUMENT_POSITION_CONTAINS === 8 && p.DOCUMENT_POSITION_CONTAINED_BY === 16 && p.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC === 32",
            &mut dom,
        );
        assert_eq!(res_const_on_node, Ok("true".to_string()));

        let res_const_on_doc = host.eval_with_dom(
            "document.DOCUMENT_POSITION_DISCONNECTED === 1 && document.DOCUMENT_POSITION_PRECEDING === 2 && document.DOCUMENT_POSITION_FOLLOWING === 4 && document.DOCUMENT_POSITION_CONTAINS === 8 && document.DOCUMENT_POSITION_CONTAINED_BY === 16 && document.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC === 32",
            &mut dom,
        );
        assert_eq!(res_const_on_doc, Ok("true".to_string()));
    }

    #[test]
    fn test_node_has_child_nodes() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            const div = document.createElement('div');
            const hasNoChildren = div.hasChildNodes();
            
            const span = document.createElement('span');
            div.appendChild(span);
            const hasChildrenNow = div.hasChildNodes();
            
            [hasNoChildren, hasChildrenNow].join(',');
        ";
        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(res, Ok("false,true".to_string()));
    }

    #[test]
    fn test_node_get_root_node() {
        let mut dom = Dom::new();
        let document = dom.document();

        // Create an element and append to document
        let parent_div = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "parent".to_string())],
        });
        let child_span = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![("id".to_string(), "child".to_string())],
        });
        dom.append_child(parent_div, child_span);
        dom.append_child(document, parent_div);

        let mut host = BoaHost::new();

        // a. An element appended into the document returns document:
        let res_child_root = host.eval_with_dom(
            "document.getElementById('child').getRootNode() === document",
            &mut dom,
        );
        assert_eq!(res_child_root, Ok("true".to_string()));

        let res_parent_root = host.eval_with_dom(
            "document.getElementById('parent').getRootNode() === document",
            &mut dom,
        );
        assert_eq!(res_parent_root, Ok("true".to_string()));

        // b. document.getRootNode() === document
        let res_doc_root = host.eval_with_dom("document.getRootNode() === document", &mut dom);
        assert_eq!(res_doc_root, Ok("true".to_string()));

        // c. A freshly created, NOT-yet-appended node returns itself
        let res_fresh_root = host.eval_with_dom(
            "const n = document.createElement('div'); n.getRootNode() === n",
            &mut dom,
        );
        assert_eq!(res_fresh_root, Ok("true".to_string()));

        // d. A nested element (grandchild) returns the same root as its parent
        let res_nested_detached = host.eval_with_dom(
            "const p = document.createElement('div'); const c = document.createElement('span'); const gc = document.createElement('a'); c.appendChild(gc); p.appendChild(c); gc.getRootNode() === p",
            &mut dom,
        );
        assert_eq!(res_nested_detached, Ok("true".to_string()));
    }

    #[test]
    fn test_node_is_equal_and_is_same() {
        let mut dom = Dom::new();
        let document = dom.document();

        // Let's build some nodes in DOM:
        // div1 with class "x" and child text "hello"
        let div1 = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("id".to_string(), "div1".to_string()),
                ("class".to_string(), "x".to_string()),
            ],
        });
        let text1 = dom.create_node(NodeData::Text("hello".to_string()));
        dom.append_child(div1, text1);
        dom.append_child(document, div1);

        // div2 with class "x" and child text "hello"
        let div2 = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("id".to_string(), "div2".to_string()),
                ("class".to_string(), "x".to_string()),
            ],
        });
        let text2 = dom.create_node(NodeData::Text("hello".to_string()));
        dom.append_child(div2, text2);
        dom.append_child(document, div2);

        // div3 with class "y" (different attribute) and child text "hello"
        let div3 = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("id".to_string(), "div3".to_string()),
                ("class".to_string(), "y".to_string()),
            ],
        });
        let text3 = dom.create_node(NodeData::Text("hello".to_string()));
        dom.append_child(div3, text3);
        dom.append_child(document, div3);

        // div4 with class "x" but child text "world" (different text)
        let div4 = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("id".to_string(), "div4".to_string()),
                ("class".to_string(), "x".to_string()),
            ],
        });
        let text4 = dom.create_node(NodeData::Text("world".to_string()));
        dom.append_child(div4, text4);
        dom.append_child(document, div4);

        // span1 with class "x" and child text "hello" (different tagName)
        let span1 = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![
                ("id".to_string(), "span1".to_string()),
                ("class".to_string(), "x".to_string()),
            ],
        });
        let text_span = dom.create_node(NodeData::Text("hello".to_string()));
        dom.append_child(span1, text_span);
        dom.append_child(document, span1);

        let mut host = BoaHost::new();

        // 1. isSameNode assertions
        let res_same_self = host.eval_with_dom(
            "document.getElementById('div1').isSameNode(document.getElementById('div1'))",
            &mut dom,
        );
        assert_eq!(res_same_self, Ok("true".to_string()));

        let res_same_other = host.eval_with_dom(
            "document.getElementById('div1').isSameNode(document.getElementById('div2'))",
            &mut dom,
        );
        assert_eq!(res_same_other, Ok("false".to_string()));

        let res_same_null =
            host.eval_with_dom("document.getElementById('div1').isSameNode(null)", &mut dom);
        assert_eq!(res_same_null, Ok("false".to_string()));

        let res_doc_same_self = host.eval_with_dom("document.isSameNode(document)", &mut dom);
        assert_eq!(res_doc_same_self, Ok("true".to_string()));

        let res_doc_same_other = host.eval_with_dom(
            "document.isSameNode(document.getElementById('div1'))",
            &mut dom,
        );
        assert_eq!(res_doc_same_other, Ok("false".to_string()));

        // 2. isEqualNode assertions
        // Same structural content but separate elements:
        // Wait, div1 and div2 have different "id" attributes (div1 vs div2),
        // so they are NOT equal as elements if we compare all attributes!
        // To make a perfect comparison, let's create two elements dynamically inside JS
        // that have exactly the same attributes and structure, OR compare child nodes directly.
        // Let's do both! First, evaluate some JS code that creates nodes:
        let res_js_equal = host.eval_with_dom(
            "const d1 = document.createElement('div'); d1.setAttribute('class', 'x'); d1.appendChild(document.createTextNode('hello')); const d2 = document.createElement('div'); d2.setAttribute('class', 'x'); d2.appendChild(document.createTextNode('hello')); d1.isEqualNode(d2)",
            &mut dom,
        );
        assert_eq!(res_js_equal, Ok("true".to_string()));

        // Now test differing attributes
        let res_js_diff_attr = host.eval_with_dom(
            "const d3 = document.createElement('div'); d3.setAttribute('class', 'x'); const d4 = document.createElement('div'); d4.setAttribute('class', 'y'); d3.isEqualNode(d4)",
            &mut dom,
        );
        assert_eq!(res_js_diff_attr, Ok("false".to_string()));

        // Test differing children length/structure
        let res_js_diff_children = host.eval_with_dom(
            "const d5 = document.createElement('div'); d5.appendChild(document.createTextNode('hello')); const d6 = document.createElement('div'); d5.isEqualNode(d6)",
            &mut dom,
        );
        assert_eq!(res_js_diff_children, Ok("false".to_string()));

        // Test null/undefined
        let res_js_null = host.eval_with_dom(
            "document.getElementById('div1').isEqualNode(null)",
            &mut dom,
        );
        assert_eq!(res_js_null, Ok("false".to_string()));

        // Test document isEqualNode
        let res_doc_equal_self = host.eval_with_dom("document.isEqualNode(document)", &mut dom);
        assert_eq!(res_doc_equal_self, Ok("true".to_string()));
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
    fn test_dom_node_is_connected() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = r#"
            const div = document.createElement('div');
            const detachedIsConnected = div.isConnected;
            
            document.appendChild(div);
            const attachedIsConnected = div.isConnected;

            const verification = {
                documentIsConnected: document.isConnected,
                detachedIsConnected: detachedIsConnected,
                attachedIsConnected: attachedIsConnected
            };
            JSON.stringify(verification);
        "#;

        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(
            res,
            Ok(r#"{"documentIsConnected":true,"detachedIsConnected":false,"attachedIsConnected":true}"#.to_string())
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

        // - prototype and instance checks (t0468)
        let res_proto_matches = host.eval_with_dom("typeof Element.prototype.matches", &mut dom);
        assert_eq!(res_proto_matches, Ok("function".to_string()));

        let res_proto_closest = host.eval_with_dom("typeof Element.prototype.closest", &mut dom);
        assert_eq!(res_proto_closest, Ok("function".to_string()));

        let res_instanceof_element = host.eval_with_dom(
            "document.getElementById('b-span') instanceof Element",
            &mut dom,
        );
        assert_eq!(res_instanceof_element, Ok("true".to_string()));

        let res_instanceof_node = host.eval_with_dom(
            "document.getElementById('b-span') instanceof Node",
            &mut dom,
        );
        assert_eq!(res_instanceof_node, Ok("true".to_string()));

        let res_instanceof_event_target = host.eval_with_dom(
            "document.getElementById('b-span') instanceof EventTarget",
            &mut dom,
        );
        assert_eq!(res_instanceof_event_target, Ok("true".to_string()));

        // - invalid selector string should not panic or throw, behaves like querySelector
        let res_invalid_matches = host.eval_with_dom(
            "document.getElementById('b-span').matches('div > > p')",
            &mut dom,
        );
        assert_eq!(res_invalid_matches, Ok("false".to_string()));

        let res_invalid_closest = host.eval_with_dom(
            "document.getElementById('b-span').closest('div > > p')",
            &mut dom,
        );
        assert_eq!(res_invalid_closest, Ok("null".to_string()));
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
    fn test_eval_with_dom_get_elements_by_name() {
        let mut dom = Dom::new();
        let document = dom.document();

        let input1_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![
                ("name".to_string(), "q".to_string()),
                ("value".to_string(), "first".to_string()),
            ],
        });
        let input2_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![
                ("name".to_string(), "q".to_string()),
                ("value".to_string(), "second".to_string()),
            ],
        });
        let input3_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![
                ("name".to_string(), "btn".to_string()),
                ("value".to_string(), "click".to_string()),
            ],
        });
        let input4_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![
                ("name".to_string(), "foo\"bar\\baz".to_string()),
                ("value".to_string(), "escaped".to_string()),
            ],
        });
        dom.append_child(document, input1_id);
        dom.append_child(document, input2_id);
        dom.append_child(document, input3_id);
        dom.append_child(document, input4_id);

        let mut host = BoaHost::new();

        // Exact name matching with multiple matches
        let res_q_len = host.eval_with_dom("document.getElementsByName('q').length", &mut dom);
        assert_eq!(res_q_len, Ok("2".to_string()));

        // Check attributes/values of retrieved elements
        let res_q_val1 = host.eval_with_dom(
            "document.getElementsByName('q')[0].getAttribute('value')",
            &mut dom,
        );
        assert_eq!(res_q_val1, Ok("first".to_string()));
        let res_q_val2 = host.eval_with_dom(
            "document.getElementsByName('q')[1].getAttribute('value')",
            &mut dom,
        );
        assert_eq!(res_q_val2, Ok("second".to_string()));

        // Single match
        let res_btn_len = host.eval_with_dom("document.getElementsByName('btn').length", &mut dom);
        assert_eq!(res_btn_len, Ok("1".to_string()));

        // Escaped name test: foo"bar\baz
        // Note: JS string literal 'foo"bar\\\\baz' corresponds to string 'foo"bar\\baz'
        let res_escaped_len = host.eval_with_dom(
            "document.getElementsByName('foo\"bar\\\\baz').length",
            &mut dom,
        );
        assert_eq!(res_escaped_len, Ok("1".to_string()));

        // Non-existent name
        let res_nonexistent =
            host.eval_with_dom("document.getElementsByName('notfound').length", &mut dom);
        assert_eq!(res_nonexistent, Ok("0".to_string()));

        // Empty string
        let res_empty = host.eval_with_dom("document.getElementsByName('').length", &mut dom);
        assert_eq!(res_empty, Ok("0".to_string()));
    }

    #[test]
    fn test_htmlcollection_methods() {
        let mut dom = Dom::new();
        let document = dom.document();

        let div1_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("id".to_string(), "first_id".to_string()),
                ("name".to_string(), "name_one".to_string()),
                ("class".to_string(), "my-class".to_string()),
            ],
        });
        let div2_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("id".to_string(), "second_id".to_string()),
                ("name".to_string(), "name_two".to_string()),
                ("class".to_string(), "my-class".to_string()),
            ],
        });
        dom.append_child(document, div1_id);
        dom.append_child(document, div2_id);

        let mut host = BoaHost::new();

        // 1. Existing index access coll[i] and coll.length still work
        assert_eq!(
            host.eval_with_dom("document.getElementsByTagName('div').length", &mut dom),
            Ok("2".to_string())
        );
        assert_eq!(
            host.eval_with_dom("document.getElementsByTagName('div')[0].id", &mut dom),
            Ok("first_id".to_string())
        );
        assert_eq!(
            host.eval_with_dom(
                "document.getElementsByClassName('my-class').length",
                &mut dom
            ),
            Ok("2".to_string())
        );

        // 2. item(index) returns the same element as coll[0], and item(999) returns null
        assert_eq!(
            host.eval_with_dom("document.getElementsByTagName('div').item(0).id", &mut dom),
            Ok("first_id".to_string())
        );
        assert_eq!(
            host.eval_with_dom("document.getElementsByTagName('div').item(1).id", &mut dom),
            Ok("second_id".to_string())
        );
        assert_eq!(
            host.eval_with_dom("document.getElementsByTagName('div').item(999)", &mut dom),
            Ok("null".to_string())
        );
        // item parameter coercion check: item("1") should resolve to index 1
        assert_eq!(
            host.eval_with_dom(
                "document.getElementsByTagName('div').item('1').id",
                &mut dom
            ),
            Ok("second_id".to_string())
        );

        // 3. namedItem(name) finds by id
        assert_eq!(
            host.eval_with_dom(
                "document.getElementsByTagName('div').namedItem('first_id').id",
                &mut dom
            ),
            Ok("first_id".to_string())
        );
        assert_eq!(
            host.eval_with_dom(
                "document.getElementsByClassName('my-class').namedItem('second_id').id",
                &mut dom
            ),
            Ok("second_id".to_string())
        );

        // 4. namedItem(name) finds by name attribute when no matching id
        assert_eq!(
            host.eval_with_dom(
                "document.getElementsByTagName('div').namedItem('name_two').id",
                &mut dom
            ),
            Ok("second_id".to_string())
        );

        // 5. namedItem(nope) returns null
        assert_eq!(
            host.eval_with_dom(
                "document.getElementsByTagName('div').namedItem('nope')",
                &mut dom
            ),
            Ok("null".to_string())
        );
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

            // 1. before: insert multiple nodes and strings before refNode
            let beforeNode1 = document.createElement('span');
            beforeNode1.textContent = 'before1';
            let beforeNode2 = document.createElement('span');
            beforeNode2.textContent = 'before2';
            refNode.before(beforeNode1, 'before_text', beforeNode2);

            // 2. after: insert multiple nodes and strings after refNode
            let afterNode1 = document.createElement('span');
            afterNode1.textContent = 'after1';
            let afterNode2 = document.createElement('span');
            afterNode2.textContent = 'after2';
            refNode.after(afterNode1, 'after_text', afterNode2);

            // 3. before / after on a node with null parentNode is a no-op (should not throw)
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

        // Order should be: before1, before_text, before2, ref, after1, after_text, after2
        assert_eq!(parent_children.len(), 7);
        assert_eq!(dom.text_content(parent_children[0]), "before1");
        assert_eq!(dom.text_content(parent_children[1]), "before_text");
        assert_eq!(dom.text_content(parent_children[2]), "before2");
        assert_eq!(dom.text_content(parent_children[3]), "ref");
        assert_eq!(dom.text_content(parent_children[4]), "after1");
        assert_eq!(dom.text_content(parent_children[5]), "after_text");
        assert_eq!(dom.text_content(parent_children[6]), "after2");
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

            // Test no-args calling (should be no-op)
            parent.append();
            parent.prepend();

            // Test non-string raw value conversions
            parent.append(true);
            parent.append(123);
        ";
        assert!(host.eval_with_dom(script, &mut dom).is_ok());

        // Verify the DOM structure from the Rust side
        let doc_children = dom.children(dom.document());
        assert_eq!(doc_children.len(), 1);
        let parent_id = doc_children[0];
        let parent_children = dom.children(parent_id);

        assert_eq!(parent_children.len(), 9);
        assert_eq!(dom.text_content(parent_children[0]), "x");
        assert_eq!(dom.text_content(parent_children[1]), "y");
        assert_eq!(dom.text_content(parent_children[2]), "z");
        assert_eq!(dom.text_content(parent_children[3]), "a");
        assert_eq!(dom.text_content(parent_children[4]), "b");
        assert_eq!(dom.text_content(parent_children[5]), "c");
        assert_eq!(dom.text_content(parent_children[6]), "hi");
        assert_eq!(dom.text_content(parent_children[7]), "true");
        assert_eq!(dom.text_content(parent_children[8]), "123");
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

            let x1 = document.createElement('span');
            x1.textContent = 'x1';
            let x2 = document.createElement('span');
            x2.textContent = 'x2';

            // Replaces b with x1, a text node, and x2:
            b.replaceWith(x1, 'replaced_text', x2);

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

        // Order should be exactly: a, x1, replaced_text, x2, c
        assert_eq!(parent_children.len(), 5);
        assert_eq!(dom.text_content(parent_children[0]), "a");
        assert_eq!(dom.text_content(parent_children[1]), "x1");
        assert_eq!(dom.text_content(parent_children[2]), "replaced_text");
        assert_eq!(dom.text_content(parent_children[3]), "x2");
        assert_eq!(dom.text_content(parent_children[4]), "c");
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
    fn test_dom_write_create_comment() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            let div = document.createElement('div');
            let comment = document.createComment('hello comment');
            div.appendChild(comment);
            document.appendChild(div);
            div.outerHTML;
        ";
        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(res, Ok("<div><!--hello comment--></div>".to_string()));

        // Verify backing Node type and content
        let root_children = dom.children(dom.document());
        assert_eq!(root_children.len(), 1);
        let div_id = root_children[0];
        let div_children = dom.children(div_id);
        assert_eq!(div_children.len(), 1);
        let comment_id = div_children[0];
        match dom.data(comment_id) {
            Some(NodeData::Comment(content)) => assert_eq!(content, "hello comment"),
            _ => panic!("Expected Comment node"),
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
    fn test_element_has_attributes() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            const div = document.createElement('div');
            div.setAttribute('id', 'x');
            div.setAttribute('class', 'y');
            document.appendChild(div);

            const r1 = document.getElementById('x').hasAttributes() === true;

            const n = document.createElement('span');
            const r2 = n.hasAttributes() === false;

            const m = document.createElement('p');
            const before = m.hasAttributes() === false;
            m.setAttribute('data-k', 'v');
            const after = m.hasAttributes() === true;
            const r3 = before && after;

            [r1, r2, r3].join(',');
        ";
        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(res, Ok("true,true,true".to_string()));
    }

    #[test]
    fn test_dom_node_value() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let script = "
            const results = [];

            // 1. document.nodeValue is null and setter is a no-op
            results.push(document.nodeValue === null);
            document.nodeValue = 'some value';
            results.push(document.nodeValue === null);

            // 2. Element.nodeValue is null and setter is a no-op
            const div = document.createElement('div');
            results.push(div.nodeValue === null);
            div.nodeValue = 'hello element';
            results.push(div.nodeValue === null);

            // 3. TextNode.nodeValue getter & setter
            const text = document.createTextNode('hello text');
            results.push(text.nodeValue === 'hello text');
            text.nodeValue = 'new text';
            results.push(text.nodeValue === 'new text');

            // 4. CommentNode.nodeValue getter & setter
            const comment = document.createComment('hello comment');
            results.push(comment.nodeValue === 'hello comment');
            comment.nodeValue = 'new comment';
            results.push(comment.nodeValue === 'new comment');

            // 5. null assignment to nodeValue behaves as empty string for Text/Comment
            text.nodeValue = null;
            results.push(text.nodeValue === '');
            comment.nodeValue = null;
            results.push(comment.nodeValue === '');

            // 6. textContent on comment and text node delegates to nodeValue
            text.nodeValue = 'text content test';
            results.push(text.textContent === 'text content test');
            text.textContent = 'new text content';
            results.push(text.nodeValue === 'new text content');

            comment.nodeValue = 'comment content test';
            results.push(comment.textContent === 'comment content test');
            comment.textContent = 'new comment content';
            results.push(comment.nodeValue === 'new comment content');

            results.join(',');
        ";
        let res = host.eval_with_dom(script, &mut dom);
        assert_eq!(
            res,
            Ok("true,true,true,true,true,true,true,true,true,true,true,true,true,true".to_string())
        );
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
    fn test_external_data_url_script_runs() {
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
            attrs: vec![(
                "src".to_string(),
                "data:text/javascript,document.getElementById('x').textContent='external_data'"
                    .to_string(),
            )],
        });
        dom.append_child(document, script_id);

        let base_url = crate::url::Url::parse("about:blank").unwrap();
        struct MockDummyLoader;
        impl crate::loader::ResourceLoader for MockDummyLoader {
            fn load(&self, _url: &crate::url::Url) -> Result<Vec<u8>, crate::loader::LoadError> {
                Err(crate::loader::LoadError::NotFound)
            }
        }

        let mutated_dom = run_scripts(
            dom,
            &std::collections::HashMap::new(),
            &base_url,
            &MockDummyLoader,
        );
        assert_eq!(mutated_dom.text_content(element_id), "external_data");
    }

    #[test]
    fn test_external_http_script_via_mock_loader() {
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
            attrs: vec![("src".to_string(), "http://example.test/app.js".to_string())],
        });
        dom.append_child(document, script_id);

        struct MockHttpLoader;
        impl crate::loader::ResourceLoader for MockHttpLoader {
            fn load(&self, url: &crate::url::Url) -> Result<Vec<u8>, crate::loader::LoadError> {
                if url.serialize() == "http://example.test/app.js" {
                    Ok(b"document.getElementById('target').textContent = 'http_loaded'".to_vec())
                } else {
                    Err(crate::loader::LoadError::NotFound)
                }
            }
        }

        let base_url = crate::url::Url::parse("http://example.test/").unwrap();
        let mutated_dom = run_scripts(
            dom,
            &std::collections::HashMap::new(),
            &base_url,
            &MockHttpLoader,
        );
        assert_eq!(mutated_dom.text_content(element_id), "http_loaded");
    }

    #[test]
    fn test_external_script_fetch_failure_is_safe() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("initial".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        // This script will fail to load
        let script_fail = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![("src".to_string(), "http://example.test/fail.js".to_string())],
        });
        dom.append_child(document, script_fail);

        // Later inline script runs successfully
        let script_ok = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        let text_ok = dom.create_node(NodeData::Text(
            "document.getElementById('target').textContent = 'after_failure'".to_string(),
        ));
        dom.append_child(script_ok, text_ok);
        dom.append_child(document, script_ok);

        struct MockFailingLoader;
        impl crate::loader::ResourceLoader for MockFailingLoader {
            fn load(&self, _url: &crate::url::Url) -> Result<Vec<u8>, crate::loader::LoadError> {
                Err(crate::loader::LoadError::NotFound)
            }
        }

        let base_url = crate::url::Url::parse("http://example.test/").unwrap();
        let mutated_dom = run_scripts(
            dom,
            &std::collections::HashMap::new(),
            &base_url,
            &MockFailingLoader,
        );
        assert_eq!(mutated_dom.text_content(element_id), "after_failure");
    }

    #[test]
    fn test_run_scripts_document_order() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "target".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        // 1. External script appends "A"
        let script_ext = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![(
                "src".to_string(),
                "data:text/javascript,document.getElementById('target').textContent += 'A'"
                    .to_string(),
            )],
        });
        dom.append_child(document, script_ext);

        // 2. Inline script appends "B"
        let script_inl = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        let text_inl = dom.create_node(NodeData::Text(
            "document.getElementById('target').textContent += 'B'".to_string(),
        ));
        dom.append_child(script_inl, text_inl);
        dom.append_child(document, script_inl);

        struct MockDummyLoader;
        impl crate::loader::ResourceLoader for MockDummyLoader {
            fn load(&self, _url: &crate::url::Url) -> Result<Vec<u8>, crate::loader::LoadError> {
                Err(crate::loader::LoadError::NotFound)
            }
        }

        let base_url = crate::url::Url::parse("about:blank").unwrap();
        let mutated_dom = run_scripts(
            dom,
            &std::collections::HashMap::new(),
            &base_url,
            &MockDummyLoader,
        );
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

        // This script is 54 characters. With limit of 20, it is skipped entirely.
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

        // A long script that would have been skipped if limits were on
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
    fn test_oversized_script_skipped_entirely() {
        set_limits_enabled(true);
        set_max_script_length(20);

        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let title_before = host.eval_with_dom("document.title", &mut dom).unwrap();
        assert_eq!(title_before, "Underrated");

        // Now run an oversized script (length = 33, which is > max of 20)
        let res = host.eval_with_dom("document.title = 'MutatedTitle';", &mut dom);

        // It must succeed (return Ok) and the title must remain "Underrated"
        assert!(res.is_ok());
        let title_after = host.eval_with_dom("document.title", &mut dom).unwrap();
        assert_eq!(title_after, "Underrated");

        // Restore defaults
        set_max_script_length(5000);
    }

    #[test]
    fn test_under_limit_script_executed_normally() {
        set_limits_enabled(true);
        set_max_script_length(50); // Greater than script size

        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let title_before = host.eval_with_dom("document.title", &mut dom).unwrap();
        assert_eq!(title_before, "Underrated");

        // Run an under-limit script (length = 33, which is <= max of 50)
        let res = host.eval_with_dom("document.title = 'MutatedTitle';", &mut dom);

        // It must succeed and the title must be mutated
        assert!(res.is_ok());
        let title_after = host.eval_with_dom("document.title", &mut dom).unwrap();
        assert_eq!(title_after, "MutatedTitle");

        // Restore defaults
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
    fn test_custom_event_t0529() {
        let mut host = BoaHost::new();
        let mut dom = crate::dom::Dom::new();

        let script = r#"
            if (typeof CustomEvent === "undefined") throw new Error("CustomEvent undefined");

            // No options
            const ev1 = new CustomEvent("test-type");
            if (ev1.type !== "test-type") throw new Error("Expected type test-type, got " + ev1.type);
            if (ev1.detail !== null) throw new Error("Expected detail to default to null, got " + ev1.detail);
            if (ev1.bubbles !== false) throw new Error("Expected bubbles to default to false, got " + ev1.bubbles);
            if (ev1.cancelable !== false) throw new Error("Expected cancelable to default to false, got " + ev1.cancelable);

            // With options
            const ev2 = new CustomEvent("custom", {
                detail: { value: "hello", count: 42 },
                bubbles: true,
                cancelable: true
            });
            if (ev2.type !== "custom") throw new Error("Expected type custom, got " + ev2.type);
            if (ev2.bubbles !== true) throw new Error("Expected bubbles to be true, got " + ev2.bubbles);
            if (ev2.cancelable !== true) throw new Error("Expected cancelable to be true, got " + ev2.cancelable);
            if (ev2.detail === null) throw new Error("Expected detail to not be null");
            if (ev2.detail.value !== "hello") throw new Error("Expected detail.value to be 'hello', got " + ev2.detail.value);
            if (ev2.detail.count !== 42) throw new Error("Expected detail.count to be 42, got " + ev2.detail.count);

            // Dispatch and listen on standard EventTarget
            const target = new EventTarget();
            let observed = null;
            target.addEventListener("custom", (e) => {
                observed = e;
                if (e.target !== target) throw new Error("e.target is not target! got: " + e.target + ", expected: " + target);
                if (e.currentTarget !== target) throw new Error("e.currentTarget is not target! got: " + e.currentTarget + ", expected: " + target);
            });

            target.dispatchEvent(ev2);

            if (observed === null) throw new Error("Event listener was not invoked");
            if (observed.type !== "custom") throw new Error("Observed incorrect type: " + observed.type);
            if (observed.detail.value !== "hello") throw new Error("Observed incorrect detail.value");
            if (observed.currentTarget !== null) throw new Error("Expected currentTarget to be null after dispatching, got: " + observed.currentTarget);
        "#;
        host.eval_with_dom(script, &mut dom).unwrap();
    }

    #[test]
    fn test_blob_t0534() {
        let mut host = BoaHost::new();
        let mut dom = crate::dom::Dom::new();

        let script = r#"
             if (typeof Blob === "undefined") throw new Error("Blob undefined");

             // 1. Default constructor
             const b1 = new Blob();
             if (b1.size !== 0) throw new Error("Expected b1.size to be 0, got " + b1.size);
             if (b1.type !== "") throw new Error("Expected b1.type to be empty string, got " + b1.type);
             if (b1.text() !== "") throw new Error("Expected b1.text() to be empty, got " + b1.text());

             // 2. ASCII and multi-byte UTF-8 parts
             const b2 = new Blob(["hello", "こんにちは"]);
             // "hello" is 5 bytes, "こんにちは" is 15 bytes. Total size = 20.
             if (b2.size !== 20) throw new Error("Expected b2.size to be 20, got " + b2.size);
             if (b2.text() !== "helloこんにちは") throw new Error("Expected b2.text() correct, got " + b2.text());

             // 3. MIME type defaulting and lowercase and invalid char filter
             const b3 = new Blob(["abc"], { type: "TEXT/html" });
             if (b3.type !== "text/html") throw new Error("Expected b3.type to be 'text/html', got " + b3.type);

             const b4 = new Blob(["abc"], { type: "text/html\x00" });
             if (b4.type !== "") throw new Error("Expected b4.type to be empty due to out of range char");

             const b5 = new Blob(["abc"], { type: "text/html\u0100" });
             if (b5.type !== "") throw new Error("Expected b5.type to be empty due to non-ASCII char");

             // 4. Nested Blob parts
             const nested = new Blob([b2, " world"]);
             if (nested.size !== 26) throw new Error("Expected nested.size to be 26, got " + nested.size);
             if (nested.text() !== "helloこんにちは world") throw new Error("Expected nested.text() correct, got " + nested.text());

             // 5. Read-only properties are read-only
             let size_val = b2.size;
             try {
                 b2.size = 999;
             } catch(e) {}
             if (b2.size !== size_val) throw new Error("b2.size should be read-only");

             let type_val = b3.type;
             try {
                 b3.type = "invalid";
             } catch(e) {}
             if (b3.type !== type_val) throw new Error("b3.type should be read-only");
        "#;
        host.eval_with_dom(script, &mut dom).unwrap();
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
    fn test_location_navigation() {
        let mut host = BoaHost::new();
        host.set_document_url("https://example.com/initial");

        // 1. With NO assignment, take_pending_navigation() returns None
        assert_eq!(host.take_pending_navigation(), None);

        // 2. Assigning window.location.href
        assert!(
            host.eval("window.location.href = 'https://example.com/next'")
                .is_ok()
        );
        assert_eq!(
            host.take_pending_navigation().as_deref(),
            Some("https://example.com/next")
        );

        // 3. A second call to take_pending_navigation() returns None (slot is cleared after taking)
        assert_eq!(host.take_pending_navigation(), None);

        // 4. location.assign('/foo')
        assert!(host.eval("location.assign('/foo')").is_ok());
        assert_eq!(host.take_pending_navigation().as_deref(), Some("/foo"));

        // 5. location.replace('/bar')
        assert!(host.eval("location.replace('/bar')").is_ok());
        assert_eq!(host.take_pending_navigation().as_deref(), Some("/bar"));

        // 6. location.reload() records current document location href
        assert!(host.eval("location.reload()").is_ok());
        assert_eq!(
            host.take_pending_navigation().as_deref(),
            Some("https://example.com/initial")
        );

        // 7. location getters still work (href/pathname unchanged) — do not regress existing test_location_initialized
        assert!(
            host.eval(
                "if (window.location.href !== 'https://example.com/initial') throw 'href mismatch';"
            )
            .is_ok()
        );
        assert!(
            host.eval("if (window.location.pathname !== '/initial') throw 'pathname mismatch';")
                .is_ok()
        );
    }

    #[test]
    fn test_history_api_basic() {
        let mut host = BoaHost::new();
        host.set_document_url("https://example.com/home");

        // Verify initial state
        assert!(
            host.eval("if (window.history.length !== 1) throw 'initial length mismatch';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.history.state !== null) throw 'initial state mismatch';")
                .is_ok()
        );

        // Test pushState with state and relative URL
        assert!(
            host.eval("window.history.pushState({a: 1}, 'title 1', '/foo')")
                .is_ok()
        );
        assert!(
            host.eval("if (window.history.length !== 2) throw 'length mismatch after push';")
                .is_ok()
        );
        assert!(
            host.eval("if (window.history.state.a !== 1) throw 'state value mismatch after push';")
                .is_ok()
        );
        assert!(
            host.eval(
                "if (window.location.pathname !== '/foo') throw 'pathname mismatch after push';"
            )
            .is_ok()
        );
        assert!(host.eval("if (window.location.href !== 'https://example.com/foo') throw 'href mismatch after push';").is_ok());

        // Test replaceState
        assert!(
            host.eval("window.history.replaceState({b: 2}, 'title 2', '/bar')")
                .is_ok()
        );
        assert!(
            host.eval(
                "if (window.history.length !== 2) throw 'length should not change after replace';"
            )
            .is_ok()
        );
        assert!(
            host.eval(
                "if (window.history.state.b !== 2) throw 'state value mismatch after replace';"
            )
            .is_ok()
        );
        assert!(host.eval("if (window.history.state.a !== undefined) throw 'old state should be gone after replace';").is_ok());
        assert!(
            host.eval(
                "if (window.location.pathname !== '/bar') throw 'pathname mismatch after replace';"
            )
            .is_ok()
        );

        // Push another one to test back and forward
        assert!(
            host.eval("window.history.pushState({c: 3}, 'title 3', 'baz')")
                .is_ok()
        );
        assert!(
            host.eval(
                "if (window.history.length !== 3) throw 'length mismatch after second push';"
            )
            .is_ok()
        );
        assert!(host.eval("if (window.location.pathname !== '/baz') throw 'pathname mismatch after second push';").is_ok());

        // Set up event listener for popstate
        assert!(
            host.eval(
                "
            window.popstateLogs = [];
            window.addEventListener('popstate', (e) => {
                window.popstateLogs.push(e.state);
            });
        "
            )
            .is_ok()
        );

        // Go back - should go from index 2 to index 1 (state {b: 2})
        assert!(host.eval("window.history.back()").is_ok());
        assert!(
            host.eval("if (window.history.state.b !== 2) throw 'state mismatch after back';")
                .is_ok()
        );
        assert!(
            host.eval(
                "if (window.location.pathname !== '/bar') throw 'pathname mismatch after back';"
            )
            .is_ok()
        );

        // Go back again - should go from index 1 to index 0 (state null)
        assert!(host.eval("window.history.back()").is_ok());
        assert!(
            host.eval(
                "if (window.history.state !== null) throw 'state mismatch after second back';"
            )
            .is_ok()
        );
        assert!(host.eval("if (window.location.pathname !== '/home') throw 'pathname mismatch after second back';").is_ok());

        // Go forward - should go from index 0 to index 1
        assert!(host.eval("window.history.forward()").is_ok());
        assert!(
            host.eval("if (window.history.state.b !== 2) throw 'state mismatch after forward';")
                .is_ok()
        );

        // Go with delta - should go from index 1 to index 2
        assert!(host.eval("window.history.go(1)").is_ok());
        assert!(
            host.eval("if (window.history.state.c !== 3) throw 'state mismatch after go';")
                .is_ok()
        );

        // Verify popstate logs
        // We did back (to {b:2}), back (to null), forward (to {b:2}), go(1) (to {c:3})
        // So popstateLogs should have: [{b:2}, null, {b:2}, {c:3}]
        assert!(
            host.eval(
                "
            if (window.popstateLogs.length !== 4) throw 'popstate event count mismatch';
            if (window.popstateLogs[0].b !== 2) throw 'first popstate mismatch';
            if (window.popstateLogs[1] !== null) throw 'second popstate mismatch';
            if (window.popstateLogs[2].b !== 2) throw 'third popstate mismatch';
            if (window.popstateLogs[3].c !== 3) throw 'fourth popstate mismatch';
        "
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
    fn test_lifecycle_window_onload_property() {
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
        // Register a window onload property during script execution
        let script_text = dom.create_node(NodeData::Text(
            r#"
            window.onload = () => {
                document.getElementById('target').textContent = 'windowloaded_via_onload';
            };
            "#
            .to_string(),
        ));
        dom.append_child(script_id, script_text);
        dom.append_child(document, script_id);

        let mutated_dom = run_inline_scripts(dom, &std::collections::HashMap::new());
        assert_eq!(
            mutated_dom.text_content(element_id),
            "windowloaded_via_onload"
        );
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

    #[test]
    fn test_element_insert_adjacent_html_and_element() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let setup_script = r#"
            let container = document.createElement('div');
            container.innerHTML = "<ul id='L'><li id='a'>a</li></ul>";
            document.appendChild(container);
            
            let L = document.getElementById('L');
            let a = document.getElementById('a');
            
            // 1. insertAdjacentHTML 'beforeend'
            L.insertAdjacentHTML('beforeend', '<li>b</li>');
            
            // 2. insertAdjacentHTML 'afterbegin'
            L.insertAdjacentHTML('afterbegin', '<li>z</li>');
            
            // 3. insertAdjacentHTML 'beforebegin' on 'a'
            a.insertAdjacentHTML('beforebegin', '<li>pre</li>');
            
            // 4. insertAdjacentHTML 'afterend' on 'a'
            a.insertAdjacentHTML('afterend', '<li>post</li>');
            
            // 5. Case insensitivity and whitespace trimming
            L.insertAdjacentHTML('  BeFoReEnD  ', '<li>case</li>');
            
            let html1 = L.innerHTML;
            
            // 6. insertAdjacentElement
            let newItem = document.createElement('li');
            newItem.textContent = 'new';
            let returnedItem = L.insertAdjacentElement('beforeend', newItem);
            
            // Check identity
            let isSame = (returnedItem === newItem);
            let html2 = L.innerHTML;
            
            // 7. Invalid/edge cases
            let parentless = document.createElement('div');
            let r1 = parentless.insertAdjacentElement('beforebegin', document.createElement('p'));
            let r2 = parentless.insertAdjacentElement('afterend', document.createElement('p'));
            let r3 = L.insertAdjacentElement('nope', document.createElement('p'));
            L.insertAdjacentHTML('nope', '<li>invalid</li>');
            let html3 = L.innerHTML;
            
            [html1, isSame, html2, String(r1), String(r2), String(r3), html3].join('|');
        "#;

        let res = host.eval_with_dom(setup_script, &mut dom).unwrap();
        // Check everything in a single formatted assertion
        assert_eq!(
            res,
            "<li>z</li><li>pre</li><li id=\"a\">a</li><li>post</li><li>b</li><li>case</li>|true|<li>z</li><li>pre</li><li id=\"a\">a</li><li>post</li><li>b</li><li>case</li><li>new</li>|null|null|null|<li>z</li><li>pre</li><li id=\"a\">a</li><li>post</li><li>b</li><li>case</li><li>new</li>"
        );
    }

    #[test]
    fn test_element_insert_adjacent_text() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let setup_script = r#"
            let container = document.createElement('div');
            container.innerHTML = "<div id='L'><span id='a'>a</span></div>";
            document.appendChild(container);

            let L = document.getElementById('L');
            let a = document.getElementById('a');

            // 1. insertAdjacentText 'beforeend'
            L.insertAdjacentText('beforeend', 'b');

            // 2. insertAdjacentText 'afterbegin'
            L.insertAdjacentText('afterbegin', 'z');

            // 3. insertAdjacentText 'beforebegin' on 'a'
            a.insertAdjacentText('beforebegin', 'pre');

            // 4. insertAdjacentText 'afterend' on 'a'
            a.insertAdjacentText('afterend', 'post');

            // 5. Case insensitivity and whitespace trimming
            L.insertAdjacentText('  BeFoReEnD  ', 'case');

            let textContent1 = L.textContent;

            // 6. Test invalid position throwing SyntaxError DOMException
            let threwSyntaxError = false;
            try {
                L.insertAdjacentText('nope', 'invalid');
            } catch (e) {
                if (e instanceof DOMException && e.name === 'SyntaxError') {
                    threwSyntaxError = true;
                }
            }

            // 7. Test literal markup insertion
            a.insertAdjacentText('beforeend', '<b>markup</b>');
            let innerHTML1 = L.innerHTML;

            [textContent1, String(threwSyntaxError), innerHTML1].join('|');
        "#;

        let res = host.eval_with_dom(setup_script, &mut dom).unwrap();
        assert_eq!(
            res,
            "zpreapostbcase|true|zpre<span id=\"a\">a&lt;b&gt;markup&lt;/b&gt;</span>postbcase"
        );
    }

    #[test]
    fn test_node_normalize() {
        let mut dom = Dom::new();
        let mut host = BoaHost::new();

        let setup_script = r#"
            let container = document.createElement('div');
            document.appendChild(container);

            // 1. Test basic adjacent text node merging
            let t1 = document.createTextNode('hello ');
            let t2 = document.createTextNode('world');
            container.appendChild(t1);
            container.appendChild(t2);

            let initial_len = container.childNodes.length; // 2
            container.normalize();
            let after_len = container.childNodes.length; // 1
            let merged_text = container.childNodes[0].textContent; // "hello world"

            // 2. Test empty text node removal
            let empty_container = document.createElement('div');
            document.appendChild(empty_container);
            let t_empty = document.createTextNode('');
            empty_container.appendChild(t_empty);
            let initial_empty_len = empty_container.childNodes.length; // 1
            empty_container.normalize();
            let after_empty_len = empty_container.childNodes.length; // 0

            // 3. Test nested elements recursion
            let parent = document.createElement('div');
            let child_elem = document.createElement('p');
            parent.appendChild(child_elem);
            let child_t1 = document.createTextNode('foo ');
            let child_t2 = document.createTextNode('bar');
            child_elem.appendChild(child_t1);
            child_elem.appendChild(child_t2);

            parent.normalize(); // Normalize on parent should recurse to child_elem
            let child_after_len = child_elem.childNodes.length; // 1
            let child_merged_text = child_elem.childNodes[0].textContent; // "foo bar"

            // 4. Calling normalize on a text node should not throw
            let standalone_text = document.createTextNode('abc');
            let standalone_ok = true;
            try {
                standalone_text.normalize();
            } catch (e) {
                standalone_ok = false;
            }

            // 5. Exposing on document object
            let doc_normalize_exists = typeof document.normalize === 'function';

            [
                initial_len,
                after_len,
                merged_text,
                initial_empty_len,
                after_empty_len,
                child_after_len,
                child_merged_text,
                standalone_ok,
                doc_normalize_exists
            ].join('|');
        "#;

        let res = host.eval_with_dom(setup_script, &mut dom).unwrap();
        assert_eq!(res, "2|1|hello world|1|0|1|foo bar|true|true");
    }

    // Guards Element.getBoundingClientRect() DOM-to-JS script layer bindings wiring.
    #[test]
    fn test_element_get_bounding_client_rect() {
        let mut dom = Dom::new();
        let document = dom.document();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "rect-div".to_string())],
        });
        dom.append_child(document, div_id);

        let mut host = BoaHost::new();

        let script = r#"
            const el = document.getElementById('rect-div');
            const rect = el.getBoundingClientRect();
            
            const isObject = typeof rect === 'object' && rect !== null;
            const x = rect.x;
            const y = rect.y;
            const width = rect.width;
            const height = rect.height;
            const top = rect.top;
            const right = rect.right;
            const bottom = rect.bottom;
            const left = rect.left;

            // Non-elements should return null
            const textNode = document.createTextNode('hello');
            const textRect = textNode.getBoundingClientRect();

            [
                isObject,
                x === 0,
                y === 0,
                width === 0,
                height === 0,
                top === 0,
                right === 0,
                bottom === 0,
                left === 0,
                textRect === null
            ].join('|');
        "#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(res, "true|true|true|true|true|true|true|true|true|true");
    }

    // Guards Element.getClientRects() DOM-to-JS script layer bindings wiring.
    #[test]
    fn test_element_get_client_rects() {
        let mut dom = Dom::new();
        let document = dom.document();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "rects-div".to_string())],
        });
        dom.append_child(document, div_id);

        let mut host = BoaHost::new();

        let script = r#"
            const el = document.getElementById('rects-div');
            el.getBoundingClientRect = () => ({ width: 100, height: 50 });

            const rects = el.getClientRects();
            const lengthOk = rects.length === 1;
            const widthMatches = rects[0].width === el.getBoundingClientRect().width;
            
            const item0 = rects.item(0);
            const item0Ok = item0 !== null && item0.width === rects[0].width;
            const item5 = rects.item(5);

            // Check non-enumerable properties on rects
            let enumerableKeys = Object.keys(rects);
            const itemIsNotEnumerable = !enumerableKeys.includes('item');

            // Non-elements should return null
            const textNode = document.createTextNode('hello');
            const textRects = textNode.getClientRects();

            [
                lengthOk,
                widthMatches,
                item0Ok,
                item5 === null,
                itemIsNotEnumerable,
                textRects === null
            ].join('|');
        "#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(res, "true|true|true|true|true|true");
    }

    #[test]
    fn test_element_offset_width_height() {
        let mut dom = Dom::new();
        let document = dom.document();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "offset-div".to_string())],
        });
        dom.append_child(document, div_id);

        let mut host = BoaHost::new();

        let script = r#"
            const el = document.getElementById('offset-div');
            
            // Check original types and rounded default values (original should be 0 because bounding client rect is 0)
            const origWidthType = typeof el.offsetWidth;
            const origHeightType = typeof el.offsetHeight;
            const origTopType = typeof el.offsetTop;
            const origLeftType = typeof el.offsetLeft;

            const origWidth = el.offsetWidth;
            const origHeight = el.offsetHeight;
            const origTop = el.offsetTop;
            const origLeft = el.offsetLeft;

            // Mock getBoundingClientRect to test rounding behavior on elements
            el.getBoundingClientRect = () => ({ width: 100.5, height: 50.1, top: 12.3, left: 34.8 });
            const mockWidth = el.offsetWidth;
            const mockHeight = el.offsetHeight;
            const mockTop = el.offsetTop;
            const mockLeft = el.offsetLeft;

            // Non-element nodeType !== 1 (Text Node)
            const textNode = document.createTextNode('hello');
            const textWidth = textNode.offsetWidth;
            const textHeight = textNode.offsetHeight;
            const textTop = textNode.offsetTop;
            const textLeft = textNode.offsetLeft;

            [
                origWidthType === 'number',
                origHeightType === 'number',
                origTopType === 'number',
                origLeftType === 'number',
                origWidth === 0,
                origHeight === 0,
                origTop === 0,
                origLeft === 0,
                mockWidth === 101,
                mockHeight === 50,
                mockTop === 12,
                mockLeft === 35,
                textWidth === 0,
                textHeight === 0,
                textTop === 0,
                textLeft === 0
            ].join('|');
        "#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(
            res,
            "true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true"
        );
    }

    #[test]
    fn test_element_client_scroll_dimensions() {
        let mut dom = Dom::new();
        let document = dom.document();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "client-scroll-div".to_string())],
        });
        dom.append_child(document, div_id);

        let mut host = BoaHost::new();

        let script = r#"
            const el = document.getElementById('client-scroll-div');
            
            // Check original types and rounded default values (original should be 0 because bounding client rect is 0)
            const origClientWidthType = typeof el.clientWidth;
            const origClientHeightType = typeof el.clientHeight;
            const origScrollWidthType = typeof el.scrollWidth;
            const origScrollHeightType = typeof el.scrollHeight;

            const origClientWidth = el.clientWidth;
            const origClientHeight = el.clientHeight;
            const origScrollWidth = el.scrollWidth;
            const origScrollHeight = el.scrollHeight;

            // Mock getBoundingClientRect to test rounding behavior on elements
            el.getBoundingClientRect = () => ({ width: 120.6, height: 80.2 });
            const mockClientWidth = el.clientWidth;
            const mockClientHeight = el.clientHeight;
            const mockScrollWidth = el.scrollWidth;
            const mockScrollHeight = el.scrollHeight;

            // Non-element nodeType !== 1 (Text Node)
            const textNode = document.createTextNode('hello');
            const textClientWidth = textNode.clientWidth;
            const textClientHeight = textNode.clientHeight;
            const textScrollWidth = textNode.scrollWidth;
            const textScrollHeight = textNode.scrollHeight;

            [
                origClientWidthType === 'number',
                origClientHeightType === 'number',
                origScrollWidthType === 'number',
                origScrollHeightType === 'number',
                origClientWidth === 0,
                origClientHeight === 0,
                origScrollWidth === 0,
                origScrollHeight === 0,
                mockClientWidth === 121,
                mockClientHeight === 80,
                mockScrollWidth === 121,
                mockScrollHeight === 80,
                textClientWidth === 0,
                textClientHeight === 0,
                textScrollWidth === 0,
                textScrollHeight === 0
            ].join('|');
        "#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(
            res,
            "true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true"
        );
    }

    #[test]
    fn test_element_scroll_top_left() {
        let mut dom = Dom::new();
        let document = dom.document();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "scroll-div".to_string())],
        });
        dom.append_child(document, div_id);

        let mut host = BoaHost::new();

        let script = r#"
            const el = document.getElementById('scroll-div');

            // Default values
            const defaultTopType = typeof el.scrollTop;
            const defaultLeftType = typeof el.scrollLeft;
            const defaultTop = el.scrollTop;
            const defaultLeft = el.scrollLeft;

            // Set positive values
            el.scrollTop = 15.5;
            el.scrollLeft = 42.8;
            const topAfterSet = el.scrollTop;
            const leftAfterSet = el.scrollLeft;

            // Set negative values (should clamp to 0)
            el.scrollTop = -10;
            el.scrollLeft = -5.5;
            const topAfterNegative = el.scrollTop;
            const leftAfterNegative = el.scrollLeft;

            // Type coercion (should convert strings to numbers)
            el.scrollTop = "100.2";
            el.scrollLeft = "200.7";
            const topAfterString = el.scrollTop;
            const leftAfterString = el.scrollLeft;

            // Non-element nodeType !== 1 (Text Node)
            const textNode = document.createTextNode('hello');
            const textTopDefault = textNode.scrollTop;
            textNode.scrollTop = 50;
            const textTopAfterSet = textNode.scrollTop;

            [
                defaultTopType === 'number',
                defaultLeftType === 'number',
                defaultTop === 0,
                defaultLeft === 0,
                topAfterSet === 15.5,
                leftAfterSet === 42.8,
                topAfterNegative === 0,
                leftAfterNegative === 0,
                topAfterString === 100.2,
                leftAfterString === 200.7,
                textTopDefault === 0,
                textTopAfterSet === 0
            ].join('|');
        "#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(
            res,
            "true|true|true|true|true|true|true|true|true|true|true|true"
        );
    }

    #[test]
    fn test_element_scroll_into_view() {
        let mut dom = Dom::new();
        let document = dom.document();

        let parent_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "parent-div".to_string())],
        });
        dom.append_child(document, parent_id);

        let child_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "child-div".to_string())],
        });
        dom.append_child(parent_id, child_id);

        let mut host = BoaHost::new();

        let script = r#"
            const parent = document.getElementById('parent-div');
            const child = document.getElementById('child-div');

            const hasScrollIntoView = typeof child.scrollIntoView === 'function';

            // Mock getComputedStyle to treat parent as a scroll container
            window.getComputedStyle = (element) => {
                if (element === parent) {
                    return {
                        getPropertyValue: (prop) => {
                            if (prop === 'overflow' || prop === 'overflow-y') return 'scroll';
                            return '';
                        }
                    };
                }
                return { getPropertyValue: () => '' };
            };

            // Default scrollTop / scrollLeft
            const initialParentScrollTop = parent.scrollTop;
            const initialParentScrollLeft = parent.scrollLeft;

            // Mock getBoundingClientRect on both elements to simulate layout positions
            parent.getBoundingClientRect = () => ({
                x: 0,
                y: 10,
                width: 100,
                height: 100,
                top: 10,
                left: 10,
                bottom: 110,
                right: 110
            });

            child.getBoundingClientRect = () => ({
                x: 0,
                y: 50,
                width: 50,
                height: 50,
                top: 50,
                left: 70,
                bottom: 100,
                right: 120
            });

            // Call scrollIntoView with no arguments
            child.scrollIntoView();

            const parentScrollTopAfterNoArgs = parent.scrollTop;
            const parentScrollLeftAfterNoArgs = parent.scrollLeft;

            // Reset scroll positions
            parent.scrollTop = 0;
            parent.scrollLeft = 0;

            // Call scrollIntoView with boolean argument
            child.scrollIntoView(true);

            const parentScrollTopAfterBool = parent.scrollTop;

            // Reset scroll positions
            parent.scrollTop = 0;
            parent.scrollLeft = 0;

            // Call scrollIntoView with options object
            child.scrollIntoView({ behavior: 'smooth', block: 'start' });

            const parentScrollTopAfterOptions = parent.scrollTop;

            // Call scrollIntoView on non-element (Text node) - should do nothing and not throw
            const textNode = document.createTextNode('hello');
            let textNodeScrollIntoViewOk = true;
            try {
                textNode.scrollIntoView();
            } catch (e) {
                textNodeScrollIntoViewOk = false;
            }

            [
                hasScrollIntoView,
                initialParentScrollTop === 0,
                initialParentScrollLeft === 0,
                parentScrollTopAfterNoArgs === 40,   // parent.scrollTop + (child.top - parent.top) = 0 + (50 - 10)
                parentScrollLeftAfterNoArgs === 60,  // parent.scrollLeft + (child.left - parent.left) = 0 + (70 - 10)
                parentScrollTopAfterBool === 40,
                parentScrollTopAfterOptions === 40,
                textNodeScrollIntoViewOk
            ].join('|');
        "#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(res, "true|true|true|true|true|true|true|true");
    }

    #[test]
    fn test_element_click() {
        let mut dom = Dom::new();
        let document = dom.document();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "clickable-div".to_string())],
        });
        dom.append_child(document, div_id);

        let mut host = BoaHost::new();

        let script = r#"
            let firedCount = 0;
            const el = document.getElementById('clickable-div');
            
            el.addEventListener('click', () => {
                firedCount++;
            });

            // Call click() and ensure listener runs
            el.click();

            // Calling click() on a non-element node should do nothing/not panic
            const textNode = document.createTextNode('hello');
            let textNodeOk = false;
            try {
                textNode.click(); // Should be undefined or do nothing since nodeType !== 1
                textNodeOk = true;
            } catch (e) {
                textNodeOk = false;
            }

            [
                firedCount === 1,
                textNodeOk,
                typeof el.click === 'function',
                typeof textNode.click === 'function'
            ].join('|');
        "#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(res, "true|true|true|true");
    }

    #[test]
    fn test_element_inner_text() {
        let mut dom = Dom::new();
        let document = dom.document();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "inner-text-div".to_string())],
        });
        dom.append_child(document, div_id);

        let mut host = BoaHost::new();

        let script = r#"
            const el = document.getElementById('inner-text-div');
            el.innerHTML = ' \n\t  Hello   <span>  Beautiful  \r\f  World  </span>\n\t ';

            const origInnerText = el.innerText;
            const origType = typeof el.innerText;

            // Set innerText
            el.innerText = 'hello world';
            const afterSetTextContent = el.textContent;
            const afterSetInnerText = el.innerText;

            // Test on Text node
            const textNode = document.createTextNode('  some  text  ');
            const textNodeInnerText = textNode.innerText;

            [
                origInnerText === 'Hello Beautiful World',
                origType === 'string',
                afterSetTextContent === 'hello world',
                afterSetInnerText === 'hello world',
                textNodeInnerText === ''
            ].join('|');
        "#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(res, "true|true|true|true|true");
    }

    #[test]
    fn test_active_element_focus_blur() {
        let mut dom = Dom::new();
        let doc_id = dom.document();

        let html_id = dom.create_node(NodeData::Element {
            name: "html".to_string(),
            attrs: vec![],
        });
        dom.append_child(doc_id, html_id);

        let body_id = dom.create_node(NodeData::Element {
            name: "body".to_string(),
            attrs: vec![],
        });
        dom.append_child(html_id, body_id);

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "parent-div".to_string())],
        });
        dom.append_child(body_id, div_id);

        let input1_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("id".to_string(), "input1".to_string())],
        });
        dom.append_child(div_id, input1_id);

        let input2_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("id".to_string(), "input2".to_string())],
        });
        dom.append_child(div_id, input2_id);

        let mut host = BoaHost::new();

        let script = r#"
            const body = document.body;
            const div = document.getElementById('parent-div');
            const input1 = document.getElementById('input1');
            const input2 = document.getElementById('input2');

            let log = [];
            function record(e) {
                log.push(`${e.type} on ${e.currentTarget.id || e.currentTarget.tagName || 'window/document'} (target: ${e.target.id || e.target.tagName})`);
            }

            const targets = [window, document, div, input1, input2];
            const eventTypes = ['focus', 'focusin', 'blur', 'focusout'];

            for (const t of targets) {
                for (const et of eventTypes) {
                    t.addEventListener(et, record);
                }
            }

            const initialActive = document.activeElement === body;

            log = [];
            input1.focus();
            const focusedOnInput1 = document.activeElement === input1;
            const focus1Log = [...log];

            log = [];
            input1.focus();
            const focus1AgainLogSize = log.length;

            log = [];
            input2.focus();
            const focusedOnInput2 = document.activeElement === input2;
            const focus2Log = [...log];

            log = [];
            input2.blur();
            const revertedToBody = document.activeElement === body;
            const blurLog = [...log];

            log = [];
            let blurAgainOk = false;
            try {
                input2.blur();
                blurAgainOk = true;
            } catch(e) {
                blurAgainOk = false;
            }
            const blurAgainLogSize = log.length;

            [
                initialActive,
                focusedOnInput1,
                focus1Log.join(','),
                focus1AgainLogSize === 0,
                focusedOnInput2,
                focus2Log.join(','),
                revertedToBody,
                blurLog.join(','),
                blurAgainOk,
                blurAgainLogSize === 0
            ].join('|');
        "#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        let parts: Vec<&str> = res.split('|').collect();

        assert_eq!(parts[0], "true", "Initial activeElement must be body");
        assert_eq!(parts[1], "true", "ActiveElement must update to input1");
        assert!(
            parts[2].contains("focus on input1"),
            "focus event must be dispatched on input1"
        );
        assert!(
            parts[2].contains("focusin on parent-div"),
            "focusin event must bubble to parent-div"
        );
        assert!(
            !parts[2].contains("focus on parent-div"),
            "focus event must NOT bubble"
        );

        assert_eq!(
            parts[3], "true",
            "Focusing already-focused element must be a no-op"
        );
        assert_eq!(parts[4], "true", "ActiveElement must update to input2");

        assert!(
            parts[5].contains("blur on input1"),
            "blur must be dispatched on input1"
        );
        assert!(
            parts[5].contains("focusout on parent-div"),
            "focusout must bubble to parent-div"
        );
        assert!(
            parts[5].contains("focus on input2"),
            "focus must be dispatched on input2"
        );
        assert!(
            parts[5].contains("focusin on parent-div"),
            "focusin must bubble to parent-div"
        );

        let blur_idx = parts[5].find("blur on input1").unwrap();
        let focus_idx = parts[5].find("focus on input2").unwrap();
        assert!(
            blur_idx < focus_idx,
            "Previously focused element must be blurred before new element is focused"
        );

        assert_eq!(
            parts[6], "true",
            "Blurring activeElement must revert activeElement to body"
        );
        assert!(
            parts[7].contains("blur on input2"),
            "blur must be dispatched on input2"
        );
        assert!(
            parts[7].contains("focusout on parent-div"),
            "focusout must bubble to parent-div"
        );
        assert!(
            !parts[7].contains("focus on"),
            "no focus/focusin should be dispatched during explicit blur()"
        );

        assert_eq!(
            parts[8], "true",
            "blurring unfocused element must not throw"
        );
        assert_eq!(
            parts[9], "true",
            "blurring unfocused element must be a no-op"
        );
    }
}
