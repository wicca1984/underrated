//! Implementation of the DOM `EventTarget` and `Event` native classes for JavaScript execution via Boa.
//!
//! Spec: <https://dom.spec.whatwg.org/#eventtarget>
//! Spec: <https://dom.spec.whatwg.org/#event>

use boa_engine::{
    Context, JsData, JsError, JsNativeError, JsResult, JsString, JsValue, NativeFunction,
    class::{Class, ClassBuilder},
    object::{FunctionObjectBuilder, ObjectInitializer},
    property::Attribute,
};
use boa_gc::{Finalize, GcRefCell, Trace};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Instant;

static EVENT_TIME_ORIGIN: OnceLock<Instant> = OnceLock::new();

fn get_event_timestamp() -> f64 {
    let origin = EVENT_TIME_ORIGIN.get_or_init(Instant::now);
    origin.elapsed().as_secs_f64() * 1000.0
}

/// The DOM `Event` interface represents an event which takes place in the DOM.
///
/// Spec: <https://dom.spec.whatwg.org/#event>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct Event {
    pub(crate) r#type: String,
    pub(crate) target: GcRefCell<Option<JsValue>>,
    pub(crate) current_target: GcRefCell<Option<JsValue>>,
    pub(crate) default_prevented: GcRefCell<bool>,
    pub(crate) propagation_stopped: GcRefCell<bool>,
    pub(crate) immediate_propagation_stopped: GcRefCell<bool>,
    pub(crate) bubbles: bool,
    pub(crate) cancelable: bool,
    pub(crate) composed: bool,
    pub(crate) is_trusted: bool,
    pub(crate) event_phase: GcRefCell<u16>,
    pub(crate) time_stamp: f64,
}

