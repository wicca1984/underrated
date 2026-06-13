use boa_engine::class::{Class, ClassBuilder};
use boa_engine::{Context, JsError, JsString, JsValue, NativeFunction};
use boa_engine::{JsData, JsNativeError, JsResult};
use boa_gc::{Finalize, GcRefCell, Trace};

/// Implementation of WHATWG Fetch `Headers` interface.
/// Spec: <https://fetch.spec.whatwg.org/#headers-class>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct Headers {
    pub(crate) pairs: GcRefCell<Vec<(String, String)>>,
}

fn normalize_value(val: &str) -> String {
    let is_http_whitespace =
        |c: char| matches!(c, '\u{0009}' | '\u{000A}' | '\u{000D}' | '\u{0020}');
    val.trim_matches(is_http_whitespace).to_string()
}

impl Class for Headers {
    const NAME: &'static str = "Headers";
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
            && let Some(obj) = arg.as_object()
        {
            if let Some(other_headers) = obj.downcast_ref::<Headers>() {
                let other_pairs = other_headers.pairs.borrow().clone();
                pairs = other_pairs;
            } else {
                // Check if the object is iterable
                let symbol_iterator = context
                    .global_object()
                    .get(JsString::from("Symbol"), context)?
                    .as_object()
                    .ok_or_else(|| {
                        JsError::from(JsNativeError::typ().with_message("Symbol not found"))
                    })?
                    .get(JsString::from("iterator"), context)?;

                let symbol_iterator_key = symbol_iterator.to_property_key(context)?;
                let is_iterable = obj
                    .has_property(symbol_iterator_key, context)
                    .unwrap_or(false);

                if is_iterable {
                    let array_from_fn = context
                        .global_object()
                        .get(JsString::from("Array"), context)?
                        .as_object()
                        .ok_or_else(|| {
                            JsError::from(JsNativeError::typ().with_message("Array not found"))
                        })?
                        .get(JsString::from("from"), context)?;
                    let array_val = array_from_fn
                        .as_callable()
                        .ok_or_else(|| {
                            JsError::from(
                                JsNativeError::typ().with_message("Array.from not callable"),
                            )
                        })?
                        .call(&JsValue::undefined(), std::slice::from_ref(arg), context)?;

                    if let Some(arr_obj) = array_val.as_object() {
                        let length_val = arr_obj.get(JsString::from("length"), context)?;
                        let length = length_val.as_number().unwrap_or(0.0) as usize;
                        for i in 0..length {
                            let item = arr_obj.get(i, context)?;
                            if let Some(item_obj) = item.as_object() {
                                let name_val = item_obj.get(0, context)?;
                                let value_val = item_obj.get(1, context)?;

                                let name = name_val
                                    .to_string(context)?
                                    .to_std_string()
                                    .unwrap_or_default();
                                let value = value_val
                                    .to_string(context)?
                                    .to_std_string()
                                    .unwrap_or_default();

                                let normalized_name = name.to_ascii_lowercase();
                                let normalized_value = normalize_value(&value);

                                // Follow append-like logic to merge duplicate headers on init
                                let mut found = false;
                                for (k, v) in pairs.iter_mut() {
                                    if k == &normalized_name {
                                        v.push_str(", ");
                                        v.push_str(&normalized_value);
                                        found = true;
                                        break;
                                    }
                                }
                                if !found {
                                    pairs.push((normalized_name, normalized_value));
                                }
                            } else {
                                return Err(JsError::from(
                                    JsNativeError::typ()
                                        .with_message("Header pair must be a sequence"),
                                ));
                            }
                        }
                    }
                } else {
                    // Otherwise treat as a record (Object)
                    let object_keys_fn = context
                        .global_object()
                        .get(JsString::from("Object"), context)?
                        .as_object()
                        .ok_or_else(|| {
                            JsError::from(JsNativeError::typ().with_message("Object not found"))
                        })?
                        .get(JsString::from("keys"), context)?;
                    let keys_val = object_keys_fn
                        .as_callable()
                        .ok_or_else(|| {
                            JsError::from(
                                JsNativeError::typ().with_message("Object.keys not callable"),
                            )
                        })?
                        .call(&JsValue::undefined(), std::slice::from_ref(arg), context)?;

                    if let Some(keys_arr) = keys_val.as_object() {
                        let length_val = keys_arr.get(JsString::from("length"), context)?;
                        let length = length_val.as_number().unwrap_or(0.0) as usize;
                        for i in 0..length {
                            let key_val = keys_arr.get(i, context)?;
                            let key_str = key_val
                                .to_string(context)?
                                .to_std_string()
                                .unwrap_or_default();
                            let val_val = obj.get(JsString::from(key_str.as_str()), context)?;
                            let val_str = val_val
                                .to_string(context)?
                                .to_std_string()
                                .unwrap_or_default();

                            let normalized_name = key_str.to_ascii_lowercase();
                            let normalized_value = normalize_value(&val_str);

                            let mut found = false;
                            for (k, v) in pairs.iter_mut() {
                                if k == &normalized_name {
                                    v.push_str(", ");
                                    v.push_str(&normalized_value);
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                pairs.push((normalized_name, normalized_value));
                            }
                        }
                    }
                }
            }
        }

        Ok(Headers {
            pairs: GcRefCell::new(pairs),
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class
            .method(
                JsString::from("append"),
                2,
                NativeFunction::from_fn_ptr(headers_append),
            )
            .method(
                JsString::from("set"),
                2,
                NativeFunction::from_fn_ptr(headers_set),
            )
            .method(
                JsString::from("get"),
                1,
                NativeFunction::from_fn_ptr(headers_get),
            )
            .method(
                JsString::from("has"),
                1,
                NativeFunction::from_fn_ptr(headers_has),
            )
            .method(
                JsString::from("delete"),
                1,
                NativeFunction::from_fn_ptr(headers_delete),
            )
            .method(
                JsString::from("forEach"),
                1,
                NativeFunction::from_fn_ptr(headers_for_each),
            )
            .method(
                JsString::from("keys"),
                0,
                NativeFunction::from_fn_ptr(headers_keys),
            )
            .method(
                JsString::from("values"),
                0,
                NativeFunction::from_fn_ptr(headers_values),
            )
            .method(
                JsString::from("entries"),
                0,
                NativeFunction::from_fn_ptr(headers_entries),
            );

        Ok(())
    }
}

pub fn headers_append(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let headers = obj.downcast_ref::<Headers>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Headers object"))
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

    let normalized_name = name.to_ascii_lowercase();
    let normalized_value = normalize_value(&val);

    let mut pairs = headers.pairs.borrow_mut();
    let mut found = false;
    for (k, v) in pairs.iter_mut() {
        if k == &normalized_name {
            v.push_str(", ");
            v.push_str(&normalized_value);
            found = true;
            break;
        }
    }
    if !found {
        pairs.push((normalized_name, normalized_value));
    }

    Ok(JsValue::undefined())
}

pub fn headers_set(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let headers = obj.downcast_ref::<Headers>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Headers object"))
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

    let normalized_name = name.to_ascii_lowercase();
    let normalized_value = normalize_value(&val);

    let mut pairs = headers.pairs.borrow_mut();
    let mut found = false;
    let mut i = 0;
    while i < pairs.len() {
        if pairs[i].0 == normalized_name {
            if !found {
                pairs[i].1 = normalized_value.clone();
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
        pairs.push((normalized_name, normalized_value));
    }

    Ok(JsValue::undefined())
}

pub fn headers_get(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let headers = obj.downcast_ref::<Headers>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Headers object"))
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let normalized_name = name.to_ascii_lowercase();

    let pairs = headers.pairs.borrow();
    for (k, v) in pairs.iter() {
        if k == &normalized_name {
            return Ok(JsValue::from(JsString::from(v.as_str())));
        }
    }

    Ok(JsValue::null())
}

pub fn headers_has(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let headers = obj.downcast_ref::<Headers>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Headers object"))
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let normalized_name = name.to_ascii_lowercase();

    let pairs = headers.pairs.borrow();
    let has_key = pairs.iter().any(|(k, _)| k == &normalized_name);
    Ok(JsValue::from(has_key))
}

pub fn headers_delete(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let headers = obj.downcast_ref::<Headers>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Headers object"))
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let normalized_name = name.to_ascii_lowercase();

    headers
        .pairs
        .borrow_mut()
        .retain(|(k, _)| k != &normalized_name);
    Ok(JsValue::undefined())
}

pub fn headers_for_each(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let headers = obj.downcast_ref::<Headers>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Headers object"))
    })?;

    let callback = args.first().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Missing callback argument"))
    })?;
    let callback_fn = callback.as_callable().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Callback must be a function"))
    })?;

    let this_arg = args.get(1).cloned().unwrap_or_default();

    // Get sorted keys
    let mut entries = headers.pairs.borrow().clone();
    // Sort lexicographically by lowercased name
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (k, v) in entries {
        callback_fn.call(
            &this_arg,
            &[
                JsValue::from(JsString::from(v)),
                JsValue::from(JsString::from(k)),
                this.clone(),
            ],
            context,
        )?;
    }

    Ok(JsValue::undefined())
}

