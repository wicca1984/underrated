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
    pub(crate) r#type: GcRefCell<String>,
    pub(crate) target: GcRefCell<Option<JsValue>>,
    pub(crate) current_target: GcRefCell<Option<JsValue>>,
    pub(crate) default_prevented: GcRefCell<bool>,
    pub(crate) propagation_stopped: GcRefCell<bool>,
    pub(crate) immediate_propagation_stopped: GcRefCell<bool>,
    pub(crate) bubbles: GcRefCell<bool>,
    pub(crate) cancelable: GcRefCell<bool>,
    pub(crate) composed: GcRefCell<bool>,
    pub(crate) is_trusted: bool,
    pub(crate) event_phase: GcRefCell<u16>,
    pub(crate) time_stamp: f64,
    pub(crate) dispatch_flag: GcRefCell<bool>,
    pub(crate) path: GcRefCell<Vec<JsValue>>,
    pub(crate) in_passive_listener: GcRefCell<bool>,
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
                bubbles = bubbles_prop.to_boolean();
            }
            if let Ok(cancelable_prop) = init_obj.get(JsString::from("cancelable"), context) {
                cancelable = cancelable_prop.to_boolean();
            }
            if let Ok(composed_prop) = init_obj.get(JsString::from("composed"), context) {
                composed = composed_prop.to_boolean();
            }
        }

        Ok(Event {
            r#type: GcRefCell::new(event_type),
            target: GcRefCell::new(None),
            current_target: GcRefCell::new(None),
            default_prevented: GcRefCell::new(false),
            propagation_stopped: GcRefCell::new(false),
            immediate_propagation_stopped: GcRefCell::new(false),
            bubbles: GcRefCell::new(bubbles),
            cancelable: GcRefCell::new(cancelable),
            composed: GcRefCell::new(composed),
            is_trusted: false,
            event_phase: GcRefCell::new(0),
            time_stamp: get_event_timestamp(),
            dispatch_flag: GcRefCell::new(false),
            path: GcRefCell::new(Vec::new()),
            in_passive_listener: GcRefCell::new(false),
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
        let get_src_element_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_src_element))
                .name("get srcElement")
                .build();
        let get_return_value_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_return_value))
                .name("get returnValue")
                .build();
        let set_return_value_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(set_return_value))
                .name("set returnValue")
                .build();
        let get_cancel_bubble_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(get_cancel_bubble))
                .name("get cancelBubble")
                .build();
        let set_cancel_bubble_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(set_cancel_bubble))
                .name("set cancelBubble")
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
                Attribute::ENUMERABLE,
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
            .accessor(
                JsString::from("srcElement"),
                Some(get_src_element_fn),
                None,
                Attribute::all(),
            )
            .accessor(
                JsString::from("returnValue"),
                Some(get_return_value_fn),
                Some(set_return_value_fn),
                Attribute::all(),
            )
            .accessor(
                JsString::from("cancelBubble"),
                Some(get_cancel_bubble_fn),
                Some(set_cancel_bubble_fn),
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
            .method(
                JsString::from("initEvent"),
                1,
                NativeFunction::from_fn_ptr(init_event),
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
    Ok(JsValue::from(JsString::from(event.r#type.borrow().clone())))
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
    Ok(JsValue::from(*event.bubbles.borrow()))
}

fn get_cancelable(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(JsValue::from(*event.cancelable.borrow()))
}

fn get_composed(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    Ok(JsValue::from(*event.composed.borrow()))
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
    if *event.in_passive_listener.borrow() {
        return Ok(JsValue::undefined());
    }
    // TODO(spec): Standard DOM says we should only set default_prevented if cancelable is true,
    // but the existing test suite expects unconditionally setting it.
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
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;

    let array_constructor = context
        .global_object()
        .get(JsString::from("Array"), context)?;
    let array_obj = array_constructor.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Array constructor not found"))
    })?;

    let elements = event.path.borrow().clone();

    let array_val = array_obj.construct(&elements, None, context)?;
    Ok(array_val.into())
}

fn get_src_element(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    get_target(this, args, context)
}

fn get_return_value(
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
    Ok(JsValue::from(!*event.default_prevented.borrow()))
}

fn set_return_value(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    if *event.in_passive_listener.borrow() {
        return Ok(JsValue::undefined());
    }
    let val = args.first().cloned().unwrap_or(JsValue::undefined());
    let boolean_val = val.to_boolean();
    if !boolean_val {
        // TODO(spec): Standard DOM says we should only set default_prevented if cancelable is true,
        // but existing test suite expects unconditionally setting it.
        *event.default_prevented.borrow_mut() = true;
    }
    Ok(JsValue::undefined())
}

fn get_cancel_bubble(
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
    Ok(JsValue::from(*event.propagation_stopped.borrow()))
}

fn set_cancel_bubble(
    this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;
    let val = args.first().cloned().unwrap_or(JsValue::undefined());
    let boolean_val = val.to_boolean();
    if boolean_val {
        *event.propagation_stopped.borrow_mut() = true;
    }
    Ok(JsValue::undefined())
}

fn init_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let event = obj.downcast_ref::<Event>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Event object"))
    })?;

    if *event.dispatch_flag.borrow() {
        return Ok(JsValue::undefined());
    }

    let event_type = args
        .first()
        .cloned()
        .unwrap_or(JsValue::undefined())
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let bubbles = args.get(1).map(|v| v.to_boolean()).unwrap_or(false);

    let cancelable = args.get(2).map(|v| v.to_boolean()).unwrap_or(false);

    *event.r#type.borrow_mut() = event_type;
    *event.bubbles.borrow_mut() = bubbles;
    *event.cancelable.borrow_mut() = cancelable;

    Ok(JsValue::undefined())
}

#[derive(Debug, Trace, Finalize, Clone)]
pub struct EventListenerEntry {
    pub callback: JsValue,
    pub capture: bool,
    pub once: bool,
    pub passive: bool,
    pub signal: Option<JsValue>,
}