impl Class for Event {
    const NAME: &'static str = "Event";
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
                        .with_message("Event constructor requires at least 1 argument"),
                )
            })?
            .to_string(context)?
            .to_std_string()
            .unwrap_or_default();

        let mut bubbles = false;
        let mut cancelable = false;
        let mut composed = false;

        if let Some(init_val) = args.get(1)
            && let Some(init_obj) = init_val.as_object()
        {
            if let Ok(bubbles_prop) = init_obj.get(JsString::from("bubbles"), context) {
                bubbles = bubbles_prop.as_boolean().unwrap_or(false);
            }
            if let Ok(cancelable_prop) = init_obj.get(JsString::from("cancelable"), context) {
                cancelable = cancelable_prop.as_boolean().unwrap_or(false);
            }
            if let Ok(composed_prop) = init_obj.get(JsString::from("composed"), context) {
                composed = composed_prop.as_boolean().unwrap_or(false);
            }
        }

        Ok(Event {
            r#type: event_type,
            target: GcRefCell::new(None),
            current_target: GcRefCell::new(None),
            default_prevented: GcRefCell::new(false),
            propagation_stopped: GcRefCell::new(false),
            immediate_propagation_stopped: GcRefCell::new(false),
            bubbles,
            cancelable,
            composed,
            is_trusted: false,
            event_phase: GcRefCell::new(0),
            time_stamp: get_event_timestamp(),
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let get_type_fn = FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_type))
            .name("get type")
            .build();
        let get_target_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_target))
                .name("get target")
                .build();
        let get_current_target_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_current_target))
                .name("get currentTarget")
                .build();
        let get_default_prevented_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_default_prevented))
                .name("get defaultPrevented")
                .build();
        let get_bubbles_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_bubbles))
                .name("get bubbles")
                .build();
        let get_cancelable_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_cancelable))
                .name("get cancelable")
                .build();
        let get_composed_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_composed))
                .name("get composed")
                .build();
        let get_is_trusted_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_is_trusted))
                .name("get isTrusted")
                .build();
        let get_event_phase_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_event_phase))
                .name("get eventPhase")
                .build();
        let get_time_stamp_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_time_stamp))
                .name("get timeStamp")
                .build();

        class
            .accessor(
                JsString::from("type"),
                Some(get_type_fn),
                None,
                Attribute::all(),
            )
            .accessor(
                JsString::from("target"),
                Some(get_target_fn),
                None,
                Attribute::all(),
            )
            .accessor(
                JsString::from("currentTarget"),
                Some(get_current_target_fn),
                None,
                Attribute::all(),
            )
            .accessor(
                JsString::from("defaultPrevented"),
                Some(get_default_prevented_fn),
                None,
                Attribute::all(),
            )
            .accessor(
                JsString::from("bubbles"),
                Some(get_bubbles_fn),
                None,
                Attribute::all(),
            )
            .accessor(
                JsString::from("cancelable"),
                Some(get_cancelable_fn),
                None,
                Attribute::all(),
            )
            .accessor(
                JsString::from("composed"),
                Some(get_composed_fn),
                None,
                Attribute::all(),
            )
            .accessor(
                JsString::from("isTrusted"),
                Some(get_is_trusted_fn),
                None,
                Attribute::all(),
            )
            .accessor(
                JsString::from("eventPhase"),
                Some(get_event_phase_fn),
                None,
                Attribute::all(),
            )
            .accessor(
                JsString::from("timeStamp"),
                Some(get_time_stamp_fn),
                None,
                Attribute::all(),
            )
            .method(
                JsString::from("preventDefault"),
                0,
                NativeFunction::from_fn_ptr(prevent_default),
            )
            .method(
                JsString::from("stopPropagation"),
                0,
                NativeFunction::from_fn_ptr(stop_propagation),
            )
            .method(
                JsString::from("stopImmediatePropagation"),
                0,
                NativeFunction::from_fn_ptr(stop_immediate_propagation),
            )
            .method(
                JsString::from("composedPath"),
                0,
                NativeFunction::from_fn_ptr(composed_path),
            )
            .property(JsString::from("NONE"), 0, Attribute::ENUMERABLE)
            .property(JsString::from("CAPTURING_PHASE"), 1, Attribute::ENUMERABLE)
            .property(JsString::from("AT_TARGET"), 2, Attribute::ENUMERABLE)
            .property(JsString::from("BUBBLING_PHASE"), 3, Attribute::ENUMERABLE)
            .static_property(JsString::from("NONE"), 0, Attribute::ENUMERABLE)
            .static_property(JsString::from("CAPTURING_PHASE"), 1, Attribute::ENUMERABLE)
            .static_property(JsString::from("AT_TARGET"), 2, Attribute::ENUMERABLE)
            .static_property(JsString::from("BUBBLING_PHASE"), 3, Attribute::ENUMERABLE);

        Ok(())
    }
}

fn get_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(JsValue::from(JsString::from(event.r#type.clone())))
}

fn get_target(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(event.target.borrow().clone().unwrap_or(JsValue::null()))
}

fn get_current_target(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(event
        .current_target
        .borrow()
        .clone()
        .unwrap_or(JsValue::null()))
}

fn get_default_prevented(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(JsValue::from(*event.default_prevented.borrow()))
}

fn get_bubbles(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(JsValue::from(event.bubbles))
}

fn get_cancelable(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(JsValue::from(event.cancelable))
}

fn get_composed(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(JsValue::from(event.composed))
}

fn get_is_trusted(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(JsValue::from(event.is_trusted))
}

fn get_event_phase(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(JsValue::from(*event.event_phase.borrow()))
}

fn get_time_stamp(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(JsValue::from(event.time_stamp))
}

fn prevent_default(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    *event.default_prevented.borrow_mut() = true;
    Ok(JsValue::undefined())
}

fn stop_propagation(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    *event.propagation_stopped.borrow_mut() = true;
    Ok(JsValue::undefined())
}

fn stop_immediate_propagation(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    *event.propagation_stopped.borrow_mut() = true;
    *event.immediate_propagation_stopped.borrow_mut() = true;
    Ok(JsValue::undefined())
}

fn composed_path(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let _event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;

    // TODO(spec): Return the real path once event propagation path tracking is fully wired.
    let array_constructor = context
        .global_object()
        .get(JsString::from("Array"), context)?;
    let array_obj = array_constructor.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Array constructor not found"))
    })?;
    let array_val = array_obj.construct(&[], None, context)?;
    Ok(array_val.into())
}

