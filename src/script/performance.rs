//! Performance JS object implementation.
//!
//! This module defines the `Performance` object which provides access to performance-related
//! information for the current context.

use boa_engine::class::{Class, ClassBuilder};
use boa_engine::object::{FunctionObjectBuilder, JsObject, ObjectInitializer};
use boa_engine::property::Attribute;
use boa_engine::{
    Context, JsData, JsError, JsNativeError, JsResult, JsString, JsValue, NativeFunction,
};
use boa_gc::{Finalize, GcRefCell, Trace};
use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static TIME_ORIGINS: OnceLock<(Instant, f64)> = OnceLock::new();

/// Returns the time origins initialized together.
fn get_origins() -> &'static (Instant, f64) {
    TIME_ORIGINS.get_or_init(|| {
        let instant = Instant::now();
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0;
        (instant, ms)
    })
}

/// Returns the time origin `Instant`. Initialized on first call.
fn get_time_origin() -> &'static Instant {
    &get_origins().0
}

/// Returns the time origin Unix epoch high-res timestamp in milliseconds.
fn get_time_origin_ms() -> f64 {
    get_origins().1
}

/// PerformanceEntry base class representation.
#[derive(Debug, Trace, Finalize, JsData)]
pub struct PerformanceEntry {}

impl Class for PerformanceEntry {
    const NAME: &'static str = "PerformanceEntry";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Err(JsError::from(
            JsNativeError::typ().with_message("TypeError: Illegal constructor"),
        ))
    }

    fn init(_class: &mut ClassBuilder<'_>) -> JsResult<()> {
        Ok(())
    }
}

/// Representation of a stored performance mark.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct PerformanceMark {
    pub name: String,
    pub start_time: f64,
    pub detail: JsValue,
}

/// Representation of a stored performance measure.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct PerformanceMeasure {
    pub name: String,
    pub start_time: f64,
    pub duration: f64,
    pub detail: JsValue,
}

/// Performance JS Class host struct.
#[derive(Debug, Trace, Finalize, JsData)]
pub struct Performance {
    pub(crate) marks: GcRefCell<Vec<JsObject>>,
    pub(crate) measures: GcRefCell<Vec<JsObject>>,
}

impl Class for Performance {
    const NAME: &'static str = "Performance";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Ok(Performance {
            marks: GcRefCell::new(Vec::new()),
            measures: GcRefCell::new(Vec::new()),
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let get_time_origin_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_get_time_origin),
        )
        .name("get timeOrigin")
        .build();

        let get_timing_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(performance_get_timing))
                .name("get timing")
                .build();

        let get_navigation_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_get_navigation),
        )
        .name("get navigation")
        .build();

        class
            .accessor(
                JsString::from("timeOrigin"),
                Some(get_time_origin_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("timing"),
                Some(get_timing_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("navigation"),
                Some(get_navigation_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(
                JsString::from("now"),
                0,
                NativeFunction::from_fn_ptr(performance_now),
            )
            .method(
                JsString::from("mark"),
                1,
                NativeFunction::from_fn_ptr(performance_mark),
            )
            .method(
                JsString::from("measure"),
                1,
                NativeFunction::from_fn_ptr(performance_measure),
            )
            .method(
                JsString::from("clearMarks"),
                0,
                NativeFunction::from_fn_ptr(performance_clear_marks),
            )
            .method(
                JsString::from("clearMeasures"),
                0,
                NativeFunction::from_fn_ptr(performance_clear_measures),
            )
            .method(
                JsString::from("clearResourceTimings"),
                0,
                NativeFunction::from_fn_ptr(performance_clear_resource_timings),
            )
            .method(
                JsString::from("setResourceTimingBufferSize"),
                1,
                NativeFunction::from_fn_ptr(performance_set_resource_timing_buffer_size),
            )
            .method(
                JsString::from("getEntries"),
                0,
                NativeFunction::from_fn_ptr(performance_get_entries),
            )
            .method(
                JsString::from("getEntriesByType"),
                1,
                NativeFunction::from_fn_ptr(performance_get_entries_by_type),
            )
            .method(
                JsString::from("getEntriesByName"),
                1,
                NativeFunction::from_fn_ptr(performance_get_entries_by_name),
            )
            .method(
                JsString::from("toJSON"),
                0,
                NativeFunction::from_fn_ptr(performance_to_json),
            );

        Ok(())
    }
}

fn performance_get_time_origin(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(JsValue::from(get_time_origin_ms()))
}

fn performance_now(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let origin = get_time_origin();
    let elapsed = origin.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    Ok(JsValue::from(elapsed_ms))
}

fn performance_to_json(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let obj = ObjectInitializer::new(context)
        .property(
            JsString::from("timeOrigin"),
            JsValue::from(get_time_origin_ms()),
            ro,
        )
        .build();
    Ok(JsValue::from(obj))
}

fn performance_timing_to_json(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("toJSON called on non-object"))
    })?;

    let keys = [
        "navigationStart",
        "unloadEventStart",
        "unloadEventEnd",
        "redirectStart",
        "redirectEnd",
        "fetchStart",
        "domainLookupStart",
        "domainLookupEnd",
        "connectStart",
        "connectEnd",
        "secureConnectionStart",
        "requestStart",
        "responseStart",
        "responseEnd",
        "domLoading",
        "domInteractive",
        "domContentLoadedEventStart",
        "domContentLoadedEventEnd",
        "domComplete",
        "loadEventStart",
        "loadEventEnd",
    ];

    let mut values = Vec::with_capacity(keys.len());
    for key in keys {
        let val = obj.get(JsString::from(key), context)?;
        values.push((JsString::from(key), val));
    }

    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let mut initializer = ObjectInitializer::new(context);
    for (key, val) in values {
        initializer.property(key, val, ro);
    }

    Ok(JsValue::from(initializer.build()))
}

