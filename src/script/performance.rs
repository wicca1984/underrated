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
    pub(crate) resource_timings: GcRefCell<Vec<JsObject>>,
    pub(crate) navigation_timings: GcRefCell<Vec<JsObject>>,
    pub(crate) resource_timing_buffer_size: GcRefCell<usize>,
}

impl Class for Performance {
    const NAME: &'static str = "Performance";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        let marks = GcRefCell::new(Vec::new());
        let measures = GcRefCell::new(Vec::new());
        let resource_timings = GcRefCell::new(Vec::new());
        let navigation_timings = GcRefCell::new(Vec::new());
        let resource_timing_buffer_size = GcRefCell::new(250);

        // We can construct the navigation timing entry if the constructor is registered
        if let Some(constructor_obj) = _context
            .global_object()
            .get(JsString::from("PerformanceNavigationTiming"), _context)
            .ok()
            .and_then(|v| v.as_object())
        {
            let args = [
                JsValue::from(JsString::from("__internal_private_key__")),
                JsValue::from(JsString::from("document")),
                JsValue::from(0.0),
                JsValue::from(0.0),
            ];
            if let Ok(nav_obj) = constructor_obj.construct(&args, None, _context) {
                navigation_timings.borrow_mut().push(nav_obj);
            }
        }

        Ok(Performance {
            marks,
            measures,
            resource_timings,
            navigation_timings,
            resource_timing_buffer_size,
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
                JsString::from("addResourceTiming"),
                1,
                NativeFunction::from_fn_ptr(performance_add_resource_timing),
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
                None => {
                    // 3. Check if name is in performance.measures (User Timing L3)
                    let measures_borrow = performance.measures.borrow();
                    let measure_opt = measures_borrow.iter().rev().find(|m_obj| {
                        if let Some(m) = m_obj.downcast_ref::<PerformanceMeasure>() {
                            m.name == name
                        } else {
                            false
                        }
                    });
                    match measure_opt {
                        Some(m_obj) => {
                            let st = m_obj
                                .downcast_ref::<PerformanceMeasure>()
                                .map(|m| m.start_time)
                                .unwrap_or(0.0);
                            Ok(st)
                        }
                        None => Err(throw_dom_exception(
                            "SyntaxError",
                            &format!("performance.measure: mark or measure '{}' not found", name),
                            context,
                        )),
                    }
                }
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
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    performance.resource_timings.borrow_mut().clear();
    Ok(JsValue::undefined())
}

fn performance_set_resource_timing_buffer_size(
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

    let size_val = args.first().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ()
                .with_message("performance.setResourceTimingBufferSize: size is required"),
        )
    })?;
    let size = size_val.to_number(context)? as usize;
    *performance.resource_timing_buffer_size.borrow_mut() = size;

    Ok(JsValue::undefined())
}

