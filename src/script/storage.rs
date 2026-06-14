//! Implementation of Web Storage (`localStorage` and `sessionStorage`) for the scripting engine.
//!
//! This module provides the Rust-side backing stores and the JS-side `Storage` class
//! and global `localStorage` / `sessionStorage` properties.

use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsNativeError, JsString, JsValue, NativeFunction};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static LOCAL_STORAGE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static SESSION_STORAGE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static STORAGE_QUOTA: RefCell<usize> = const { RefCell::new(5_000_000) };
}

/// Sets the storage quota limit (in characters) for testing.
pub fn set_storage_quota(quota: usize) {
    STORAGE_QUOTA.with(|q| *q.borrow_mut() = quota);
}

/// Clears both local and session storage and resets the quota limit.
///
/// This is particularly useful for resetting state between test cases.
pub fn clear_storages() {
    LOCAL_STORAGE.with(|store| store.borrow_mut().clear());
    SESSION_STORAGE.with(|store| store.borrow_mut().clear());
    STORAGE_QUOTA.with(|q| *q.borrow_mut() = 5_000_000);
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
            const initToken = Symbol('StorageInitToken');
            const typeSymbol = Symbol('StorageType');

            class Storage {
                constructor(token, type) {
                    if (token !== initToken) {
                        throw new TypeError("Illegal constructor");
                    }
                    this[typeSymbol] = type;
                }

                getItem(key) {
                    if (!(this instanceof Storage) || (this[typeSymbol] !== 'local' && this[typeSymbol] !== 'session')) {
                        throw new TypeError("Failed to execute 'getItem' on 'Storage': Value of 'this' is not a Storage object.");
                    }
                    if (arguments.length < 1) {
                        throw new TypeError("Failed to execute 'getItem' on 'Storage': 1 argument required, but only 0 present.");
                    }
                    return bridge.getItem(this[typeSymbol], String(key));
                }

                setItem(key, value) {
                    if (!(this instanceof Storage) || (this[typeSymbol] !== 'local' && this[typeSymbol] !== 'session')) {
                        throw new TypeError("Failed to execute 'setItem' on 'Storage': Value of 'this' is not a Storage object.");
                    }
                    if (arguments.length < 2) {
                        throw new TypeError("Failed to execute 'setItem' on 'Storage': 2 arguments required, but only " + arguments.length + " present.");
                    }
                    bridge.setItem(this[typeSymbol], String(key), String(value));
                }

                removeItem(key) {
                    if (!(this instanceof Storage) || (this[typeSymbol] !== 'local' && this[typeSymbol] !== 'session')) {
                        throw new TypeError("Failed to execute 'removeItem' on 'Storage': Value of 'this' is not a Storage object.");
                    }
                    if (arguments.length < 1) {
                        throw new TypeError("Failed to execute 'removeItem' on 'Storage': 1 argument required, but only 0 present.");
                    }
                    bridge.removeItem(this[typeSymbol], String(key));
                }

                clear() {
                    if (!(this instanceof Storage) || (this[typeSymbol] !== 'local' && this[typeSymbol] !== 'session')) {
                        throw new TypeError("Failed to execute 'clear' on 'Storage': Value of 'this' is not a Storage object.");
                    }
                    bridge.clear(this[typeSymbol]);
                }

                key(index) {
                    if (!(this instanceof Storage) || (this[typeSymbol] !== 'local' && this[typeSymbol] !== 'session')) {
                        throw new TypeError("Failed to execute 'key' on 'Storage': Value of 'this' is not a Storage object.");
                    }
                    return bridge.getKey(this[typeSymbol], Number(index));
                }

                get length() {
                    if (!(this instanceof Storage) || (this[typeSymbol] !== 'local' && this[typeSymbol] !== 'session')) {
                        throw new TypeError("Failed to execute 'length' on 'Storage': Value of 'this' is not a Storage object.");
                    }
                    return bridge.getLength(this[typeSymbol]);
                }
            }

            Object.defineProperty(Storage.prototype, Symbol.toStringTag, {
                value: 'Storage',
                configurable: true
            });

            // Create window.localStorage and window.sessionStorage
            const localStorageInstance = new Storage(initToken, 'local');
            const sessionStorageInstance = new Storage(initToken, 'session');

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
                    has(target, prop) {
                        if (prop in target || typeof prop === 'symbol') {
                            return true;
                        }
                        return target.getItem(prop) !== null;
                    },
                    defineProperty(target, prop, descriptor) {
                        if (prop in target || typeof prop === 'symbol') {
                            return Reflect.defineProperty(target, prop, descriptor);
                        }
                        if (descriptor.value !== undefined) {
                            target.setItem(prop, String(descriptor.value));
                            return true;
                        }
                        return false;
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
                configurable: false
            });

            Object.defineProperty(window, 'sessionStorage', {
                value: sessionProxy,
                writable: false,
                enumerable: true,
                configurable: false
            });

            // Also define Storage globally
            Object.defineProperty(window, 'Storage', {
                value: Storage,
                writable: true,
                enumerable: false,
                configurable: true
            });

            // Register global StorageEvent class if Event is available
            if (typeof window.Event === 'function') {
                class StorageEvent extends window.Event {
                    constructor(type, eventInitDict = {}) {
                        if (arguments.length < 1) {
                            throw new TypeError("Failed to construct 'StorageEvent': 1 argument required, but only 0 present.");
                        }
                        super(type, eventInitDict);
                        this._key = eventInitDict.key !== undefined ? eventInitDict.key : null;
                        this._oldValue = eventInitDict.oldValue !== undefined ? eventInitDict.oldValue : null;
                        this._newValue = eventInitDict.newValue !== undefined ? eventInitDict.newValue : null;
                        this._url = eventInitDict.url !== undefined ? String(eventInitDict.url) : "";
                        this._storageArea = eventInitDict.storageArea !== undefined ? eventInitDict.storageArea : null;
                    }

                    get key() {
                        if (!(this instanceof StorageEvent)) {
                            throw new TypeError("Failed to read the 'key' property from 'StorageEvent': Receiver does not implement interface 'StorageEvent'.");
                        }
                        return this._key;
                    }

                    get oldValue() {
                        if (!(this instanceof StorageEvent)) {
                            throw new TypeError("Failed to read the 'oldValue' property from 'StorageEvent': Receiver does not implement interface 'StorageEvent'.");
                        }
                        return this._oldValue;
                    }

                    get newValue() {
                        if (!(this instanceof StorageEvent)) {
                            throw new TypeError("Failed to read the 'newValue' property from 'StorageEvent': Receiver does not implement interface 'StorageEvent'.");
                        }
                        return this._newValue;
                    }

                    get url() {
                        if (!(this instanceof StorageEvent)) {
                            throw new TypeError("Failed to read the 'url' property from 'StorageEvent': Receiver does not implement interface 'StorageEvent'.");
                        }
                        return this._url;
                    }

                    get storageArea() {
                        if (!(this instanceof StorageEvent)) {
                            throw new TypeError("Failed to read the 'storageArea' property from 'StorageEvent': Receiver does not implement interface 'StorageEvent'.");
                        }
                        return this._storageArea;
                    }

                    initStorageEvent(type, bubbles = false, cancelable = false, key = null, oldValue = null, newValue = null, url = "", storageArea = null) {
                        if (!(this instanceof StorageEvent)) {
                            throw new TypeError("Failed to execute 'initStorageEvent' on 'StorageEvent': Receiver does not implement interface 'StorageEvent'.");
                        }
                        this._key = key;
                        this._oldValue = oldValue;
                        this._newValue = newValue;
                        this._url = String(url);
                        this._storageArea = storageArea;
                    }
                }

                Object.defineProperty(StorageEvent.prototype, Symbol.toStringTag, {
                    value: 'StorageEvent',
                    configurable: true
                });

                Object.defineProperty(window, 'StorageEvent', {
                    value: StorageEvent,
                    writable: true,
                    enumerable: false,
                    configurable: true
                });
            }
        })();
    "#;

    let source = boa_engine::Source::from_bytes(setup_code.as_bytes());
    if let Err(e) = context.eval(source) {
        eprintln!("Failed to initialize Storage bindings: {:?}", e);
    }
}