pub fn headers_keys(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let headers = obj.downcast_ref::<Headers>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Headers object"))
    })?;

    let mut entries = headers.pairs.borrow().clone();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let elements: Vec<JsValue> = entries
        .iter()
        .map(|(k, _)| JsValue::from(JsString::from(k.as_str())))
        .collect();

    let array = boa_engine::object::builtins::JsArray::from_iter(elements, context);
    let values_fn = array.get(JsString::from("values"), context)?;
    let values_iterator = values_fn
        .as_callable()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Array.prototype.values not callable"))
        })?
        .call(&array.into(), &[], context)?;
    Ok(values_iterator)
}

pub fn headers_values(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let headers = obj.downcast_ref::<Headers>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Headers object"))
    })?;

    let mut entries = headers.pairs.borrow().clone();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let elements: Vec<JsValue> = entries
        .iter()
        .map(|(_, v)| JsValue::from(JsString::from(v.as_str())))
        .collect();

    let array = boa_engine::object::builtins::JsArray::from_iter(elements, context);
    let values_fn = array.get(JsString::from("values"), context)?;
    let values_iterator = values_fn
        .as_callable()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Array.prototype.values not callable"))
        })?
        .call(&array.into(), &[], context)?;
    Ok(values_iterator)
}

pub fn headers_entries(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let headers = obj.downcast_ref::<Headers>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Headers object"))
    })?;

    let mut entries = headers.pairs.borrow().clone();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut elements = Vec::with_capacity(entries.len());
    for (k, v) in entries.iter() {
        let pair = vec![
            JsValue::from(JsString::from(k.as_str())),
            JsValue::from(JsString::from(v.as_str())),
        ];
        let pair_array = boa_engine::object::builtins::JsArray::from_iter(pair, context);
        elements.push(JsValue::from(pair_array));
    }

    let array = boa_engine::object::builtins::JsArray::from_iter(elements, context);
    let values_fn = array.get(JsString::from("values"), context)?;
    let values_iterator = values_fn
        .as_callable()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Array.prototype.values not callable"))
        })?
        .call(&array.into(), &[], context)?;
    Ok(values_iterator)
}

