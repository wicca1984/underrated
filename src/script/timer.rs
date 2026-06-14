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
    pub nesting_level: u32,
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Timer {}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Clone, Debug)]
pub struct AnimationFrame {
    pub id: i32,
}

#[derive(Clone, Debug)]
pub struct IdleCallback {
    pub id: i32,
    pub timeout: Option<u32>,
}

thread_local! {
    static NEXT_TIMER_ID: RefCell<i32> = const { RefCell::new(1) };
    static TIMERS: RefCell<HashMap<i32, Timer>> = RefCell::new(HashMap::new());
    static NEXT_RAF_ID: RefCell<i32> = const { RefCell::new(1) };
    static ANIMATION_FRAMES: RefCell<HashMap<i32, AnimationFrame>> = RefCell::new(HashMap::new());
    static NEXT_IDLE_ID: RefCell<i32> = const { RefCell::new(1) };
    static IDLE_CALLBACKS: RefCell<HashMap<i32, IdleCallback>> = RefCell::new(HashMap::new());
    static MICROTASK_QUEUE: RefCell<std::collections::VecDeque<JsValue>> = const { RefCell::new(std::collections::VecDeque::new()) };
    static CURRENT_NESTING_LEVEL: RefCell<u32> = const { RefCell::new(0) };
}

/// Clear all timers and reset the ID counter (mainly for test isolation).
pub fn clear_all_timers() {
    NEXT_TIMER_ID.with(|cell| {
        *cell.borrow_mut() = 1;
    });
    TIMERS.with(|cell| {
        cell.borrow_mut().clear();
    });
    NEXT_RAF_ID.with(|cell| {
        *cell.borrow_mut() = 1;
    });
    ANIMATION_FRAMES.with(|cell| {
        cell.borrow_mut().clear();
    });
    NEXT_IDLE_ID.with(|cell| {
        *cell.borrow_mut() = 1;
    });
    IDLE_CALLBACKS.with(|cell| {
        cell.borrow_mut().clear();
    });
    CURRENT_NESTING_LEVEL.with(|cell| {
        *cell.borrow_mut() = 0;
    });
}