fn performance_add_resource_timing(
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
            JsNativeError::typ().with_message("performance.addResourceTiming: name is required"),
        )
    })?;
    let name = name_val.to_string(context)?;

    let start_time_val = args.get(1).cloned().unwrap_or(JsValue::undefined());
    let start_time = if start_time_val.is_undefined() || start_time_val.is_null() {
        0.0
    } else {
        start_time_val.to_number(context)?
    };

    let duration_val = args.get(2).cloned().unwrap_or(JsValue::undefined());
    let duration = if duration_val.is_undefined() || duration_val.is_null() {
        0.0
    } else {
        duration_val.to_number(context)?
    };

    let options_val = args.get(3).cloned().unwrap_or(JsValue::undefined());

    let resource_constructor = context
        .global_object()
        .get(JsString::from("PerformanceResourceTiming"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("PerformanceResourceTiming constructor not found"),
            )
        })?;

    let constructor_args = [
        JsValue::from(JsString::from("__internal_private_key__")),
        JsValue::from(name),
        JsValue::from(start_time),
        JsValue::from(duration),
        options_val,
    ];

    let resource_obj = resource_constructor.construct(&constructor_args, None, context)?;

    if performance.resource_timings.borrow().len()
        >= *performance.resource_timing_buffer_size.borrow()
    {
        return Ok(JsValue::from(resource_obj));
    }

    performance
        .resource_timings
        .borrow_mut()
        .push(resource_obj.clone());

    Ok(JsValue::from(resource_obj))
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

    for resource_obj in performance.resource_timings.borrow().iter() {
        if let Some(resource) = resource_obj.downcast_ref::<PerformanceResourceTiming>() {
            entries.push((resource.start_time, JsValue::from(resource_obj.clone())));
        }
    }

    for nav_obj in performance.navigation_timings.borrow().iter() {
        if let Some(nav) = nav_obj.downcast_ref::<PerformanceNavigationTiming>() {
            entries.push((
                nav.resource_timing.start_time,
                JsValue::from(nav_obj.clone()),
            ));
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
    } else if entry_type == "resource" {
        for resource_obj in performance.resource_timings.borrow().iter() {
            if let Some(resource) = resource_obj.downcast_ref::<PerformanceResourceTiming>() {
                entries.push((resource.start_time, JsValue::from(resource_obj.clone())));
            }
        }
    } else if entry_type == "navigation" {
        for nav_obj in performance.navigation_timings.borrow().iter() {
            if let Some(nav) = nav_obj.downcast_ref::<PerformanceNavigationTiming>() {
                entries.push((
                    nav.resource_timing.start_time,
                    JsValue::from(nav_obj.clone()),
                ));
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
    let check_resource = entry_type.as_deref().is_none_or(|t| t == "resource");
    let check_navigation = entry_type.as_deref().is_none_or(|t| t == "navigation");

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

    if check_resource {
        for resource_obj in performance.resource_timings.borrow().iter() {
            if let Some(resource) = resource_obj
                .downcast_ref::<PerformanceResourceTiming>()
                .filter(|m| m.name == name)
            {
                entries.push((resource.start_time, JsValue::from(resource_obj.clone())));
            }
        }
    }

    if check_navigation {
        for nav_obj in performance.navigation_timings.borrow().iter() {
            if let Some(nav) = nav_obj
                .downcast_ref::<PerformanceNavigationTiming>()
                .filter(|m| m.resource_timing.name == name)
            {
                entries.push((
                    nav.resource_timing.start_time,
                    JsValue::from(nav_obj.clone()),
                ));
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

type PerformanceGetterFn = fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>;

/// Representation of a stored performance resource timing.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct PerformanceResourceTiming {
    pub name: String,
    pub start_time: f64,
    pub duration: f64,
    pub initiator_type: String,
    pub next_hop_protocol: String,
    pub worker_start: f64,
    pub redirect_start: f64,
    pub redirect_end: f64,
    pub fetch_start: f64,
    pub domain_lookup_start: f64,
    pub domain_lookup_end: f64,
    pub connect_start: f64,
    pub connect_end: f64,
    pub secure_connection_start: f64,
    pub request_start: f64,
    pub response_start: f64,
    pub response_end: f64,
    pub transfer_size: f64,
    pub encoded_body_size: f64,
    pub decoded_body_size: f64,
}

impl Class for PerformanceResourceTiming {
    const NAME: &'static str = "PerformanceResourceTiming";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
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

        let mut initiator_type = "other".to_string();
        let mut next_hop_protocol = "".to_string();
        let mut worker_start = 0.0;
        let mut redirect_start = 0.0;
        let mut redirect_end = 0.0;
        let mut fetch_start = start_time;
        let mut domain_lookup_start = start_time;
        let mut domain_lookup_end = start_time;
        let mut connect_start = start_time;
        let mut connect_end = start_time;
        let mut secure_connection_start = 0.0;
        let mut request_start = start_time;
        let mut response_start = start_time;
        let mut response_end = start_time + duration;
        let mut transfer_size = 0.0;
        let mut encoded_body_size = 0.0;
        let mut decoded_body_size = 0.0;

        if let Some(options_obj) = args.get(4).and_then(|v| v.as_object()) {
            if let Some(val) = options_obj
                .get(JsString::from("initiatorType"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                initiator_type = val
                    .to_string(context)?
                    .to_std_string()
                    .unwrap_or(initiator_type);
            }
            if let Some(val) = options_obj
                .get(JsString::from("nextHopProtocol"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                next_hop_protocol = val
                    .to_string(context)?
                    .to_std_string()
                    .unwrap_or(next_hop_protocol);
            }
            if let Some(val) = options_obj
                .get(JsString::from("workerStart"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                worker_start = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("redirectStart"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                redirect_start = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("redirectEnd"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                redirect_end = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("fetchStart"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                fetch_start = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("domainLookupStart"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                domain_lookup_start = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("domainLookupEnd"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                domain_lookup_end = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("connectStart"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                connect_start = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("connectEnd"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                connect_end = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("secureConnectionStart"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                secure_connection_start = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("requestStart"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                request_start = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("responseStart"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                response_start = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("responseEnd"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                response_end = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("transferSize"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                transfer_size = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("encodedBodySize"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                encoded_body_size = val.to_number(context)?;
            }
            if let Some(val) = options_obj
                .get(JsString::from("decodedBodySize"), context)
                .ok()
                .filter(|v| !v.is_undefined())
            {
                decoded_body_size = val.to_number(context)?;
            }
        }

        Ok(PerformanceResourceTiming {
            name,
            start_time,
            duration,
            initiator_type,
            next_hop_protocol,
            worker_start,
            redirect_start,
            redirect_end,
            fetch_start,
            domain_lookup_start,
            domain_lookup_end,
            connect_start,
            connect_end,
            secure_connection_start,
            request_start,
            response_start,
            response_end,
            transfer_size,
            encoded_body_size,
            decoded_body_size,
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let getters: &[(&str, PerformanceGetterFn)] = &[
            ("name", performance_resource_get_name),
            ("entryType", performance_resource_get_entry_type),
            ("startTime", performance_resource_get_start_time),
            ("duration", performance_resource_get_duration),
            ("initiatorType", performance_resource_get_initiator_type),
            (
                "nextHopProtocol",
                performance_resource_get_next_hop_protocol,
            ),
            ("workerStart", performance_resource_get_worker_start),
            ("redirectStart", performance_resource_get_redirect_start),
            ("redirectEnd", performance_resource_get_redirect_end),
            ("fetchStart", performance_resource_get_fetch_start),
            (
                "domainLookupStart",
                performance_resource_get_domain_lookup_start,
            ),
            (
                "domainLookupEnd",
                performance_resource_get_domain_lookup_end,
            ),
            ("connectStart", performance_resource_get_connect_start),
            ("connectEnd", performance_resource_get_connect_end),
            (
                "secureConnectionStart",
                performance_resource_get_secure_connection_start,
            ),
            ("requestStart", performance_resource_get_request_start),
            ("responseStart", performance_resource_get_response_start),
            ("responseEnd", performance_resource_get_response_end),
            ("transferSize", performance_resource_get_transfer_size),
            (
                "encodedBodySize",
                performance_resource_get_encoded_body_size,
            ),
            (
                "decodedBodySize",
                performance_resource_get_decoded_body_size,
            ),
        ];

        for &(prop, func) in getters {
            let get_fn = FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(func))
                .name(format!("get {prop}"))
                .build();
            class.accessor(
                JsString::from(prop),
                Some(get_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            );
        }

        class.method(
            JsString::from("toJSON"),
            0,
            NativeFunction::from_fn_ptr(performance_resource_to_json),
        );

        Ok(())
    }
}

fn performance_resource_get_name(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(JsString::from(entry.name.clone())))
}
fn performance_resource_get_entry_type(
    _: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    Ok(JsValue::from(JsString::from("resource")))
}
fn performance_resource_get_start_time(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.start_time))
}
fn performance_resource_get_duration(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.duration))
}
fn performance_resource_get_initiator_type(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(JsString::from(entry.initiator_type.clone())))
}
fn performance_resource_get_next_hop_protocol(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(JsString::from(
        entry.next_hop_protocol.clone(),
    )))
}
fn performance_resource_get_worker_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.worker_start))
}
fn performance_resource_get_redirect_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.redirect_start))
}
fn performance_resource_get_redirect_end(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.redirect_end))
}
fn performance_resource_get_fetch_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.fetch_start))
}
fn performance_resource_get_domain_lookup_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.domain_lookup_start))
}
fn performance_resource_get_domain_lookup_end(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.domain_lookup_end))
}
fn performance_resource_get_connect_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.connect_start))
}
fn performance_resource_get_connect_end(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.connect_end))
}
fn performance_resource_get_secure_connection_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.secure_connection_start))
}
fn performance_resource_get_request_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.request_start))
}
fn performance_resource_get_response_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.response_start))
}
fn performance_resource_get_response_end(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.response_end))
}
fn performance_resource_get_transfer_size(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.transfer_size))
}
fn performance_resource_get_encoded_body_size(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.encoded_body_size))
}
fn performance_resource_get_decoded_body_size(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceResourceTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.decoded_body_size))
}