#[cfg(test)]
mod tests {
    use super::super::BoaHost;
    use super::super::ScriptHost;

    #[test]
    fn test_headers_interface() {
        let mut host = BoaHost::new();

        // 1. Basic empty constructor
        host.eval(
            r#"{
                const h = new Headers();
                if (h.get("any") !== null) throw "new Headers() should be empty";
            }"#,
        )
        .unwrap();

        // 2. Constructor with object (record) and case-insensitive check and trimming
        host.eval(
            r#"{
                const h = new Headers({
                    "Content-Type": " text/html \r\n",
                    "X-Custom-Header": "foo"
                });
                if (h.get("content-type") !== "text/html") throw "Should trim and lowercase";
                if (h.get("x-custom-header") !== "foo") throw "Should retrieve custom header";
            }"#,
        )
        .unwrap();

        // 3. Constructor with sequence (array of arrays)
        host.eval(
            r#"{
                const h = new Headers([
                    ["a", "1"],
                    ["b", "2"],
                    ["a", "3"]
                ]);
                if (h.get("a") !== "1, 3") throw "Should combine duplicate keys";
                if (h.get("b") !== "2") throw "Should retrieve b";
            }"#,
        )
        .unwrap();

        // 4. Constructor with another Headers
        host.eval(
            r#"{
                const h1 = new Headers({ "a": "1" });
                const h2 = new Headers(h1);
                if (h2.get("a") !== "1") throw "Should clone other Headers";
            }"#,
        )
        .unwrap();

        // 5. Append combining, Set replacing, Has/Delete
        host.eval(
            r#"{
                const h = new Headers();
                h.append("a", "1");
                h.append("a", "2");
                if (h.get("a") !== "1, 2") throw "Append should combine: " + h.get("a");

                h.set("a", "x");
                if (h.get("a") !== "x") throw "Set should replace: " + h.get("a");

                if (h.has("a") !== true) throw "Has should be true";
                h.delete("a");
                if (h.has("a") !== false) throw "Has should be false after delete";
                if (h.get("a") !== null) throw "Get should be null after delete";
            }"#,
        )
        .unwrap();

        // 6. forEach visiting sorted names
        host.eval(
            r#"{
                const h = new Headers([
                    ["b", "2"],
                    ["a", "1"],
                    ["c", "3"]
                ]);
                let out = [];
                h.forEach((value, name) => {
                    out.push(name + "=" + value);
                });
                if (out.join(";") !== "a=1;b=2;c=3") throw "forEach ordering mismatch: " + out.join(";");
            }"#
        ).unwrap();

        // 7. keys(), values(), entries() and Symbol.iterator
        host.eval(
            r#"{
                const h = new Headers([
                    ["b", "2"],
                    ["a", "1"]
                ]);
                const keys = [...h.keys()];
                if (keys[0] !== "a" || keys[1] !== "b") throw "keys mismatch";

                const values = [...h.values()];
                if (values[0] !== "1" || values[1] !== "2") throw "values mismatch";

                const entries = [...h.entries()];
                if (entries[0][0] !== "a" || entries[0][1] !== "1" || entries[1][0] !== "b" || entries[1][1] !== "2") throw "entries mismatch";

                // Symbol.iterator
                const iterEntries = [...h];
                if (iterEntries[0][0] !== "a" || iterEntries[0][1] !== "1") throw "Symbol.iterator mismatch";
            }"#
        ).unwrap();
    }
}