fn performance_navigation_to_json(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("toJSON called on non-object"))
    })?;

    let keys = ["type", "redirectCount"];
    let mut values = Vec::with_capacity(keys.len());
    for key in keys {
        let val = obj.get(JsString::from(key), context)?;
        values.push((JsString::from(key), val));
    }

    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let mut initializer = ObjectInitializer::new(context);
    for (key, val) in values {
        initializer.property(key, val, ro);
    }

    Ok(JsValue::from(initializer.build()))
}

fn performance_get_timing(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let origin_ms = get_time_origin_ms();
    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let realm = context.realm().clone();
    let to_json_fn = FunctionObjectBuilder::new(
        &realm,
        NativeFunction::from_fn_ptr(performance_timing_to_json),
    )
    .name("toJSON")
    .build();

    let timing_obj = ObjectInitializer::new(context)
        .property(
            JsString::from("navigationStart"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(JsString::from("unloadEventStart"), JsValue::from(0), ro)
        .property(JsString::from("unloadEventEnd"), JsValue::from(0), ro)
        .property(JsString::from("redirectStart"), JsValue::from(0), ro)
        .property(JsString::from("redirectEnd"), JsValue::from(0), ro)
        .property(JsString::from("fetchStart"), JsValue::from(origin_ms), ro)
        .property(
            JsString::from("domainLookupStart"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(
            JsString::from("domainLookupEnd"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(JsString::from("connectStart"), JsValue::from(origin_ms), ro)
        .property(JsString::from("connectEnd"), JsValue::from(origin_ms), ro)
        .property(
            JsString::from("secureConnectionStart"),
            JsValue::from(0),
            ro,
        )
        .property(JsString::from("requestStart"), JsValue::from(origin_ms), ro)
        .property(
            JsString::from("responseStart"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(JsString::from("responseEnd"), JsValue::from(origin_ms), ro)
        .property(JsString::from("domLoading"), JsValue::from(origin_ms), ro)
        .property(
            JsString::from("domInteractive"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(
            JsString::from("domContentLoadedEventStart"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(
            JsString::from("domContentLoadedEventEnd"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(JsString::from("domComplete"), JsValue::from(origin_ms), ro)
        .property(
            JsString::from("loadEventStart"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(JsString::from("loadEventEnd"), JsValue::from(origin_ms), ro)
        .property(JsString::from("toJSON"), to_json_fn, ro)
        .build();
    Ok(JsValue::from(timing_obj))
}

fn performance_get_navigation(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let realm = context.realm().clone();
    let to_json_fn = FunctionObjectBuilder::new(
        &realm,
        NativeFunction::from_fn_ptr(performance_navigation_to_json),
    )
    .name("toJSON")
    .build();

    let nav_obj = ObjectInitializer::new(context)
        .property(JsString::from("type"), JsValue::from(0), ro)
        .property(JsString::from("redirectCount"), JsValue::from(0), ro)
        .property(JsString::from("toJSON"), to_json_fn, ro)
        .build();
    Ok(JsValue::from(nav_obj))
}

fn performance_mark(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    let performance_mark_constructor = context
        .global_object()
        .get(JsString::from("PerformanceMark"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ().with_message("PerformanceMark constructor not found"),
            )
        })?;

    let mark_obj = performance_mark_constructor.construct(args, None, context)?;

    performance.marks.borrow_mut().push(mark_obj.clone());

    Ok(JsValue::from(mark_obj))
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

const TIMING_PROPERTIES: &[(&str, bool)] = &[
    ("navigationStart", true),
    ("unloadEventStart", false),
    ("unloadEventEnd", false),
    ("redirectStart", false),
    ("redirectEnd", false),
    ("fetchStart", true),
    ("domainLookupStart", true),
    ("domainLookupEnd", true),
    ("connectStart", true),
    ("connectEnd", true),
    ("secureConnectionStart", false),
    ("requestStart", true),
    ("responseStart", true),
    ("responseEnd", true),
    ("domLoading", true),
    ("domInteractive", true),
    ("domContentLoadedEventStart", true),
    ("domContentLoadedEventEnd", true),
    ("domComplete", true),
    ("loadEventStart", true),
    ("loadEventEnd", true),
];

fn resolve_mark_or_value(
    val: &JsValue,
    performance: &Performance,
    context: &mut Context,
) -> JsResult<f64> {
    if val.is_undefined() || val.is_null() {
        Ok(0.0)
    } else if val.is_number() {
        Ok(val.as_number().unwrap_or(0.0))
    } else {
        let name = val.to_string(context)?.to_std_string().unwrap_or_default();

        // 1. Check if name is a PerformanceTiming property
        if let Some(&(_, is_nonzero)) = TIMING_PROPERTIES.iter().find(|&&(k, _)| k == name) {
            if is_nonzero {
                Ok(0.0)
            } else {
                Err(throw_dom_exception(
                    "InvalidAccessError",
                    &format!("performance.measure: timing attribute '{}' is 0", name),
                    context,
                ))
            }
        } else {
            // 2. Check if name is in performance.marks
            let marks_borrow = performance.marks.borrow();
            let mark_opt = marks_borrow.iter().rev().find(|m_obj| {
                if let Some(m) = m_obj.downcast_ref::<PerformanceMark>() {
                    m.name == name
                } else {
                    false
                }
            });
            match mark_opt {
                Some(m_obj) => {
                    let st = m_obj
                        .downcast_ref::<PerformanceMark>()
                        .map(|m| m.start_time)
                        .unwrap_or(0.0);
                    Ok(st)
                }
                None => Err(throw_dom_exception(
                    "SyntaxError",
                    &format!("performance.measure: mark '{}' not found", name),
                    context,
                )),
            }
        }
    }
}

fn performance_measure(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    let name_val = args.first().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("performance.measure: name is required"))
    })?;
    let name = name_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let start_or_options_val = args.get(1);

    // Check if start_or_options_val is an options object
    let is_options = start_or_options_val.is_some_and(|val| {
        !val.is_undefined() && !val.is_null() && val.is_object() && !val.is_callable()
    });

    let (start_time, duration, detail) = if is_options {
        // If startOrMeasureOptions is a PerformanceMeasureOptions object and endMark is given, throw a TypeError.
        if args.get(2).filter(|v| !v.is_undefined()).is_some() {
            return Err(JsError::from(JsNativeError::typ().with_message(
                "performance.measure: endMark cannot be specified when options are used",
            )));
        }

        let options_obj = start_or_options_val
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                JsError::from(
                    JsNativeError::typ()
                        .with_message("performance.measure: options must be an object"),
                )
            })?;

        let mut detail = JsValue::undefined();
        let det = options_obj.get(JsString::from("detail"), context)?;
        if !det.is_undefined() {
            detail = det;
        }

        let start_val = options_obj.get(JsString::from("start"), context)?;
        let end_val = options_obj.get(JsString::from("end"), context)?;
        let duration_val = options_obj.get(JsString::from("duration"), context)?;

        let has_start = !start_val.is_undefined() && !start_val.is_null();
        let has_end = !end_val.is_undefined() && !end_val.is_null();
        let has_duration = !duration_val.is_undefined() && !duration_val.is_null();

        // If start, duration, and end are all present, throw a TypeError
        if has_start && has_duration && has_end {
            return Err(JsError::from(JsNativeError::typ().with_message(
                "performance.measure: start, duration, and end cannot all be specified",
            )));
        }

        // If duration is present and neither start nor end is present, throw a TypeError
        if has_duration && !has_start && !has_end {
            return Err(JsError::from(JsNativeError::typ().with_message(
                "performance.measure: duration cannot be specified without start or end",
            )));
        }

        let duration_f64 = if has_duration {
            let d = duration_val.to_number(context)?;
            if d < 0.0 {
                return Err(JsError::from(
                    JsNativeError::typ()
                        .with_message("performance.measure: duration cannot be negative"),
                ));
            }
            Some(d)
        } else {
            None
        };

        let resolved_start;
        let resolved_duration;

        if has_start && has_end {
            let s = resolve_mark_or_value(&start_val, &performance, context)?;
            let e = resolve_mark_or_value(&end_val, &performance, context)?;
            resolved_start = s;
            resolved_duration = e - s;
        } else if has_start && has_duration {
            let s = resolve_mark_or_value(&start_val, &performance, context)?;
            let dur = duration_f64.unwrap_or(0.0);
            resolved_start = s;
            resolved_duration = dur;
        } else if has_duration && has_end {
            let e = resolve_mark_or_value(&end_val, &performance, context)?;
            let dur = duration_f64.unwrap_or(0.0);
            resolved_start = e - dur;
            resolved_duration = dur;
        } else if has_start {
            let s = resolve_mark_or_value(&start_val, &performance, context)?;
            let e = get_time_origin().elapsed().as_secs_f64() * 1000.0;
            resolved_start = s;
            resolved_duration = e - s;
        } else if has_end {
            let e = resolve_mark_or_value(&end_val, &performance, context)?;
            resolved_start = 0.0;
            resolved_duration = e;
        } else {
            resolved_start = 0.0;
            let e = get_time_origin().elapsed().as_secs_f64() * 1000.0;
            resolved_duration = e;
        }

        (resolved_start, resolved_duration, detail)
    } else {
        let start_val = args.get(1).cloned().unwrap_or(JsValue::undefined());
        let end_val = args.get(2).cloned().unwrap_or(JsValue::undefined());

        let start_time = if start_val.is_undefined() || start_val.is_null() {
            0.0
        } else {
            resolve_mark_or_value(&start_val, &performance, context)?
        };

        let end_time = if end_val.is_undefined() || end_val.is_null() {
            get_time_origin().elapsed().as_secs_f64() * 1000.0
        } else {
            resolve_mark_or_value(&end_val, &performance, context)?
        };

        (start_time, end_time - start_time, JsValue::undefined())
    };

    let performance_measure_constructor = context
        .global_object()
        .get(JsString::from("PerformanceMeasure"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ().with_message("PerformanceMeasure constructor not found"),
            )
        })?;

    let measure_args = [
        JsValue::from(JsString::from("__internal_private_key__")),
        JsValue::from(JsString::from(name.clone())),
        JsValue::from(start_time),
        JsValue::from(duration),
        detail.clone(),
    ];

    let measure_obj = performance_measure_constructor.construct(&measure_args, None, context)?;

    performance.measures.borrow_mut().push(measure_obj.clone());

    Ok(JsValue::from(measure_obj))
}

fn performance_clear_marks(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    if let Some(name_val) = args.first().filter(|v| !v.is_undefined() && !v.is_null()) {
        let name = name_val
            .to_string(context)?
            .to_std_string()
            .unwrap_or_default();
        performance.marks.borrow_mut().retain(|m_obj| {
            if let Some(m) = m_obj.downcast_ref::<PerformanceMark>() {
                m.name != name
            } else {
                true
            }
        });
        return Ok(JsValue::undefined());
    }

    performance.marks.borrow_mut().clear();
    Ok(JsValue::undefined())
}

fn performance_clear_measures(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    if let Some(name_val) = args.first().filter(|v| !v.is_undefined() && !v.is_null()) {
        let name = name_val
            .to_string(context)?
            .to_std_string()
            .unwrap_or_default();
        performance.measures.borrow_mut().retain(|m_obj| {
            if let Some(m) = m_obj.downcast_ref::<PerformanceMeasure>() {
                m.name != name
            } else {
                true
            }
        });
        return Ok(JsValue::undefined());
    }

    performance.measures.borrow_mut().clear();
    Ok(JsValue::undefined())
}

fn performance_clear_resource_timings(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn performance_set_resource_timing_buffer_size(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn performance_get_entries(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    let mut entries = Vec::new();

    for mark_obj in performance.marks.borrow().iter() {
        if let Some(mark) = mark_obj.downcast_ref::<PerformanceMark>() {
            entries.push((mark.start_time, JsValue::from(mark_obj.clone())));
        }
    }

    for measure_obj in performance.measures.borrow().iter() {
        if let Some(measure) = measure_obj.downcast_ref::<PerformanceMeasure>() {
            entries.push((measure.start_time, JsValue::from(measure_obj.clone())));
        }
    }

    entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let js_entries: Vec<JsValue> = entries.into_iter().map(|(_, val)| val).collect();
    let array = boa_engine::object::builtins::JsArray::from_iter(js_entries, context);

    Ok(JsValue::from(array))
}

fn performance_get_entries_by_type(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    let type_val = args.first().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("performance.getEntriesByType: type is required"),
        )
    })?;
    let entry_type = type_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let mut entries = Vec::new();

    if entry_type == "mark" {
        for mark_obj in performance.marks.borrow().iter() {
            if let Some(mark) = mark_obj.downcast_ref::<PerformanceMark>() {
                entries.push((mark.start_time, JsValue::from(mark_obj.clone())));
            }
        }
    } else if entry_type == "measure" {
        for measure_obj in performance.measures.borrow().iter() {
            if let Some(measure) = measure_obj.downcast_ref::<PerformanceMeasure>() {
                entries.push((measure.start_time, JsValue::from(measure_obj.clone())));
            }
        }
    }

    entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let js_entries: Vec<JsValue> = entries.into_iter().map(|(_, val)| val).collect();
    let array = boa_engine::object::builtins::JsArray::from_iter(js_entries, context);

    Ok(JsValue::from(array))
}

fn performance_get_entries_by_name(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    let name_val = args.first().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("performance.getEntriesByName: name is required"),
        )
    })?;
    let name = name_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let entry_type_opt = args.get(1).filter(|v| !v.is_undefined() && !v.is_null());
    let entry_type = if let Some(type_val) = entry_type_opt {
        Some(
            type_val
                .to_string(context)?
                .to_std_string()
                .unwrap_or_default(),
        )
    } else {
        None
    };

    let mut entries = Vec::new();

    let check_mark = entry_type.as_deref().is_none_or(|t| t == "mark");
    let check_measure = entry_type.as_deref().is_none_or(|t| t == "measure");

    if check_mark {
        for mark_obj in performance.marks.borrow().iter() {
            if let Some(mark) = mark_obj
                .downcast_ref::<PerformanceMark>()
                .filter(|m| m.name == name)
            {
                entries.push((mark.start_time, JsValue::from(mark_obj.clone())));
            }
        }
    }

    if check_measure {
        for measure_obj in performance.measures.borrow().iter() {
            if let Some(measure) = measure_obj
                .downcast_ref::<PerformanceMeasure>()
                .filter(|m| m.name == name)
            {
                entries.push((measure.start_time, JsValue::from(measure_obj.clone())));
            }
        }
    }

    entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let js_entries: Vec<JsValue> = entries.into_iter().map(|(_, val)| val).collect();
    let array = boa_engine::object::builtins::JsArray::from_iter(js_entries, context);

    Ok(JsValue::from(array))
}

impl Class for PerformanceMark {
    const NAME: &'static str = "PerformanceMark";
    const LENGTH: usize = 1;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        let name_val = args.first().ok_or_else(|| {
            JsError::from(
                JsNativeError::typ().with_message("PerformanceMark constructor: name is required"),
            )
        })?;
        let name = name_val
            .to_string(context)?
            .to_std_string()
            .unwrap_or_default();

        let mut detail = JsValue::undefined();
        let mut start_time = get_time_origin().elapsed().as_secs_f64() * 1000.0;

        if let Some(options_obj) = args.get(1).and_then(|v| {
            if !v.is_undefined() && !v.is_null() {
                v.as_object()
            } else {
                None
            }
        }) {
            let det = options_obj.get(JsString::from("detail"), context)?;
            if !det.is_undefined() {
                detail = det;
            }

            let start_val = options_obj.get(JsString::from("startTime"), context)?;
            if !start_val.is_undefined() && !start_val.is_null() {
                let st = start_val.to_number(context)?;
                if st < 0.0 {
                    return Err(JsError::from(JsNativeError::typ().with_message(
                        "PerformanceMark constructor: startTime cannot be negative",
                    )));
                }
                start_time = st;
            }
        }

        Ok(PerformanceMark {
            name,
            start_time,
            detail,
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let get_name_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_mark_get_name),
        )
        .name("get name")
        .build();
        let get_entry_type_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_mark_get_entry_type),
        )
        .name("get entryType")
        .build();
        let get_start_time_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_mark_get_start_time),
        )
        .name("get startTime")
        .build();
        let get_duration_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_mark_get_duration),
        )
        .name("get duration")
        .build();
        let get_detail_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_mark_get_detail),
        )
        .name("get detail")
        .build();

        class
            .accessor(
                JsString::from("name"),
                Some(get_name_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("entryType"),
                Some(get_entry_type_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("startTime"),
                Some(get_start_time_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("duration"),
                Some(get_duration_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("detail"),
                Some(get_detail_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(
                JsString::from("toJSON"),
                0,
                NativeFunction::from_fn_ptr(performance_mark_to_json),
            );

        Ok(())
    }
}

fn performance_mark_get_name(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let mark = obj.downcast_ref::<PerformanceMark>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-PerformanceMark object"),
        )
    })?;
    Ok(JsValue::from(JsString::from(mark.name.clone())))
}

fn performance_mark_get_entry_type(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(JsValue::from(JsString::from("mark")))
}

fn performance_mark_get_start_time(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let mark = obj.downcast_ref::<PerformanceMark>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-PerformanceMark object"),
        )
    })?;
    Ok(JsValue::from(mark.start_time))
}