fn throw_dom_exception(name: &str, message: &str, context: &mut Context) -> JsError {
    let dom_exception_constructor = context
        .global_object()
        .get(JsString::from("DOMException"), context);
    if let Some(constructor_obj) = dom_exception_constructor
        .ok()
        .as_ref()
        .and_then(|val| val.as_object())
    {
        let args = [
            JsValue::from(JsString::from(message)),
            JsValue::from(JsString::from(name)),
        ];
        if let Ok(exception_obj) = constructor_obj.construct(&args, None, context) {
            return JsError::from_opaque(JsValue::from(exception_obj));
        }
    }
    JsError::from(JsNativeError::typ().with_message(format!("{}: {}", name, message)))
}

fn get_storage_size(store: &HashMap<String, String>) -> usize {
    store
        .iter()
        .map(|(k, v)| {
            k.chars().map(|c| c.len_utf16()).sum::<usize>()
                + v.chars().map(|c| c.len_utf16()).sum::<usize>()
        })
        .sum()
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

    let quota = STORAGE_QUOTA.with(|q| *q.borrow());
    let mut quota_exceeded = false;

    if storage_type == "local" {
        LOCAL_STORAGE.with(|store| {
            let mut s = store.borrow_mut();
            let current_size = get_storage_size(&s);
            let old_size = s
                .get(&key)
                .map(|v| {
                    key.chars().map(|c| c.len_utf16()).sum::<usize>()
                        + v.chars().map(|c| c.len_utf16()).sum::<usize>()
                })
                .unwrap_or(0);
            let new_size = key.chars().map(|c| c.len_utf16()).sum::<usize>()
                + value.chars().map(|c| c.len_utf16()).sum::<usize>();
            if current_size - old_size + new_size > quota {
                quota_exceeded = true;
            } else {
                s.insert(key, value);
            }
        });
    } else if storage_type == "session" {
        SESSION_STORAGE.with(|store| {
            let mut s = store.borrow_mut();
            let current_size = get_storage_size(&s);
            let old_size = s
                .get(&key)
                .map(|v| {
                    key.chars().map(|c| c.len_utf16()).sum::<usize>()
                        + v.chars().map(|c| c.len_utf16()).sum::<usize>()
                })
                .unwrap_or(0);
            let new_size = key.chars().map(|c| c.len_utf16()).sum::<usize>()
                + value.chars().map(|c| c.len_utf16()).sum::<usize>();
            if current_size - old_size + new_size > quota {
                quota_exceeded = true;
            } else {
                s.insert(key, value);
            }
        });
    }

    if quota_exceeded {
        return Err(throw_dom_exception(
            "QuotaExceededError",
            "The storage quota has been exceeded.",
            context,
        ));
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

    let index_f = if let Some(arg) = args.get(1) {
        arg.to_number(context)?
    } else {
        return Ok(JsValue::null());
    };

    let index = if index_f.is_nan() || index_f.is_infinite() {
        0
    } else {
        let int_part = index_f.trunc();
        let rem = int_part % 4294967296.0;
        let u_val = if rem < 0.0 { rem + 4294967296.0 } else { rem };
        u_val as u32 as usize
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
    fn test_storage_quota_limit() {
        let mut host = new_host();

        // Set quota limit to small value (10 chars total)
        set_storage_quota(10);

        // This fits (4 + 5 = 9 chars)
        assert!(host.eval("localStorage.setItem('abcd', '12345')").is_ok());
        assert_eq!(
            host.context
                .eval(boa_engine::Source::from_bytes(
                    b"localStorage.getItem('abcd')"
                ))
                .and_then(|v| v.to_string(&mut host.context))
                .map(|js_str| js_str.to_std_string().unwrap_or_default()),
            Ok("12345".to_string())
        );

        // Setting a larger value under the same key that still fits (4 + 6 = 10 chars)
        assert!(host.eval("localStorage.setItem('abcd', '123456')").is_ok());

        // This should fail (4 + 7 = 11 chars > 10 quota limit)
        // It must throw QuotaExceededError DOMException
        assert!(
            host.eval(
                r#"
            let threwQuota = false;
            try {
                localStorage.setItem('abcd', '1234567');
            } catch (e) {
                if (e instanceof DOMException && e.name === "QuotaExceededError") {
                    threwQuota = true;
                }
            }
            if (!threwQuota) throw new Error("Expected QuotaExceededError");
        "#
            )
            .is_ok()
        );

        // Also check that a different key failing quota works correctly
        assert!(
            host.eval(
                r#"
            let threwQuota2 = false;
            try {
                localStorage.setItem('e', '123456789');
            } catch (e) {
                if (e instanceof DOMException && e.name === "QuotaExceededError") {
                    threwQuota2 = true;
                }
            }
            if (!threwQuota2) throw new Error("Expected QuotaExceededError for new key");
        "#
            )
            .is_ok()
        );
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

        // Test WebIDL key index conversions & coercion
        // - negative indices should wrap or be invalid (returning null)
        assert!(
            host.eval("if (localStorage.key(-1) !== null) throw 'error1';")
                .is_ok()
        );
        assert!(
            host.eval("if (localStorage.key(-1.5) !== null) throw 'error2';")
                .is_ok()
        );
        // - floats should be truncated: key(1.5) should be key(1) which is 'k2'
        assert!(
            host.eval("if (localStorage.key(1.5) !== 'k2') throw 'error3';")
                .is_ok()
        );
        // - wrap around: key(4294967297) should wrap to key(1) which is 'k2'
        assert!(
            host.eval("if (localStorage.key(4294967297) !== 'k2') throw 'error4';")
                .is_ok()
        );
        // - wrap around: key(4294967296) should wrap to key(0) which is 'k1'
        assert!(
            host.eval("if (localStorage.key(4294967296) !== 'k1') throw 'error5';")
                .is_ok()
        );
        // - NaN / missing / invalid: should convert to 0, returning 'k1'
        assert!(
            host.eval("if (localStorage.key(NaN) !== 'k1') throw 'error6';")
                .is_ok()
        );
        assert!(
            host.eval("if (localStorage.key('abc') !== 'k1') throw 'error7';")
                .is_ok()
        );
        assert!(
            host.eval("if (localStorage.key() !== 'k1') throw 'error8';")
                .is_ok()
        );
    }

    #[test]
    fn test_storage_validation_and_webidl() {
        let mut host = new_host();

        assert!(host.eval(r#"
            (function() {
                // 1. Storage constructor cannot be called directly
                let threw1 = false;
                try {
                    new Storage();
                } catch (e) {
                    if (e.message === "Illegal constructor") {
                        threw1 = true;
                    }
                }
                if (!threw1) throw new Error("Storage constructor did not throw TypeError");

                // 2. Storage methods require Storage instance as 'this' context
                let threw2 = false;
                try {
                    Storage.prototype.getItem.call({}, "foo");
                } catch (e) {
                    if (e.message && e.message.indexOf("Value of 'this' is not a Storage object") !== -1) {
                        threw2 = true;
                    }
                }
                if (!threw2) throw new Error("Storage.prototype.getItem on plain object did not throw TypeError");

                // 3. Method argument count validations
                // - getItem requires 1 argument
                let threw3 = false;
                try {
                    localStorage.getItem();
                } catch (e) {
                    if (e.message && e.message.indexOf("1 argument required, but only 0 present") !== -1) {
                        threw3 = true;
                    }
                }
                if (!threw3) throw new Error("localStorage.getItem() with 0 arguments did not throw TypeError");

                // - setItem requires 2 arguments
                let threw4 = false;
                try {
                    localStorage.setItem("foo");
                } catch (e) {
                    if (e.message && e.message.indexOf("2 arguments required, but only 1 present") !== -1) {
                        threw4 = true;
                    }
                }
                if (!threw4) throw new Error("localStorage.setItem('foo') with 1 argument did not throw TypeError");

                // - removeItem requires 1 argument
                let threw5 = false;
                try {
                    localStorage.removeItem();
                } catch (e) {
                    if (e.message && e.message.indexOf("1 argument required, but only 0 present") !== -1) {
                        threw5 = true;
                    }
                }
                if (!threw5) throw new Error("localStorage.removeItem() with 0 arguments did not throw TypeError");

                // 4. Object.prototype.toString.call(localStorage) returns "[object Storage]"
                const tag = Object.prototype.toString.call(localStorage);
                if (tag !== "[object Storage]") {
                    throw new Error("Expected [object Storage], got " + tag);
                }

                // 5. window.localStorage non-configurability
                const desc = Object.getOwnPropertyDescriptor(window, 'localStorage');
                if (desc.configurable !== false) {
                    throw new Error("window.localStorage must be non-configurable");
                }
            })()
        "#).is_ok());
    }

    #[test]
    fn test_storage_proxy_has_trap() {
        let mut host = new_host();

        // 1. Initially, a key should not be 'in' localStorage
        assert!(
            host.eval("if ('foo' in localStorage) throw 'error1';")
                .is_ok()
        );

        // 2. Set the key, and it should now be 'in' localStorage
        assert!(host.eval("localStorage.setItem('foo', 'bar')").is_ok());
        assert!(
            host.eval("if (!('foo' in localStorage)) throw 'error2';")
                .is_ok()
        );

        // 3. Remove the key, and it should no longer be 'in' localStorage
        assert!(host.eval("localStorage.removeItem('foo')").is_ok());
        assert!(
            host.eval("if ('foo' in localStorage) throw 'error3';")
                .is_ok()
        );

        // 4. Built-in methods and properties should always be 'in' localStorage
        assert!(
            host.eval("if (!('getItem' in localStorage)) throw 'error4';")
                .is_ok()
        );
        assert!(
            host.eval("if (!('setItem' in localStorage)) throw 'error5';")
                .is_ok()
        );
        assert!(
            host.eval("if (!('removeItem' in localStorage)) throw 'error6';")
                .is_ok()
        );
        assert!(
            host.eval("if (!('clear' in localStorage)) throw 'error7';")
                .is_ok()
        );
        assert!(
            host.eval("if (!('key' in localStorage)) throw 'error8';")
                .is_ok()
        );
        assert!(
            host.eval("if (!('length' in localStorage)) throw 'error9';")
                .is_ok()
        );
    }

    #[test]
    fn test_storage_proxy_define_property_trap() {
        let mut host = new_host();

        // 1. Defining a property via Object.defineProperty should set the key in Web Storage
        assert!(host.eval("Object.defineProperty(localStorage, 'greeting', { value: 'hello', enumerable: true, configurable: true, writable: true })").is_ok());
        assert_eq!(
            host.context
                .eval(boa_engine::Source::from_bytes(
                    b"localStorage.getItem('greeting')"
                ))
                .and_then(|v| v.to_string(&mut host.context))
                .map(|js_str| js_str.to_std_string().unwrap_or_default()),
            Ok("hello".to_string())
        );

        // 2. The defined property should also be accessible via dynamic lookup
        assert_eq!(
            host.context
                .eval(boa_engine::Source::from_bytes(b"localStorage.greeting"))
                .and_then(|v| v.to_string(&mut host.context))
                .map(|js_str| js_str.to_std_string().unwrap_or_default()),
            Ok("hello".to_string())
        );
    }

    #[test]
    fn test_storage_symbol_encapsulation() {
        let mut host = new_host();

        // 1. Check that '__type__' is no longer present as a property on localStorage target
        assert!(
            host.eval("if ('__type__' in localStorage) throw 'error1';")
                .is_ok()
        );
        assert!(
            host.eval("if (localStorage.__type__ !== undefined) throw 'error2';")
                .is_ok()
        );

        // 2. Check that Object.getOwnPropertyNames does not contain '__type__'
        assert!(host.eval("const names = Object.getOwnPropertyNames(localStorage); if (names.indexOf('__type__') !== -1) throw 'error3';").is_ok());
    }

    #[test]
    fn test_storage_event_class() {
        let mut host = new_host();

        // 1. Verify StorageEvent exists on window
        assert!(
            host.eval("if (typeof StorageEvent !== 'function') throw 'error1';")
                .is_ok()
        );

        // 2. Verify StorageEvent can be constructed and has correct properties
        assert!(host.eval("const ev = new StorageEvent('storage');").is_ok());
        assert!(
            host.eval("if (!(ev instanceof Event)) throw 'error2';")
                .is_ok()
        );
        assert!(
            host.eval("if (!(ev instanceof StorageEvent)) throw 'error3';")
                .is_ok()
        );

        // 3. Verify standard defaults
        assert!(host.eval("if (ev.key !== null) throw 'error4';").is_ok());
        assert!(
            host.eval("if (ev.oldValue !== null) throw 'error5';")
                .is_ok()
        );
        assert!(
            host.eval("if (ev.newValue !== null) throw 'error6';")
                .is_ok()
        );
        assert!(host.eval("if (ev.url !== '') throw 'error7';").is_ok());
        assert!(
            host.eval("if (ev.storageArea !== null) throw 'error8';")
                .is_ok()
        );

        // 4. Verify Symbol.toStringTag
        assert_eq!(
            host.context
                .eval(boa_engine::Source::from_bytes(
                    b"Object.prototype.toString.call(ev)"
                ))
                .and_then(|v| v.to_string(&mut host.context))
                .map(|js_str| js_str.to_std_string().unwrap_or_default()),
            Ok("[object StorageEvent]".to_string())
        );

        // 5. Verify custom eventInitDict values
        assert!(
            host.eval(
                r#"
            const ev2 = new StorageEvent('storage', {
                key: 'user',
                oldValue: 'alice',
                newValue: 'bob',
                url: 'http://example.com/',
                storageArea: localStorage
            });
            if (ev2.key !== 'user') throw 'error9';
            if (ev2.oldValue !== 'alice') throw 'error10';
            if (ev2.newValue !== 'bob') throw 'error11';
            if (ev2.url !== 'http://example.com/') throw 'error12';
            if (ev2.storageArea !== localStorage) throw 'error13';
        "#
            )
            .is_ok()
        );
    }

    #[test]
    fn test_storage_event_compliance_edge_cases() {
        let mut host = new_host();

        assert!(
            host.eval(
                r#"
            (function() {
                // 1. Missing type argument throws TypeError
                let constructorThrew = false;
                try {
                    new StorageEvent();
                } catch (e) {
                    if (e instanceof TypeError && e.message.indexOf("1 argument required") !== -1) {
                        constructorThrew = true;
                    }
                }
                if (!constructorThrew) throw new Error("Expected StorageEvent constructor to throw TypeError on missing type argument");

                // 2. Prototype properties are accessors (getters), not plain data values
                const desc = Object.getOwnPropertyDescriptor(StorageEvent.prototype, 'key');
                if (!desc || typeof desc.get !== 'function') {
                    throw new Error("Expected StorageEvent.prototype.key to be an accessor property with a getter");
                }
                if (desc.set !== undefined) {
                    throw new Error("Expected StorageEvent.prototype.key setter to be undefined");
                }

                // 3. Receiver validation (getters throw TypeError when called on non-StorageEvent)
                const ev = new StorageEvent('storage', { key: 'foo' });
                const plainObj = {};
                let getterThrew = false;
                try {
                    desc.get.call(plainObj);
                } catch (e) {
                    if (e instanceof TypeError && e.message.indexOf("Receiver does not implement interface") !== -1) {
                        getterThrew = true;
                    }
                }
                if (!getterThrew) throw new Error("Expected StorageEvent.prototype.key getter to throw TypeError on non-StorageEvent receiver");

                // 4. initStorageEvent exists and initializes correctly
                if (typeof StorageEvent.prototype.initStorageEvent !== 'function') {
                    throw new Error("Expected initStorageEvent to be a function on StorageEvent.prototype");
                }
                const evInit = new StorageEvent('storage');
                evInit.initStorageEvent('storage', false, false, 'initKey', 'oldVal', 'newVal', 'http://init.url/', localStorage);
                if (evInit.key !== 'initKey') throw new Error("initStorageEvent failed to set key");
                if (evInit.oldValue !== 'oldVal') throw new Error("initStorageEvent failed to set oldValue");
                if (evInit.newValue !== 'newVal') throw new Error("initStorageEvent failed to set newValue");
                if (evInit.url !== 'http://init.url/') throw new Error("initStorageEvent failed to set url");
                if (evInit.storageArea !== localStorage) throw new Error("initStorageEvent failed to set storageArea");
            })()
        "#
            )
            .is_ok()
        );
    }

    #[test]
    fn test_storage_quota_utf16_compliance() {
        let mut host = new_host();

        // Set quota limit to 6 characters (measured in UTF-16 code units)
        set_storage_quota(6);

        // Key: 'abc' (3 UTF-16 units)
        // Value: 'de' (2 UTF-16 units)
        // Total: 5 units <= 6 quota (Fits!)
        assert!(host.eval("localStorage.setItem('abc', 'de')").is_ok());

        // Now change value to 'de🚀'
        // '🚀' is U+1F680, which is represented as surrogate pairs in UTF-16 (length 2 code units)
        // Key: 'abc' (3 units)
        // Value: 'de🚀' (4 units)
        // Total: 7 units > 6 quota (Should throw QuotaExceededError!)
        assert!(
            host.eval(
                r#"
            let threwQuota = false;
            try {
                localStorage.setItem('abc', 'de🚀');
            } catch (e) {
                if (e instanceof DOMException && e.name === "QuotaExceededError") {
                    threwQuota = true;
                }
            }
            if (!threwQuota) throw new Error("Expected QuotaExceededError because of surrogate pair UTF-16 length");
        "#
            )
            .is_ok()
        );
    }
}