/// Clear all microtasks (mainly for test isolation).
pub fn clear_all_microtasks() {
    MICROTASK_QUEUE.with(|cell| {
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

/// Get the list of all active registered timers, sorted by their ID (handle).
pub fn get_active_timers_sorted() -> Vec<Timer> {
    TIMERS.with(|cell| {
        let mut timers: Vec<Timer> = cell.borrow().values().cloned().collect();
        timers.sort();
        timers
    })
}

/// Get the count of active registered animation frames.
pub fn get_animation_frame_count() -> usize {
    ANIMATION_FRAMES.with(|cell| cell.borrow().len())
}

/// Get the count of active registered idle callbacks.
pub fn get_idle_callback_count() -> usize {
    IDLE_CALLBACKS.with(|cell| cell.borrow().len())
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

fn get_or_create_animation_frames_obj(
    context: &mut Context,
) -> Result<boa_engine::object::JsObject, JsError> {
    let global_obj = context.global_object().clone();
    let raf_prop = JsString::from("__animation_frames__");
    let raf_val = global_obj.get(raf_prop.clone(), context)?;
    if raf_val.is_undefined() || raf_val.is_null() {
        let new_obj = ObjectInitializer::new(context).build();
        global_obj.set(raf_prop, JsValue::from(new_obj.clone()), false, context)?;
        Ok(new_obj)
    } else {
        raf_val.as_object().ok_or_else(|| {
            JsError::from_opaque(JsValue::from(JsString::from(
                "__animation_frames__ is not an object",
            )))
        })
    }
}

fn get_or_create_idle_callbacks_obj(
    context: &mut Context,
) -> Result<boa_engine::object::JsObject, JsError> {
    let global_obj = context.global_object().clone();
    let idle_prop = JsString::from("__idle_callbacks__");
    let idle_val = global_obj.get(idle_prop.clone(), context)?;
    if idle_val.is_undefined() || idle_val.is_null() {
        let new_obj = ObjectInitializer::new(context).build();
        global_obj.set(idle_prop, JsValue::from(new_obj.clone()), false, context)?;
        Ok(new_obj)
    } else {
        idle_val.as_object().ok_or_else(|| {
            JsError::from_opaque(JsValue::from(JsString::from(
                "__idle_callbacks__ is not an object",
            )))
        })
    }
}

/// Trigger a timer callback manually (for testing or event loop MVP).
/// If it is a timeout (is_interval == false), it is removed from the collection.
/// If it is an interval (is_interval == true), it remains in the collection.
pub fn trigger_timer(id: i32, context: &mut Context) -> Result<JsValue, JsError> {
    let rust_timer = TIMERS.with(|cell| {
        let mut timers = cell.borrow_mut();
        if let Some(t) = timers.get_mut(&id) {
            let t_clone = t.clone();
            if !t.is_interval {
                timers.remove(&id);
            } else {
                t.nesting_level += 1;
                if t.nesting_level > 5 && t.delay < 4 {
                    t.delay = 4;
                }
            }
            Some(t_clone)
        } else {
            None
        }
    });

    let (is_interval, nesting_level) = if let Some(t) = rust_timer {
        (t.is_interval, t.nesting_level)
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

    // Temporarily set CURRENT_NESTING_LEVEL to nesting_level during callback execution
    struct NestingLevelGuard {
        old_level: u32,
    }
    impl Drop for NestingLevelGuard {
        fn drop(&mut self) {
            CURRENT_NESTING_LEVEL.with(|cell| {
                *cell.borrow_mut() = self.old_level;
            });
        }
    }

    let res = {
        let _guard = CURRENT_NESTING_LEVEL.with(|cell| {
            let mut borrow = cell.borrow_mut();
            let old_level = *borrow;
            *borrow = nesting_level;
            NestingLevelGuard { old_level }
        });

        if callback.is_callable() {
            if let Some(callback_obj) = callback.as_object() {
                let global_this = context.global_object().clone();
                callback_obj.call(&JsValue::from(global_this), &callback_args, context)
            } else {
                Ok(JsValue::undefined())
            }
        } else {
            // Per WebIDL TimerHandler DOMString/Function rules, convert to string and evaluate.
            // This ensures non-callable objects (e.g. { toString() { return "code"; } }) are correctly evaluated.
            match callback.to_string(context) {
                Ok(code_str) => {
                    let std_str = code_str.to_std_string().unwrap_or_default();
                    let source = Source::from_bytes(std_str.as_bytes());
                    context.eval(source)
                }
                Err(err) => Err(err),
            }
        }
    };

    let drain_res = drain_microtasks(context);

    match res {
        Err(err) => Err(err),
        Ok(val) => drain_res.map(|_| val),
    }
}

fn get_current_performance_timestamp(context: &mut Context) -> Result<JsValue, JsError> {
    let perf_opt = context
        .global_object()
        .get(JsString::from("performance"), context)
        .ok()
        .and_then(|val| val.as_object());

    if let Some(perf_obj) = perf_opt {
        let now_opt = perf_obj
            .get(JsString::from("now"), context)
            .ok()
            .and_then(|val| val.as_object());
        if let Some(now_fn) = now_opt {
            return now_fn
                .call(&JsValue::from(perf_obj), &[], context)
                .or_else(|_| Ok(JsValue::from(16.0)));
        }
    }
    Ok(JsValue::from(16.0))
}

/// Trigger all currently registered animation frame callbacks.
/// In accordance with the spec, all callbacks run in this frame must receive the exact same timestamp.
pub fn trigger_all_animation_frames(
    context: &mut Context,
) -> Result<Vec<Result<JsValue, JsError>>, JsError> {
    let ids: Vec<i32> = ANIMATION_FRAMES.with(|cell| {
        let keys = cell.borrow().keys().cloned().collect::<Vec<i32>>();
        let mut sorted = keys;
        sorted.sort();
        sorted
    });

    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let timestamp = get_current_performance_timestamp(context)?;
    let mut results = Vec::new();

    for id in ids {
        let exists = ANIMATION_FRAMES.with(|cell| cell.borrow_mut().remove(&id).is_some());
        if !exists {
            continue;
        }

        let raf_obj = get_or_create_animation_frames_obj(context)?;
        let frame_info_val = raf_obj.get(id, context)?;
        if frame_info_val.is_undefined() || frame_info_val.is_null() {
            continue;
        }

        if let Some(frame_info_obj) = frame_info_val.as_object() {
            let _ = raf_obj.delete_property_or_throw(id, context)?;

            let callback = frame_info_obj.get(JsString::from("callback"), context)?;
            let callback_args = vec![timestamp.clone()];

            let res = if let Some(callback_obj) = callback.as_object() {
                let global_this = context.global_object().clone();
                callback_obj.call(&JsValue::from(global_this), &callback_args, context)
            } else if callback.is_string() {
                match callback.to_string(context) {
                    Ok(code_str) => {
                        let std_str = code_str.to_std_string().unwrap_or_default();
                        let source = Source::from_bytes(std_str.as_bytes());
                        context.eval(source)
                    }
                    Err(err) => Err(err),
                }
            } else {
                Ok(JsValue::undefined())
            };

            let drain_res = drain_microtasks(context);

            let final_res = match res {
                Err(err) => Err(err),
                Ok(val) => drain_res.map(|_| val),
            };

            results.push(final_res);
        }
    }

    Ok(results)
}

/// Trigger an animation frame callback manually (for testing or event loop MVP).
/// Always removes the callback since requestAnimationFrame is one-shot.
pub fn trigger_animation_frame(id: i32, context: &mut Context) -> Result<JsValue, JsError> {
    let exists = ANIMATION_FRAMES.with(|cell| cell.borrow_mut().remove(&id).is_some());

    if !exists {
        return Err(JsError::from_opaque(JsValue::from(JsString::from(
            "Animation frame not found",
        ))));
    }

    let raf_obj = get_or_create_animation_frames_obj(context)?;
    let frame_info_val = raf_obj.get(id, context)?;
    if frame_info_val.is_undefined() || frame_info_val.is_null() {
        return Err(JsError::from_opaque(JsValue::from(JsString::from(
            "Animation frame info not found in JS state",
        ))));
    }

    let frame_info_obj = frame_info_val.as_object().ok_or_else(|| {
        JsError::from_opaque(JsValue::from(JsString::from(
            "Animation frame info is not an object",
        )))
    })?;

    // Clean up the JS-side state
    let _ = raf_obj.delete_property_or_throw(id, context)?;

    let callback = frame_info_obj.get(JsString::from("callback"), context)?;

    let timestamp = get_current_performance_timestamp(context)?;
    let callback_args = vec![timestamp];

    let res = if let Some(callback_obj) = callback.as_object() {
        let global_this = context.global_object().clone();
        callback_obj.call(&JsValue::from(global_this), &callback_args, context)
    } else if callback.is_string() {
        match callback.to_string(context) {
            Ok(code_str) => {
                let std_str = code_str.to_std_string().unwrap_or_default();
                let source = Source::from_bytes(std_str.as_bytes());
                context.eval(source)
            }
            Err(err) => Err(err),
        }
    } else {
        // No-op or ignore non-callable callbacks gracefully
        Ok(JsValue::undefined())
    };

    let drain_res = drain_microtasks(context);

    match res {
        Err(err) => Err(err),
        Ok(val) => drain_res.map(|_| val),
    }
}

fn time_remaining_fixed(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    Ok(JsValue::from(50.0))
}

// TODO(spec): timeRemaining() returns a fixed budget and didTimeout is always false; a real idle scheduler requires event-loop deadline tracking (out of scope for this single-module task).
/// Trigger an idle callback manually (for testing or event loop MVP).
/// Always removes the callback since requestIdleCallback is one-shot.
pub fn trigger_idle_callback(id: i32, context: &mut Context) -> Result<JsValue, JsError> {
    let exists = IDLE_CALLBACKS.with(|cell| cell.borrow_mut().remove(&id).is_some());

    if !exists {
        return Err(JsError::from_opaque(JsValue::from(JsString::from(
            "Idle callback not found",
        ))));
    }

    let idle_obj = get_or_create_idle_callbacks_obj(context)?;
    let callback_info_val = idle_obj.get(id, context)?;
    if callback_info_val.is_undefined() || callback_info_val.is_null() {
        return Err(JsError::from_opaque(JsValue::from(JsString::from(
            "Idle callback info not found in JS state",
        ))));
    }

    let callback_info_obj = callback_info_val.as_object().ok_or_else(|| {
        JsError::from_opaque(JsValue::from(JsString::from(
            "Idle callback info is not an object",
        )))
    })?;

    // Clean up the JS-side state
    let _ = idle_obj.delete_property_or_throw(id, context)?;

    let callback = callback_info_obj.get(JsString::from("callback"), context)?;

    let deadline_obj = ObjectInitializer::new(context)
        .property(
            JsString::from("didTimeout"),
            JsValue::from(false),
            Attribute::all(),
        )
        .function(
            NativeFunction::from_fn_ptr(time_remaining_fixed),
            JsString::from("timeRemaining"),
            0,
        )
        .build();

    let callback_args = vec![JsValue::from(deadline_obj)];

    let res = if let Some(callback_obj) = callback.as_object() {
        let global_this = context.global_object().clone();
        callback_obj.call(&JsValue::from(global_this), &callback_args, context)
    } else if callback.is_string() {
        match callback.to_string(context) {
            Ok(code_str) => {
                let std_str = code_str.to_std_string().unwrap_or_default();
                let source = Source::from_bytes(std_str.as_bytes());
                context.eval(source)
            }
            Err(err) => Err(err),
        }
    } else {
        // No-op or ignore non-callable callbacks gracefully
        Ok(JsValue::undefined())
    };

    let drain_res = drain_microtasks(context);

    match res {
        Err(err) => Err(err),
        Ok(val) => drain_res.map(|_| val),
    }
}

fn js_value_to_i32(val: &JsValue, context: &mut Context) -> Result<i32, JsError> {
    let num = val.to_number(context)?;
    if num.is_nan() || num.is_infinite() {
        Ok(0)
    } else {
        // Implement standard WebIDL 32-bit signed integer (long) wrapping/modulo conversion
        let truncated = num.trunc();
        let modulo_val = truncated % 4294967296.0;
        let i64_val = modulo_val as i64;
        let u32_val = i64_val as u32;
        Ok(u32_val as i32)
    }
}

fn next_timer_id() -> i32 {
    NEXT_TIMER_ID.with(|cell| {
        let mut next_id = cell.borrow_mut();
        loop {
            let cur = *next_id;
            if cur == i32::MAX {
                *next_id = 1;
            } else {
                *next_id += 1;
            }
            // Check if cur is already in use
            let in_use = TIMERS.with(|timers_cell| timers_cell.borrow().contains_key(&cur));
            if !in_use {
                break cur;
            }
        }
    })
}

fn next_raf_id() -> i32 {
    NEXT_RAF_ID.with(|cell| {
        let mut next_id = cell.borrow_mut();
        loop {
            let cur = *next_id;
            if cur == i32::MAX {
                *next_id = 1;
            } else {
                *next_id += 1;
            }
            // Check if cur is already in use
            let in_use = ANIMATION_FRAMES.with(|cell_raf| cell_raf.borrow().contains_key(&cur));
            if !in_use {
                break cur;
            }
        }
    })
}

fn next_idle_id() -> i32 {
    NEXT_IDLE_ID.with(|cell| {
        let mut next_id = cell.borrow_mut();
        loop {
            let cur = *next_id;
            if cur == i32::MAX {
                *next_id = 1;
            } else {
                *next_id += 1;
            }
            // Check if cur is already in use
            let in_use = IDLE_CALLBACKS.with(|cell_idle| cell_idle.borrow().contains_key(&cur));
            if !in_use {
                break cur;
            }
        }
    })
}

pub fn set_timeout(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    let current_level = CURRENT_NESTING_LEVEL.with(|cell| *cell.borrow());
    let new_level = current_level + 1;

    let mut delay = if let Some(delay_val) = args.get(1) {
        js_value_to_i32(delay_val, context)?.max(0)
    } else {
        0
    };

    if new_level > 5 && delay < 4 {
        delay = 4;
    }

    let callback_args = if args.len() > 2 {
        args[2..].to_vec()
    } else {
        Vec::new()
    };

    let id = next_timer_id();

    // Store in Rust side
    let timer = Timer {
        id,
        delay,
        is_interval: false,
        nesting_level: new_level,
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
    let id_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let id = js_value_to_i32(&id_val, context)?;
    // Remove from Rust side
    TIMERS.with(|cell| {
        cell.borrow_mut().remove(&id);
    });
    // Remove from JS side
    if let Ok(timers_obj) = get_or_create_timers_obj(context) {
        let _ = timers_obj.delete_property_or_throw(id, context)?;
    }
    Ok(JsValue::undefined())
}

pub fn set_interval(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    let current_level = CURRENT_NESTING_LEVEL.with(|cell| *cell.borrow());
    let new_level = current_level + 1;

    let mut delay = if let Some(delay_val) = args.get(1) {
        js_value_to_i32(delay_val, context)?.max(0)
    } else {
        0
    };

    if new_level > 5 && delay < 4 {
        delay = 4;
    }

    let callback_args = if args.len() > 2 {
        args[2..].to_vec()
    } else {
        Vec::new()
    };

    let id = next_timer_id();

    // Store in Rust side
    let timer = Timer {
        id,
        delay,
        is_interval: true,
        nesting_level: new_level,
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
    let id_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let id = js_value_to_i32(&id_val, context)?;
    // Remove from Rust side
    TIMERS.with(|cell| {
        cell.borrow_mut().remove(&id);
    });
    // Remove from JS side
    if let Ok(timers_obj) = get_or_create_timers_obj(context) {
        let _ = timers_obj.delete_property_or_throw(id, context)?;
    }
    Ok(JsValue::undefined())
}

pub fn request_animation_frame(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    if !callback.is_callable() {
        return Err(JsError::from(
            boa_engine::JsNativeError::typ()
                .with_message("requestAnimationFrame callback must be a function"),
        ));
    }

    let id = next_raf_id();

    // Store in Rust side
    let frame = AnimationFrame { id };
    ANIMATION_FRAMES.with(|cell| {
        cell.borrow_mut().insert(id, frame);
    });

    // Store callback and args in JS side __animation_frames__ object
    let raf_obj = get_or_create_animation_frames_obj(context)?;

    let timer_info = ObjectInitializer::new(context)
        .property(JsString::from("callback"), callback, Attribute::all())
        .build();

    raf_obj.set(id, JsValue::from(timer_info), false, context)?;

    Ok(JsValue::from(id))
}

pub fn cancel_animation_frame(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let id_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let id = js_value_to_i32(&id_val, context)?;
    // Remove from Rust side
    ANIMATION_FRAMES.with(|cell| {
        cell.borrow_mut().remove(&id);
    });
    // Remove from JS side
    if let Ok(raf_obj) = get_or_create_animation_frames_obj(context) {
        let _ = raf_obj.delete_property_or_throw(id, context)?;
    }
    Ok(JsValue::undefined())
}

pub fn request_idle_callback(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    if !callback.is_callable() {
        return Err(JsError::from(
            boa_engine::JsNativeError::typ()
                .with_message("requestIdleCallback callback must be a function"),
        ));
    }

    let timeout = if let Some(options_val) = args.get(1) {
        if let Some(options_obj) = options_val.as_object() {
            let timeout_val = options_obj.get(JsString::from("timeout"), context)?;
            if !timeout_val.is_undefined() && !timeout_val.is_null() {
                let t_num = timeout_val.to_number(context)?;
                if t_num >= 0.0 && t_num.is_finite() {
                    Some(t_num as u32)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let id = next_idle_id();

    // Store in Rust side
    let idle = IdleCallback { id, timeout };
    IDLE_CALLBACKS.with(|cell| {
        cell.borrow_mut().insert(id, idle);
    });

    // Store callback and args in JS side __idle_callbacks__ object
    let idle_obj = get_or_create_idle_callbacks_obj(context)?;

    let timer_info = if let Some(t) = timeout {
        ObjectInitializer::new(context)
            .property(JsString::from("callback"), callback, Attribute::all())
            .property(
                JsString::from("timeout"),
                JsValue::from(t),
                Attribute::all(),
            )
            .build()
    } else {
        ObjectInitializer::new(context)
            .property(JsString::from("callback"), callback, Attribute::all())
            .build()
    };

    idle_obj.set(id, JsValue::from(timer_info), false, context)?;

    Ok(JsValue::from(id))
}

pub fn cancel_idle_callback(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let id_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let id = js_value_to_i32(&id_val, context)?;
    // Remove from Rust side
    IDLE_CALLBACKS.with(|cell| {
        cell.borrow_mut().remove(&id);
    });
    // Remove from JS side
    if let Ok(idle_obj) = get_or_create_idle_callbacks_obj(context) {
        let _ = idle_obj.delete_property_or_throw(id, context)?;
    }
    Ok(JsValue::undefined())
}

pub fn queue_microtask(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    if !callback.is_callable() {
        return Err(JsError::from(
            boa_engine::JsNativeError::typ()
                .with_message("queueMicrotask callback must be a function"),
        ));
    }

    MICROTASK_QUEUE.with(|cell| {
        cell.borrow_mut().push_back(callback);
    });

    Ok(JsValue::undefined())
}

// TODO(spec): microtask checkpoint should also run after top-level script evaluation (requires a drain call in src/script/mod.rs run loop) — out of scope for this single-module task.
/// Drain the microtask queue FIFO, calling each callback with undefined `this` and no args.
/// Per the WHATWG microtask checkpoint, callbacks enqueued during the drain must also run in the same drain,
/// and even if a callback throws an exception, the draining continues.
pub fn drain_microtasks(context: &mut Context) -> Result<(), JsError> {
    let mut first_error = None;
    loop {
        let next_callback = MICROTASK_QUEUE.with(|cell| cell.borrow_mut().pop_front());
        match next_callback {
            Some(callback) => {
                if let Some(callback_obj) = callback.as_object() {
                    let undefined_this = JsValue::undefined();
                    if let Err(err) = callback_obj.call(&undefined_this, &[], context) {
                        first_error.get_or_insert(err);
                    }
                }
            }
            None => break,
        }
    }
    if let Some(err) = first_error {
        Err(err)
    } else {
        Ok(())
    }
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
    context.register_global_builtin_callable(
        JsString::from("requestAnimationFrame"),
        1,
        NativeFunction::from_fn_ptr(request_animation_frame),
    )?;
    context.register_global_builtin_callable(
        JsString::from("cancelAnimationFrame"),
        1,
        NativeFunction::from_fn_ptr(cancel_animation_frame),
    )?;
    context.register_global_builtin_callable(
        JsString::from("requestIdleCallback"),
        1,
        NativeFunction::from_fn_ptr(request_idle_callback),
    )?;
    context.register_global_builtin_callable(
        JsString::from("cancelIdleCallback"),
        1,
        NativeFunction::from_fn_ptr(cancel_idle_callback),
    )?;
    context.register_global_builtin_callable(
        JsString::from("queueMicrotask"),
        1,
        NativeFunction::from_fn_ptr(queue_microtask),
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

    #[test]
    fn test_request_animation_frame_t0500() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        // 1. Assert requestAnimationFrame registration and count
        let source1 = Source::from_bytes(r#"requestAnimationFrame(() => {})"#);
        let id1_val = context.eval(source1).unwrap();
        let id1 = id1_val.as_number().unwrap() as i32;
        assert_eq!(id1, 1);
        assert_eq!(get_animation_frame_count(), 1);

        // 2. Assert cancelAnimationFrame removes it
        context
            .eval(Source::from_bytes(
                format!("cancelAnimationFrame({})", id1).as_bytes(),
            ))
            .unwrap();
        assert_eq!(get_animation_frame_count(), 0);

        // 3. Assert requestAnimationFrame and trigger execution
        context
            .eval(Source::from_bytes(
                r#"
            var y = 0;
            var rafId = requestAnimationFrame((timestamp) => { y = timestamp; });
        "#,
            ))
            .unwrap();

        let id2_val = context.eval(Source::from_bytes("rafId")).unwrap();
        let id2 = id2_val.as_number().unwrap() as i32;
        assert_eq!(id2, 2);
        assert_eq!(get_animation_frame_count(), 1);

        // Trigger the callback
        let result = trigger_animation_frame(id2, &mut context);
        assert!(result.is_ok());

        // Check if the callback set the global variable to the timestamp (16.0)
        let y_val = context.eval(Source::from_bytes("y")).unwrap();
        assert_eq!(y_val.as_number().unwrap(), 16.0);

        // Since it's requestAnimationFrame (one-shot), triggering it should remove it
        assert_eq!(get_animation_frame_count(), 0);
    }

    #[test]
    fn test_request_idle_callback_t0548() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        // 1. After request_idle_callback, get_idle_callback_count() == 1 and the returned id is 1
        let source1 = Source::from_bytes(r#"requestIdleCallback(() => {})"#);
        let id1_val = context.eval(source1).unwrap();
        let id1 = id1_val.as_number().unwrap() as i32;
        assert_eq!(id1, 1);
        assert_eq!(get_idle_callback_count(), 1);

        // 2. A second request_idle_callback returns id 2 and count becomes 2
        let source2 = Source::from_bytes(r#"requestIdleCallback(() => {})"#);
        let id2_val = context.eval(source2).unwrap();
        let id2 = id2_val.as_number().unwrap() as i32;
        assert_eq!(id2, 2);
        assert_eq!(get_idle_callback_count(), 2);

        // 3. cancel_idle_callback with id 1 drops the count back to 1 and removes that id
        context
            .eval(Source::from_bytes(
                format!("cancelIdleCallback({})", id1).as_bytes(),
            ))
            .unwrap();
        assert_eq!(get_idle_callback_count(), 1);

        // 4. trigger_idle_callback invokes the JS callback (assert an observable side effect on a global)
        context
            .eval(Source::from_bytes(
                r#"
            var ran = false;
            var time_remaining = -1;
            var did_timeout = null;
            var idleId = requestIdleCallback((deadline) => {
                ran = true;
                time_remaining = deadline.timeRemaining();
                did_timeout = deadline.didTimeout;
            });
        "#,
            ))
            .unwrap();

        let id3_val = context.eval(Source::from_bytes("idleId")).unwrap();
        let id3 = id3_val.as_number().unwrap() as i32;
        // id should be 3 because NEXT_IDLE_ID is 3 now
        assert_eq!(id3, 3);
        assert_eq!(get_idle_callback_count(), 2); // id 2 is still there, and id 3 is added

        // Trigger the callback for id 3
        let result = trigger_idle_callback(id3, &mut context);
        assert!(result.is_ok());

        // afterwards get_idle_callback_count() decreased (one-shot removal)
        assert_eq!(get_idle_callback_count(), 1); // only id 2 remains

        // Check side effects
        let ran_val = context.eval(Source::from_bytes("ran")).unwrap();
        assert!(ran_val.as_boolean().unwrap());

        // 5. Inside the triggered callback, the deadline arg's timeRemaining() returns 50 and didTimeout is false
        let time_remaining_val = context.eval(Source::from_bytes("time_remaining")).unwrap();
        assert_eq!(time_remaining_val.as_number().unwrap(), 50.0);

        let did_timeout_val = context.eval(Source::from_bytes("did_timeout")).unwrap();
        assert!(!did_timeout_val.as_boolean().unwrap());
    }

    #[test]
    fn test_queue_microtask_t0504() {
        clear_all_timers();
        clear_all_microtasks();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        context
            .eval(Source::from_bytes(
                r#"
            globalThis.__ran = false;
            queueMicrotask(() => { globalThis.__ran = true; });
            "#,
            ))
            .unwrap();

        let ran_before = context
            .eval(Source::from_bytes("globalThis.__ran"))
            .unwrap();
        assert!(!ran_before.as_boolean().unwrap());

        drain_microtasks(&mut context).unwrap();

        let ran_after = context
            .eval(Source::from_bytes("globalThis.__ran"))
            .unwrap();
        assert!(ran_after.as_boolean().unwrap());
    }

    #[test]
    fn test_queue_microtask_nested_t0504() {
        clear_all_timers();
        clear_all_microtasks();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        context
            .eval(Source::from_bytes(
                r#"
            globalThis.__order = [];
            queueMicrotask(() => {
                globalThis.__order.push(1);
                queueMicrotask(() => {
                    globalThis.__order.push(3);
                });
            });
            queueMicrotask(() => {
                globalThis.__order.push(2);
            });
            "#,
            ))
            .unwrap();

        drain_microtasks(&mut context).unwrap();

        let order_val = context
            .eval(Source::from_bytes("globalThis.__order.toString()"))
            .unwrap();
        assert_eq!(
            order_val
                .to_string(&mut context)
                .unwrap()
                .to_std_string()
                .unwrap(),
            "1,2,3"
        );
    }

    #[test]
    fn test_queue_microtask_type_error_t0504() {
        clear_all_timers();
        clear_all_microtasks();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        let result = context.eval(Source::from_bytes("queueMicrotask(123)"));
        assert!(result.is_err());
        let err_str = result.err().unwrap().to_string();
        assert!(err_str.contains("TypeError") || err_str.contains("must be a function"));
    }

    #[test]
    fn test_timer_nesting_clamp_set_timeout() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        // Register the 1st timer with 0ms delay
        let id1_val = context
            .eval(Source::from_bytes(
                r#"
            var last_id = 0;
            var id1 = setTimeout(() => {
                var id2 = setTimeout(() => {
                    var id3 = setTimeout(() => {
                        var id4 = setTimeout(() => {
                            var id5 = setTimeout(() => {
                                var id6 = setTimeout(() => {
                                    var id7 = setTimeout(() => {}, 0);
                                    last_id = id7;
                                }, 0);
                            }, 0);
                        }, 0);
                    }, 0);
                }, 0);
            }, 0);
            id1;
        "#,
            ))
            .unwrap();
        let id1 = id1_val.as_number().unwrap() as i32;
        let t1 = get_timer(id1).unwrap();
        assert_eq!(t1.delay, 0);
        assert_eq!(t1.nesting_level, 1);

        // Trigger 1st
        trigger_timer(id1, &mut context).unwrap();
        // Now id2 should be registered (ID 2), with delay 0, nesting level 2
        let t2 = get_timer(2).unwrap();
        assert_eq!(t2.delay, 0);
        assert_eq!(t2.nesting_level, 2);

        // Trigger 2nd
        trigger_timer(2, &mut context).unwrap();
        // Now id3 should be registered (ID 3), with delay 0, nesting level 3
        let t3 = get_timer(3).unwrap();
        assert_eq!(t3.delay, 0);
        assert_eq!(t3.nesting_level, 3);

        // Trigger 3rd
        trigger_timer(3, &mut context).unwrap();
        // Now id4 should be registered (ID 4), with delay 0, nesting level 4
        let t4 = get_timer(4).unwrap();
        assert_eq!(t4.delay, 0);
        assert_eq!(t4.nesting_level, 4);

        // Trigger 4th
        trigger_timer(4, &mut context).unwrap();
        // Now id5 should be registered (ID 5), with delay 0, nesting level 5
        let t5 = get_timer(5).unwrap();
        assert_eq!(t5.delay, 0);
        assert_eq!(t5.nesting_level, 5);

        // Trigger 5th
        trigger_timer(5, &mut context).unwrap();
        // Now id6 should be registered (ID 6). Since its nesting level is 6 (which is > 5), delay must be clamped to 4!
        let t6 = get_timer(6).unwrap();
        assert_eq!(t6.delay, 4);
        assert_eq!(t6.nesting_level, 6);

        // Trigger 6th
        trigger_timer(6, &mut context).unwrap();
        // Now id7 should be registered (ID 7). Since its nesting level is 7 (which is > 5), delay must be clamped to 4!
        let t7 = get_timer(7).unwrap();
        assert_eq!(t7.delay, 4);
        assert_eq!(t7.nesting_level, 7);
    }

    #[test]
    fn test_timer_nesting_clamp_set_interval() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        let id_val = context
            .eval(Source::from_bytes(
                r#"
            setInterval(() => {}, 0);
        "#,
            ))
            .unwrap();
        let id = id_val.as_number().unwrap() as i32;
        let t = get_timer(id).unwrap();
        assert_eq!(t.delay, 0);
        assert_eq!(t.nesting_level, 1);

        // Trigger 1st time
        trigger_timer(id, &mut context).unwrap();
        let t = get_timer(id).unwrap();
        assert_eq!(t.delay, 0);
        assert_eq!(t.nesting_level, 2);

        // Trigger 2nd time
        trigger_timer(id, &mut context).unwrap();
        let t = get_timer(id).unwrap();
        assert_eq!(t.delay, 0);
        assert_eq!(t.nesting_level, 3);

        // Trigger 3rd time
        trigger_timer(id, &mut context).unwrap();
        let t = get_timer(id).unwrap();
        assert_eq!(t.delay, 0);
        assert_eq!(t.nesting_level, 4);

        // Trigger 4th time
        trigger_timer(id, &mut context).unwrap();
        let t = get_timer(id).unwrap();
        assert_eq!(t.delay, 0);
        assert_eq!(t.nesting_level, 5);

        // Trigger 5th time -> next execution nesting level becomes 6! So delay must clamp to 4.
        trigger_timer(id, &mut context).unwrap();
        let t = get_timer(id).unwrap();
        assert_eq!(t.delay, 4);
        assert_eq!(t.nesting_level, 6);
    }

    #[test]
    fn test_timer_id_coercion() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        // Register a timeout, which gets ID 1
        let id_val = context
            .eval(Source::from_bytes(r#"setTimeout(() => {}, 100)"#))
            .unwrap();
        let id = id_val.as_number().unwrap() as i32;
        assert_eq!(id, 1);
        assert_eq!(get_timer_count(), 1);

        // Coerce ID to float or string to clear it
        context
            .eval(Source::from_bytes(r#"clearTimeout("1.8")"#))
            .unwrap();
        assert_eq!(get_timer_count(), 0);

        // Register another timeout, gets ID 2
        let id_val2 = context
            .eval(Source::from_bytes(r#"setTimeout(() => {}, 100)"#))
            .unwrap();
        let id2 = id_val2.as_number().unwrap() as i32;
        assert_eq!(id2, 2);
        assert_eq!(get_timer_count(), 1);

        // Coerce using a float value
        context
            .eval(Source::from_bytes(r#"clearTimeout(2.5)"#))
            .unwrap();
        assert_eq!(get_timer_count(), 0);
    }

    #[test]
    fn test_drain_microtasks_with_exception() {
        clear_all_microtasks();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        context
            .eval(Source::from_bytes(
                r#"
            globalThis.__ran_first = false;
            globalThis.__ran_second = false;
            queueMicrotask(() => {
                globalThis.__ran_first = true;
                throw new Error("microtask failed");
            });
            queueMicrotask(() => {
                globalThis.__ran_second = true;
            });
            "#,
            ))
            .unwrap();

        // drain_microtasks should return Err because the first microtask throws an error,
        // but it MUST run the second microtask and empty the queue anyway!
        let drain_res = drain_microtasks(&mut context);
        assert!(drain_res.is_err());

        // Check that BOTH ran!
        let ran_first = context
            .eval(Source::from_bytes("globalThis.__ran_first"))
            .unwrap()
            .as_boolean()
            .unwrap();
        let ran_second = context
            .eval(Source::from_bytes("globalThis.__ran_second"))
            .unwrap()
            .as_boolean()
            .unwrap();
        assert!(ran_first);
        assert!(ran_second);

        // The queue is indeed empty
        MICROTASK_QUEUE.with(|cell| {
            assert_eq!(cell.borrow().len(), 0);
        });
    }

    #[test]
    fn test_raf_perf_now_timestamp() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        // Register a mock performance object with a now() function that returns 12345.0
        context
            .eval(Source::from_bytes(
                r#"
            globalThis.performance = {
                now() { return 12345.0; }
            };
            globalThis.__received_timestamp = 0;
            var rafId = requestAnimationFrame((ts) => {
                globalThis.__received_timestamp = ts;
            });
            "#,
            ))
            .unwrap();

        let raf_id = context
            .eval(Source::from_bytes("rafId"))
            .unwrap()
            .as_number()
            .unwrap() as i32;
        trigger_animation_frame(raf_id, &mut context).unwrap();

        let received_timestamp = context
            .eval(Source::from_bytes("globalThis.__received_timestamp"))
            .unwrap()
            .as_number()
            .unwrap();
        assert_eq!(received_timestamp, 12345.0);
    }

    #[test]
    fn test_request_idle_callback_options_timeout() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        let id_val = context
            .eval(Source::from_bytes(
                r#"
            requestIdleCallback(() => {}, { timeout: 150 });
            "#,
            ))
            .unwrap();
        let id = id_val.as_number().unwrap() as i32;

        let idle_opt = IDLE_CALLBACKS.with(|cell| cell.borrow().get(&id).cloned());
        assert!(idle_opt.is_some());
        let idle = idle_opt.unwrap();
        assert_eq!(idle.timeout, Some(150));
    }

    #[test]
    fn test_timer_id_wrapping_and_collision() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        // Let's populate some active timers
        let id1_val = context
            .eval(Source::from_bytes(r#"setTimeout(() => {}, 100)"#))
            .unwrap();
        let id1 = id1_val.as_number().unwrap() as i32;
        assert_eq!(id1, 1);

        // Manually set NEXT_TIMER_ID to i32::MAX
        NEXT_TIMER_ID.with(|cell| {
            *cell.borrow_mut() = i32::MAX;
        });

        // Register a new timer when NEXT_TIMER_ID is i32::MAX
        let id2_val = context
            .eval(Source::from_bytes(r#"setTimeout(() => {}, 100)"#))
            .unwrap();
        let id2 = id2_val.as_number().unwrap() as i32;
        // Since cur was i32::MAX, it should be registered with ID = i32::MAX
        assert_eq!(id2, i32::MAX);

        // After registering id2, NEXT_TIMER_ID would wrap around to 1.
        // But ID 1 is currently in use!
        // Therefore, registering the next timer should skip 1 and get 2!
        let id3_val = context
            .eval(Source::from_bytes(r#"setTimeout(() => {}, 100)"#))
            .unwrap();
        let id3 = id3_val.as_number().unwrap() as i32;
        assert_eq!(id3, 2);
    }

    #[test]
    fn test_raf_and_idle_callback_non_callable_throws_type_error() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        // requestAnimationFrame with string
        let res_raf = context.eval(Source::from_bytes(
            r#"requestAnimationFrame("non-callable")"#,
        ));
        assert!(res_raf.is_err());
        let err_raf = res_raf.err().unwrap().to_string();
        assert!(err_raf.contains("TypeError") || err_raf.contains("must be a function"));

        // requestIdleCallback with number
        let res_idle = context.eval(Source::from_bytes(r#"requestIdleCallback(123)"#));
        assert!(res_idle.is_err());
        let err_idle = res_idle.err().unwrap().to_string();
        assert!(err_idle.contains("TypeError") || err_idle.contains("must be a function"));
    }

    #[test]
    fn test_clear_and_cancel_no_arguments_safe_no_ops() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        assert!(context.eval(Source::from_bytes("clearTimeout()")).is_ok());
        assert!(context.eval(Source::from_bytes("clearInterval()")).is_ok());
        assert!(
            context
                .eval(Source::from_bytes("cancelAnimationFrame()"))
                .is_ok()
        );
        assert!(
            context
                .eval(Source::from_bytes("cancelIdleCallback()"))
                .is_ok()
        );
    }

    #[test]
    fn test_timer_callback_exception_still_drains_microtasks() {
        clear_all_timers();
        clear_all_microtasks();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        context
            .eval(Source::from_bytes(
                r#"
            globalThis.__microtask_ran = false;
            var timerId = setTimeout(() => {
                queueMicrotask(() => {
                    globalThis.__microtask_ran = true;
                });
                throw new Error("timer callback threw exception");
            }, 100);
            "#,
            ))
            .unwrap();

        let id_val = context.eval(Source::from_bytes("timerId")).unwrap();
        let id = id_val.as_number().unwrap() as i32;

        // Triggering the timer should return an error because the callback threw an exception.
        let trigger_res = trigger_timer(id, &mut context);
        assert!(trigger_res.is_err());

        // However, the microtask queued during the timer's run must still be drained!
        let microtask_ran = context
            .eval(Source::from_bytes("globalThis.__microtask_ran"))
            .unwrap()
            .as_boolean()
            .unwrap();
        assert!(microtask_ran);
    }

    #[test]
    fn test_trigger_all_animation_frames_same_timestamp() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        context
            .eval(Source::from_bytes(
                r#"
            globalThis.performance = {
                now() { return 54321.0; }
            };
            globalThis.__ts1 = 0;
            globalThis.__ts2 = 0;
            requestAnimationFrame((ts) => {
                globalThis.__ts1 = ts;
            });
            requestAnimationFrame((ts) => {
                globalThis.__ts2 = ts;
            });
            "#,
            ))
            .unwrap();

        assert_eq!(get_animation_frame_count(), 2);

        let results = trigger_all_animation_frames(&mut context).unwrap();
        assert_eq!(results.len(), 2);
        for res in results {
            assert!(res.is_ok());
        }

        assert_eq!(get_animation_frame_count(), 0);

        let ts1 = context
            .eval(Source::from_bytes("globalThis.__ts1"))
            .unwrap()
            .as_number()
            .unwrap();
        let ts2 = context
            .eval(Source::from_bytes("globalThis.__ts2"))
            .unwrap()
            .as_number()
            .unwrap();

        assert_eq!(ts1, 54321.0);
        assert_eq!(ts2, 54321.0);
    }

    #[test]
    fn test_clear_timeout_interval_idempotency_t1012() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        // 1. Call clearTimeout with non-existent IDs and invalid inputs - must be safe and not throw
        let res1 = context.eval(Source::from_bytes("clearTimeout(999)"));
        assert!(res1.is_ok());
        let res2 = context.eval(Source::from_bytes("clearInterval(999)"));
        assert!(res2.is_ok());
        let res3 = context.eval(Source::from_bytes("clearTimeout(null)"));
        assert!(res3.is_ok());
        let res4 = context.eval(Source::from_bytes("clearTimeout(undefined)"));
        assert!(res4.is_ok());
        let res5 = context.eval(Source::from_bytes("clearTimeout('non-numeric')"));
        assert!(res5.is_ok());

        // 2. Set timeout, clear it multiple times
        context
            .eval(Source::from_bytes(
                r#"
            var tId = setTimeout(() => {}, 100);
            "#,
            ))
            .unwrap();
        let t_id = context
            .eval(Source::from_bytes("tId"))
            .unwrap()
            .as_number()
            .unwrap() as i32;
        assert_eq!(get_timer_count(), 1);

        // First clear
        context
            .eval(Source::from_bytes(&format!("clearTimeout({})", t_id)))
            .unwrap();
        assert_eq!(get_timer_count(), 0);

        // Second clear (idempotent call)
        let double_clear_res = context.eval(Source::from_bytes(&format!("clearTimeout({})", t_id)));
        assert!(double_clear_res.is_ok());
        assert_eq!(get_timer_count(), 0);

        // 3. Set interval, clear it multiple times
        context
            .eval(Source::from_bytes(
                r#"
            var intId = setInterval(() => {}, 100);
            "#,
            ))
            .unwrap();
        let int_id = context
            .eval(Source::from_bytes("intId"))
            .unwrap()
            .as_number()
            .unwrap() as i32;
        assert_eq!(get_timer_count(), 1);

        // First clear
        context
            .eval(Source::from_bytes(&format!("clearInterval({})", int_id)))
            .unwrap();
        assert_eq!(get_timer_count(), 0);

        // Second clear (idempotent call)
        let double_clear_int =
            context.eval(Source::from_bytes(&format!("clearInterval({})", int_id)));
        assert!(double_clear_int.is_ok());
        assert_eq!(get_timer_count(), 0);
    }

    #[test]
    fn test_timer_argument_passing_t1012() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        context
            .eval(Source::from_bytes(
                r#"
            globalThis.__args_received = null;
            var tId = setTimeout((a, b, c, d) => {
                globalThis.__args_received = [a, b, c, d];
            }, 100, "hello", 42, { x: 1 }, [2, 3]);
            "#,
            ))
            .unwrap();

        let t_id = context
            .eval(Source::from_bytes("tId"))
            .unwrap()
            .as_number()
            .unwrap() as i32;
        trigger_timer(t_id, &mut context).unwrap();

        let args_val = context
            .eval(Source::from_bytes("globalThis.__args_received"))
            .unwrap();
        let args_obj = args_val.as_object().unwrap();

        let arg0 = args_obj
            .get(0, &mut context)
            .unwrap()
            .to_string(&mut context)
            .unwrap()
            .to_std_string()
            .unwrap();
        let arg1 = args_obj.get(1, &mut context).unwrap().as_number().unwrap() as i32;
        let arg2_obj = args_obj
            .get(2, &mut context)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        let arg2_x = arg2_obj
            .get(JsString::from("x"), &mut context)
            .unwrap()
            .as_number()
            .unwrap() as i32;

        assert_eq!(arg0, "hello");
        assert_eq!(arg1, 42);
        assert_eq!(arg2_x, 1);
    }

    #[test]
    fn test_timer_ordering_by_handle_t1012() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        // 1. Register multiple timers with various delays.
        // Handles/IDs are sequentially assigned starting at 1.
        context
            .eval(Source::from_bytes("setTimeout(() => {}, 50)"))
            .unwrap(); // ID 1
        context
            .eval(Source::from_bytes("setTimeout(() => {}, 10)"))
            .unwrap(); // ID 2
        context
            .eval(Source::from_bytes("setTimeout(() => {}, 100)"))
            .unwrap(); // ID 3
        context
            .eval(Source::from_bytes("setTimeout(() => {}, 0)"))
            .unwrap(); // ID 4

        let timers = get_active_timers_sorted();
        assert_eq!(timers.len(), 4);

        // Assert they are sorted in strictly ascending order by handle/ID
        assert_eq!(timers[0].id, 1);
        assert_eq!(timers[1].id, 2);
        assert_eq!(timers[2].id, 3);
        assert_eq!(timers[3].id, 4);

        // Check our Ord/PartialOrd implementations
        assert!(timers[0] < timers[1]);
        assert!(timers[1] < timers[2]);
        assert!(timers[2] < timers[3]);

        assert_eq!(timers[0].cmp(&timers[1]), std::cmp::Ordering::Less);
        assert_eq!(timers[3].cmp(&timers[2]), std::cmp::Ordering::Greater);
        assert_eq!(timers[2].cmp(&timers[2]), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_t1038_wrapping_clamping_clear() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        // 1. Verify delay wrapping and clamping in setTimeout:
        // Delay 2147483648 (2^31) should wrap to -2147483648, which gets clamped to 0.
        let id1_val = context
            .eval(Source::from_bytes("setTimeout(() => {}, 2147483648)"))
            .unwrap();
        let id1 = id1_val.as_number().unwrap() as i32;
        let timer1 = get_timer(id1).unwrap();
        assert_eq!(timer1.delay, 0);

        // Delay 4294967396 (2^32 + 100) should wrap to 100.
        let id2_val = context
            .eval(Source::from_bytes("setTimeout(() => {}, 4294967396)"))
            .unwrap();
        let id2 = id2_val.as_number().unwrap() as i32;
        let timer2 = get_timer(id2).unwrap();
        assert_eq!(timer2.delay, 100);

        // 2. Verify WebIDL long wrapping inside clearTimeout:
        // Timer registered above has ID equal to id1 or id2.
        // Let's clear_all_timers and register a new one to get ID 1.
        clear_all_timers();
        let id3_val = context
            .eval(Source::from_bytes("setTimeout(() => {}, 100)"))
            .unwrap();
        let id3 = id3_val.as_number().unwrap() as i32;
        assert_eq!(id3, 1);
        assert_eq!(get_timer_count(), 1);

        // Clearing with 4294967297 (2^32 + 1) which wraps to 1, should successfully clear ID 1.
        context
            .eval(Source::from_bytes("clearTimeout(4294967297)"))
            .unwrap();
        assert_eq!(get_timer_count(), 0);

        // 3. Interchangeability of clearTimeout and clearInterval:
        // Register a timeout, clear with clearInterval
        clear_all_timers();
        let id_timeout_val = context
            .eval(Source::from_bytes("setTimeout(() => {}, 100)"))
            .unwrap();
        let id_timeout = id_timeout_val.as_number().unwrap() as i32;
        assert_eq!(get_timer_count(), 1);
        context
            .eval(Source::from_bytes(&format!(
                "clearInterval({})",
                id_timeout
            )))
            .unwrap();
        assert_eq!(get_timer_count(), 0);

        // Register an interval, clear with clearTimeout
        let id_interval_val = context
            .eval(Source::from_bytes("setInterval(() => {}, 100)"))
            .unwrap();
        let id_interval = id_interval_val.as_number().unwrap() as i32;
        assert_eq!(get_timer_count(), 1);
        context
            .eval(Source::from_bytes(&format!(
                "clearTimeout({})",
                id_interval
            )))
            .unwrap();
        assert_eq!(get_timer_count(), 0);
    }

    #[test]
    fn test_t1056_set_interval_clear_interval_and_clamping() {
        clear_all_timers();
        let mut context = Context::default();
        register_timer_builtins(&mut context).unwrap();

        // 1. Verify negative delays in setInterval are clamped to 0
        let id_neg_val = context
            .eval(Source::from_bytes("setInterval(() => {}, -50)"))
            .unwrap();
        let id_neg = id_neg_val.as_number().unwrap() as i32;
        let timer_neg = get_timer(id_neg).unwrap();
        assert_eq!(timer_neg.delay, 0);

        // 2. Verify non-callable object with custom toString converts to code string and evaluates correctly
        context
            .eval(Source::from_bytes(
                r#"
            globalThis.__test_val = 0;
            var customObj = {
                toString() {
                    return "globalThis.__test_val = 42;";
                }
            };
            var id_custom = setInterval(customObj, 100);
            "#,
            ))
            .unwrap();
        let id_custom = context
            .eval(Source::from_bytes("id_custom"))
            .unwrap()
            .as_number()
            .unwrap() as i32;
        trigger_timer(id_custom, &mut context).unwrap();
        let test_val = context
            .eval(Source::from_bytes("globalThis.__test_val"))
            .unwrap()
            .as_number()
            .unwrap() as i32;
        assert_eq!(test_val, 42);

        // 3. Verify various clearInterval argument types WebIDL long wrapping/coercion
        clear_all_timers();
        let id_t_val = context
            .eval(Source::from_bytes("setInterval(() => {}, 100)"))
            .unwrap();
        let id_t = id_t_val.as_number().unwrap() as i32;
        assert_eq!(id_t, 1);
        assert_eq!(get_timer_count(), 1);

        // Clear with float (1.5 converts to 1)
        context
            .eval(Source::from_bytes("clearInterval(1.5)"))
            .unwrap();
        assert_eq!(get_timer_count(), 0);

        // Register again
        let id_t2_val = context
            .eval(Source::from_bytes("setInterval(() => {}, 100)"))
            .unwrap();
        let id_t2 = id_t2_val.as_number().unwrap() as i32;
        assert_eq!(get_timer_count(), 1);

        // Clear with string ("2" converts to 2)
        context
            .eval(Source::from_bytes(&format!("clearInterval('{}')", id_t2)))
            .unwrap();
        assert_eq!(get_timer_count(), 0);

        // 4. Verify nested timeout clamping under depth > 5 inside setInterval callback
        clear_all_timers();
        // Set up a deeply nested timer tree inside setInterval:
        context
            .eval(Source::from_bytes(
                r#"
            globalThis.__nested_delay = -1;
            setInterval(() => {
                setTimeout(() => {
                    setTimeout(() => {
                        setTimeout(() => {
                            setTimeout(() => {
                                var last_t = setTimeout(() => {}, 0);
                                globalThis.__nested_delay = last_t;
                            }, 0);
                        }, 0);
                    }, 0);
                }, 0);
            }, 100);
            "#,
            ))
            .unwrap();

        let interval_id = 1;
        // Trigger interval execution (nesting level becomes 2 inside execution)
        trigger_timer(interval_id, &mut context).unwrap();
        // Now trigger timeout 2 (nesting level 3 inside execution)
        trigger_timer(2, &mut context).unwrap();
        // Now trigger timeout 3 (nesting level 4 inside execution)
        trigger_timer(3, &mut context).unwrap();
        // Now trigger timeout 4 (nesting level 5 inside execution)
        trigger_timer(4, &mut context).unwrap();
        // Now trigger timeout 5 (nesting level 6 inside execution)
        trigger_timer(5, &mut context).unwrap();

        // The scheduled timeout 6 should have nesting level 6, so delay clamped to 4!
        let last_id_val = context
            .eval(Source::from_bytes("globalThis.__nested_delay"))
            .unwrap();
        let last_id = last_id_val.as_number().unwrap() as i32;
        let last_timer = get_timer(last_id).unwrap();
        assert_eq!(last_timer.delay, 4);
        assert_eq!(last_timer.nesting_level, 6);
    }
}