fn performance_mark_get_duration(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(JsValue::from(0.0))
}

fn performance_mark_get_detail(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let mark = obj.downcast_ref::<PerformanceMark>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-PerformanceMark object"),
        )
    })?;
    Ok(mark.detail.clone())
}

fn performance_mark_to_json(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("toJSON called on non-object"))
    })?;
    let mark = obj.downcast_ref::<PerformanceMark>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("toJSON called on non-PerformanceMark object"),
        )
    })?;

    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let mut initializer = ObjectInitializer::new(context);
    initializer.property(
        JsString::from("name"),
        JsString::from(mark.name.clone()),
        ro,
    );
    initializer.property(JsString::from("entryType"), JsString::from("mark"), ro);
    initializer.property(
        JsString::from("startTime"),
        JsValue::from(mark.start_time),
        ro,
    );
    initializer.property(JsString::from("duration"), JsValue::from(0.0), ro);

    if !mark.detail.is_undefined() {
        initializer.property(JsString::from("detail"), mark.detail.clone(), ro);
    }

    Ok(JsValue::from(initializer.build()))
}

impl Class for PerformanceMeasure {
    const NAME: &'static str = "PerformanceMeasure";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        if args
            .first()
            .and_then(|v| v.as_string())
            .map(|s| s.to_std_string().unwrap_or_default())
            != Some("__internal_private_key__".to_string())
        {
            return Err(JsError::from(
                JsNativeError::typ().with_message("TypeError: Illegal constructor"),
            ));
        }

