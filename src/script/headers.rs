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

fn is_header_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    name.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(
                c,
                '!' | '#'
                    | '$'
                    | '%'
                    | '&'
                    | '\''
                    | '*'
                    | '+'
                    | '-'
                    | '.'
                    | '^'
                    | '_'
                    | '`'
                    | '|'
                    | '~'
            )
    })
}

fn is_header_value(val: &str) -> bool {
    val.chars().all(|c| {
        let code = c as u32;
        code <= 0xFF && code != 0x00 && code != 0x0A && code != 0x0D
    })
}

impl Headers {
    fn get_iteration_entries(&self) -> Vec<(String, String)> {
        let pairs = self.pairs.borrow();
        // 1. Get unique keys in their first occurrence order, then sort them lexicographically
        let mut unique_keys = Vec::new();
        for (k, _) in pairs.iter() {
            if !unique_keys.contains(k) {
                unique_keys.push(k.clone());
            }
        }
        unique_keys.sort();

        // 2. Build the iteration entries
        let mut result = Vec::new();
        for key in unique_keys {
            if key == "set-cookie" {
                for (k, v) in pairs.iter() {
                    if k == &key {
                        result.push((key.clone(), v.clone()));
                    }
                }
            } else {
                let matched_values: Vec<&str> = pairs
                    .iter()
                    .filter(|(k, _)| k == &key)
                    .map(|(_, v)| v.as_str())
                    .collect();
                let combined = matched_values.join(", ");
                result.push((key, combined));
            }
        }
        result
    }
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
                                let length_val = item_obj.get(JsString::from("length"), context)?;
                                let inner_len = length_val.as_number().unwrap_or(0.0) as usize;
                                if inner_len != 2 {
                                    return Err(JsError::from(
                                        JsNativeError::typ()
                                            .with_message("Header pair must have a length of 2"),
                                    ));
                                }

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

                                if !is_header_name(&name) {
                                    return Err(JsError::from(JsNativeError::typ().with_message(
                                        format!("Invalid header name: '{}'", name),
                                    )));
                                }

                                let normalized_name = name.to_ascii_lowercase();
                                let normalized_value = normalize_value(&value);

                                if !is_header_value(&normalized_value) {
                                    return Err(JsError::from(JsNativeError::typ().with_message(
                                        format!("Invalid header value: '{}'", value),
                                    )));
                                }

                                pairs.push((normalized_name, normalized_value));
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

                            if !is_header_name(&key_str) {
                                return Err(JsError::from(
                                    JsNativeError::typ().with_message(format!(
                                        "Invalid header name: '{}'",
                                        key_str
                                    )),
                                ));
                            }

                            let val_val = obj.get(JsString::from(key_str.as_str()), context)?;
                            let val_str = val_val
                                .to_string(context)?
                                .to_std_string()
                                .unwrap_or_default();

                            let normalized_name = key_str.to_ascii_lowercase();
                            let normalized_value = normalize_value(&val_str);

                            if !is_header_value(&normalized_value) {
                                return Err(JsError::from(JsNativeError::typ().with_message(
                                    format!("Invalid header value: '{}'", val_str),
                                )));
                            }

                            pairs.push((normalized_name, normalized_value));
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
                JsString::from("getSetCookie"),
                0,
                NativeFunction::from_fn_ptr(headers_get_set_cookie),
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

    if !is_header_name(&name) {
        return Err(JsError::from(
            JsNativeError::typ().with_message(format!("Invalid header name: '{}'", name)),
        ));
    }

    let normalized_name = name.to_ascii_lowercase();
    let normalized_value = normalize_value(&val);

    if !is_header_value(&normalized_value) {
        return Err(JsError::from(
            JsNativeError::typ().with_message(format!("Invalid header value: '{}'", val)),
        ));
    }

    headers
        .pairs
        .borrow_mut()
        .push((normalized_name, normalized_value));

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

    if !is_header_name(&name) {
        return Err(JsError::from(
            JsNativeError::typ().with_message(format!("Invalid header name: '{}'", name)),
        ));
    }

    let normalized_name = name.to_ascii_lowercase();
    let normalized_value = normalize_value(&val);

    if !is_header_value(&normalized_value) {
        return Err(JsError::from(
            JsNativeError::typ().with_message(format!("Invalid header value: '{}'", val)),
        ));
    }

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

    if !is_header_name(&name) {
        return Err(JsError::from(
            JsNativeError::typ().with_message(format!("Invalid header name: '{}'", name)),
        ));
    }

    let normalized_name = name.to_ascii_lowercase();

    let pairs = headers.pairs.borrow();
    let matched_values: Vec<&str> = pairs
        .iter()
        .filter(|(k, _)| k == &normalized_name)
        .map(|(_, v)| v.as_str())
        .collect();

    if matched_values.is_empty() {
        Ok(JsValue::null())
    } else {
        let joined = matched_values.join(", ");
        Ok(JsValue::from(JsString::from(joined)))
    }
}