/// The `EventTarget` interface is implemented by objects that can receive events and may have listeners for them.
///
/// Spec: <https://dom.spec.whatwg.org/#eventtarget>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct EventTarget {
    pub(crate) listeners: GcRefCell<HashMap<String, Vec<JsValue>>>,
}

impl Class for EventTarget {
    const NAME: &'static str = "EventTarget";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Ok(EventTarget {
            listeners: GcRefCell::new(HashMap::new()),
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class
            .method(
                JsString::from("addEventListener"),
                2,
                NativeFunction::from_fn_ptr(add_event_listener),
            )
            .method(
                JsString::from("removeEventListener"),
                2,
                NativeFunction::from_fn_ptr(remove_event_listener),
            )
            .method(
                JsString::from("dispatchEvent"),
                1,
                NativeFunction::from_fn_ptr(dispatch_event),
            );

        Ok(())
    }
}

pub fn add_event_listener(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;

    let event_type = args
        .first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let listener = args.get(1).cloned().unwrap_or(JsValue::undefined());

    if let Some(event_target) = obj.downcast_ref::<EventTarget>() {
        if listener.is_callable() || listener.is_object() {
            let mut listeners = event_target.listeners.borrow_mut();
            let entry = listeners.entry(event_type).or_insert_with(Vec::new);
            if !entry
                .iter()
                .any(|existing| existing.strict_equals(&listener))
            {
                entry.push(listener);
            }
        }
    } else {
        // Fallback/Legacy DOM bridge path: store in JS property `__events__`
        let events_prop = JsString::from("__events__");
        let mut events_val = obj.get(events_prop.clone(), context)?;
        if events_val.is_undefined() || events_val.is_null() {
            let new_events_obj = ObjectInitializer::new(context).build();
            obj.set(
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
                    JsError::from(JsNativeError::typ().with_message("Array constructor not found"))
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
                let length_val = handlers_obj.get(JsString::from("length"), context)?;
                let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);
                let mut already_exists = false;
                for i in 0..length {
                    if let Ok(existing) = handlers_obj.get(i, context)
                        && existing.strict_equals(&listener)
                    {
                        already_exists = true;
                        break;
                    }
                }
                if !already_exists {
                    let push_val = handlers_obj.get(JsString::from("push"), context)?;
                    if let Some(push_fn) = push_val.as_object() {
                        push_fn.call(&handlers_val, &[listener], context)?;
                    }
                }
            }
        }
    }

    Ok(JsValue::undefined())
}