        let name = args
            .get(1)
            .and_then(|v| v.as_string())
            .map(|s| s.to_std_string().unwrap_or_default())
            .unwrap_or_default();
        let start_time = args.get(2).and_then(|v| v.as_number()).unwrap_or(0.0);
        let duration = args.get(3).and_then(|v| v.as_number()).unwrap_or(0.0);
        let detail = args.get(4).cloned().unwrap_or(JsValue::undefined());

        Ok(PerformanceMeasure {
            name,
            start_time,
            duration,
            detail,
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let get_name_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_measure_get_name),
        )
        .name("get name")
        .build();
        let get_entry_type_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_measure_get_entry_type),
        )
        .name("get entryType")
        .build();
        let get_start_time_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_measure_get_start_time),
        )
        .name("get startTime")
        .build();
        let get_duration_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_measure_get_duration),
        )
        .name("get duration")
        .build();
        let get_detail_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_measure_get_detail),
        )
        .name("get detail")
        .build();

        class
            .accessor(
                JsString::from("name"),
                Some(get_name_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("entryType"),
                Some(get_entry_type_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("startTime"),
                Some(get_start_time_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("duration"),
                Some(get_duration_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("detail"),
                Some(get_detail_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(
                JsString::from("toJSON"),
                0,
                NativeFunction::from_fn_ptr(performance_measure_to_json),
            );

        Ok(())
    }
}

fn performance_measure_get_name(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let measure = obj.downcast_ref::<PerformanceMeasure>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-PerformanceMeasure object"),
        )
    })?;
    Ok(JsValue::from(JsString::from(measure.name.clone())))
}

fn performance_measure_get_entry_type(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(JsValue::from(JsString::from("measure")))
}

fn performance_measure_get_start_time(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let measure = obj.downcast_ref::<PerformanceMeasure>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-PerformanceMeasure object"),
        )
    })?;
    Ok(JsValue::from(measure.start_time))
}