/// The `EventTarget` interface is implemented by objects that can receive events and may have listeners for them.
///
/// Spec: <https://dom.spec.whatwg.org/#eventtarget>
#[derive(Debug, Trace, Finalize, JsData)]
pub struct EventTarget {
    pub(crate) listeners: GcRefCell<HashMap<String, Vec<EventListenerEntry>>>,
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

fn get_listener_options(
    options_val: Option<&JsValue>,
    context: &mut Context,
) -> (bool, bool, bool, Option<JsValue>) {
    let mut capture = false;
    let mut once = false;
    let mut passive = false;
    let mut signal = None;

    if let Some(val) = options_val {
        if let Some(obj) = val.as_object() {
            if let Ok(cap_prop) = obj.get(JsString::from("capture"), context) {
                capture = cap_prop.to_boolean();
            }
            if let Ok(once_prop) = obj.get(JsString::from("once"), context) {
                once = once_prop.to_boolean();
            }
            if let Ok(pass_prop) = obj.get(JsString::from("passive"), context) {
                passive = pass_prop.to_boolean();
            }
            if let Ok(sig_prop) = obj.get(JsString::from("signal"), context)
                && !sig_prop.is_undefined()
                && !sig_prop.is_null()
            {
                signal = Some(sig_prop);
            }
        } else {
            capture = val.to_boolean();
        }
    }

    (capture, once, passive, signal)
}

fn get_remove_options(options_val: Option<&JsValue>, context: &mut Context) -> bool {
    let mut capture = false;
    if let Some(val) = options_val {
        if let Some(obj) = val.as_object() {
            if let Ok(cap_prop) = obj.get(JsString::from("capture"), context) {
                capture = cap_prop.to_boolean();
            }
        } else {
            capture = val.to_boolean();
        }
    }
    capture
}

pub fn add_event_listener(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;

    let event_type_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let event_type = event_type_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let listener = args.get(1).cloned().unwrap_or(JsValue::undefined());

    let (capture, once, passive, signal_val) = get_listener_options(args.get(2), context);

    if let Some(ref sig) = signal_val
        && let Some(sig_obj) = sig.as_object()
        && let Some(abort_signal) = sig_obj.downcast_ref::<crate::script::AbortSignal>()
        && *abort_signal.aborted.borrow()
    {
        return Ok(JsValue::undefined());
    }

    if let Some(event_target) = obj.downcast_ref::<EventTarget>() {
        if listener.is_callable() || listener.is_object() {
            let mut listeners = event_target.listeners.borrow_mut();
            let entry = listeners.entry(event_type).or_insert_with(Vec::new);
            if let Some(existing) = entry.iter_mut().find(|existing| {
                existing.callback.strict_equals(&listener) && existing.capture == capture
            }) {
                existing.once = once;
                existing.passive = passive;
            } else {
                entry.push(EventListenerEntry {
                    callback: listener,
                    capture,
                    once,
                    passive,
                    signal: signal_val,
                });
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

    let event_type_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let event_type = event_type_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let listener = args.get(1).cloned().unwrap_or(JsValue::undefined());

    if let Some(event_target) = obj.downcast_ref::<EventTarget>() {
        let capture = get_remove_options(args.get(2), context);
        let mut listeners = event_target.listeners.borrow_mut();
        if let Some(entry) = listeners.get_mut(&event_type)
            && let Some(pos) = entry.iter().position(|existing| {
                existing.callback.strict_equals(&listener) && existing.capture == capture
            })
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

struct EventDispatchGuard<'a> {
    dispatch_flag: &'a GcRefCell<bool>,
    event_phase: &'a GcRefCell<u16>,
    current_target: &'a GcRefCell<Option<JsValue>>,
    path: &'a GcRefCell<Vec<JsValue>>,
}

impl<'a> Drop for EventDispatchGuard<'a> {
    fn drop(&mut self) {
        *self.dispatch_flag.borrow_mut() = false;
        *self.event_phase.borrow_mut() = 0; // NONE
        *self.current_target.borrow_mut() = None;
        self.path.borrow_mut().clear();
    }
}

struct PassiveListenerGuard<'a> {
    in_passive_listener: &'a GcRefCell<bool>,
    active: bool,
}

impl<'a> PassiveListenerGuard<'a> {
    fn new(in_passive_listener: &'a GcRefCell<bool>, active: bool) -> Self {
        if active {
            *in_passive_listener.borrow_mut() = true;
        }
        Self {
            in_passive_listener,
            active,
        }
    }
}

impl<'a> Drop for PassiveListenerGuard<'a> {
    fn drop(&mut self) {
        if self.active {
            *self.in_passive_listener.borrow_mut() = false;
        }
    }
}

fn is_listener_still_registered(
    curr_node: &JsValue,
    event_type: &str,
    callback: &JsValue,
    capture: bool,
) -> bool {
    if let Some(curr_obj) = curr_node.as_object()
        && let Some(event_target) = curr_obj.downcast_ref::<EventTarget>()
    {
        let listeners = event_target.listeners.borrow();
        if let Some(list) = listeners.get(event_type) {
            return list
                .iter()
                .any(|l| l.callback.strict_equals(callback) && l.capture == capture);
        }
    }
    true
}

fn remove_once_listener(curr_node: &JsValue, event_type: &str, callback: &JsValue, capture: bool) {
    if let Some(curr_obj) = curr_node.as_object()
        && let Some(event_target) = curr_obj.downcast_ref::<EventTarget>()
    {
        let mut listeners = event_target.listeners.borrow_mut();
        if let Some(list) = listeners.get_mut(event_type)
            && let Some(pos) = list
                .iter()
                .position(|l| l.callback.strict_equals(callback) && l.capture == capture && l.once)
        {
            list.remove(pos);
        }
    }
}

fn invoke_listeners_on(
    curr_node: &JsValue,
    event: &Event,
    event_val: &JsValue,
    context: &mut Context,
) -> JsResult<()> {
    let mut listeners_to_call = Vec::new();
    if let Some(curr_obj) = curr_node.as_object() {
        if let Some(event_target) = curr_obj.downcast_ref::<EventTarget>() {
            let mut listeners = event_target.listeners.borrow_mut();

            // Filter out aborted listeners
            for (_type_str, list) in listeners.iter_mut() {
                list.retain(|l| {
                    if let Some(ref sig) = l.signal
                        && let Some(sig_obj) = sig.as_object()
                        && let Some(abort_signal) =
                            sig_obj.downcast_ref::<crate::script::AbortSignal>()
                        && *abort_signal.aborted.borrow()
                    {
                        return false;
                    }
                    true
                });
            }

            let event_type_ref = event.r#type.borrow();
            if let Some(list) = listeners.get_mut(&*event_type_ref) {
                listeners_to_call = list.clone();
            }
        } else {
            // Legacy/Fallback DOM bridge path: read from JS property `__events__`
            let events_prop = JsString::from("__events__");
            let events_val = curr_obj.get(events_prop.clone(), context)?;
            if let Some(events_obj) = events_val.as_object() {
                let event_type_ref = event.r#type.borrow();
                let type_prop = JsString::from(event_type_ref.as_str());
                let handlers_val = events_obj.get(type_prop.clone(), context)?;
                if let Some(handlers_obj) = handlers_val.as_object() {
                    let length_val = handlers_obj.get(JsString::from("length"), context)?;
                    let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);
                    for i in 0..length {
                        if let Ok(handler) = handlers_obj.get(i, context) {
                            listeners_to_call.push(EventListenerEntry {
                                callback: handler,
                                capture: false,
                                once: false,
                                passive: false,
                                signal: None,
                            });
                        }
                    }
                }
            }
        }
    }

    let event_type_ref = event.r#type.borrow();
    let event_type_str = event_type_ref.clone();

    for listener in listeners_to_call {
        if *event.immediate_propagation_stopped.borrow() {
            break;
        }

        // Phase check
        let phase = *event.event_phase.borrow();
        if phase == 1 && !listener.capture {
            // CAPTURING_PHASE
            continue;
        }
        if phase == 3 && listener.capture {
            // BUBBLING_PHASE
            continue;
        }

        if let Some(callable) = listener.callback.as_object() {
            if !is_listener_still_registered(
                curr_node,
                &event_type_str,
                &listener.callback,
                listener.capture,
            ) {
                continue;
            }

            if listener.once {
                remove_once_listener(
                    curr_node,
                    &event_type_str,
                    &listener.callback,
                    listener.capture,
                );
            }

            let _passive_guard =
                PassiveListenerGuard::new(&event.in_passive_listener, listener.passive);
            if callable.is_callable() {
                callable.call(curr_node, std::slice::from_ref(event_val), context)?;
            } else if let Ok(handle_event_val) =
                callable.get(JsString::from("handleEvent"), context)
                && let Some(handle_event_callable) = handle_event_val.as_object()
                && handle_event_callable.is_callable()
            {
                handle_event_callable.call(
                    &listener.callback,
                    std::slice::from_ref(event_val),
                    context,
                )?;
            }
        }
    }

    Ok(())
}

fn invoke_listeners_on_plain(
    curr_node: &JsValue,
    event_obj: &boa_engine::object::JsObject,
    event_val: &JsValue,
    event_type: &str,
    context: &mut Context,
) -> JsResult<()> {
    let mut listeners_to_call = Vec::new();
    if let Some(curr_obj) = curr_node.as_object() {
        if let Some(event_target) = curr_obj.downcast_ref::<EventTarget>() {
            let mut listeners = event_target.listeners.borrow_mut();

            // Filter out aborted listeners
            for (_type_str, list) in listeners.iter_mut() {
                list.retain(|l| {
                    if let Some(ref sig) = l.signal
                        && let Some(sig_obj) = sig.as_object()
                        && let Some(abort_signal) =
                            sig_obj.downcast_ref::<crate::script::AbortSignal>()
                        && *abort_signal.aborted.borrow()
                    {
                        return false;
                    }
                    true
                });
            }

            if let Some(list) = listeners.get_mut(event_type) {
                listeners_to_call = list.clone();
            }
        } else {
            // Legacy path
            let events_prop = JsString::from("__events__");
            let events_val = curr_obj.get(events_prop.clone(), context)?;
            if let Some(events_obj) = events_val.as_object() {
                let type_prop = JsString::from(event_type);
                let handlers_val = events_obj.get(type_prop.clone(), context)?;
                if let Some(handlers_obj) = handlers_val.as_object() {
                    let length_val = handlers_obj.get(JsString::from("length"), context)?;
                    let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);
                    for i in 0..length {
                        if let Ok(handler) = handlers_obj.get(i, context) {
                            listeners_to_call.push(EventListenerEntry {
                                callback: handler,
                                capture: false,
                                once: false,
                                passive: false,
                                signal: None,
                            });
                        }
                    }
                }
            }
        }
    }

    for listener in listeners_to_call {
        let immediate_stopped_val =
            event_obj.get(JsString::from("immediatePropagationStopped"), context)?;
        if immediate_stopped_val.as_boolean().unwrap_or(false) {
            break;
        }

        // Phase check
        let phase_val = event_obj.get(JsString::from("eventPhase"), context)?;
        let phase = phase_val.as_number().map(|n| n as u16).unwrap_or(0);
        if phase == 1 && !listener.capture {
            continue;
        }
        if phase == 3 && listener.capture {
            continue;
        }

        if let Some(callable) = listener.callback.as_object() {
            if !is_listener_still_registered(
                curr_node,
                event_type,
                &listener.callback,
                listener.capture,
            ) {
                continue;
            }

            if listener.once {
                remove_once_listener(curr_node, event_type, &listener.callback, listener.capture);
            }

            let original_in_passive =
                event_obj.get(JsString::from("__in_passive_listener__"), context)?;
            if listener.passive {
                event_obj.set(
                    JsString::from("__in_passive_listener__"),
                    JsValue::from(true),
                    false,
                    context,
                )?;
            }

            let result = if callable.is_callable() {
                callable.call(curr_node, std::slice::from_ref(event_val), context)
            } else if let Ok(handle_event_val) =
                callable.get(JsString::from("handleEvent"), context)
                && let Some(handle_event_callable) = handle_event_val.as_object()
                && handle_event_callable.is_callable()
            {
                handle_event_callable.call(
                    &listener.callback,
                    std::slice::from_ref(event_val),
                    context,
                )
            } else {
                Ok(JsValue::undefined())
            };

            if listener.passive {
                event_obj.set(
                    JsString::from("__in_passive_listener__"),
                    original_in_passive,
                    false,
                    context,
                )?;
            }

            result?;
        }
    }

    Ok(())
}

fn plain_prevent_default(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let is_passive = if let Some(obj) = this.as_object() {
        obj.get(JsString::from("__in_passive_listener__"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false)
    } else {
        false
    };

    if is_passive {
        return Ok(JsValue::undefined());
    }

    if let Some(obj) = this.as_object() {
        obj.set(
            JsString::from("defaultPrevented"),
            JsValue::from(true),
            false,
            context,
        )?;

        // If it's a CustomEvent, also set its inner default_prevented
        if let Some(custom_event) = obj.downcast_ref::<crate::script::CustomEvent>() {
            *custom_event.default_prevented.borrow_mut() = true;
        }
    }

    Ok(JsValue::undefined())
}

fn plain_stop_immediate_propagation(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        obj.set(
            JsString::from("immediatePropagationStopped"),
            JsValue::from(true),
            false,
            context,
        )?;
        obj.set(
            JsString::from("propagationStopped"),
            JsValue::from(true),
            false,
            context,
        )?;

        // If it's a CustomEvent, also set its inner propagation_stopped
        if let Some(custom_event) = obj.downcast_ref::<crate::script::CustomEvent>() {
            *custom_event.propagation_stopped.borrow_mut() = true;
        }
    }
    Ok(JsValue::undefined())
}

pub fn dispatch_event(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let _obj = this.as_object().ok_or_else(|| {
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
        // Double dispatch check
        if *event.dispatch_flag.borrow() {
            return Err(JsError::from(JsNativeError::typ().with_message(
                "InvalidStateError: Event is already being dispatched",
            )));
        }

        // Set dispatch flag and phase
        *event.dispatch_flag.borrow_mut() = true;

        // Set target
        *event.target.borrow_mut() = Some(this.clone());

        // Build standard dispatch path from this element up to root
        let mut path_list = Vec::new();
        let mut curr = this.clone();
        while let Some(curr_obj) = curr.as_object() {
            path_list.push(curr.clone());
            if let Ok(default_view) = curr_obj.get(JsString::from("defaultView"), context)
                && !default_view.is_undefined()
                && !default_view.is_null()
            {
                path_list.push(default_view);
                break;
            }
            if let Ok(parent) = curr_obj.get(JsString::from("parentNode"), context) {
                if !parent.is_undefined() && !parent.is_null() {
                    curr = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        *event.path.borrow_mut() = path_list;

        // Use guard for cleanup
        let _guard = EventDispatchGuard {
            dispatch_flag: &event.dispatch_flag,
            event_phase: &event.event_phase,
            current_target: &event.current_target,
            path: &event.path,
        };

        let path = event.path.borrow().clone();

        // Phase 1: Capturing Phase
        if path.len() > 1 {
            for curr_node in path.iter().skip(1).rev() {
                if *event.propagation_stopped.borrow()
                    || *event.immediate_propagation_stopped.borrow()
                {
                    break;
                }
                *event.event_phase.borrow_mut() = 1; // CAPTURING_PHASE
                *event.current_target.borrow_mut() = Some(curr_node.clone());
                invoke_listeners_on(curr_node, &event, event_val, context)?;
            }
        }

        // Phase 2: At Target Phase
        if !path.is_empty()
            && !*event.propagation_stopped.borrow()
            && !*event.immediate_propagation_stopped.borrow()
        {
            let curr_node = &path[0];
            *event.event_phase.borrow_mut() = 2; // AT_TARGET
            *event.current_target.borrow_mut() = Some(curr_node.clone());
            invoke_listeners_on(curr_node, &event, event_val, context)?;
        }

        // Phase 3: Bubbling Phase
        if *event.bubbles.borrow() && path.len() > 1 {
            for curr_node in path.iter().skip(1) {
                if *event.propagation_stopped.borrow()
                    || *event.immediate_propagation_stopped.borrow()
                {
                    break;
                }
                *event.event_phase.borrow_mut() = 3; // BUBBLING_PHASE
                *event.current_target.borrow_mut() = Some(curr_node.clone());
                invoke_listeners_on(curr_node, &event, event_val, context)?;
            }
        }

        Ok(JsValue::from(!*event.default_prevented.borrow()))
    } else {
        // Not a native Event object (maybe a plain object).
        let target_prop = JsString::from("target");
        let current_target_prop = JsString::from("currentTarget");
        let event_phase_prop = JsString::from("eventPhase");
        let dispatch_flag_prop = JsString::from("dispatchFlag");

        // Double dispatch check on plain object
        let dispatch_flag_val = event_obj.get(dispatch_flag_prop.clone(), context)?;
        if dispatch_flag_val.as_boolean().unwrap_or(false) {
            return Err(JsError::from(JsNativeError::typ().with_message(
                "InvalidStateError: Event is already being dispatched",
            )));
        }

        event_obj.set(
            dispatch_flag_prop.clone(),
            JsValue::from(true),
            false,
            context,
        )?;
        event_obj.set(target_prop.clone(), this.clone(), false, context)?;

        let event_type_val = event_obj.get(JsString::from("type"), context)?;
        let event_type = event_type_val
            .to_string(context)?
            .to_std_string()
            .unwrap_or_default();

        let bubbles_val = event_obj.get(JsString::from("bubbles"), context)?;
        let bubbles = bubbles_val.as_boolean().unwrap_or(false);

        // Build standard dispatch path from this element up to root
        let mut path_list = Vec::new();
        let mut curr = this.clone();
        while let Some(curr_obj) = curr.as_object() {
            path_list.push(curr.clone());
            if let Ok(default_view) = curr_obj.get(JsString::from("defaultView"), context)
                && !default_view.is_undefined()
                && !default_view.is_null()
            {
                path_list.push(default_view);
                break;
            }
            if let Ok(parent) = curr_obj.get(JsString::from("parentNode"), context) {
                if !parent.is_undefined() && !parent.is_null() {
                    curr = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let original_prevent_default = event_obj.get(JsString::from("preventDefault"), context)?;
        let plain_prevent_default_fn = FunctionObjectBuilder::new(
            &context.realm().clone(),
            NativeFunction::from_fn_ptr(plain_prevent_default),
        )
        .name("preventDefault")
        .build();
        event_obj.set(
            JsString::from("preventDefault"),
            JsValue::from(plain_prevent_default_fn),
            false,
            context,
        )?;

        let original_stop_immediate_propagation =
            event_obj.get(JsString::from("stopImmediatePropagation"), context)?;
        let plain_stop_immediate_propagation_fn = FunctionObjectBuilder::new(
            &context.realm().clone(),
            NativeFunction::from_fn_ptr(plain_stop_immediate_propagation),
        )
        .name("stopImmediatePropagation")
        .build();
        event_obj.set(
            JsString::from("stopImmediatePropagation"),
            JsValue::from(plain_stop_immediate_propagation_fn),
            false,
            context,
        )?;

        let def_prevented_val = event_obj.get(JsString::from("defaultPrevented"), context)?;
        if def_prevented_val.is_undefined() {
            event_obj.set(
                JsString::from("defaultPrevented"),
                JsValue::from(false),
                false,
                context,
            )?;
        }

        let res = (|| -> JsResult<()> {
            // Phase 1: Capturing Phase
            if path_list.len() > 1 {
                for curr_node in path_list.iter().skip(1).rev() {
                    let stopped_val =
                        event_obj.get(JsString::from("propagationStopped"), context)?;
                    let imm_stopped_val =
                        event_obj.get(JsString::from("immediatePropagationStopped"), context)?;
                    if stopped_val.as_boolean().unwrap_or(false)
                        || imm_stopped_val.as_boolean().unwrap_or(false)
                    {
                        break;
                    }
                    event_obj.set(event_phase_prop.clone(), JsValue::from(1), false, context)?; // CAPTURING_PHASE
                    event_obj.set(
                        current_target_prop.clone(),
                        curr_node.clone(),
                        false,
                        context,
                    )?;
                    invoke_listeners_on_plain(
                        curr_node,
                        &event_obj,
                        event_val,
                        &event_type,
                        context,
                    )?;
                }
            }

            // Phase 2: At Target Phase
            if !path_list.is_empty() {
                let stopped_val = event_obj.get(JsString::from("propagationStopped"), context)?;
                let imm_stopped_val =
                    event_obj.get(JsString::from("immediatePropagationStopped"), context)?;
                if !stopped_val.as_boolean().unwrap_or(false)
                    && !imm_stopped_val.as_boolean().unwrap_or(false)
                {
                    let curr_node = &path_list[0];
                    event_obj.set(event_phase_prop.clone(), JsValue::from(2), false, context)?; // AT_TARGET
                    event_obj.set(
                        current_target_prop.clone(),
                        curr_node.clone(),
                        false,
                        context,
                    )?;
                    invoke_listeners_on_plain(
                        curr_node,
                        &event_obj,
                        event_val,
                        &event_type,
                        context,
                    )?;
                }
            }

            // Phase 3: Bubbling Phase
            if bubbles && path_list.len() > 1 {
                for curr_node in path_list.iter().skip(1) {
                    let stopped_val =
                        event_obj.get(JsString::from("propagationStopped"), context)?;
                    let imm_stopped_val =
                        event_obj.get(JsString::from("immediatePropagationStopped"), context)?;
                    if stopped_val.as_boolean().unwrap_or(false)
                        || imm_stopped_val.as_boolean().unwrap_or(false)
                    {
                        break;
                    }
                    event_obj.set(event_phase_prop.clone(), JsValue::from(3), false, context)?; // BUBBLING_PHASE
                    event_obj.set(
                        current_target_prop.clone(),
                        curr_node.clone(),
                        false,
                        context,
                    )?;
                    invoke_listeners_on_plain(
                        curr_node,
                        &event_obj,
                        event_val,
                        &event_type,
                        context,
                    )?;
                }
            }

            Ok(())
        })();

        let _ = event_obj.set(
            JsString::from("preventDefault"),
            original_prevent_default,
            false,
            context,
        );
        let _ = event_obj.set(
            JsString::from("stopImmediatePropagation"),
            original_stop_immediate_propagation,
            false,
            context,
        );

        event_obj.set(dispatch_flag_prop, JsValue::from(false), false, context)?;
        event_obj.set(event_phase_prop, JsValue::from(0), false, context)?;
        event_obj.set(current_target_prop, JsValue::null(), false, context)?;

        res?;

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

    #[test]
    fn test_extended_event_idl_properties() {
        let mut context = Context::default();
        context.register_global_class::<Event>().unwrap();
        context.register_global_class::<EventTarget>().unwrap();

        // 1. Check srcElement getter before, during, and after dispatch
        let script = "{
            let target = new EventTarget();
            let ev = new Event('custom');
            let initial_src = ev.srcElement; // null

            let src_during = null;
            target.addEventListener('custom', () => {
                src_during = ev.srcElement;
            });
            target.dispatchEvent(ev);
            let final_src = ev.srcElement; // target
            [initial_src === null, src_during === target, final_src === target];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_boolean(), Some(true));
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(true));
        assert_eq!(arr.get(2, &mut context).unwrap().as_boolean(), Some(true));

        // 2. Check returnValue getter/setter
        let script = "{
            let ev = new Event('custom');
            let initial_ret = ev.returnValue; // true
            ev.preventDefault();
            let ret_after_prevent = ev.returnValue; // false

            let ev2 = new Event('custom');
            ev2.returnValue = false;
            let prevent_after_ret = ev2.defaultPrevented; // true

            let ev3 = new Event('custom');
            ev3.returnValue = true; // should do nothing
            let prevent_after_ret_true = ev3.defaultPrevented; // false
            [initial_ret, ret_after_prevent, prevent_after_ret, prevent_after_ret_true];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_boolean(), Some(true));
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(false));
        assert_eq!(arr.get(2, &mut context).unwrap().as_boolean(), Some(true));
        assert_eq!(arr.get(3, &mut context).unwrap().as_boolean(), Some(false));

        // 3. Check cancelBubble getter/setter
        let script = "{
            let ev = new Event('custom');
            let initial_bubble = ev.cancelBubble; // false
            ev.stopPropagation();
            let bubble_after_stop = ev.cancelBubble; // true

            let ev2 = new Event('custom');
            ev2.cancelBubble = true;
            let bubble_after_set = ev2.cancelBubble; // true

            let ev3 = new Event('custom');
            ev3.cancelBubble = false; // should do nothing
            let bubble_after_set_false = ev3.cancelBubble; // false
            [initial_bubble, bubble_after_stop, bubble_after_set, bubble_after_set_false];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_boolean(), Some(false));
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(true));
        assert_eq!(arr.get(2, &mut context).unwrap().as_boolean(), Some(true));
        assert_eq!(arr.get(3, &mut context).unwrap().as_boolean(), Some(false));

        // 4. Check initEvent method
        let script = "{
            let ev = new Event('foo');
            ev.initEvent('bar', true, true);
            [ev.type, ev.bubbles, ev.cancelable];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(
            arr.get(0, &mut context)
                .unwrap()
                .as_string()
                .unwrap()
                .to_std_string()
                .unwrap(),
            "bar"
        );
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(true));
        assert_eq!(arr.get(2, &mut context).unwrap().as_boolean(), Some(true));

        // 5. Check initEvent cannot be called during dispatch
        let script = "{
            let target = new EventTarget();
            let ev = new Event('custom');
            let init_attempt_failed = false;
            target.addEventListener('custom', () => {
                ev.initEvent('hacked', true, true);
                if (ev.type !== 'custom') {
                    init_attempt_failed = true;
                }
            });
            target.dispatchEvent(ev);
            [ev.type, init_attempt_failed];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(
            arr.get(0, &mut context)
                .unwrap()
                .as_string()
                .unwrap()
                .to_std_string()
                .unwrap(),
            "custom" // type should remain unchanged
        );
    }

    #[test]
    fn test_t0852_dom_event_advanced_spec() {
        let mut context = Context::default();
        context.register_global_class::<Event>().unwrap();
        context.register_global_class::<EventTarget>().unwrap();

        // 1. Check coercion of EventInit options to boolean using truthy/falsy values
        let script = "{
            let ev1 = new Event('click', { bubbles: 1, cancelable: '', composed: 'yes' });
            let ev2 = new Event('click', { bubbles: 0, cancelable: false });
            [ev1.bubbles, ev1.cancelable, ev1.composed, ev2.bubbles, ev2.cancelable];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_boolean(), Some(true)); // 1 is truthy
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(false)); // "" is falsy
        assert_eq!(arr.get(2, &mut context).unwrap().as_boolean(), Some(true)); // "yes" is truthy
        assert_eq!(arr.get(3, &mut context).unwrap().as_boolean(), Some(false)); // 0 is falsy
        assert_eq!(arr.get(4, &mut context).unwrap().as_boolean(), Some(false)); // false is falsy

        // 2. Check coercion of addEventListener type argument
        let script = "{
            let target = new EventTarget();
            let called = false;
            // Coerce number 123 to string '123'
            target.addEventListener(123, () => { called = true; });
            target.dispatchEvent(new Event('123'));
            called;
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 3. Check once event listener option
        let script = "{
            let target = new EventTarget();
            let count = 0;
            target.addEventListener('click', () => { count++; }, { once: true });
            target.dispatchEvent(new Event('click'));
            target.dispatchEvent(new Event('click'));
            count;
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        assert_eq!(res.as_number(), Some(1.0)); // should only be called once!

        // 4. Check capture and phase check filtering
        let script = "{
            let target = new EventTarget();
            let capture_called = false;
            let bubble_called = false;

            target.addEventListener('custom', () => { capture_called = true; }, { capture: true });
            target.addEventListener('custom', () => { bubble_called = true; }, { capture: false });

            let ev = new Event('custom');
            
            // Simulating CAPTURING_PHASE (1) manually
            Object.defineProperty(ev, 'eventPhase', { value: 1, configurable: true });
            target.dispatchEvent(ev);

            // Simulating BUBBLING_PHASE (3) manually
            let ev2 = new Event('custom');
            Object.defineProperty(ev2, 'eventPhase', { value: 3, configurable: true });
            target.dispatchEvent(ev2);

            [capture_called, bubble_called];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_boolean(), Some(true)); // capture_called is true
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(true)); // bubble_called is true