fn performance_resource_to_json(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("toJSON called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceResourceTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("toJSON called on non-PerformanceResourceTiming object"),
            )
        })?;

    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let mut initializer = ObjectInitializer::new(context);
    initializer.property(
        JsString::from("name"),
        JsString::from(entry.name.clone()),
        ro,
    );
    initializer.property(JsString::from("entryType"), JsString::from("resource"), ro);
    initializer.property(
        JsString::from("startTime"),
        JsValue::from(entry.start_time),
        ro,
    );
    initializer.property(
        JsString::from("duration"),
        JsValue::from(entry.duration),
        ro,
    );
    initializer.property(
        JsString::from("initiatorType"),
        JsString::from(entry.initiator_type.clone()),
        ro,
    );
    initializer.property(
        JsString::from("nextHopProtocol"),
        JsString::from(entry.next_hop_protocol.clone()),
        ro,
    );
    initializer.property(
        JsString::from("workerStart"),
        JsValue::from(entry.worker_start),
        ro,
    );
    initializer.property(
        JsString::from("redirectStart"),
        JsValue::from(entry.redirect_start),
        ro,
    );
    initializer.property(
        JsString::from("redirectEnd"),
        JsValue::from(entry.redirect_end),
        ro,
    );
    initializer.property(
        JsString::from("fetchStart"),
        JsValue::from(entry.fetch_start),
        ro,
    );
    initializer.property(
        JsString::from("domainLookupStart"),
        JsValue::from(entry.domain_lookup_start),
        ro,
    );
    initializer.property(
        JsString::from("domainLookupEnd"),
        JsValue::from(entry.domain_lookup_end),
        ro,
    );
    initializer.property(
        JsString::from("connectStart"),
        JsValue::from(entry.connect_start),
        ro,
    );
    initializer.property(
        JsString::from("connectEnd"),
        JsValue::from(entry.connect_end),
        ro,
    );
    initializer.property(
        JsString::from("secureConnectionStart"),
        JsValue::from(entry.secure_connection_start),
        ro,
    );
    initializer.property(
        JsString::from("requestStart"),
        JsValue::from(entry.request_start),
        ro,
    );
    initializer.property(
        JsString::from("responseStart"),
        JsValue::from(entry.response_start),
        ro,
    );
    initializer.property(
        JsString::from("responseEnd"),
        JsValue::from(entry.response_end),
        ro,
    );
    initializer.property(
        JsString::from("transferSize"),
        JsValue::from(entry.transfer_size),
        ro,
    );
    initializer.property(
        JsString::from("encodedBodySize"),
        JsValue::from(entry.encoded_body_size),
        ro,
    );
    initializer.property(
        JsString::from("decodedBodySize"),
        JsValue::from(entry.decoded_body_size),
        ro,
    );

    Ok(JsValue::from(initializer.build()))
}