fn performance_measure_get_duration(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let measure = obj.downcast_ref::<PerformanceMeasure>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-PerformanceMeasure object"),
        )
    })?;
    Ok(JsValue::from(measure.duration))
}

fn performance_measure_get_detail(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let measure = obj.downcast_ref::<PerformanceMeasure>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("Method called on non-PerformanceMeasure object"),
        )
    })?;
    Ok(measure.detail.clone())
}

fn performance_measure_to_json(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("toJSON called on non-object"))
    })?;
    let measure = obj.downcast_ref::<PerformanceMeasure>().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("toJSON called on non-PerformanceMeasure object"),
        )
    })?;

    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let mut initializer = ObjectInitializer::new(context);
    initializer.property(
        JsString::from("name"),
        JsString::from(measure.name.clone()),
        ro,
    );
    initializer.property(JsString::from("entryType"), JsString::from("measure"), ro);
    initializer.property(
        JsString::from("startTime"),
        JsValue::from(measure.start_time),
        ro,
    );
    initializer.property(
        JsString::from("duration"),
        JsValue::from(measure.duration),
        ro,
    );

    if !measure.detail.is_undefined() {
        initializer.property(JsString::from("detail"), measure.detail.clone(), ro);
    }

    Ok(JsValue::from(initializer.build()))
}