pub fn headers_get_set_cookie(
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

    let pairs = headers.pairs.borrow();
    let elements: Vec<JsValue> = pairs
        .iter()
        .filter(|(k, _)| k == "set-cookie")
        .map(|(_, v)| JsValue::from(JsString::from(v.as_str())))
        .collect();

    let array = boa_engine::object::builtins::JsArray::from_iter(elements, context);
    Ok(JsValue::from(array))
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

    if !is_header_name(&name) {
        return Err(JsError::from(
            JsNativeError::typ().with_message(format!("Invalid header name: '{}'", name)),
        ));
    }

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

    if !is_header_name(&name) {
        return Err(JsError::from(
            JsNativeError::typ().with_message(format!("Invalid header name: '{}'", name)),
        ));
    }

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

    let entries = headers.get_iteration_entries();

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

    let entries = headers.get_iteration_entries();

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

    let entries = headers.get_iteration_entries();

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

    let entries = headers.get_iteration_entries();

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

        // 8. getSetCookie and Set-Cookie duplicate-preserving iteration behavior (t0792)
        host.eval(
            r#"{
                const h = new Headers([
                    ["Set-Cookie", "a=1"],
                    ["Set-Cookie", "b=2"],
                    ["X-Custom", "x1"],
                    ["X-Custom", "x2"]
                ]);
                const cookies = h.getSetCookie();
                if (cookies.length !== 2) throw "getSetCookie length mismatch";
                if (cookies[0] !== "a=1" || cookies[1] !== "b=2") throw "getSetCookie values mismatch";

                // Set-Cookie should yield individual entries in iterator and forEach,
                // but X-Custom should combine!
                const entries = [...h.entries()];
                if (entries.length !== 3) throw "entries length mismatch (expected 3)";
                
                // Sorted alphabetically: Set-Cookie vs X-Custom
                // "set-cookie" vs "x-custom" -> "set-cookie" is first
                if (entries[0][0] !== "set-cookie" || entries[0][1] !== "a=1") throw "First entry mismatch";
                if (entries[1][0] !== "set-cookie" || entries[1][1] !== "b=2") throw "Second entry mismatch";
                if (entries[2][0] !== "x-custom" || entries[2][1] !== "x1, x2") throw "Third entry mismatch";

                // has and get behavior on Set-Cookie
                if (!h.has("set-cookie")) throw "has('set-cookie') should be true";
                if (h.get("set-cookie") !== "a=1, b=2") throw "get('set-cookie') should combine";

                // delete works on Set-Cookie
                h.delete("set-cookie");
                if (h.has("set-cookie")) throw "has('set-cookie') should be false after delete";
                if (h.getSetCookie().length !== 0) throw "getSetCookie should be empty after delete";
            }"#
        ).unwrap();

        // 9. Validation of header names and values
        host.eval(
            r#"{
                // Invalid names
                try {
                    new Headers({ "Invalid:": "val" });
                    throw "Should throw on invalid header name in object key";
                } catch (e) {
                    if (!(e instanceof TypeError)) throw "Expected TypeError for invalid key, got: " + e;
                }

                try {
                    new Headers([["Invalid:", "val"]]);
                    throw "Should throw on invalid header name in sequence key";
                } catch (e) {
                    if (!(e instanceof TypeError)) throw "Expected TypeError for invalid sequence key, got: " + e;
                }

                try {
                    const h = new Headers();
                    h.append("Invalid:", "val");
                    throw "Should throw on append with invalid header name";
                } catch (e) {
                    if (!(e instanceof TypeError)) throw "Expected TypeError for append invalid name, got: " + e;
                }

                try {
                    const h = new Headers();
                    h.get("Invalid:");
                    throw "Should throw on get with invalid header name";
                } catch (e) {
                    if (!(e instanceof TypeError)) throw "Expected TypeError for get invalid name, got: " + e;
                }

                // Invalid values (non-ByteString or containing CR/LF/NUL inside)
                try {
                    const h = new Headers();
                    h.append("ok", "val\x00withNull");
                    throw "Should throw on append with NUL in value";
                } catch (e) {
                    if (!(e instanceof TypeError)) throw "Expected TypeError for NUL, got: " + e;
                }

                try {
                    const h = new Headers();
                    h.append("ok", "val\nwithLF");
                    throw "Should throw on append with LF in value";
                } catch (e) {
                    if (!(e instanceof TypeError)) throw "Expected TypeError for LF, got: " + e;
                }

                try {
                    const h = new Headers();
                    h.append("ok", "unicode\u0100");
                    throw "Should throw on append with code point > 255";
                } catch (e) {
                    if (!(e instanceof TypeError)) throw "Expected TypeError for >255, got: " + e;
                }

                // Sequence length validation
                try {
                    new Headers([["a"]]);
                    throw "Should throw on 1-element sequence";
                } catch (e) {
                    if (!(e instanceof TypeError)) throw "Expected TypeError for 1-element sequence, got: " + e;
                }

                try {
                    new Headers([["a", "b", "c"]]);
                    throw "Should throw on 3-element sequence";
                } catch (e) {
                    if (!(e instanceof TypeError)) throw "Expected TypeError for 3-element sequence, got: " + e;
                }
            }"#
        ).unwrap();
    }
}
