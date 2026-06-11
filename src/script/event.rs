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

        Ok(Event {
            r#type: event_type,
            target: GcRefCell::new(None),
            current_target: GcRefCell::new(None),
            default_prevented: GcRefCell::new(false),
            propagation_stopped: GcRefCell::new(false),
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
            .method(
                JsString::from("preventDefault"),
                0,
                NativeFunction::from_fn_ptr(prevent_default),
            )
            .method(
                JsString::from("stopPropagation"),
                0,
                NativeFunction::from_fn_ptr(stop_propagation),
            );

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
            if *event.propagation_stopped.borrow() {
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
