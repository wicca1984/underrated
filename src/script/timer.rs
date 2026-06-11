use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsString, JsValue, NativeFunction, Source};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Timer {
    pub id: i32,
    pub delay: i32,
    pub is_interval: bool,
}

thread_local! {
    static NEXT_TIMER_ID: RefCell<i32> = const { RefCell::new(1) };
    static TIMERS: RefCell<HashMap<i32, Timer>> = RefCell::new(HashMap::new());
}

/// Clear all timers and reset the ID counter (mainly for test isolation).
pub fn clear_all_timers() {
    NEXT_TIMER_ID.with(|cell| {
        *cell.borrow_mut() = 1;
    });
    TIMERS.with(|cell| {
        cell.borrow_mut().clear();
    });
}

/// Get a copy of a registered timer by its ID.
pub fn get_timer(id: i32) -> Option<Timer> {
    TIMERS.with(|cell| cell.borrow().get(&id).cloned())
}

/// Get the count of active registered timers.
pub fn get_timer_count() -> usize {
    TIMERS.with(|cell| cell.borrow().len())
}

fn get_or_create_timers_obj(
    context: &mut Context,
) -> Result<boa_engine::object::JsObject, JsError> {
    let global_obj = context.global_object().clone();
    let timers_prop = JsString::from("__timers__");
    let timers_val = global_obj.get(timers_prop.clone(), context)?;
    if timers_val.is_undefined() || timers_val.is_null() {
        let new_obj = ObjectInitializer::new(context).build();
        global_obj.set(timers_prop, JsValue::from(new_obj.clone()), false, context)?;
        Ok(new_obj)
    } else {
        timers_val.as_object().ok_or_else(|| {
            JsError::from_opaque(JsValue::from(JsString::from("__timers__ is not an object")))
        })
    }
}

/// Trigger a timer callback manually (for testing or event loop MVP).
/// If it is a timeout (is_interval == false), it is removed from the collection.
/// If it is an interval (is_interval == true), it remains in the collection.
pub fn trigger_timer(id: i32, context: &mut Context) -> Result<JsValue, JsError> {
    let rust_timer = TIMERS.with(|cell| {
        let mut timers = cell.borrow_mut();
        if let Some(t) = timers.get(&id) {
            let t_clone = t.clone();
            if !t.is_interval {
                timers.remove(&id);
            }
            Some(t_clone)
        } else {
            None
        }
    });

    let is_interval = if let Some(t) = rust_timer {
        t.is_interval
    } else {
        return Err(JsError::from_opaque(JsValue::from(JsString::from(
            "Timer not found",
        ))));
    };

    let timers_obj = get_or_create_timers_obj(context)?;
    let timer_info_val = timers_obj.get(id, context)?;
    if timer_info_val.is_undefined() || timer_info_val.is_null() {
        return Err(JsError::from_opaque(JsValue::from(JsString::from(
            "Timer info not found in JS state",
        ))));
    }

    let timer_info_obj = timer_info_val.as_object().ok_or_else(|| {
        JsError::from_opaque(JsValue::from(JsString::from("Timer info is not an object")))
    })?;

    // If it is a timeout, clean up the JS-side state as well
    if !is_interval {
        let _ = timers_obj.delete_property_or_throw(id, context)?;
    }

    let callback = timer_info_obj.get(JsString::from("callback"), context)?;
    let args_val = timer_info_obj.get(JsString::from("args"), context)?;

    let mut callback_args = Vec::new();
    if let Some(args_arr_obj) = args_val.as_object() {
        let length_val = args_arr_obj.get(JsString::from("length"), context)?;
        let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);
        for i in 0..length {
            let arg_val = args_arr_obj.get(i, context)?;
            callback_args.push(arg_val);
        }
    }

    if let Some(callback_obj) = callback.as_object() {
        let global_this = context.global_object().clone();
        callback_obj.call(&JsValue::from(global_this), &callback_args, context)
    } else if callback.is_string() {
        let code_str = callback.to_string(context)?;
        let std_str = code_str.to_std_string().unwrap_or_default();
        let source = Source::from_bytes(std_str.as_bytes());
        context.eval(source)
    } else {
        // No-op or ignore non-callable callbacks gracefully
        Ok(JsValue::undefined())
    }
}

fn js_value_to_i32(val: &JsValue, context: &mut Context) -> i32 {
    if let Some(num) = val.as_number() {
        num as i32
    } else if let Ok(s) = val.to_string(context) {
        s.to_std_string()
            .unwrap_or_default()
            .parse::<i32>()
            .unwrap_or(0)
    } else {
        0
    }
}