/// Creates the standard `performance` object.
pub fn create_performance(context: &mut Context) -> JsObject {
    let _ = context.register_global_class::<Performance>();
    let _ = context.register_global_class::<PerformanceEntry>();
    let _ = context.register_global_class::<PerformanceMark>();
    let _ = context.register_global_class::<PerformanceMeasure>();

    let performance_obj = context
        .global_object()
        .get(boa_engine::JsString::from("Performance"), context)
        .ok()
        .and_then(|constructor| constructor.as_object());

    if let Some(inst) = performance_obj.and_then(|obj| obj.construct(&[], None, context).ok()) {
        return inst;
    }

    ObjectInitializer::new(context).build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    #[test]
    fn test_performance_time_origin_and_now() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance.clone(),
            Attribute::all(),
        );

        let res = context
            .eval(Source::from_bytes("performance.timeOrigin > 0"))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        let res = context
            .eval(Source::from_bytes("performance.now() >= 0"))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_user_timing_mark_measure_clear() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance.clone(),
            Attribute::all(),
        );

        // 1. Mark creation
        let source = Source::from_bytes(
            r#"
            const m1 = performance.mark("start");
            const m2 = performance.mark("end");
            m1.name === "start" && m1.entryType === "mark" && m1.startTime >= 0 && m1.duration === 0 &&
            m2.name === "end" && m2.entryType === "mark" && m2.startTime >= m1.startTime
            "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Verify state via Rust downcast
        {
            let perf = performance.downcast_ref::<Performance>().unwrap();
            let marks = perf.marks.borrow();
            assert_eq!(marks.len(), 2);
            assert_eq!(
                marks[0].downcast_ref::<PerformanceMark>().unwrap().name,
                "start"
            );
            assert_eq!(
                marks[1].downcast_ref::<PerformanceMark>().unwrap().name,
                "end"
            );
        }

        // 2. Measure creation
        let source = Source::from_bytes(
            r#"
            const m = performance.measure("my-duration", "start", "end");
            m.name === "my-duration" && m.entryType === "measure" && m.startTime === m1.startTime && m.duration >= 0
            "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        {
            let perf = performance.downcast_ref::<Performance>().unwrap();
            let measures = perf.measures.borrow();
            assert_eq!(measures.len(), 1);
            assert_eq!(
                measures[0]
                    .downcast_ref::<PerformanceMeasure>()
                    .unwrap()
                    .name,
                "my-duration"
            );
            assert!(
                measures[0]
                    .downcast_ref::<PerformanceMeasure>()
                    .unwrap()
                    .duration
                    >= 0.0
            );
        }

        // 3. Clear marks by name
        let source = Source::from_bytes(
            r#"
            performance.clearMarks("start");
            "#,
        );
        let _ = context.eval(source).unwrap();

        {
            let perf = performance.downcast_ref::<Performance>().unwrap();
            let marks = perf.marks.borrow();
            assert_eq!(marks.len(), 1);
            assert_eq!(
                marks[0].downcast_ref::<PerformanceMark>().unwrap().name,
                "end"
            );
        }

        // 4. Clear all marks
        let source = Source::from_bytes(
            r#"
            performance.clearMarks();
            "#,
        );
        let _ = context.eval(source).unwrap();

        {
            let perf = performance.downcast_ref::<Performance>().unwrap();
            let marks = perf.marks.borrow();
            assert!(marks.is_empty());
        }

        // 5. Clear measures by name
        let source = Source::from_bytes(
            r#"
            performance.clearMeasures("my-duration");
            "#,
        );
        let _ = context.eval(source).unwrap();

        {
            let perf = performance.downcast_ref::<Performance>().unwrap();
            let measures = perf.measures.borrow();
            assert!(measures.is_empty());
        }
    }

    #[test]
    fn test_performance_measure_error_handling() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // measure with unknown mark name throws an error
        let source = Source::from_bytes(
            r#"
            let threw = false;
            try {
                performance.measure("test", "non-existent");
            } catch (e) {
                threw = true;
            }
            threw
            "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_get_entries_api() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // Add some marks and measures
        context
            .eval(Source::from_bytes("performance.mark('mark1');"))
            .unwrap();
        context
            .eval(Source::from_bytes("performance.mark('mark2');"))
            .unwrap();
        context
            .eval(Source::from_bytes(
                "performance.measure('measure1', 'mark1', 'mark2');",
            ))
            .unwrap();

        // 1. Check getEntries returns all 3 entries
        let res = context
            .eval(Source::from_bytes("performance.getEntries().length === 3"))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check chronological ordering of getEntries()
        let res = context
            .eval(Source::from_bytes(
                r#"
                const entries = performance.getEntries();
                entries[0].name === "mark1" && entries[1].name === "measure1" && entries[2].name === "mark2"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. Check getEntriesByType('mark') returns only marks
        let res = context
            .eval(Source::from_bytes(
                r#"
                const marks = performance.getEntriesByType("mark");
                marks.length === 2 && marks[0].name === "mark1" && marks[1].name === "mark2"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check getEntriesByType('measure') returns only measures
        let res = context
            .eval(Source::from_bytes(
                r#"
                const measures = performance.getEntriesByType("measure");
                measures.length === 1 && measures[0].name === "measure1"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check getEntriesByType with unsupported type returns empty array
        let res = context
            .eval(Source::from_bytes(
                "performance.getEntriesByType('invalid_type').length === 0",
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 3. Check getEntriesByName('mark1') returns the specific mark
        let res = context
            .eval(Source::from_bytes(
                r#"
                const named = performance.getEntriesByName("mark1");
                named.length === 1 && named[0].name === "mark1" && named[0].entryType === "mark"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check getEntriesByName with correct type
        let res = context
            .eval(Source::from_bytes(
                r#"
                const named_typed = performance.getEntriesByName("mark1", "mark");
                named_typed.length === 1 && named_typed[0].name === "mark1"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check getEntriesByName with non-matching type returns empty array
        let res = context
            .eval(Source::from_bytes(
                r#"
                performance.getEntriesByName("mark1", "measure").length === 0
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_timing_and_navigation_api_surface() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // Check timing object exists and has valid numbers
        let res = context
            .eval(Source::from_bytes(
                r#"
                performance.timing &&
                typeof performance.timing.navigationStart === "number" &&
                performance.timing.navigationStart > 0 &&
                performance.timing.unloadEventStart === 0 &&
                performance.timing.fetchStart === performance.timing.navigationStart
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check navigation object exists and has valid type and redirectCount
        let res = context
            .eval(Source::from_bytes(
                r#"
                performance.navigation &&
                performance.navigation.type === 0 &&
                performance.navigation.redirectCount === 0
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_user_timing_l3_mark_options() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // 1. Mark with startTime and detail
        let res = context
            .eval(Source::from_bytes(
                r#"
                {
                    const m = performance.mark("custom-mark", {
                        detail: { foo: "bar" },
                        startTime: 123.45
                    });
                    m.name === "custom-mark" &&
                    m.startTime === 123.45 &&
                    m.detail.foo === "bar"
                }
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. Mark with negative startTime should throw TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                {
                    let threw = false;
                    try {
                        performance.mark("negative", { startTime: -5 });
                    } catch (e) {
                        threw = e instanceof TypeError;
                    }
                    threw
                }
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_user_timing_l3_measure_options() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // Setup custom marks
        context
            .eval(Source::from_bytes(
                r#"
                performance.mark("m1", { startTime: 100 });
                performance.mark("m2", { startTime: 250 });
                "#,
            ))
            .unwrap();

        // 1. Measure with start and end
        let res = context
            .eval(Source::from_bytes(
                r#"
                const m1 = performance.measure("meas-1", {
                    start: "m1",
                    end: "m2",
                    detail: "metadata-1"
                });
                m1.startTime === 100 &&
                m1.duration === 150 &&
                m1.detail === "metadata-1"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. Measure with start and duration
        let res = context
            .eval(Source::from_bytes(
                r#"
                const m2 = performance.measure("meas-2", {
                    start: "m1",
                    duration: 50
                });
                m2.startTime === 100 &&
                m2.duration === 50
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 3. Measure with duration and end
        let res = context
            .eval(Source::from_bytes(
                r#"
                const m3 = performance.measure("meas-3", {
                    end: "m2",
                    duration: 75
                });
                m3.startTime === 175 &&
                m3.duration === 75
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 4. Measure with negative duration throws TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                let threw4 = false;
                try {
                    performance.measure("meas-neg", {
                        start: "m1",
                        duration: -10
                    });
                } catch (e) {
                    threw4 = e instanceof TypeError;
                }
                threw4
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 5. Measure with start, duration, and end throws TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                let threw5 = false;
                try {
                    performance.measure("meas-all", {
                        start: "m1",
                        end: "m2",
                        duration: 100
                    });
                } catch (e) {
                    threw5 = e instanceof TypeError;
                }
                threw5
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 6. Measure with duration and neither start nor end throws TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                let threw6 = false;
                try {
                    performance.measure("meas-dur-only", {
                        duration: 100
                    });
                } catch (e) {
                    threw6 = e instanceof TypeError;
                }
                threw6
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 7. Measure with options and endMark throws TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                let threw7 = false;
                try {
                    performance.measure("meas-conflict", { start: "m1" }, "m2");
                } catch (e) {
                    threw7 = e instanceof TypeError;
                }
                threw7
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_to_json_api_surface() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // 1. Performance.toJSON()
        let res = context
            .eval(Source::from_bytes(
                r#"
                const pJson = performance.toJSON();
                typeof pJson === "object" &&
                typeof pJson.timeOrigin === "number" &&
                pJson.timeOrigin > 0
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. PerformanceTiming.toJSON()
        let res = context
            .eval(Source::from_bytes(
                r#"
                const tJson = performance.timing.toJSON();
                typeof tJson === "object" &&
                tJson.navigationStart === performance.timing.navigationStart &&
                tJson.fetchStart === performance.timing.fetchStart &&
                tJson.loadEventEnd === performance.timing.loadEventEnd
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 3. PerformanceNavigation.toJSON()
        let res = context
            .eval(Source::from_bytes(
                r#"
                const nJson = performance.navigation.toJSON();
                typeof nJson === "object" &&
                nJson.type === performance.navigation.type &&
                nJson.redirectCount === performance.navigation.redirectCount
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 4. PerformanceEntry.toJSON() - Mark and Measure
        let res = context
            .eval(Source::from_bytes(
                r#"
                const m = performance.mark("test-mark", { startTime: 123.45, detail: "some-detail" });
                const mJson = m.toJSON();
                mJson.name === "test-mark" &&
                mJson.entryType === "mark" &&
                mJson.startTime === 123.45 &&
                mJson.duration === 0 &&
                mJson.detail === "some-detail"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        let res = context
            .eval(Source::from_bytes(
                r#"
                const meas = performance.measure("test-meas", { start: "test-mark", duration: 50, detail: "meas-detail" });
                const measJson = meas.toJSON();
                measJson.name === "test-meas" &&
                measJson.entryType === "measure" &&
                measJson.startTime === 123.45 &&
                measJson.duration === 50 &&
                measJson.detail === "meas-detail"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_timing_resolution_and_exceptions() {
        let mut context = Context::default();

        // Register DOMException so that throw_dom_exception can construct it
        let source_init = Source::from_bytes(
            r#"
            class DOMException extends Error {
                constructor(message, name) {
                    super(message);
                    this.name = name || "DOMException";
                }
            }
            globalThis.DOMException = DOMException;
            "#,
        );
        context.eval(source_init).unwrap();

        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // 1. Measure using non-zero PerformanceTiming property ("fetchStart")
        let res = context
            .eval(Source::from_bytes(
                r#"
                const m = performance.measure("from-fetch", "fetchStart");
                m.startTime === 0 && m.duration >= 0
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. Measure using non-zero start and end PerformanceTiming properties
        let res = context
            .eval(Source::from_bytes(
                r#"
                const m2 = performance.measure("fetch-to-load", "fetchStart", "loadEventEnd");
                m2.startTime === 0 && m2.duration === 0
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 3. Measure using options with PerformanceTiming properties
        let res = context
            .eval(Source::from_bytes(
                r#"
                const m3 = performance.measure("meas-opt-timing", {
                    start: "fetchStart",
                    end: "loadEventEnd"
                });
                m3.startTime === 0 && m3.duration === 0
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 4. Measure using a zero PerformanceTiming property ("unloadEventStart") should throw InvalidAccessError DOMException
        let res = context
            .eval(Source::from_bytes(
                r#"
                let threwInvalidAccess = false;
                try {
                    performance.measure("invalid", "unloadEventStart");
                } catch (e) {
                    threwInvalidAccess = (e instanceof DOMException) && (e.name === "InvalidAccessError");
                }
                threwInvalidAccess
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 5. Measure using nonexistent mark should throw SyntaxError DOMException
        let res = context
            .eval(Source::from_bytes(
                r#"
                let threwSyntax = false;
                try {
                    performance.measure("invalid-syntax", "non-existent-mark-name");
                } catch (e) {
                    threwSyntax = (e instanceof DOMException) && (e.name === "SyntaxError");
                }
                threwSyntax
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_resource_timing_stubs() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        let res = context
            .eval(Source::from_bytes(
                r#"
                const res1 = performance.clearResourceTimings();
                const res2 = performance.setResourceTimingBufferSize(150);
                res1 === undefined && res2 === undefined
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_classes_global_surface() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // 1. PerformanceMark constructor exposure and properties
        let res = context
            .eval(Source::from_bytes(
                r#"
                const mark = new PerformanceMark("my-custom-mark", {
                    startTime: 500,
                    detail: { status: "success" }
                });
                mark.name === "my-custom-mark" &&
                mark.entryType === "mark" &&
                mark.startTime === 500 &&
                mark.duration === 0 &&
                mark.detail.status === "success"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. Constructed mark is NOT in the timeline
        let res = context
            .eval(Source::from_bytes(
                r#"
                performance.getEntries().length === 0
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 3. toJSON of constructed mark works perfectly
        let res = context
            .eval(Source::from_bytes(
                r#"
                const json = mark.toJSON();
                json.name === "my-custom-mark" &&
                json.entryType === "mark" &&
                json.startTime === 500 &&
                json.duration === 0 &&
                json.detail.status === "success"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 4. PerformanceMeasure constructor is not constructible and throws TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                let threwMeasure = false;
                try {
                    new PerformanceMeasure();
                } catch (e) {
                    threwMeasure = e instanceof TypeError;
                }
                threwMeasure
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 5. PerformanceEntry constructor is not constructible and throws TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                let threwEntry = false;
                try {
                    new PerformanceEntry();
                } catch (e) {
                    threwEntry = e instanceof TypeError;
                }
                threwEntry
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }
}
