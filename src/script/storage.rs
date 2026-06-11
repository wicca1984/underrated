//! Implementation of Web Storage (`localStorage` and `sessionStorage`) for the scripting engine.
//!
//! This module provides the Rust-side backing stores and the JS-side `Storage` class
//! and global `localStorage` / `sessionStorage` properties.

use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsString, JsValue, NativeFunction};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static LOCAL_STORAGE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static SESSION_STORAGE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Clears both local and session storage.
///
/// This is particularly useful for resetting state between test cases.
pub fn clear_storages() {
    LOCAL_STORAGE.with(|store| store.borrow_mut().clear());
    SESSION_STORAGE.with(|store| store.borrow_mut().clear());
}

/// Sets up the Storage bindings in the provided Boa context.
///
/// This registers a private bridge object and runs a wrapper script to define the standard
/// global `localStorage`, `sessionStorage`, and `Storage` interface in JS.
pub fn setup_storage(context: &mut Context) {
    let bridge = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(storage_set_item),
            JsString::from("setItem"),
            3,
        )
        .function(
            NativeFunction::from_fn_ptr(storage_get_item),
            JsString::from("getItem"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(storage_remove_item),
            JsString::from("removeItem"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(storage_clear),
            JsString::from("clear"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(storage_get_length),
            JsString::from("getLength"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(storage_get_key),
            JsString::from("getKey"),
            2,
        )
        .build();

    let _ = context.register_global_property(
        JsString::from("__storage_bridge__"),
        bridge,
        Attribute::all(),
    );

    // Evaluate the JS wrapper code to build the dynamic Storage API.
    // spec: https://html.spec.whatwg.org/multipage/webstorage.html
    let setup_code = r#"
        (function() {
            const bridge = window.__storage_bridge__;

            class Storage {
                constructor(type) {
                    this.__type__ = type;
                }

                getItem(key) {
                    return bridge.getItem(this.__type__, String(key));
                }

                setItem(key, value) {
                    bridge.setItem(this.__type__, String(key), String(value));
                }

                removeItem(key) {
                    bridge.removeItem(this.__type__, String(key));
                }

                clear() {
                    bridge.clear(this.__type__);
                }

                key(index) {
                    const idx = Number(index);
                    if (isNaN(idx)) return null;
                    return bridge.getKey(this.__type__, idx);
                }

                get length() {
                    return bridge.getLength(this.__type__);
                }
            }

            // Create window.localStorage and window.sessionStorage
            const localStorageInstance = new Storage('local');
            const sessionStorageInstance = new Storage('session');

            // Expose them as custom Proxies so that property-based gets/sets work!
            // e.g. localStorage.foo = 'bar' or localStorage['foo']
            function createStorageProxy(instance) {
                return new Proxy(instance, {
                    get(target, prop) {
                        // If it's a built-in method/property or symbol, return it
                        if (prop in target || typeof prop === 'symbol') {
                            const val = target[prop];
                            if (typeof val === 'function') {
                                return val.bind(target);
                            }
                            return val;
                        }
                        // Otherwise treat it as a key lookup
                        const item = target.getItem(prop);
                        return item === null ? undefined : item;
                    },
                    set(target, prop, value) {
                        if (prop in target || typeof prop === 'symbol') {
                            target[prop] = value;
                            return true;
                        }
                        target.setItem(prop, value);
                        return true;
                    },
                    deleteProperty(target, prop) {
                        if (prop in target || typeof prop === 'symbol') {
                            return delete target[prop];
                        }
                        target.removeItem(prop);
                        return true;
                    },
                    ownKeys(target) {
                        const len = target.length;
                        const keys = [];
                        for (let i = 0; i < len; i++) {
                            const k = target.key(i);
                            if (k !== null) {
                                keys.push(k);
                            }
                        }
                        return keys;
                    },
                    getOwnPropertyDescriptor(target, prop) {
                        if (prop in target || typeof prop === 'symbol') {
                            return Object.getOwnPropertyDescriptor(target, prop);
                        }
                        const item = target.getItem(prop);
                        if (item !== null) {
                            return {
                                value: item,
                                writable: true,
                                enumerable: true,
                                configurable: true
                            };
                        }
                        return undefined;
                    }
                });
            }

            const localProxy = createStorageProxy(localStorageInstance);
            const sessionProxy = createStorageProxy(sessionStorageInstance);

            Object.defineProperty(window, 'localStorage', {
                value: localProxy,
                writable: false,
                enumerable: true,
                configurable: true
            });

            Object.defineProperty(window, 'sessionStorage', {
                value: sessionProxy,
                writable: false,
                enumerable: true,
                configurable: true
            });

            // Also define Storage globally
            Object.defineProperty(window, 'Storage', {
                value: Storage,
                writable: true,
                enumerable: false,
                configurable: true
            });
        })();
    "#;

    let source = boa_engine::Source::from_bytes(setup_code.as_bytes());
    if let Err(e) = context.eval(source) {
        eprintln!("Failed to initialize Storage bindings: {:?}", e);
    }
}

fn storage_set_item(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let storage_type = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let key = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let value = if let Some(arg) = args.get(2) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    if storage_type == "local" {
        LOCAL_STORAGE.with(|store| {
            store.borrow_mut().insert(key, value);
        });
    } else if storage_type == "session" {
        SESSION_STORAGE.with(|store| {
            store.borrow_mut().insert(key, value);
        });
    }

    Ok(JsValue::undefined())
}

fn storage_get_item(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let storage_type = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let key = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let val_opt = if storage_type == "local" {
        LOCAL_STORAGE.with(|store| store.borrow().get(&key).cloned())
    } else if storage_type == "session" {
        SESSION_STORAGE.with(|store| store.borrow().get(&key).cloned())
    } else {
        None
    };

    if let Some(val) = val_opt {
        Ok(JsValue::from(JsString::from(val)))
    } else {
        Ok(JsValue::null())
    }
}

fn storage_remove_item(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let storage_type = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    let key = if let Some(arg) = args.get(1) {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    if storage_type == "local" {
        LOCAL_STORAGE.with(|store| {
            store.borrow_mut().remove(&key);
        });
    } else if storage_type == "session" {
        SESSION_STORAGE.with(|store| {
            store.borrow_mut().remove(&key);
        });
    }

    Ok(JsValue::undefined())
}

fn storage_clear(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let storage_type = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::undefined());
    };

    if storage_type == "local" {
        LOCAL_STORAGE.with(|store| {
            store.borrow_mut().clear();
        });
    } else if storage_type == "session" {
        SESSION_STORAGE.with(|store| {
            store.borrow_mut().clear();
        });
    }

    Ok(JsValue::undefined())
}

fn storage_get_length(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let storage_type = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::from(0));
    };

    let len = if storage_type == "local" {
        LOCAL_STORAGE.with(|store| store.borrow().len())
    } else if storage_type == "session" {
        SESSION_STORAGE.with(|store| store.borrow().len())
    } else {
        0
    };

    Ok(JsValue::from(len))
}