/// Representation of a stored performance navigation timing.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct PerformanceNavigationTiming {
    pub resource_timing: PerformanceResourceTiming,
    pub unload_event_start: f64,
    pub unload_event_end: f64,
    pub dom_loading: f64,
    pub dom_interactive: f64,
    pub dom_content_loaded_event_start: f64,
    pub dom_content_loaded_event_end: f64,
    pub dom_complete: f64,
    pub load_event_start: f64,
    pub load_event_end: f64,
    pub nav_type: String,
    pub redirect_count: f64,
}

impl Class for PerformanceNavigationTiming {
    const NAME: &'static str = "PerformanceNavigationTiming";
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

        let resource_timing = PerformanceResourceTiming {
            name,
            start_time,
            duration,
            initiator_type: "navigation".to_string(),
            next_hop_protocol: "".to_string(),
            worker_start: 0.0,
            redirect_start: 0.0,
            redirect_end: 0.0,
            fetch_start: 0.0,
            domain_lookup_start: 0.0,
            domain_lookup_end: 0.0,
            connect_start: 0.0,
            connect_end: 0.0,
            secure_connection_start: 0.0,
            request_start: 0.0,
            response_start: 0.0,
            response_end: 0.0,
            transfer_size: 0.0,
            encoded_body_size: 0.0,
            decoded_body_size: 0.0,
        };

