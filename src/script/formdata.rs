use boa_engine::class::{Class, ClassBuilder};
use boa_engine::{Context, JsError, JsString, JsValue, NativeFunction};
use boa_engine::{JsData, JsNativeError, JsResult};
use boa_gc::{Finalize, GcRefCell, Trace};

/// Implementation of WHATWG HTML `FormData` interface.
/// Spec: <https://xhr.spec.whatwg.org/#interface-formdata>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct FormData {
    pub(crate) entries: GcRefCell<Vec<(String, String)>>,
}

impl Class for FormData {
    const NAME: &'static str = "FormData";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        // // TODO(spec): Support optional HTMLFormElement argument
        Ok(FormData {
            entries: GcRefCell::new(Vec::new()),
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class
            .method(
                JsString::from("append"),
                2,
                NativeFunction::from_fn_ptr(form_data_append),
            )
            .method(
                JsString::from("set"),
                2,
                NativeFunction::from_fn_ptr(form_data_set),
            )
            .method(
                JsString::from("get"),
                1,
                NativeFunction::from_fn_ptr(form_data_get),
            )
            .method(
                JsString::from("getAll"),
                1,
                NativeFunction::from_fn_ptr(form_data_get_all),
            )
            .method(
                JsString::from("has"),
                1,
                NativeFunction::from_fn_ptr(form_data_has),
            )
            .method(
                JsString::from("delete"),
                1,
                NativeFunction::from_fn_ptr(form_data_delete),
            )
            .method(
                JsString::from("keys"),
                0,
                NativeFunction::from_fn_ptr(form_data_keys),
            )
            .method(
                JsString::from("values"),
                0,
                NativeFunction::from_fn_ptr(form_data_values),
            )
            .method(
                JsString::from("entries"),
                0,
                NativeFunction::from_fn_ptr(form_data_entries),
            );

        Ok(())
    }
}

pub fn form_data_append(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let form_data = obj.downcast_ref::<FormData>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-FormData object"))
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

    form_data.entries.borrow_mut().push((name, val));
    Ok(JsValue::undefined())
}

pub fn form_data_set(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let form_data = obj.downcast_ref::<FormData>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-FormData object"))
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

    let mut entries = form_data.entries.borrow_mut();
    entries.retain(|(k, _)| k != &name);
    entries.push((name, val));
    Ok(JsValue::undefined())
}

pub fn form_data_get(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let form_data = obj.downcast_ref::<FormData>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-FormData object"))
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let entries = form_data.entries.borrow();
    for (k, v) in entries.iter() {
        if k == &name {
            return Ok(JsValue::from(JsString::from(v.as_str())));
        }
    }

    Ok(JsValue::null())
}

pub fn form_data_get_all(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let form_data = obj.downcast_ref::<FormData>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-FormData object"))
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let entries = form_data.entries.borrow();
    let elements: Vec<JsValue> = entries
        .iter()
        .filter(|(k, _)| k == &name)
        .map(|(_, v)| JsValue::from(JsString::from(v.as_str())))
        .collect();

    let array = boa_engine::object::builtins::JsArray::from_iter(elements, context);
    Ok(JsValue::from(array))
}

pub fn form_data_has(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let form_data = obj.downcast_ref::<FormData>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-FormData object"))
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let entries = form_data.entries.borrow();
    let has_key = entries.iter().any(|(k, _)| k == &name);
    Ok(JsValue::from(has_key))
}

pub fn form_data_delete(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let form_data = obj.downcast_ref::<FormData>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-FormData object"))
    })?;

    let name = args
        .first()
        .map(|v| v.to_string(context))
        .transpose()?
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    form_data.entries.borrow_mut().retain(|(k, _)| k != &name);
    Ok(JsValue::undefined())
}

pub fn form_data_keys(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let form_data = obj.downcast_ref::<FormData>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-FormData object"))
    })?;

    // // TODO(spec): Use live iterators instead of static arrays
    let entries = form_data.entries.borrow();
    let elements: Vec<JsValue> = entries
        .iter()
        .map(|(k, _)| JsValue::from(JsString::from(k.as_str())))
        .collect();

    let array = boa_engine::object::builtins::JsArray::from_iter(elements, context);
    Ok(JsValue::from(array))
}

pub fn form_data_values(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let form_data = obj.downcast_ref::<FormData>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-FormData object"))
    })?;

    // // TODO(spec): Use live iterators instead of static arrays
    let entries = form_data.entries.borrow();
    let elements: Vec<JsValue> = entries
        .iter()
        .map(|(_, v)| JsValue::from(JsString::from(v.as_str())))
        .collect();

    let array = boa_engine::object::builtins::JsArray::from_iter(elements, context);
    Ok(JsValue::from(array))
}

pub fn form_data_entries(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let form_data = obj.downcast_ref::<FormData>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-FormData object"))
    })?;

    // // TODO(spec): Use live iterators instead of static arrays
    let entries = form_data.entries.borrow();
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
    Ok(JsValue::from(array))
}

#[cfg(test)]
mod tests {
    use super::super::BoaHost;
    use super::super::ScriptHost;

    #[test]
    fn test_formdata_t0502() {
        let mut host = BoaHost::new();

        host.eval(
            r#"{
            const fd = new FormData();
            if (fd.has("name") !== false) throw "has(name) should be false initially";

            fd.append("name", "Alice");
            fd.append("name", "Bob");
            fd.append("age", 30); // test coercion to string

            if (fd.has("name") !== true) throw "has(name) should be true after append";
            if (fd.get("name") !== "Alice") throw "get(name) mismatch: " + fd.get("name");
            if (fd.get("age") !== "30") throw "get(age) coercion mismatch: " + fd.get("age");

            const nameAll = fd.getAll("name");
            if (nameAll.length !== 2 || nameAll[0] !== "Alice" || nameAll[1] !== "Bob") {
                throw "getAll(name) mismatch: " + JSON.stringify(nameAll);
            }

            // Test set
            fd.set("name", "Charlie");
            if (fd.get("name") !== "Charlie") throw "set(name) mismatch: " + fd.get("name");
            const charlieAll = fd.getAll("name");
            if (charlieAll.length !== 1 || charlieAll[0] !== "Charlie") {
                throw "set(name) getAll mismatch: " + JSON.stringify(charlieAll);
            }

            // Test keys, values, entries
            const keys = fd.keys();
            if (keys.length !== 2 || keys[0] !== "age" || keys[1] !== "name") {
                throw "keys() mismatch: " + JSON.stringify(keys);
            }

            const values = fd.values();
            if (values.length !== 2 || values[0] !== "30" || values[1] !== "Charlie") {
                throw "values() mismatch: " + JSON.stringify(values);
            }

            const entries = fd.entries();
            if (entries.length !== 2 || entries[0][0] !== "age" || entries[0][1] !== "30" || entries[1][0] !== "name" || entries[1][1] !== "Charlie") {
                throw "entries() mismatch: " + JSON.stringify(entries);
            }

            // Test delete
            fd.delete("age");
            if (fd.has("age") !== false) throw "delete(age) failed";
            if (fd.get("age") !== null) throw "get(age) after delete should be null";
        }"#
        ).unwrap();
    }
}