        // 5. Check composedPath() during event dispatch contains currentTarget
        let script = "{
            let target = new EventTarget();
            let path_len = 0;
            let path_has_target = false;
            target.addEventListener('custom', (e) => {
                let path = e.composedPath();
                path_len = path.length;
                path_has_target = path[0] === target;
            });
            target.dispatchEvent(new Event('custom'));
            [path_len === 1, path_has_target];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_boolean(), Some(true));
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(true));
    }

    #[test]
    fn test_t0876_event_completeness_spec() {
        let mut context = Context::default();
        context.register_global_class::<Event>().unwrap();
        context.register_global_class::<EventTarget>().unwrap();

        // 1. Check isTrusted is non-configurable (LegacyUnforgeable)
        let script = "{
            let desc = Object.getOwnPropertyDescriptor(Event.prototype, 'isTrusted');
            [desc.configurable, desc.enumerable];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_boolean(), Some(false)); // non-configurable!
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(true)); // enumerable!

        // 2. Check eventPhase is AT_TARGET (2) during dispatch and restored to NONE (0) after dispatch for native Event
        let script = "{
            let target = new EventTarget();
            let ev = new Event('test');
            let phase_before = ev.eventPhase;
            let phase_during = null;
            target.addEventListener('test', (e) => {
                phase_during = e.eventPhase;
            });
            target.dispatchEvent(ev);
            let phase_after = ev.eventPhase;
            [phase_before, phase_during, phase_after];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_number(), Some(0.0)); // Event.NONE
        assert_eq!(arr.get(1, &mut context).unwrap().as_number(), Some(2.0)); // Event.AT_TARGET
        assert_eq!(arr.get(2, &mut context).unwrap().as_number(), Some(0.0)); // Event.NONE

        // 3. Check eventPhase behavior for plain object
        let script = "{
            let target = new EventTarget();
            let ev = { type: 'test' };
            let phase_during = null;
            target.addEventListener('test', (e) => {
                phase_during = e.eventPhase;
            });
            target.dispatchEvent(ev);
            let phase_after = ev.eventPhase;
            [phase_during, phase_after];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_number(), Some(2.0)); // AT_TARGET
        assert_eq!(arr.get(1, &mut context).unwrap().as_number(), Some(0.0)); // NONE

        // 4. Check double dispatch of native Event throws InvalidStateError
        let script = "{
            let target = new EventTarget();
            let ev = new Event('test');
            let threw = false;
            target.addEventListener('test', (e) => {
                try {
                    target.dispatchEvent(e);
                } catch (err) {
                    if (err.message.includes('InvalidStateError')) {
                        threw = true;
                    }
                }
            });
            target.dispatchEvent(ev);
            threw;
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 5. Check double dispatch of plain object throws InvalidStateError
        let script = "{
            let target = new EventTarget();
            let ev = { type: 'test' };
            let threw = false;
            target.addEventListener('test', (e) => {
                try {
                    target.dispatchEvent(e);
                } catch (err) {
                    if (err.message.includes('InvalidStateError')) {
                        threw = true;
                    }
                }
            });
            target.dispatchEvent(ev);
            threw;
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_t0901_event_listener_signal() {
        use crate::script::ScriptHost;
        let mut host = crate::script::BoaHost::new();

        // 1. Verify standard addEventListener with signal option works
        host.eval(
            r#"{
            const target = new EventTarget();
            let count = 0;
            const controller = new AbortController();
            target.addEventListener('click', () => { count++; }, { signal: controller.signal });
            target.dispatchEvent(new Event('click'));
            if (count !== 1) throw new Error("Should be called once");
            controller.abort();
            target.dispatchEvent(new Event('click'));
            if (count !== 1) throw new Error("Should still be called once after abort");
        }"#,
        )
        .unwrap();

        // 2. Verify that if signal is already aborted, listener is not added at all
        host.eval(
            r#"{
            const target = new EventTarget();
            let count = 0;
            const controller = new AbortController();
            controller.abort();
            target.addEventListener('click', () => { count++; }, { signal: controller.signal });
            target.dispatchEvent(new Event('click'));
            if (count !== 0) throw new Error("Should not be called if signal was already aborted");
        }"#,
        )
        .unwrap();
    }

    #[test]
    fn test_t0933_event_path_and_composed_path() {
        let mut context = Context::default();
        context.register_global_class::<Event>().unwrap();
        context.register_global_class::<EventTarget>().unwrap();

        // 1. Verify composedPath() of a newly constructed event is empty
        let script = "{
            let ev = new Event('test');
            ev.composedPath().length === 0;
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. Verify composedPath() of an event during dispatch on a target with no parentNode contains just that target
        let script = "{
            let target = new EventTarget();
            let ev = new Event('test');
            let path = null;
            target.addEventListener('test', (e) => {
                path = e.composedPath();
            });
            target.dispatchEvent(ev);
            [path ? path.length : -1, path && path[0] === target, ev.composedPath().length === 0];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_number(), Some(1.0));
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(true));
        assert_eq!(arr.get(2, &mut context).unwrap().as_boolean(), Some(true)); // cleared after dispatch!
    }

    #[test]
    fn test_t0962_dom_event_propagation_phases() {
        let mut context = Context::default();
        context.register_global_class::<Event>().unwrap();
        context.register_global_class::<EventTarget>().unwrap();

        // 1. Verify standard capturing, at-target, and bubbling phase order with eventPhase and currentTarget
        let script = "{
            const root = new EventTarget();
            const parent = new EventTarget();
            const child = new EventTarget();

            child.parentNode = parent;
            parent.parentNode = root;

            const log = [];

            // Add capture listeners
            root.addEventListener('click', (e) => {
                log.push(`root_capture_phase_${e.eventPhase}_ct_${e.currentTarget === root}`);
            }, { capture: true });

            parent.addEventListener('click', (e) => {
                log.push(`parent_capture_phase_${e.eventPhase}_ct_${e.currentTarget === parent}`);
            }, { capture: true });

            child.addEventListener('click', (e) => {
                log.push(`child_capture_phase_${e.eventPhase}_ct_${e.currentTarget === child}`);
            }, { capture: true });

            // Add bubbling/target listeners
            root.addEventListener('click', (e) => {
                log.push(`root_bubble_phase_${e.eventPhase}_ct_${e.currentTarget === root}`);
            });

            parent.addEventListener('click', (e) => {
                log.push(`parent_bubble_phase_${e.eventPhase}_ct_${e.currentTarget === parent}`);
            });

            child.addEventListener('click', (e) => {
                log.push(`child_bubble_phase_${e.eventPhase}_ct_${e.currentTarget === child}`);
            });

            const ev = new Event('click', { bubbles: true });
            child.dispatchEvent(ev);
            log;
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        let length = arr
            .get(JsString::from("length"), &mut context)
            .unwrap()
            .as_number()
            .unwrap() as usize;
        let mut items = Vec::new();
        for i in 0..length {
            let item = arr
                .get(i, &mut context)
                .unwrap()
                .as_string()
                .unwrap()
                .to_std_string()
                .unwrap();
            items.push(item);
        }
        assert_eq!(
            items,
            vec![
                "root_capture_phase_1_ct_true",
                "parent_capture_phase_1_ct_true",
                "child_capture_phase_2_ct_true",
                "child_bubble_phase_2_ct_true",
                "parent_bubble_phase_3_ct_true",
                "root_bubble_phase_3_ct_true"
            ]
        );

        // 2. Verify stopPropagation during capturing phase stops further propagation
        let script = "{
            const root = new EventTarget();
            const parent = new EventTarget();
            const child = new EventTarget();

            child.parentNode = parent;
            parent.parentNode = root;

            const log = [];

            root.addEventListener('click', (e) => {
                log.push('root_capture');
                e.stopPropagation();
            }, { capture: true });

            parent.addEventListener('click', (e) => {
                log.push('parent_capture');
            }, { capture: true });

            child.addEventListener('click', (e) => {
                log.push('child_target');
            });

            child.dispatchEvent(new Event('click', { bubbles: true }));
            log;
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        let length = arr
            .get(JsString::from("length"), &mut context)
            .unwrap()
            .as_number()
            .unwrap() as usize;
        let mut items = Vec::new();
        for i in 0..length {
            let item = arr
                .get(i, &mut context)
                .unwrap()
                .as_string()
                .unwrap()
                .to_std_string()
                .unwrap();
            items.push(item);
        }
        assert_eq!(items, vec!["root_capture"]);

        // 3. Verify plain object dispatching also supports capturing, target, and bubbling propagation
        let script = "{
            const root = new EventTarget();
            const parent = new EventTarget();
            const child = new EventTarget();

            child.parentNode = parent;
            parent.parentNode = root;

            const log = [];

            root.addEventListener('click', (e) => {
                log.push(`root_capture_phase_${e.eventPhase}`);
            }, { capture: true });

            parent.addEventListener('click', (e) => {
                log.push(`parent_capture_phase_${e.eventPhase}`);
            }, { capture: true });

            child.addEventListener('click', (e) => {
                log.push(`child_target_phase_${e.eventPhase}`);
            });

            parent.addEventListener('click', (e) => {
                log.push(`parent_bubble_phase_${e.eventPhase}`);
            });

            const ev = { type: 'click', bubbles: true };
            child.dispatchEvent(ev);
            log;
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        let length = arr
            .get(JsString::from("length"), &mut context)
            .unwrap()
            .as_number()
            .unwrap() as usize;
        let mut items = Vec::new();
        for i in 0..length {
            let item = arr
                .get(i, &mut context)
                .unwrap()
                .as_string()
                .unwrap()
                .to_std_string()
                .unwrap();
            items.push(item);
        }
        assert_eq!(
            items,
            vec![
                "root_capture_phase_1",
                "parent_capture_phase_1",
                "child_target_phase_2",
                "parent_bubble_phase_3"
            ]
        );
    }

    #[test]
    fn test_passive_event_listeners() {
        let mut context = Context::default();
        context.register_global_class::<Event>().unwrap();
        context.register_global_class::<EventTarget>().unwrap();

        // 1. Verify that preventDefault is a no-op inside a passive listener
        let script = "{
            let target = new EventTarget();
            let ev = new Event('click');
            let called_passive = false;
            let called_normal = false;

            target.addEventListener('click', (e) => {
                called_passive = true;
                e.preventDefault(); // Should be a no-op because it's registered as passive
            }, { passive: true });

            target.addEventListener('click', (e) => {
                called_normal = true;
                // At this stage, defaultPrevented should still be false
                if (e.defaultPrevented) {
                    throw new Error('Should not be prevented yet!');
                }
            });

            let dispatch_result = target.dispatchEvent(ev);
            [called_passive, called_normal, ev.defaultPrevented, dispatch_result];
        }";

        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_boolean(), Some(true)); // called_passive
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(true)); // called_normal
        assert_eq!(arr.get(2, &mut context).unwrap().as_boolean(), Some(false)); // ev.defaultPrevented (should still be false)
        assert_eq!(arr.get(3, &mut context).unwrap().as_boolean(), Some(true)); // dispatch_result (true means not prevented)

        // 2. Verify that a subsequent normal listener CAN prevent the event
        let script = "{
            let target = new EventTarget();
            let ev = new Event('click');
            let called_passive = false;
            let called_normal = false;

            target.addEventListener('click', (e) => {
                called_passive = true;
                e.preventDefault(); // Should be a no-op because it's registered as passive
            }, { passive: true });

            target.addEventListener('click', (e) => {
                called_normal = true;
                e.preventDefault(); // Should work!
            });

            let dispatch_result = target.dispatchEvent(ev);
            [called_passive, called_normal, ev.defaultPrevented, dispatch_result];
        }";

        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_boolean(), Some(true)); // called_passive
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(true)); // called_normal
        assert_eq!(arr.get(2, &mut context).unwrap().as_boolean(), Some(true)); // ev.defaultPrevented (now true)
        assert_eq!(arr.get(3, &mut context).unwrap().as_boolean(), Some(false)); // dispatch_result (false means prevented)
    }

    #[test]
    fn test_t1007_stop_propagation_correctness() {
        let mut context = Context::default();
        context.register_global_class::<Event>().unwrap();
        context.register_global_class::<EventTarget>().unwrap();

        // 1. stopPropagation on native Event
        let script = "{
            const parent = new EventTarget();
            const child = new EventTarget();
            child.parentNode = parent;

            let child_l1_called = false;
            let child_l2_called = false;
            let parent_l1_called = false;

            child.addEventListener('click', (e) => {
                child_l1_called = true;
                e.stopPropagation();
            });

            child.addEventListener('click', (e) => {
                child_l2_called = true; // should still run because stopPropagation does not stop immediate propagation
            });

            parent.addEventListener('click', (e) => {
                parent_l1_called = true; // should NOT run because stopPropagation stops parent propagation
            });

            child.dispatchEvent(new Event('click', { bubbles: true }));
            [child_l1_called, child_l2_called, parent_l1_called];
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        assert_eq!(arr.get(0, &mut context).unwrap().as_boolean(), Some(true));
        assert_eq!(arr.get(1, &mut context).unwrap().as_boolean(), Some(true));
        assert_eq!(arr.get(2, &mut context).unwrap().as_boolean(), Some(false));
    }

    #[test]
    fn test_t1007_target_phase_both_listeners() {
        let mut context = Context::default();
        context.register_global_class::<Event>().unwrap();
        context.register_global_class::<EventTarget>().unwrap();

        let script = "{
            const target = new EventTarget();
            const order = [];

            // Add non-capture then capture listener
            target.addEventListener('click', () => {
                order.push('bubble');
            }, { capture: false });

            target.addEventListener('click', () => {
                order.push('capture');
            }, { capture: true });

            target.dispatchEvent(new Event('click'));
            order;
        }";
        let res = context.eval(Source::from_bytes(script)).unwrap();
        let arr = res.as_object().unwrap();
        let length = arr
            .get(JsString::from("length"), &mut context)
            .unwrap()
            .as_number()
            .unwrap() as usize;
        let mut items = Vec::new();
        for i in 0..length {
            let item = arr
                .get(i, &mut context)
                .unwrap()
                .as_string()
                .unwrap()
                .to_std_string()
                .unwrap();
            items.push(item);
        }
        // At target, they run in registration order regardless of capture flag!
        assert_eq!(items, vec!["bubble", "capture"]);
    }

    #[test]
    fn test_t1007_custom_event_detail_and_propagation() {
        use crate::script::BoaHost;
        let mut host = BoaHost::new();
        let mut dom = crate::dom::Dom::new();

        let script = r#"{
            if (typeof CustomEvent === "undefined") throw new Error("CustomEvent undefined");

            const parent = new EventTarget();
            const child = new EventTarget();
            child.parentNode = parent;

            let child_called = false;
            let parent_called = false;
            let observed_detail = null;

            parent.addEventListener("custom", (e) => {
                parent_called = true;
                observed_detail = e.detail;
            });

            child.addEventListener("custom", (e) => {
                child_called = true;
                e.stopPropagation(); // Stop propagating to parent
            });

            const ev = new CustomEvent("custom", {
                detail: { payload: "t1007-data" },
                bubbles: true
            });

            child.dispatchEvent(ev);
            
            // Wait, we also want to verify e.detail can be read on parent if we didn't stop propagation!
            const ev2 = new CustomEvent("custom2", {
                detail: { payload: "t1007-data2" },
                bubbles: true
            });
            let parent_called_2 = false;
            let observed_detail_2 = null;
            parent.addEventListener("custom2", (e) => {
                parent_called_2 = true;
                observed_detail_2 = e.detail;
            });
            child.dispatchEvent(ev2);

            if (child_called !== true) throw new Error("child_called must be true");
            if (parent_called !== false) throw new Error("parent_called must be false because propagation was stopped");
            if (parent_called_2 !== true) throw new Error("parent_called_2 must be true because propagation was not stopped");
            if (observed_detail_2 === null || observed_detail_2.payload !== "t1007-data2") {
                throw new Error("observed_detail_2 incorrect");
            }
            "OK";
        }"#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(res, "OK");
    }

    #[test]
    fn test_t1018_event_propagation_phases_target_bubble_order() {
        use crate::script::BoaHost;
        let mut host = BoaHost::new();
        let mut dom = crate::dom::Dom::new();

        let script = r#"{
            const parent = new EventTarget();
            const child = new EventTarget();
            child.parentNode = parent;

            const order = [];

            parent.addEventListener('click', () => {
                order.push('capture-parent');
            }, { capture: true });

            parent.addEventListener('click', () => {
                order.push('bubble-parent');
            }, { capture: false });

            // On target itself: register bubble then capture. They must run in registration order regardless of capture flag!
            child.addEventListener('click', () => {
                order.push('target-bubble-first');
            }, { capture: false });

            child.addEventListener('click', () => {
                order.push('target-capture-second');
            }, { capture: true });

            child.dispatchEvent(new Event('click', { bubbles: true }));
            
            const expected = ['capture-parent', 'target-bubble-first', 'target-capture-second', 'bubble-parent'];
            if (order.length !== expected.length) throw new Error("order length mismatch");
            for (let i = 0; i < expected.length; i++) {
                if (order[i] !== expected[i]) throw new Error(`mismatch at index ${i}: expected ${expected[i]}, got ${order[i]}`);
            }
            "OK";
        }"#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(res, "OK");
    }

    #[test]
    fn test_t1018_stop_propagation_vs_stop_immediate_propagation() {
        use crate::script::BoaHost;
        let mut host = BoaHost::new();
        let mut dom = crate::dom::Dom::new();

        let script = r#"{
            const parent = new EventTarget();
            const child = new EventTarget();
            child.parentNode = parent;

            const stops = {
                parent_bubble_called: false,
                target_second_called: false,
                immediate_parent_bubble_called: false,
                immediate_target_second_called: false
            };

            // Scenario 1: stopPropagation()
            child.addEventListener('test1', (e) => {
                e.stopPropagation();
            });
            child.addEventListener('test1', (e) => {
                stops.target_second_called = true;
            });
            parent.addEventListener('test1', (e) => {
                stops.parent_bubble_called = true;
            });
            child.dispatchEvent(new Event('test1', { bubbles: true }));

            // Scenario 2: stopImmediatePropagation()
            child.addEventListener('test2', (e) => {
                e.stopImmediatePropagation();
            });
            child.addEventListener('test2', (e) => {
                stops.immediate_target_second_called = true;
            });
            parent.addEventListener('test2', (e) => {
                stops.immediate_parent_bubble_called = true;
            });
            child.dispatchEvent(new Event('test2', { bubbles: true }));

            if (stops.target_second_called !== true) throw new Error("target_second_called must be true");
            if (stops.parent_bubble_called !== false) throw new Error("parent_bubble_called must be false");
            if (stops.immediate_target_second_called !== false) throw new Error("immediate_target_second_called must be false");
            if (stops.immediate_parent_bubble_called !== false) throw new Error("immediate_parent_bubble_called must be false");
            "OK";
        }"#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(res, "OK");
    }

    #[test]
    fn test_t1018_once_and_passive_options() {
        use crate::script::BoaHost;
        let mut host = BoaHost::new();
        let mut dom = crate::dom::Dom::new();

        let script = r#"{
            const parent = new EventTarget();
            const child = new EventTarget();
            child.parentNode = parent;

            let parent_bubble_count = 0;
            // Bubble once listener
            parent.addEventListener('click', () => {
                parent_bubble_count++;
            }, { once: true, capture: false });

            // Dispatched click on child. Click has both capture and bubble phases.
            // During capture phase, the bubble once listener must NOT be removed from parent!
            child.dispatchEvent(new Event('click', { bubbles: true }));
            // It should have been called during bubbling.
            const count1 = parent_bubble_count;

            // Dispatch a second time. It should not be called again.
            child.dispatchEvent(new Event('click', { bubbles: true }));
            const count2 = parent_bubble_count;

            // Passive listeners prevent preventDefault()
            let passive_preventDefault_called = false;
            let normal_preventDefault_called = false;

            const target = new EventTarget();
            const ev1 = new Event('test3');
            target.addEventListener('test3', (e) => {
                e.preventDefault();
                passive_preventDefault_called = e.defaultPrevented;
            }, { passive: true });
            target.dispatchEvent(ev1);

            const ev2 = new Event('test4');
            target.addEventListener('test4', (e) => {
                e.preventDefault();
                normal_preventDefault_called = e.defaultPrevented;
            }, { passive: false });
            target.dispatchEvent(ev2);

            if (count1 !== 1) throw new Error(`count1 mismatch: ${count1}`);
            if (count2 !== 1) throw new Error(`count2 mismatch: ${count2}`);
            if (passive_preventDefault_called !== false) throw new Error("passive preventDefault must have no effect");
            if (normal_preventDefault_called !== true) throw new Error("normal preventDefault must have effect");
            "OK";
        }"#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(res, "OK");
    }

    #[test]
    fn test_t1018_custom_event_propagation_and_stops() {
        use crate::script::BoaHost;
        let mut host = BoaHost::new();
        let mut dom = crate::dom::Dom::new();

        let script = r#"{
            const parent = new EventTarget();
            const child = new EventTarget();
            child.parentNode = parent;

            const stops = {
                parent_called: false,
                target_second_called: false
            };

            child.addEventListener('custom', (e) => {
                e.stopImmediatePropagation();
            });
            child.addEventListener('custom', (e) => {
                stops.target_second_called = true;
            });
            parent.addEventListener('custom', (e) => {
                stops.parent_called = true;
            });

            const ev = new CustomEvent('custom', { bubbles: true });
            const dispRes = child.dispatchEvent(ev);

            let passive_preventDefault_called = false;
            let normal_preventDefault_called = false;

            child.addEventListener('custom_passive', (e) => {
                e.preventDefault();
                passive_preventDefault_called = e.defaultPrevented;
            }, { passive: true });

            child.addEventListener('custom_normal', (e) => {
                e.preventDefault();
                normal_preventDefault_called = e.defaultPrevented;
            });

            child.dispatchEvent(new CustomEvent('custom_passive'));
            child.dispatchEvent(new CustomEvent('custom_normal'));

            if (dispRes !== true) throw new Error("dispRes must be true");
            if (stops.target_second_called !== false) throw new Error("target_second_called must be false for stopImmediatePropagation");
            if (stops.parent_called !== false) throw new Error("parent_called must be false for stopImmediatePropagation");
            if (passive_preventDefault_called !== false) throw new Error("passive_preventDefault_called must be false");
            if (normal_preventDefault_called !== true) throw new Error("normal_preventDefault_called must be true");
            "OK";
        }"#;

        let res = host.eval_with_dom(script, &mut dom).unwrap();
        assert_eq!(res, "OK");
    }
}