        Ok(PerformanceNavigationTiming {
            resource_timing,
            unload_event_start: 0.0,
            unload_event_end: 0.0,
            dom_loading: 0.0,
            dom_interactive: 0.0,
            dom_content_loaded_event_start: 0.0,
            dom_content_loaded_event_end: 0.0,
            dom_complete: 0.0,
            load_event_start: 0.0,
            load_event_end: 0.0,
            nav_type: "navigate".to_string(),
            redirect_count: 0.0,
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let getters: &[(&str, PerformanceGetterFn)] = &[
            ("name", performance_navigation_get_name),
            ("entryType", performance_navigation_get_entry_type),
            ("startTime", performance_navigation_get_start_time),
            ("duration", performance_navigation_get_duration),
            ("initiatorType", performance_navigation_get_initiator_type),
            (
                "nextHopProtocol",
                performance_navigation_get_next_hop_protocol,
            ),
            ("workerStart", performance_navigation_get_worker_start),
            ("redirectStart", performance_navigation_get_redirect_start),
            ("redirectEnd", performance_navigation_get_redirect_end),
            ("fetchStart", performance_navigation_get_fetch_start),
            (
                "domainLookupStart",
                performance_navigation_get_domain_lookup_start,
            ),
            (
                "domainLookupEnd",
                performance_navigation_get_domain_lookup_end,
            ),
            ("connectStart", performance_navigation_get_connect_start),
            ("connectEnd", performance_navigation_get_connect_end),
            (
                "secureConnectionStart",
                performance_navigation_get_secure_connection_start,
            ),
            ("requestStart", performance_navigation_get_request_start),
            ("responseStart", performance_navigation_get_response_start),
            ("responseEnd", performance_navigation_get_response_end),
            ("transferSize", performance_navigation_get_transfer_size),
            (
                "encodedBodySize",
                performance_navigation_get_encoded_body_size,
            ),
            (
                "decodedBodySize",
                performance_navigation_get_decoded_body_size,
            ),
            (
                "unloadEventStart",
                performance_navigation_get_unload_event_start,
            ),
            (
                "unloadEventEnd",
                performance_navigation_get_unload_event_end,
            ),
            ("domLoading", performance_navigation_get_dom_loading),
            ("domInteractive", performance_navigation_get_dom_interactive),
            (
                "domContentLoadedEventStart",
                performance_navigation_get_dom_content_loaded_event_start,
            ),
            (
                "domContentLoadedEventEnd",
                performance_navigation_get_dom_content_loaded_event_end,
            ),
            ("domComplete", performance_navigation_get_dom_complete),
            (
                "loadEventStart",
                performance_navigation_get_load_event_start,
            ),
            ("loadEventEnd", performance_navigation_get_load_event_end),
            ("type", performance_navigation_get_type),
            ("redirectCount", performance_navigation_get_redirect_count),
        ];

        for &(prop, func) in getters {
            let get_fn = FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(func))
                .name(format!("get {prop}"))
                .build();
            class.accessor(
                JsString::from(prop),
                Some(get_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            );
        }

        class.method(
            JsString::from("toJSON"),
            0,
            NativeFunction::from_fn_ptr(performance_navigation_entry_to_json),
        );

        Ok(())
    }
}

fn performance_navigation_get_name(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(JsString::from(
        entry.resource_timing.name.clone(),
    )))
}
fn performance_navigation_get_entry_type(
    _: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    Ok(JsValue::from(JsString::from("navigation")))
}
fn performance_navigation_get_start_time(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.start_time))
}
fn performance_navigation_get_duration(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.duration))
}
fn performance_navigation_get_initiator_type(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(JsString::from(
        entry.resource_timing.initiator_type.clone(),
    )))
}
fn performance_navigation_get_next_hop_protocol(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(JsString::from(
        entry.resource_timing.next_hop_protocol.clone(),
    )))
}
fn performance_navigation_get_worker_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.worker_start))
}
fn performance_navigation_get_redirect_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.redirect_start))
}
fn performance_navigation_get_redirect_end(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.redirect_end))
}
fn performance_navigation_get_fetch_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.fetch_start))
}
fn performance_navigation_get_domain_lookup_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.domain_lookup_start))
}
fn performance_navigation_get_domain_lookup_end(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.domain_lookup_end))
}
fn performance_navigation_get_connect_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.connect_start))
}
fn performance_navigation_get_connect_end(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.connect_end))
}
fn performance_navigation_get_secure_connection_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.secure_connection_start))
}
fn performance_navigation_get_request_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.request_start))
}
fn performance_navigation_get_response_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.response_start))
}
fn performance_navigation_get_response_end(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.response_end))
}
fn performance_navigation_get_transfer_size(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.transfer_size))
}
fn performance_navigation_get_encoded_body_size(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.encoded_body_size))
}
fn performance_navigation_get_decoded_body_size(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.resource_timing.decoded_body_size))
}
fn performance_navigation_get_unload_event_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.unload_event_start))
}
fn performance_navigation_get_unload_event_end(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.unload_event_end))
}
fn performance_navigation_get_dom_loading(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.dom_loading))
}
fn performance_navigation_get_dom_interactive(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.dom_interactive))
}
fn performance_navigation_get_dom_content_loaded_event_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.dom_content_loaded_event_start))
}
fn performance_navigation_get_dom_content_loaded_event_end(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.dom_content_loaded_event_end))
}
fn performance_navigation_get_dom_complete(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.dom_complete))
}
fn performance_navigation_get_load_event_start(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.load_event_start))
}
fn performance_navigation_get_load_event_end(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.load_event_end))
}
fn performance_navigation_get_type(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(JsString::from(entry.nav_type.clone())))
}
fn performance_navigation_get_redirect_count(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("Method called on non-PerformanceNavigationTiming object"),
            )
        })?;
    Ok(JsValue::from(entry.redirect_count))
}