fn storage_get_key(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let storage_type = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        return Ok(JsValue::null());
    };

    let index = if let Some(arg) = args.get(1) {
        arg.to_number(context)? as usize
    } else {
        return Ok(JsValue::null());
    };

    let key_opt = if storage_type == "local" {
        LOCAL_STORAGE.with(|store| {
            let s = store.borrow();
            let mut keys: Vec<&String> = s.keys().collect();
            keys.sort();
            keys.get(index).map(|&k| k.clone())
        })
    } else if storage_type == "session" {
        SESSION_STORAGE.with(|store| {
            let s = store.borrow();
            let mut keys: Vec<&String> = s.keys().collect();
            keys.sort();
            keys.get(index).map(|&k| k.clone())
        })
    } else {
        None
    };

    if let Some(k) = key_opt {
        Ok(JsValue::from(JsString::from(k)))
    } else {
        Ok(JsValue::null())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{BoaHost, ScriptHost};

    fn new_host() -> BoaHost {
        clear_storages();
        BoaHost::new()
    }

    #[test]
    fn test_storage_basic_get_set() {
        let mut host = new_host();

        // Test localStorage setItem / getItem
        assert!(host.eval("localStorage.setItem('foo', 'bar')").is_ok());
        assert_eq!(
            host.context
                .eval(boa_engine::Source::from_bytes(
                    b"localStorage.getItem('foo')"
                ))
                .and_then(|v| v.to_string(&mut host.context))
                .map(|js_str| js_str.to_std_string().unwrap_or_default()),
            Ok("bar".to_string())
        );

        // Test sessionStorage setItem / getItem
        assert!(
            host.eval("sessionStorage.setItem('hello', 'world')")
                .is_ok()
        );
        assert_eq!(
            host.context
                .eval(boa_engine::Source::from_bytes(
                    b"sessionStorage.getItem('hello')"
                ))
                .and_then(|v| v.to_string(&mut host.context))
                .map(|js_str| js_str.to_std_string().unwrap_or_default()),
            Ok("world".to_string())
        );

        // They are isolated from each other
        assert!(
            host.eval("if (localStorage.getItem('hello') !== null) throw 'error';")
                .is_ok()
        );
        assert!(
            host.eval("if (sessionStorage.getItem('foo') !== null) throw 'error';")
                .is_ok()
        );
    }

    #[test]
    fn test_storage_property_access() {
        let mut host = new_host();

        // Test direct property assignment and lookup
        assert!(host.eval("localStorage.abc = '123'").is_ok());
        assert!(
            host.eval("if (localStorage.abc !== '123') throw 'error';")
                .is_ok()
        );
        assert!(
            host.eval("if (localStorage.getItem('abc') !== '123') throw 'error';")
                .is_ok()
        );

        // Test delete property
        assert!(host.eval("delete localStorage.abc").is_ok());
        assert!(
            host.eval("if (localStorage.abc !== undefined) throw 'error';")
                .is_ok()
        );
        assert!(
            host.eval("if (localStorage.getItem('abc') !== null) throw 'error';")
                .is_ok()
        );
    }

    #[test]
    fn test_storage_remove_and_clear() {
        let mut host = new_host();

        assert!(host.eval("localStorage.setItem('a', '1')").is_ok());
        assert!(host.eval("localStorage.setItem('b', '2')").is_ok());
        assert!(
            host.eval("if (localStorage.length !== 2) throw 'error';")
                .is_ok()
        );

        assert!(host.eval("localStorage.removeItem('a')").is_ok());
        assert!(
            host.eval("if (localStorage.length !== 1) throw 'error';")
                .is_ok()
        );
        assert!(
            host.eval("if (localStorage.getItem('a') !== null) throw 'error';")
                .is_ok()
        );

        assert!(host.eval("localStorage.clear()").is_ok());
        assert!(
            host.eval("if (localStorage.length !== 0) throw 'error';")
                .is_ok()
        );
    }

    #[test]
    fn test_storage_keys_and_length() {
        let mut host = new_host();

        assert!(host.eval("localStorage.setItem('k2', 'val2')").is_ok());
        assert!(host.eval("localStorage.setItem('k1', 'val1')").is_ok());

        assert!(
            host.eval("if (localStorage.length !== 2) throw 'error';")
                .is_ok()
        );

        // Sorted keys should mean key(0) is 'k1', key(1) is 'k2'
        assert!(
            host.eval("if (localStorage.key(0) !== 'k1') throw 'error';")
                .is_ok()
        );
        assert!(
            host.eval("if (localStorage.key(1) !== 'k2') throw 'error';")
                .is_ok()
        );
        assert!(
            host.eval("if (localStorage.key(2) !== null) throw 'error';")
                .is_ok()
        );

        // Object.keys(localStorage) should also work and be sorted
        assert!(host.eval("const keys = Object.keys(localStorage); if (keys.length !== 2 || keys[0] !== 'k1' || keys[1] !== 'k2') throw 'error';").is_ok());
    }
}