pub fn set_timeout(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    let delay = if let Some(delay_val) = args.get(1) {
        js_value_to_i32(delay_val, context)
    } else {
        0
    };

    let callback_args = if args.len() > 2 {
        args[2..].to_vec()
    } else {
        Vec::new()
    };

    let id = NEXT_TIMER_ID.with(|cell| {
        let mut next_id = cell.borrow_mut();
        let cur = *next_id;
        *next_id += 1;
        cur
    });

    // Store in Rust side
    let timer = Timer {
        id,
        delay,
        is_interval: false,
    };
    TIMERS.with(|cell| {
        cell.borrow_mut().insert(id, timer);
    });

    // Store callback and args in JS side __timers__ object
    let timers_obj = get_or_create_timers_obj(context)?;

    // Create a JS array for arguments
    let array_constructor = context
        .global_object()
        .get(JsString::from("Array"), context)?;
    let array_obj = array_constructor.as_object().ok_or_else(|| {
        JsError::from_opaque(JsValue::from(JsString::from("Array constructor not found")))
    })?;
    let args_val = array_obj.construct(&[], None, context)?;
    let push_fn = args_val.get(JsString::from("push"), context)?;
    if let Some(push_obj) = push_fn.as_object() {
        for arg in callback_args {
            push_obj.call(&JsValue::from(args_val.clone()), &[arg], context)?;
        }
    }

    let timer_info = ObjectInitializer::new(context)
        .property(JsString::from("callback"), callback, Attribute::all())
        .property(
            JsString::from("args"),
            JsValue::from(args_val),
            Attribute::all(),
        )
        .build();

    timers_obj.set(id, JsValue::from(timer_info), false, context)?;

    Ok(JsValue::from(id))
}

pub fn clear_timeout(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    if let Some(id_val) = args.first() {
        let id = js_value_to_i32(id_val, context);
        // Remove from Rust side
        TIMERS.with(|cell| {
            cell.borrow_mut().remove(&id);
        });
        // Remove from JS side
        if let Ok(timers_obj) = get_or_create_timers_obj(context) {
            let _ = timers_obj.delete_property_or_throw(id, context)?;
        }
    }
    Ok(JsValue::undefined())
}

pub fn set_interval(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    let delay = if let Some(delay_val) = args.get(1) {
        js_value_to_i32(delay_val, context)
    } else {
        0
    };

    let callback_args = if args.len() > 2 {
        args[2..].to_vec()
    } else {
        Vec::new()
    };

    let id = NEXT_TIMER_ID.with(|cell| {
        let mut next_id = cell.borrow_mut();
        let cur = *next_id;
        *next_id += 1;
        cur
    });

    // Store in Rust side
    let timer = Timer {
        id,
        delay,
        is_interval: true,
    };
    TIMERS.with(|cell| {
        cell.borrow_mut().insert(id, timer);
    });

    // Store callback and args in JS side __timers__ object
    let timers_obj = get_or_create_timers_obj(context)?;

    // Create a JS array for arguments
    let array_constructor = context
        .global_object()
        .get(JsString::from("Array"), context)?;
    let array_obj = array_constructor.as_object().ok_or_else(|| {
        JsError::from_opaque(JsValue::from(JsString::from("Array constructor not found")))
    })?;
    let args_val = array_obj.construct(&[], None, context)?;
    let push_fn = args_val.get(JsString::from("push"), context)?;
    if let Some(push_obj) = push_fn.as_object() {
        for arg in callback_args {
            push_obj.call(&JsValue::from(args_val.clone()), &[arg], context)?;
        }
    }

    let timer_info = ObjectInitializer::new(context)
        .property(JsString::from("callback"), callback, Attribute::all())
        .property(
            JsString::from("args"),
            JsValue::from(args_val),
            Attribute::all(),
        )
        .build();

    timers_obj.set(id, JsValue::from(timer_info), false, context)?;

    Ok(JsValue::from(id))
}

pub fn clear_interval(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    if let Some(id_val) = args.first() {
        let id = js_value_to_i32(id_val, context);
        // Remove from Rust side
        TIMERS.with(|cell| {
            cell.borrow_mut().remove(&id);
        });
        // Remove from JS side
        if let Ok(timers_obj) = get_or_create_timers_obj(context) {
            let _ = timers_obj.delete_property_or_throw(id, context)?;
        }
    }
    Ok(JsValue::undefined())
}