fn performance_navigation_entry_to_json(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("toJSON called on non-object"))
    })?;
    let entry = obj
        .downcast_ref::<PerformanceNavigationTiming>()
        .ok_or_else(|| {
            JsError::from(
                JsNativeError::typ()
                    .with_message("toJSON called on non-PerformanceNavigationTiming object"),
            )
        })?;

    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let mut initializer = ObjectInitializer::new(context);
    initializer.property(
        JsString::from("name"),
        JsString::from(entry.resource_timing.name.clone()),
        ro,
    );
    initializer.property(
        JsString::from("entryType"),
        JsString::from("navigation"),
        ro,
    );
    initializer.property(
        JsString::from("startTime"),
        JsValue::from(entry.resource_timing.start_time),
        ro,
    );
    initializer.property(
        JsString::from("duration"),
        JsValue::from(entry.resource_timing.duration),
        ro,
    );
    initializer.property(
        JsString::from("initiatorType"),
        JsString::from(entry.resource_timing.initiator_type.clone()),
        ro,
    );
    initializer.property(
        JsString::from("nextHopProtocol"),
        JsString::from(entry.resource_timing.next_hop_protocol.clone()),
        ro,
    );
    initializer.property(
        JsString::from("workerStart"),
        JsValue::from(entry.resource_timing.worker_start),
        ro,
    );
    initializer.property(
        JsString::from("redirectStart"),
        JsValue::from(entry.resource_timing.redirect_start),
        ro,
    );
    initializer.property(
        JsString::from("redirectEnd"),
        JsValue::from(entry.resource_timing.redirect_end),
        ro,
    );
    initializer.property(
        JsString::from("fetchStart"),
        JsValue::from(entry.resource_timing.fetch_start),
        ro,
    );
    initializer.property(
        JsString::from("domainLookupStart"),
        JsValue::from(entry.resource_timing.domain_lookup_start),
        ro,
    );
    initializer.property(
        JsString::from("domainLookupEnd"),
        JsValue::from(entry.resource_timing.domain_lookup_end),
        ro,
    );
    initializer.property(
        JsString::from("connectStart"),
        JsValue::from(entry.resource_timing.connect_start),
        ro,
    );
    initializer.property(
        JsString::from("connectEnd"),
        JsValue::from(entry.resource_timing.connect_end),
        ro,
    );
    initializer.property(
        JsString::from("secureConnectionStart"),
        JsValue::from(entry.resource_timing.secure_connection_start),
        ro,
    );
    initializer.property(
        JsString::from("requestStart"),
        JsValue::from(entry.resource_timing.request_start),
        ro,
    );
    initializer.property(
        JsString::from("responseStart"),
        JsValue::from(entry.resource_timing.response_start),
        ro,
    );
    initializer.property(
        JsString::from("responseEnd"),
        JsValue::from(entry.resource_timing.response_end),
        ro,
    );
    initializer.property(
        JsString::from("transferSize"),
        JsValue::from(entry.resource_timing.transfer_size),
        ro,
    );
    initializer.property(
        JsString::from("encodedBodySize"),
        JsValue::from(entry.resource_timing.encoded_body_size),
        ro,
    );
    initializer.property(
        JsString::from("decodedBodySize"),
        JsValue::from(entry.resource_timing.decoded_body_size),
        ro,
    );

    initializer.property(
        JsString::from("unloadEventStart"),
        JsValue::from(entry.unload_event_start),
        ro,
    );
    initializer.property(
        JsString::from("unloadEventEnd"),
        JsValue::from(entry.unload_event_end),
        ro,
    );
    initializer.property(
        JsString::from("domLoading"),
        JsValue::from(entry.dom_loading),
        ro,
    );
    initializer.property(
        JsString::from("domInteractive"),
        JsValue::from(entry.dom_interactive),
        ro,
    );
    initializer.property(
        JsString::from("domContentLoadedEventStart"),
        JsValue::from(entry.dom_content_loaded_event_start),
        ro,
    );
    initializer.property(
        JsString::from("domContentLoadedEventEnd"),
        JsValue::from(entry.dom_content_loaded_event_end),
        ro,
    );
    initializer.property(
        JsString::from("domComplete"),
        JsValue::from(entry.dom_complete),
        ro,
    );
    initializer.property(
        JsString::from("loadEventStart"),
        JsValue::from(entry.load_event_start),
        ro,
    );
    initializer.property(
        JsString::from("loadEventEnd"),
        JsValue::from(entry.load_event_end),
        ro,
    );
    initializer.property(
        JsString::from("type"),
        JsString::from(entry.nav_type.clone()),
        ro,
    );
    initializer.property(
        JsString::from("redirectCount"),
        JsValue::from(entry.redirect_count),
        ro,
    );

    Ok(JsValue::from(initializer.build()))
}