pub fn remove_event_listener(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;

    let event_type = args
        .first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let listener = args.get(1).cloned().unwrap_or(JsValue::undefined());

    if let Some(event_target) = obj.downcast_ref::<EventTarget>() {
        let mut listeners = event_target.listeners.borrow_mut();
        if let Some(entry) = listeners.get_mut(&event_type)
            && let Some(pos) = entry
                .iter()
                .position(|existing| existing.strict_equals(&listener))
        {
            entry.remove(pos);
        }
    } else {
        // Fallback/Legacy path: look in JS property `__events__`
        let events_prop = JsString::from("__events__");
        let events_val = obj.get(events_prop.clone(), context)?;
        if let Some(events_obj) = events_val.as_object() {
            let type_prop = JsString::from(event_type.as_str());
            let handlers_val = events_obj.get(type_prop.clone(), context)?;
            if let Some(handlers_obj) = handlers_val.as_object() {
                let length_val = handlers_obj.get(JsString::from("length"), context)?;
                let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);
                for i in 0..length {
                    if let Ok(existing) = handlers_obj.get(i, context)
                        && existing.strict_equals(&listener)
                    {
                        let splice_val = handlers_obj.get(JsString::from("splice"), context)?;
                        if let Some(splice_fn) = splice_val.as_object() {
                            splice_fn.call(
                                &handlers_val,
                                &[JsValue::from(i), JsValue::from(1)],
                                context,
                            )?;
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(JsValue::undefined())
}

pub fn dispatch_event(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;

    let event_val = args.first().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("dispatchEvent requires an Event argument"))
    })?;

    let event_obj = event_val.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Event must be an object"))
    })?;

    // Try to downcast to our Event class
    if let Some(event) = event_obj.downcast_ref::<Event>() {
        // Set target and current_target
        *event.target.borrow_mut() = Some(this.clone());
        *event.current_target.borrow_mut() = Some(this.clone());

        // Get list of listeners (either native or legacy)
        let mut listeners_to_call = Vec::new();
        if let Some(event_target) = obj.downcast_ref::<EventTarget>() {
            let listeners = event_target.listeners.borrow();
            if let Some(list) = listeners.get(&event.r#type) {
                listeners_to_call = list.clone();
            }
        } else {
            // Legacy/Fallback DOM bridge path: read from JS property `__events__`
            let events_prop = JsString::from("__events__");
            let events_val = obj.get(events_prop.clone(), context)?;
            if let Some(events_obj) = events_val.as_object() {
                let type_prop = JsString::from(event.r#type.as_str());
                let handlers_val = events_obj.get(type_prop.clone(), context)?;
                if let Some(handlers_obj) = handlers_val.as_object() {
                    let length_val = handlers_obj.get(JsString::from("length"), context)?;
                    let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);
                    for i in 0..length {
                        if let Ok(handler) = handlers_obj.get(i, context) {
                            listeners_to_call.push(handler);
                        }
                    }
                }
            }
        }

        for listener in listeners_to_call {
            if *event.propagation_stopped.borrow() || *event.immediate_propagation_stopped.borrow()
            {
                break;
            }

            if let Some(callable) = listener.as_object() {
                if callable.is_callable() {
                    callable.call(this, std::slice::from_ref(event_val), context)?;
                } else if let Ok(handle_event_val) =
                    callable.get(JsString::from("handleEvent"), context)
                    && let Some(handle_event_callable) = handle_event_val.as_object()
                    && handle_event_callable.is_callable()
                {
                    handle_event_callable.call(
                        &listener,
                        std::slice::from_ref(event_val),
                        context,
                    )?;
                }
            }
        }

        *event.current_target.borrow_mut() = None;
        Ok(JsValue::from(!*event.default_prevented.borrow()))
    } else {
        // Not a native Event object (maybe a plain object).
        let target_prop = JsString::from("target");
        let current_target_prop = JsString::from("currentTarget");
        event_obj.set(target_prop.clone(), this.clone(), false, context)?;
        event_obj.set(current_target_prop.clone(), this.clone(), false, context)?;

        let event_type_val = event_obj.get(JsString::from("type"), context)?;
        let event_type = event_type_val
            .to_string(context)?
            .to_std_string()
            .unwrap_or_default();

        let mut listeners_to_call = Vec::new();
        if let Some(event_target) = obj.downcast_ref::<EventTarget>() {
            let listeners = event_target.listeners.borrow();
            if let Some(list) = listeners.get(&event_type) {
                listeners_to_call = list.clone();
            }
        } else {
            // Legacy path
            let events_prop = JsString::from("__events__");
            let events_val = obj.get(events_prop.clone(), context)?;
            if let Some(events_obj) = events_val.as_object() {
                let type_prop = JsString::from(event_type.as_str());
                let handlers_val = events_obj.get(type_prop.clone(), context)?;
                if let Some(handlers_obj) = handlers_val.as_object() {
                    let length_val = handlers_obj.get(JsString::from("length"), context)?;
                    let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);
                    for i in 0..length {
                        if let Ok(handler) = handlers_obj.get(i, context) {
                            listeners_to_call.push(handler);
                        }
                    }
                }
            }
        }

        for listener in listeners_to_call {
            let stopped_val = event_obj.get(JsString::from("propagationStopped"), context)?;
            if stopped_val.as_boolean().unwrap_or(false) {
                break;
            }

            if let Some(callable) = listener.as_object() {
                if callable.is_callable() {
                    callable.call(this, std::slice::from_ref(event_val), context)?;
                } else if let Ok(handle_event_val) =
                    callable.get(JsString::from("handleEvent"), context)
                    && let Some(handle_event_callable) = handle_event_val.as_object()
                    && handle_event_callable.is_callable()
                {
                    handle_event_callable.call(
                        &listener,
                        std::slice::from_ref(event_val),
                        context,
                    )?;
                }
            }
        }

        event_obj.set(current_target_prop.clone(), JsValue::null(), false, context)?;

        let prevented_val = event_obj.get(JsString::from("defaultPrevented"), context)?;
        Ok(JsValue::from(!prevented_val.as_boolean().unwrap_or(false)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    #[test]
    fn test_event_constants_and_getters() {
        let mut context = Context::default();
        context.register_global_class::<Event>().unwrap();

        // 1. Check constants on constructor and prototype/instance
        let res = context
            .eval(Source::from_bytes("Event.CAPTURING_PHASE === 1"))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        let res = context
            .eval(Source::from_bytes("Event.BUBBLING_PHASE === 3"))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        let res = context
            .eval(Source::from_bytes(
                "{ let ev = new Event('foo'); ev.CAPTURING_PHASE === 1 }",
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. Check default values for getters
        let res = context
            .eval(Source::from_bytes("{ let ev = new Event('foo'); [ev.type, ev.bubbles, ev.cancelable, ev.composed, ev.isTrusted, ev.eventPhase, ev.defaultPrevented] }"))
            .unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(
            arr.get(0, &mut context)
                .unwrap()
                .as_string()
                .unwrap()
                .to_std_string()
                .unwrap(),
            "foo"
        );
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(false)); // bubbles
        assert_eq!(arr.get(2, &mut context).unwrap().as_boolean(), Some(false)); // cancelable
        assert_eq!(arr.get(3, &mut context).unwrap().as_boolean(), Some(false)); // composed
        assert_eq!(arr.get(4, &mut context).unwrap().as_boolean(), Some(false)); // isTrusted
        assert_eq!(arr.get(5, &mut context).unwrap().as_number(), Some(0.0)); // eventPhase
        assert_eq!(arr.get(6, &mut context).unwrap().as_boolean(), Some(false)); // defaultPrevented

        // 3. Check Constructor with EventInit options
        let res = context
            .eval(Source::from_bytes("{ let ev = new Event('bar', { bubbles: true, cancelable: true, composed: true }); [ev.type, ev.bubbles, ev.cancelable, ev.composed] }"))
            .unwrap();
        let arr2 = res.as_object().unwrap();
        assert_eq!(
            arr2.get(0, &mut context)
                .unwrap()
                .as_string()
                .unwrap()
                .to_std_string()
                .unwrap(),
            "bar"
        );
        assert_eq!(arr2.get(1, &mut context).unwrap().as_boolean(), Some(true)); // bubbles
        assert_eq!(arr2.get(2, &mut context).unwrap().as_boolean(), Some(true)); // cancelable
        assert_eq!(arr2.get(3, &mut context).unwrap().as_boolean(), Some(true)); // composed

        // 4. Check timeStamp is a number >= 0
        let res = context
            .eval(Source::from_bytes("{ let ev = new Event('foo'); typeof ev.timeStamp === 'number' && ev.timeStamp >= 0 }"))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 5. Check composedPath() returns an empty array
        let res = context
            .eval(Source::from_bytes("{ let ev = new Event('foo'); Array.isArray(ev.composedPath()) && ev.composedPath().length === 0 }"))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_event_stop_immediate_propagation() {
        let mut context = Context::default();
        context.register_global_class::<Event>().unwrap();
        context.register_global_class::<EventTarget>().unwrap();

        // Let's set up an EventTarget and dispatch an event
        let register_and_dispatch_script = "
            let target = new EventTarget();
            let ev = new Event('click');
            let called1 = false;
            let called2 = false;

            target.addEventListener('click', () => {
                called1 = true;
                ev.stopImmediatePropagation();
            });

            target.addEventListener('click', () => {
                called2 = true;
            });

            target.dispatchEvent(ev);
            [called1, called2];
        ";
        let res = context
            .eval(Source::from_bytes(register_and_dispatch_script))
            .unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_boolean(), Some(true)); // called1 should be true
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(false)); // called2 should be false because of stopImmediatePropagation!
    }
}