pub fn register_timer_builtins(context: &mut Context) -> Result<(), JsError> {
    context.register_global_builtin_callable(
        JsString::from("setTimeout"),
        2,
        NativeFunction::from_fn_ptr(set_timeout),
    )?;
    context.register_global_builtin_callable(
        JsString::from("clearTimeout"),
        1,
        NativeFunction::from_fn_ptr(clear_timeout),
    )?;
    context.register_global_builtin_callable(
        JsString::from("setInterval"),
        2,
        NativeFunction::from_fn_ptr(set_interval),
    )?;
    context.register_global_builtin_callable(
        JsString::from("clearInterval"),
        1,
        NativeFunction::from_fn_ptr(clear_interval),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_timeout_registers_and_increments_id() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        let source1 = Source::from_bytes(r#"setTimeout(() => {}, 100)"#);
        let id1_val = context.eval(source1).unwrap();
        let id1 = id1_val.as_number().unwrap() as i32;
        assert_eq!(id1, 1);
        assert_eq!(get_timer_count(), 1);

        let source2 = Source::from_bytes(r#"setTimeout(() => {}, 200)"#);
        let id2_val = context.eval(source2).unwrap();
        let id2 = id2_val.as_number().unwrap() as i32;
        assert_eq!(id2, 2);
        assert_eq!(get_timer_count(), 2);

        let t1 = get_timer(1).unwrap();
        assert_eq!(t1.delay, 100);
        assert!(!t1.is_interval);

        let t2 = get_timer(2).unwrap();
        assert_eq!(t2.delay, 200);
        assert!(!t2.is_interval);
    }

    #[test]
    fn test_clear_timeout_removes_timer() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        let id_val = context
            .eval(Source::from_bytes(r#"setTimeout(() => {}, 100)"#))
            .unwrap();
        let id = id_val.as_number().unwrap() as i32;
        assert_eq!(get_timer_count(), 1);

        context
            .eval(Source::from_bytes(
                format!("clearTimeout({})", id).as_bytes(),
            ))
            .unwrap();
        assert_eq!(get_timer_count(), 0);
        assert!(get_timer(id).is_none());
    }

    #[test]
    fn test_trigger_timer_executes_callback() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        context
            .eval(Source::from_bytes(
                r#"
            var x = 0;
            var timerId = setTimeout(() => { x = 42; }, 100);
        "#,
            ))
            .unwrap();

        let id_val = context.eval(Source::from_bytes("timerId")).unwrap();
        let id = id_val.as_number().unwrap() as i32;

        let result = trigger_timer(id, &mut context);
        assert!(result.is_ok());

        let x_val = context.eval(Source::from_bytes("x")).unwrap();
        assert_eq!(x_val.as_number().unwrap() as i32, 42);

        // Since it's a timeout, triggering it should remove it from the map
        assert_eq!(get_timer_count(), 0);
    }

    #[test]
    fn test_trigger_timer_with_args() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        context
            .eval(Source::from_bytes(
                r#"
            var x = 0;
            var timerId = setTimeout((a, b) => { x = a + b; }, 100, 10, 32);
        "#,
            ))
            .unwrap();

        let id_val = context.eval(Source::from_bytes("timerId")).unwrap();
        let id = id_val.as_number().unwrap() as i32;

        let result = trigger_timer(id, &mut context);
        assert!(result.is_ok());

        let x_val = context.eval(Source::from_bytes("x")).unwrap();
        assert_eq!(x_val.as_number().unwrap() as i32, 42);
    }

    #[test]
    fn test_trigger_timer_string_callback() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        context
            .eval(Source::from_bytes(
                r#"
            var x = 0;
            var timerId = setTimeout("x = 99", 100);
        "#,
            ))
            .unwrap();

        let id_val = context.eval(Source::from_bytes("timerId")).unwrap();
        let id = id_val.as_number().unwrap() as i32;

        let result = trigger_timer(id, &mut context);
        assert!(result.is_ok());

        let x_val = context.eval(Source::from_bytes("x")).unwrap();
        assert_eq!(x_val.as_number().unwrap() as i32, 99);
    }

    #[test]
    fn test_set_interval_and_clear_interval() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        context
            .eval(Source::from_bytes(
                r#"
            var x = 0;
            var timerId = setInterval(() => { x += 1; }, 100);
        "#,
            ))
            .unwrap();

        let id_val = context.eval(Source::from_bytes("timerId")).unwrap();
        let id = id_val.as_number().unwrap() as i32;

        assert_eq!(get_timer_count(), 1);
        let timer = get_timer(id).unwrap();
        assert!(timer.is_interval);

        // Triggering an interval should run it and keep it in the collection
        trigger_timer(id, &mut context).unwrap();
        assert_eq!(get_timer_count(), 1);

        let x_val = context.eval(Source::from_bytes("x")).unwrap();
        assert_eq!(x_val.as_number().unwrap() as i32, 1);

        trigger_timer(id, &mut context).unwrap();
        let x_val_2 = context.eval(Source::from_bytes("x")).unwrap();
        assert_eq!(x_val_2.as_number().unwrap() as i32, 2);

        // Now clear it
        context
            .eval(Source::from_bytes(
                format!("clearInterval({})", id).as_bytes(),
            ))
            .unwrap();
        assert_eq!(get_timer_count(), 0);
    }
}