/// Creates the standard `performance` object.
pub fn create_performance(context: &mut Context) -> JsObject {
    let _ = context.register_global_class::<Performance>();
    let _ = context.register_global_class::<PerformanceEntry>();
    let _ = context.register_global_class::<PerformanceMark>();
    let _ = context.register_global_class::<PerformanceMeasure>();
    let _ = context.register_global_class::<PerformanceResourceTiming>();
    let _ = context.register_global_class::<PerformanceNavigationTiming>();

    // Link prototype chain so subclasses of PerformanceEntry inherit correctly.
    let inheritance_js = r#"
        if (typeof PerformanceEntry === "function") {
            const classes = [PerformanceMark, PerformanceMeasure, PerformanceResourceTiming, PerformanceNavigationTiming];
            for (const cls of classes) {
                if (typeof cls === "function") {
                    Object.setPrototypeOf(cls, PerformanceEntry);
                    Object.setPrototypeOf(cls.prototype, PerformanceEntry.prototype);
                }
            }
        }
    "#;
    let _ = context.eval(boa_engine::Source::from_bytes(inheritance_js));

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

        // 1. Check getEntries returns all entries (3 user-added + 1 default navigation entry)
        let res = context
            .eval(Source::from_bytes("performance.getEntries().length === 4"))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check chronological ordering of getEntries()
        let res = context
            .eval(Source::from_bytes(
                r#"
                const entries = performance.getEntries();
                entries[0].entryType === "navigation" && entries[1].name === "mark1" && entries[2].name === "measure1" && entries[3].name === "mark2"
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
                performance.getEntriesByType("mark").length === 0
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

    #[test]
    fn test_performance_navigation_and_resource_timings() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // 1. Navigation Timing entry exists by default
        let res = context
            .eval(Source::from_bytes(
                r#"
                const navs = performance.getEntriesByType("navigation");
                navs.length === 1 &&
                navs[0].name === "document" &&
                navs[0].entryType === "navigation" &&
                navs[0].startTime === 0 &&
                navs[0].duration === 0 &&
                navs[0].initiatorType === "navigation" &&
                navs[0].type === "navigate" &&
                navs[0].redirectCount === 0
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. Add resource timing
        let res = context
            .eval(Source::from_bytes(
                r#"
                const resEntry = performance.addResourceTiming("https://example.com/logo.png", 100, 50, {
                    initiatorType: "img",
                    nextHopProtocol: "h2",
                    transferSize: 1024,
                    encodedBodySize: 500,
                    decodedBodySize: 800
                });
                resEntry.name === "https://example.com/logo.png" &&
                resEntry.entryType === "resource" &&
                resEntry.startTime === 100 &&
                resEntry.duration === 50 &&
                resEntry.initiatorType === "img" &&
                resEntry.nextHopProtocol === "h2" &&
                resEntry.transferSize === 1024 &&
                resEntry.encodedBodySize === 500 &&
                resEntry.decodedBodySize === 800
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 3. Retrieval by type/name
        let res = context
            .eval(Source::from_bytes(
                r#"
                const resources = performance.getEntriesByType("resource");
                const named = performance.getEntriesByName("https://example.com/logo.png");
                resources.length === 1 &&
                resources[0].name === "https://example.com/logo.png" &&
                named.length === 1 &&
                named[0].startTime === 100
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 4. toJSON for Resource and Navigation timing entries
        let res = context
            .eval(Source::from_bytes(
                r#"
                const resJson = resources[0].toJSON();
                const navJson = navs[0].toJSON();
                resJson.name === "https://example.com/logo.png" &&
                resJson.entryType === "resource" &&
                resJson.initiatorType === "img" &&
                resJson.transferSize === 1024 &&
                navJson.name === "document" &&
                navJson.entryType === "navigation" &&
                navJson.type === "navigate"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 5. Buffer size limit
        let res = context
            .eval(Source::from_bytes(
                r#"
                performance.setResourceTimingBufferSize(2);
                // currently we have 1 resource timing ("logo.png"). Add a 2nd one.
                performance.addResourceTiming("https://example.com/script.js", 150, 10);
                // adding a 3rd one should be ignored / not stored in the timeline.
                performance.addResourceTiming("https://example.com/style.css", 200, 5);

                const resourcesAfter = performance.getEntriesByType("resource");
                resourcesAfter.length === 2 &&
                resourcesAfter[0].name === "https://example.com/logo.png" &&
                resourcesAfter[1].name === "https://example.com/script.js"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 6. Clear resource timings
        let res = context
            .eval(Source::from_bytes(
                r#"
                performance.clearResourceTimings();
                performance.getEntriesByType("resource").length === 0
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_entry_subclassing_and_prototype_chain() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // Verify PerformanceMark inherits from PerformanceEntry
        let res = context
            .eval(Source::from_bytes(
                r#"
                const mark = performance.mark("test-subclass");
                mark instanceof PerformanceEntry &&
                Object.getPrototypeOf(PerformanceMark) === PerformanceEntry &&
                Object.getPrototypeOf(PerformanceMark.prototype) === PerformanceEntry.prototype
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Verify PerformanceMeasure inherits from PerformanceEntry
        let res = context
            .eval(Source::from_bytes(
                r#"
                const measure = performance.measure("test-meas-subclass", "test-subclass");
                measure instanceof PerformanceEntry &&
                Object.getPrototypeOf(PerformanceMeasure) === PerformanceEntry &&
                Object.getPrototypeOf(PerformanceMeasure.prototype) === PerformanceEntry.prototype
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_measure_resolving_measures() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // 1. Create a measure directly with timestamps (which is valid under options)
        context
            .eval(Source::from_bytes(
                r#"
                performance.measure("meas-base", { start: 100, duration: 50 });
                "#,
            ))
            .unwrap();

        // 2. Resolve "meas-base" as the start mark in a subsequent measure
        let res = context
            .eval(Source::from_bytes(
                r#"
                const m = performance.measure("meas-derived", "meas-base", 150);
                m.startTime === 100 && m.duration === 50
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }
}
