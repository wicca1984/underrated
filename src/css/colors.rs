use crate::css::values::Color;

/// Returns the RGBA color for a given CSS named color.
/// name is compared ASCII case-insensitively.
/// spec: <https://www.w3.org/TR/css-color-4/#named-colors>
pub fn named_color(name: &str) -> Option<Color> {
    match name.to_ascii_lowercase().as_str() {
        "aliceblue" => Some(Color::Rgba(240, 248, 255, 255)),
        "antiquewhite" => Some(Color::Rgba(250, 235, 215, 255)),
        "aqua" => Some(Color::Rgba(0, 255, 255, 255)),
        "aquamarine" => Some(Color::Rgba(127, 255, 212, 255)),
        "azure" => Some(Color::Rgba(240, 255, 255, 255)),
        "beige" => Some(Color::Rgba(245, 245, 220, 255)),
        "bisque" => Some(Color::Rgba(255, 228, 196, 255)),
        "black" => Some(Color::Rgba(0, 0, 0, 255)),
        "blanchedalmond" => Some(Color::Rgba(255, 235, 205, 255)),
        "blue" => Some(Color::Rgba(0, 0, 255, 255)),
        "blueviolet" => Some(Color::Rgba(138, 43, 226, 255)),
        "brown" => Some(Color::Rgba(165, 42, 42, 255)),
        "burlywood" => Some(Color::Rgba(222, 184, 135, 255)),
        "cadetblue" => Some(Color::Rgba(95, 158, 160, 255)),
        "chartreuse" => Some(Color::Rgba(127, 255, 0, 255)),
        "chocolate" => Some(Color::Rgba(210, 105, 30, 255)),
        "coral" => Some(Color::Rgba(255, 127, 80, 255)),
        "cornflowerblue" => Some(Color::Rgba(100, 149, 237, 255)),
        "cornsilk" => Some(Color::Rgba(255, 248, 220, 255)),
        "crimson" => Some(Color::Rgba(220, 20, 60, 255)),
        "cyan" => Some(Color::Rgba(0, 255, 255, 255)),
        "darkblue" => Some(Color::Rgba(0, 0, 139, 255)),
        "darkcyan" => Some(Color::Rgba(0, 139, 139, 255)),
        "darkgoldenrod" => Some(Color::Rgba(184, 134, 11, 255)),
        "darkgray" => Some(Color::Rgba(169, 169, 169, 255)),
        "darkgreen" => Some(Color::Rgba(0, 100, 0, 255)),
        "darkgrey" => Some(Color::Rgba(169, 169, 169, 255)),
        "darkkhaki" => Some(Color::Rgba(189, 183, 107, 255)),
        "darkmagenta" => Some(Color::Rgba(139, 0, 139, 255)),
        "darkolivegreen" => Some(Color::Rgba(85, 107, 47, 255)),
        "darkorange" => Some(Color::Rgba(255, 140, 0, 255)),
        "darkorchid" => Some(Color::Rgba(153, 50, 204, 255)),
        "darkred" => Some(Color::Rgba(139, 0, 0, 255)),
        "darksalmon" => Some(Color::Rgba(233, 150, 122, 255)),
        "darkseagreen" => Some(Color::Rgba(143, 188, 143, 255)),
        "darkslateblue" => Some(Color::Rgba(72, 61, 139, 255)),
        "darkslategray" => Some(Color::Rgba(47, 79, 79, 255)),
        "darkslategrey" => Some(Color::Rgba(47, 79, 79, 255)),
        "darkturquoise" => Some(Color::Rgba(0, 206, 209, 255)),
        "darkviolet" => Some(Color::Rgba(148, 0, 211, 255)),
        "deeppink" => Some(Color::Rgba(255, 20, 147, 255)),
        "deepskyblue" => Some(Color::Rgba(0, 191, 255, 255)),
        "dimgray" => Some(Color::Rgba(105, 105, 105, 255)),
        "dimgrey" => Some(Color::Rgba(105, 105, 105, 255)),
        "dodgerblue" => Some(Color::Rgba(30, 144, 255, 255)),
        "firebrick" => Some(Color::Rgba(178, 34, 34, 255)),
        "floralwhite" => Some(Color::Rgba(255, 250, 240, 255)),
        "forestgreen" => Some(Color::Rgba(34, 139, 34, 255)),
        "fuchsia" => Some(Color::Rgba(255, 0, 255, 255)),
        "gainsboro" => Some(Color::Rgba(220, 220, 220, 255)),
        "ghostwhite" => Some(Color::Rgba(248, 248, 255, 255)),
        "gold" => Some(Color::Rgba(255, 215, 0, 255)),
        "goldenrod" => Some(Color::Rgba(218, 165, 32, 255)),
        "gray" => Some(Color::Rgba(128, 128, 128, 255)),
        "green" => Some(Color::Rgba(0, 128, 0, 255)),
        "greenyellow" => Some(Color::Rgba(173, 255, 47, 255)),
        "grey" => Some(Color::Rgba(128, 128, 128, 255)),
        "honeydew" => Some(Color::Rgba(240, 255, 240, 255)),
        "hotpink" => Some(Color::Rgba(255, 105, 180, 255)),
        "indianred" => Some(Color::Rgba(205, 92, 92, 255)),
        "indigo" => Some(Color::Rgba(75, 0, 130, 255)),
        "ivory" => Some(Color::Rgba(255, 255, 240, 255)),
        "khaki" => Some(Color::Rgba(240, 230, 140, 255)),
        "lavender" => Some(Color::Rgba(230, 230, 250, 255)),
        "lavenderblush" => Some(Color::Rgba(255, 240, 245, 255)),
        "lawngreen" => Some(Color::Rgba(124, 252, 0, 255)),
        "lemonchiffon" => Some(Color::Rgba(255, 250, 205, 255)),
        "lightblue" => Some(Color::Rgba(173, 216, 230, 255)),
        "lightcoral" => Some(Color::Rgba(240, 128, 128, 255)),
        "lightcyan" => Some(Color::Rgba(224, 255, 255, 255)),
        "lightgoldenrodyellow" => Some(Color::Rgba(250, 250, 210, 255)),
        "lightgray" => Some(Color::Rgba(211, 211, 211, 255)),
        "lightgreen" => Some(Color::Rgba(144, 238, 144, 255)),
        "lightgrey" => Some(Color::Rgba(211, 211, 211, 255)),
        "lightpink" => Some(Color::Rgba(255, 182, 193, 255)),
        "lightsalmon" => Some(Color::Rgba(255, 160, 122, 255)),
        "lightseagreen" => Some(Color::Rgba(32, 178, 170, 255)),
        "lightskyblue" => Some(Color::Rgba(135, 206, 250, 255)),
        "lightslategray" => Some(Color::Rgba(119, 136, 153, 255)),
        "lightslategrey" => Some(Color::Rgba(119, 136, 153, 255)),
        "lightsteelblue" => Some(Color::Rgba(176, 196, 222, 255)),
        "lightyellow" => Some(Color::Rgba(255, 255, 224, 255)),
        "lime" => Some(Color::Rgba(0, 255, 0, 255)),
        "limegreen" => Some(Color::Rgba(50, 205, 50, 255)),
        "linen" => Some(Color::Rgba(250, 240, 230, 255)),
        "magenta" => Some(Color::Rgba(255, 0, 255, 255)),
        "maroon" => Some(Color::Rgba(128, 0, 0, 255)),
        "mediumaquamarine" => Some(Color::Rgba(102, 205, 170, 255)),
        "mediumblue" => Some(Color::Rgba(0, 0, 205, 255)),
        "mediumorchid" => Some(Color::Rgba(186, 85, 211, 255)),
        "mediumpurple" => Some(Color::Rgba(147, 112, 219, 255)),
        "mediumseagreen" => Some(Color::Rgba(60, 179, 113, 255)),
        "mediumslateblue" => Some(Color::Rgba(123, 104, 238, 255)),
        "mediumspringgreen" => Some(Color::Rgba(0, 250, 154, 255)),
        "mediumturquoise" => Some(Color::Rgba(72, 209, 204, 255)),
        "mediumvioletred" => Some(Color::Rgba(199, 21, 133, 255)),
        "midnightblue" => Some(Color::Rgba(25, 25, 112, 255)),
        "mintcream" => Some(Color::Rgba(245, 255, 250, 255)),
        "mistyrose" => Some(Color::Rgba(255, 228, 225, 255)),
        "moccasin" => Some(Color::Rgba(255, 228, 181, 255)),
        "navajowhite" => Some(Color::Rgba(255, 222, 173, 255)),
        "navy" => Some(Color::Rgba(0, 0, 128, 255)),
        "oldlace" => Some(Color::Rgba(253, 245, 230, 255)),
        "olive" => Some(Color::Rgba(128, 128, 0, 255)),
        "olivedrab" => Some(Color::Rgba(107, 142, 35, 255)),
        "orange" => Some(Color::Rgba(255, 165, 0, 255)),
        "orangered" => Some(Color::Rgba(255, 69, 0, 255)),
        "orchid" => Some(Color::Rgba(218, 112, 214, 255)),
        "palegoldenrod" => Some(Color::Rgba(238, 232, 170, 255)),
        "palegreen" => Some(Color::Rgba(152, 251, 152, 255)),
        "paleturquoise" => Some(Color::Rgba(175, 238, 238, 255)),
        "palevioletred" => Some(Color::Rgba(219, 112, 147, 255)),
        "papayawhip" => Some(Color::Rgba(255, 239, 213, 255)),
        "peachpuff" => Some(Color::Rgba(255, 218, 185, 255)),
        "peru" => Some(Color::Rgba(205, 133, 63, 255)),
        "pink" => Some(Color::Rgba(255, 192, 203, 255)),
        "plum" => Some(Color::Rgba(221, 160, 221, 255)),
        "powderblue" => Some(Color::Rgba(176, 224, 230, 255)),
        "purple" => Some(Color::Rgba(128, 0, 128, 255)),
        "rebeccapurple" => Some(Color::Rgba(102, 51, 153, 255)),
        "red" => Some(Color::Rgba(255, 0, 0, 255)),
        "rosybrown" => Some(Color::Rgba(188, 143, 143, 255)),
        "royalblue" => Some(Color::Rgba(65, 105, 225, 255)),
        "saddlebrown" => Some(Color::Rgba(139, 69, 19, 255)),
        "salmon" => Some(Color::Rgba(250, 128, 114, 255)),
        "sandybrown" => Some(Color::Rgba(244, 164, 96, 255)),
        "seagreen" => Some(Color::Rgba(46, 139, 87, 255)),
        "seashell" => Some(Color::Rgba(255, 245, 238, 255)),
        "sienna" => Some(Color::Rgba(160, 82, 45, 255)),
        "silver" => Some(Color::Rgba(192, 192, 192, 255)),
        "skyblue" => Some(Color::Rgba(135, 206, 235, 255)),
        "slateblue" => Some(Color::Rgba(106, 90, 205, 255)),
        "slategray" => Some(Color::Rgba(112, 128, 144, 255)),
        "slategrey" => Some(Color::Rgba(112, 128, 144, 255)),
        "snow" => Some(Color::Rgba(255, 250, 250, 255)),
        "springgreen" => Some(Color::Rgba(0, 255, 127, 255)),
        "steelblue" => Some(Color::Rgba(70, 130, 180, 255)),
        "tan" => Some(Color::Rgba(210, 180, 140, 255)),
        "teal" => Some(Color::Rgba(0, 128, 128, 255)),
        "thistle" => Some(Color::Rgba(216, 191, 216, 255)),
        "tomato" => Some(Color::Rgba(255, 99, 71, 255)),
        "transparent" => Some(Color::Rgba(0, 0, 0, 0)),
        "turquoise" => Some(Color::Rgba(64, 224, 208, 255)),
        "violet" => Some(Color::Rgba(238, 130, 238, 255)),
        "wheat" => Some(Color::Rgba(245, 222, 179, 255)),
        "white" => Some(Color::Rgba(255, 255, 255, 255)),
        "whitesmoke" => Some(Color::Rgba(245, 245, 245, 255)),
        "yellow" => Some(Color::Rgba(255, 255, 0, 255)),
        "yellowgreen" => Some(Color::Rgba(154, 205, 50, 255)),
        // CSS System Colors (CSS Color Module Level 4)
        // These are UA-chosen light-theme defaults.
        "canvas" => Some(Color::Rgba(255, 255, 255, 255)),
        "canvastext" => Some(Color::Rgba(0, 0, 0, 255)),
        "linktext" => Some(Color::Rgba(0, 0, 238, 255)),
        "visitedtext" => Some(Color::Rgba(85, 26, 139, 255)),
        "activetext" => Some(Color::Rgba(238, 0, 0, 255)),
        "buttonface" => Some(Color::Rgba(240, 240, 240, 255)),
        "buttontext" => Some(Color::Rgba(0, 0, 0, 255)),
        "buttonborder" => Some(Color::Rgba(118, 118, 118, 255)),
        "field" => Some(Color::Rgba(255, 255, 255, 255)),
        "fieldtext" => Some(Color::Rgba(0, 0, 0, 255)),
        "highlight" => Some(Color::Rgba(51, 153, 255, 255)),
        "highlighttext" => Some(Color::Rgba(255, 255, 255, 255)),
        "selecteditem" => Some(Color::Rgba(0, 90, 158, 255)),
        "selecteditemtext" => Some(Color::Rgba(255, 255, 255, 255)),
        "mark" => Some(Color::Rgba(255, 255, 0, 255)),
        "marktext" => Some(Color::Rgba(0, 0, 0, 255)),
        "graytext" => Some(Color::Rgba(128, 128, 128, 255)),
        "accentcolor" => Some(Color::Rgba(0, 120, 215, 255)),
        "accentcolortext" => Some(Color::Rgba(255, 255, 255, 255)),
        // Deprecated CSS2 System Colors (CSS Color Module Level 4, Section 14.2)
        "activeborder" => Some(Color::Rgba(118, 118, 118, 255)),
        "activecaption" => Some(Color::Rgba(204, 204, 204, 255)),
        "appworkspace" => Some(Color::Rgba(240, 240, 240, 255)),
        "background" => Some(Color::Rgba(0, 120, 215, 255)),
        "buttonhighlight" => Some(Color::Rgba(255, 255, 255, 255)),
        "buttonshadow" => Some(Color::Rgba(128, 128, 128, 255)),
        "captiontext" => Some(Color::Rgba(0, 0, 0, 255)),
        "inactiveborder" => Some(Color::Rgba(244, 244, 244, 255)),
        "inactivecaption" => Some(Color::Rgba(244, 244, 244, 255)),
        "inactivecaptiontext" => Some(Color::Rgba(128, 128, 128, 255)),
        "infobackground" => Some(Color::Rgba(255, 255, 225, 255)),
        "infotext" => Some(Color::Rgba(0, 0, 0, 255)),
        "menu" => Some(Color::Rgba(240, 240, 240, 255)),
        "menutext" => Some(Color::Rgba(0, 0, 0, 255)),
        "scrollbar" => Some(Color::Rgba(200, 200, 200, 255)),
        "threeddarkshadow" => Some(Color::Rgba(0, 0, 0, 255)),
        "threedface" => Some(Color::Rgba(240, 240, 240, 255)),
        "threedhighlight" => Some(Color::Rgba(255, 255, 255, 255)),
        "threedlightshadow" => Some(Color::Rgba(224, 224, 224, 255)),
        "threedshadow" => Some(Color::Rgba(160, 160, 160, 255)),
        "window" => Some(Color::Rgba(255, 255, 255, 255)),
        "windowframe" => Some(Color::Rgba(118, 118, 118, 255)),
        "windowtext" => Some(Color::Rgba(0, 0, 0, 255)),
        _ => None,
    }
}

/// Parses an HSL color into RGBA.
/// h is in degrees, s, l, and a are in the range [0.0, 1.0].
/// spec: <https://www.w3.org/TR/css-color-4/#hsl-color>
pub fn parse_hsl(h: f32, s: f32, l: f32, a: f32) -> Color {
    // Sanitize non-finite inputs (NaN / infinity) to 0 so they cannot propagate
    // into the output (a NaN would otherwise cast to a meaningless channel).
    fn finite_or_zero(v: f32) -> f32 {
        if v.is_finite() { v } else { 0.0 }
    }
    let s = finite_or_zero(s).clamp(0.0, 1.0);
    let l = finite_or_zero(l).clamp(0.0, 1.0);
    let a = finite_or_zero(a).clamp(0.0, 1.0);

    // Normalize hue to [0, 360)
    let h = finite_or_zero(h) % 360.0;
    let h = if h < 0.0 { h + 360.0 } else { h };

    fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }

    let (r, g, b) = if s == 0.0 {
        (l, l, l)
    } else {
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        (
            hue_to_rgb(p, q, h / 360.0 + 1.0 / 3.0),
            hue_to_rgb(p, q, h / 360.0),
            hue_to_rgb(p, q, h / 360.0 - 1.0 / 3.0),
        )
    };

    Color::Rgba(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        (a * 255.0).round() as u8,
    )
}

/// Converts a standard sRGB channel value (0 to 255) to a linear sRGB f32 value in the range [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#srgb-to-linear>
pub fn srgb_to_linear(val: u8) -> f32 {
    let c = val as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts a linear sRGB channel value in the range [0.0, 1.0] to a standard sRGB channel value (0 to 255).
/// Spec: <https://www.w3.org/TR/css-color-4/#linear-to-srgb>
pub fn linear_to_srgb(c: f32) -> u8 {
    let c = if c.is_finite() { c } else { 0.0 }.clamp(0.0, 1.0);
    let s = if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Parses an HWB color into RGBA.
/// h is in degrees, w, b, and a are in the range [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#hwb-to-rgb>
pub fn parse_hwb(h: f32, w: f32, b: f32, a: f32) -> Color {
    fn finite_or_zero(v: f32) -> f32 {
        if v.is_finite() { v } else { 0.0 }
    }
    let w = finite_or_zero(w).clamp(0.0, 1.0);
    let b = finite_or_zero(b).clamp(0.0, 1.0);
    let a = finite_or_zero(a).clamp(0.0, 1.0);

    // Normalize hue to [0, 360)
    let h = finite_or_zero(h) % 360.0;
    let h = if h < 0.0 { h + 360.0 } else { h };

    // If w + b > 1.0, they are normalized by dividing both by (w + b)
    let sum = w + b;
    let (w_norm, b_norm) = if sum > 1.0 {
        (w / sum, b / sum)
    } else {
        (w, b)
    };

    fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }

    let p = 0.0;
    let q = 1.0;
    let r1 = hue_to_rgb(p, q, h / 360.0 + 1.0 / 3.0);
    let g1 = hue_to_rgb(p, q, h / 360.0);
    let b1 = hue_to_rgb(p, q, h / 360.0 - 1.0 / 3.0);

    // Blend with whiteness and blackness
    let factor = 1.0 - w_norm - b_norm;
    let r = r1 * factor + w_norm;
    let g = g1 * factor + w_norm;
    let b = b1 * factor + w_norm;

    Color::Rgba(
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8,
        (a * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// Converts an RGBA Color to HSL components.
/// Returns (h, s, l, a) where h is in [0.0, 360.0], s, l, a are in [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#rgb-to-hsl>
pub fn color_to_hsl(color: Color) -> (f32, f32, f32, f32) {
    let Color::Rgba(r_u, g_u, b_u, a_u) = color;
    let r = r_u as f32 / 255.0;
    let g = g_u as f32 / 255.0;
    let b = b_u as f32 / 255.0;
    let a = a_u as f32 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let d = max - min;

    let l = (max + min) / 2.0;

    let s = if d == 0.0 {
        0.0
    } else {
        let denom = 1.0 - (2.0 * l - 1.0).abs();
        if denom == 0.0 { 0.0 } else { d / denom }
    };

    let mut h = if d == 0.0 {
        0.0
    } else if max == r {
        let mut val = (g - b) / d;
        if val < 0.0 {
            val += 6.0;
        }
        val % 6.0
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    h *= 60.0;
    if h < 0.0 {
        h += 360.0;
    } else if h >= 360.0 {
        h -= 360.0;
    }

    (h, s, l, a)
}

/// Converts an RGBA Color to HWB components.
/// Returns (h, w, b, a) where h is in [0.0, 360.0], w, b, a are in [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#rgb-to-hwb>
pub fn color_to_hwb(color: Color) -> (f32, f32, f32, f32) {
    let Color::Rgba(r_u, g_u, b_u, a_u) = color;
    let r = r_u as f32 / 255.0;
    let g = g_u as f32 / 255.0;
    let b = b_u as f32 / 255.0;
    let a = a_u as f32 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let d = max - min;

    let mut h = if d == 0.0 {
        0.0
    } else if max == r {
        let mut val = (g - b) / d;
        if val < 0.0 {
            val += 6.0;
        }
        val % 6.0
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    h *= 60.0;
    if h < 0.0 {
        h += 360.0;
    } else if h >= 360.0 {
        h -= 360.0;
    }

    let w = min;
    let b_val = 1.0 - max;

    (h, w, b_val, a)
}

/// Parses a Lab color into RGBA.
/// l, a, b are in the standard Lab ranges, alpha is in [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#lab-to-rgb>
pub fn parse_lab(l: f32, a: f32, b: f32, alpha: f32) -> Color {
    fn finite_or_zero(v: f32) -> f32 {
        if v.is_finite() { v } else { 0.0 }
    }
    let l_val = finite_or_zero(l).max(0.0) as f64;
    let a_val = finite_or_zero(a) as f64;
    let b_val = finite_or_zero(b) as f64;
    let alpha_val = (finite_or_zero(alpha).clamp(0.0, 1.0) * 255.0).round() as u8;

    let fy = (l_val + 16.0) / 116.0;
    let fx = fy + a_val / 500.0;
    let fz = fy - b_val / 200.0;

    let finv = |t: f64| {
        let d = 6.0 / 29.0;
        if t > d {
            t * t * t
        } else {
            3.0 * d * d * (t - 4.0 / 29.0)
        }
    };

    let xr = finv(fx);
    let yr = finv(fy);
    let zr = finv(fz);

    let x = xr * 0.96422;
    let y = yr * 1.0;
    let z = zr * 0.82521;

    // Bradford-adapted D50 XYZ -> linear sRGB:
    let r_lin = 3.1338561 * x - 1.6168667 * y - 0.4906146 * z;
    let g_lin = -0.9787684 * x + 1.9161415 * y + 0.0334540 * z;
    let b_lin = 0.0719453 * x - 0.2289914 * y + 1.4052427 * z;

    let gamma_encode = |c: f64| {
        let c = c.clamp(0.0, 1.0);
        let s = if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0).round().clamp(0.0, 255.0) as u8
    };

    let r = gamma_encode(r_lin);
    let g = gamma_encode(g_lin);
    let b = gamma_encode(b_lin);

    Color::Rgba(r, g, b, alpha_val)
}

/// Parses an LCH color into RGBA.
/// l, c are in standard ranges, h is in degrees, alpha is in [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#lch-to-rgb>
pub fn parse_lch(l: f32, c: f32, h: f32, alpha: f32) -> Color {
    fn finite_or_zero(v: f32) -> f32 {
        if v.is_finite() { v } else { 0.0 }
    }
    let l_val = finite_or_zero(l).max(0.0);
    let c_val = finite_or_zero(c).max(0.0);
    let h_deg = finite_or_zero(h);
    let h_deg = ((h_deg % 360.0) + 360.0) % 360.0;
    let h_rad = h_deg.to_radians();

    let a_val = c_val * h_rad.cos();
    let b_val = c_val * h_rad.sin();

    parse_lab(l_val, a_val, b_val, alpha)
}

/// Parses an OKLAB color into RGBA.
/// l, a, b are in standard OKLAB ranges, alpha is in [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#oklab-to-rgb>
pub fn parse_oklab(l: f32, a: f32, b: f32, alpha: f32) -> Color {
    fn finite_or_zero(v: f32) -> f32 {
        if v.is_finite() { v } else { 0.0 }
    }
    let l_val = finite_or_zero(l).max(0.0) as f64;
    let a_val = finite_or_zero(a) as f64;
    let b_val = finite_or_zero(b) as f64;
    let alpha_val = (finite_or_zero(alpha).clamp(0.0, 1.0) * 255.0).round() as u8;

    let l_ = l_val + 0.3963377774 * a_val + 0.2158037573 * b_val;
    let m_ = l_val - 0.1055613458 * a_val - 0.0638541728 * b_val;
    let s_ = l_val - 0.0894841775 * a_val - 1.2914855480 * b_val;

    let l_cube = l_ * l_ * l_;
    let m_cube = m_ * m_ * m_;
    let s_cube = s_ * s_ * s_;

    let r_lin = 4.0767416621 * l_cube - 3.3077115913 * m_cube + 0.2309699292 * s_cube;
    let g_lin = -1.2684380046 * l_cube + 2.6097574011 * m_cube - 0.3413193965 * s_cube;
    let b_lin = -0.0041960863 * l_cube - 0.7034186147 * m_cube + 1.7076147010 * s_cube;

    let gamma_encode = |c: f64| {
        let c = c.clamp(0.0, 1.0);
        let s = if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0).round().clamp(0.0, 255.0) as u8
    };

    let r = gamma_encode(r_lin);
    let g = gamma_encode(g_lin);
    let b = gamma_encode(b_lin);

    Color::Rgba(r, g, b, alpha_val)
}

/// Parses an OKLCH color into RGBA.
/// l, c are in standard ranges, h is in degrees, alpha is in [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#oklch-to-rgb>
pub fn parse_oklch(l: f32, c: f32, h: f32, alpha: f32) -> Color {
    fn finite_or_zero(v: f32) -> f32 {
        if v.is_finite() { v } else { 0.0 }
    }
    let l_val = finite_or_zero(l).max(0.0);
    let c_val = finite_or_zero(c).max(0.0);
    let h_deg = finite_or_zero(h);
    let h_deg = ((h_deg % 360.0) + 360.0) % 360.0;
    let h_rad = h_deg.to_radians();

    let a_val = c_val * h_rad.cos();
    let b_val = c_val * h_rad.sin();

    parse_oklab(l_val, a_val, b_val, alpha)
}

/// Converts an RGBA Color to Lab components.
/// Returns (l, a, b, alpha) where l, a, b are in standard Lab ranges, alpha is in [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#rgb-to-lab>
pub fn color_to_lab(color: Color) -> (f32, f32, f32, f32) {
    let Color::Rgba(r_u, g_u, b_u, a_u) = color;
    let r_lin = srgb_to_linear(r_u) as f64;
    let g_lin = srgb_to_linear(g_u) as f64;
    let b_lin = srgb_to_linear(b_u) as f64;

    // srgb_to_xyz_d50 matrix multiplication:
    let x = 0.4360747 * r_lin + 0.3850649 * g_lin + 0.1430804 * b_lin;
    let y = 0.2225045 * r_lin + 0.7168786 * g_lin + 0.0606169 * b_lin;
    let z = 0.0139322 * r_lin + 0.0971045 * g_lin + 0.7141733 * b_lin;

    let f = |t: f64| {
        let d = 6.0 / 29.0;
        if t > d * d * d {
            t.powf(1.0 / 3.0)
        } else {
            t / (3.0 * d * d) + 4.0 / 29.0
        }
    };

    let xr = x / 0.96422;
    let yr = y / 1.0;
    let zr = z / 0.82521;

    let fx = f(xr);
    let fy = f(yr);
    let fz = f(zr);

    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    let alpha = a_u as f32 / 255.0;

    (l as f32, a as f32, b as f32, alpha)
}

/// Converts an RGBA Color to LCH components.
/// Returns (l, c, h, alpha) where l, c are in standard ranges, h is in degrees, alpha is in [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#rgb-to-lch>
pub fn color_to_lch(color: Color) -> (f32, f32, f32, f32) {
    let (l, a, b, alpha) = color_to_lab(color);
    let c = (a * a + b * b).sqrt();
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    (l, c, h, alpha)
}

/// Converts an RGBA Color to OKLAB components.
/// Returns (l, a, b, alpha) where l, a, b are in standard ranges, alpha is in [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#rgb-to-oklab>
pub fn color_to_oklab(color: Color) -> (f32, f32, f32, f32) {
    let Color::Rgba(r_u, g_u, b_u, a_u) = color;
    let r_lin = srgb_to_linear(r_u) as f64;
    let g_lin = srgb_to_linear(g_u) as f64;
    let b_lin = srgb_to_linear(b_u) as f64;

    // linear_srgb_to_lms:
    let l_lms = 0.4122214708 * r_lin + 0.5363325363 * g_lin + 0.0514459929 * b_lin;
    let m_lms = 0.2119034982 * r_lin + 0.6806995451 * g_lin + 0.1073969566 * b_lin;
    let s_lms = 0.0883024619 * r_lin + 0.2817188376 * g_lin + 0.6299787005 * b_lin;

    let l_ = l_lms.cbrt();
    let m_ = m_lms.cbrt();
    let s_ = s_lms.cbrt();

    // lms_to_oklab:
    let l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    let alpha = a_u as f32 / 255.0;

    (l as f32, a as f32, b as f32, alpha)
}

/// Converts an RGBA Color to OKLCH components.
/// Returns (l, c, h, alpha) where l, c are in standard ranges, h is in degrees, alpha is in [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#rgb-to-oklch>
pub fn color_to_oklch(color: Color) -> (f32, f32, f32, f32) {
    let (l, a, b, alpha) = color_to_oklab(color);
    let c = (a * a + b * b).sqrt();
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    (l, c, h, alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_named_colors() {
        assert_eq!(named_color("red"), Some(Color::Rgba(255, 0, 0, 255)));
        assert_eq!(named_color("RED"), Some(Color::Rgba(255, 0, 0, 255)));
        assert_eq!(named_color("blue"), Some(Color::Rgba(0, 0, 255, 255)));
        assert_eq!(named_color("black"), Some(Color::Rgba(0, 0, 0, 255)));
        assert_eq!(named_color("white"), Some(Color::Rgba(255, 255, 255, 255)));
        assert_eq!(named_color("transparent"), Some(Color::Rgba(0, 0, 0, 0)));
        assert_eq!(
            named_color("rebeccapurple"),
            Some(Color::Rgba(102, 51, 153, 255))
        );
        assert_eq!(named_color("unknown"), None);
    }

    #[test]
    fn test_system_colors() {
        // Test case-insensitive resolution of Level 4 system color keywords
        assert_eq!(named_color("Canvas"), Some(Color::Rgba(255, 255, 255, 255)));
        assert_eq!(named_color("canvas"), Some(Color::Rgba(255, 255, 255, 255)));
        assert_eq!(named_color("CANVAS"), Some(Color::Rgba(255, 255, 255, 255)));
        assert_eq!(named_color("CanvasText"), Some(Color::Rgba(0, 0, 0, 255)));
        assert_eq!(named_color("canvastext"), Some(Color::Rgba(0, 0, 0, 255)));
        assert_eq!(named_color("LinkText"), Some(Color::Rgba(0, 0, 238, 255)));
        assert_eq!(
            named_color("VisitedText"),
            Some(Color::Rgba(85, 26, 139, 255))
        );
        assert_eq!(named_color("ActiveText"), Some(Color::Rgba(238, 0, 0, 255)));
        assert_eq!(
            named_color("ButtonFace"),
            Some(Color::Rgba(240, 240, 240, 255))
        );
        assert_eq!(named_color("ButtonText"), Some(Color::Rgba(0, 0, 0, 255)));
        assert_eq!(
            named_color("ButtonBorder"),
            Some(Color::Rgba(118, 118, 118, 255))
        );
        assert_eq!(named_color("Field"), Some(Color::Rgba(255, 255, 255, 255)));
        assert_eq!(named_color("FieldText"), Some(Color::Rgba(0, 0, 0, 255)));
        assert_eq!(
            named_color("Highlight"),
            Some(Color::Rgba(51, 153, 255, 255))
        );
        assert_eq!(
            named_color("HighlightText"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(
            named_color("SelectedItem"),
            Some(Color::Rgba(0, 90, 158, 255))
        );
        assert_eq!(
            named_color("SelectedItemText"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(named_color("Mark"), Some(Color::Rgba(255, 255, 0, 255)));
        assert_eq!(named_color("MarkText"), Some(Color::Rgba(0, 0, 0, 255)));
        assert_eq!(
            named_color("GrayText"),
            Some(Color::Rgba(128, 128, 128, 255))
        );
        assert_eq!(
            named_color("AccentColor"),
            Some(Color::Rgba(0, 120, 215, 255))
        );
        assert_eq!(
            named_color("AccentColorText"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(named_color("UnknownSystemColor"), None);
    }

    #[test]
    fn test_parse_hsl() {
        // Red
        assert_eq!(parse_hsl(0.0, 1.0, 0.5, 1.0), Color::Rgba(255, 0, 0, 255));
        // Green (120 degrees)
        assert_eq!(parse_hsl(120.0, 1.0, 0.5, 1.0), Color::Rgba(0, 255, 0, 255));
        // Blue (240 degrees)
        assert_eq!(parse_hsl(240.0, 1.0, 0.5, 1.0), Color::Rgba(0, 0, 255, 255));
        // White
        assert_eq!(
            parse_hsl(0.0, 0.0, 1.0, 1.0),
            Color::Rgba(255, 255, 255, 255)
        );
        // Black
        assert_eq!(parse_hsl(0.0, 0.0, 0.0, 1.0), Color::Rgba(0, 0, 0, 255));
        // Gray
        assert_eq!(
            parse_hsl(0.0, 0.0, 0.5, 1.0),
            Color::Rgba(128, 128, 128, 255)
        );
        // Semi-transparent red
        assert_eq!(parse_hsl(0.0, 1.0, 0.5, 0.5), Color::Rgba(255, 0, 0, 128));

        // Hue wrap-around
        assert_eq!(parse_hsl(360.0, 1.0, 0.5, 1.0), Color::Rgba(255, 0, 0, 255));
        assert_eq!(
            parse_hsl(-360.0, 1.0, 0.5, 1.0),
            Color::Rgba(255, 0, 0, 255)
        );
        assert_eq!(parse_hsl(720.0, 1.0, 0.5, 1.0), Color::Rgba(255, 0, 0, 255));
    }

    #[test]
    fn test_deprecated_system_colors() {
        assert_eq!(
            named_color("activeborder"),
            Some(Color::Rgba(118, 118, 118, 255))
        );
        assert_eq!(
            named_color("infobackground"),
            Some(Color::Rgba(255, 255, 225, 255))
        );
        assert_eq!(named_color("window"), Some(Color::Rgba(255, 255, 255, 255)));
        assert_eq!(
            named_color("activecaption"),
            Some(Color::Rgba(204, 204, 204, 255))
        );
    }

    #[test]
    fn test_srgb_conversions() {
        // srgb_to_linear and linear_to_srgb round-tripping or specific values
        assert_eq!(linear_to_srgb(srgb_to_linear(128)), 128);
        assert_eq!(linear_to_srgb(srgb_to_linear(0)), 0);
        assert_eq!(linear_to_srgb(srgb_to_linear(255)), 255);

        // check non-finite or out-of-range safety
        assert_eq!(linear_to_srgb(f32::NAN), 0);
        assert_eq!(linear_to_srgb(f32::INFINITY), 0);
        assert_eq!(linear_to_srgb(-10.0), 0);
        assert_eq!(linear_to_srgb(10.0), 255);
    }

    #[test]
    fn test_parse_hwb() {
        // hwb(0 0% 0%) -> red
        assert_eq!(parse_hwb(0.0, 0.0, 0.0, 1.0), Color::Rgba(255, 0, 0, 255));
        // hwb(120 0% 0%) -> green
        assert_eq!(parse_hwb(120.0, 0.0, 0.0, 1.0), Color::Rgba(0, 255, 0, 255));
        // hwb(240 0% 0%) -> blue
        assert_eq!(parse_hwb(240.0, 0.0, 0.0, 1.0), Color::Rgba(0, 0, 255, 255));
        // hwb(0 100% 0%) -> white
        assert_eq!(
            parse_hwb(0.0, 1.0, 0.0, 1.0),
            Color::Rgba(255, 255, 255, 255)
        );
        // hwb(0 0% 100%) -> black
        assert_eq!(parse_hwb(0.0, 0.0, 1.0, 1.0), Color::Rgba(0, 0, 0, 255));

        // Clamping and normalization
        // w + b > 1.0 (e.g. w=0.4, b=0.7) -> sum = 1.1 -> w_norm = 0.4/1.1, b_norm = 0.7/1.1
        let clamped = parse_hwb(0.0, 0.4, 0.7, 1.0);
        let expected_gray = ((0.4f32 / 1.1f32) * 255.0f32).round() as u8;
        assert_eq!(
            clamped,
            Color::Rgba(expected_gray, expected_gray, expected_gray, 255)
        );

        // Non-finite values
        assert_eq!(
            parse_hwb(f32::NAN, f32::INFINITY, f32::NAN, 1.0),
            Color::Rgba(255, 0, 0, 255)
        );
    }

    #[test]
    fn test_color_to_hsl_and_back() {
        let colors = [
            Color::Rgba(255, 0, 0, 255),
            Color::Rgba(0, 255, 0, 255),
            Color::Rgba(0, 0, 255, 255),
            Color::Rgba(128, 128, 128, 255),
            Color::Rgba(255, 255, 255, 255),
            Color::Rgba(0, 0, 0, 255),
        ];

        for color in colors {
            let Color::Rgba(r1, g1, b1, a1) = color;
            let (h, s, l, a) = color_to_hsl(Color::Rgba(r1, g1, b1, a1));
            let converted = parse_hsl(h, s, l, a);
            // Allow tiny rounding tolerance
            let Color::Rgba(r2, g2, b2, a2) = converted;
            assert!((r1 as i32 - r2 as i32).abs() <= 1);
            assert!((g1 as i32 - g2 as i32).abs() <= 1);
            assert!((b1 as i32 - b2 as i32).abs() <= 1);
            assert_eq!(a1, a2);
        }
    }

    #[test]
    fn test_color_to_hwb_and_back() {
        let colors = [
            Color::Rgba(255, 0, 0, 255),
            Color::Rgba(0, 255, 0, 255),
            Color::Rgba(0, 0, 255, 255),
            Color::Rgba(128, 128, 128, 255),
            Color::Rgba(255, 255, 255, 255),
            Color::Rgba(0, 0, 0, 255),
        ];

        for color in colors {
            let Color::Rgba(r1, g1, b1, a1) = color;
            let (h, w, b, a) = color_to_hwb(Color::Rgba(r1, g1, b1, a1));
            let converted = parse_hwb(h, w, b, a);
            let Color::Rgba(r2, g2, b2, a2) = converted;
            assert!((r1 as i32 - r2 as i32).abs() <= 1);
            assert!((g1 as i32 - g2 as i32).abs() <= 1);
            assert!((b1 as i32 - b2 as i32).abs() <= 1);
            assert_eq!(a1, a2);
        }
    }

    #[test]
    fn test_parse_lab_and_back() {
        // Test lab(100 0 0) -> white
        assert_eq!(
            parse_lab(100.0, 0.0, 0.0, 1.0),
            Color::Rgba(255, 255, 255, 255)
        );
        // Test lab(0 0 0) -> black
        assert_eq!(parse_lab(0.0, 0.0, 0.0, 1.0), Color::Rgba(0, 0, 0, 255));

        // Test roundtrip
        let red = Color::Rgba(255, 0, 0, 255);
        let (l, a, b, alpha) = color_to_lab(red);
        let back = parse_lab(l, a, b, alpha);
        let Color::Rgba(r, g, b_val, a_val) = back;
        assert!((255 - r as i32).abs() <= 1);
        assert!((g as i32).abs() <= 1);
        assert!((b_val as i32).abs() <= 1);
        assert_eq!(a_val, 255);
    }

    #[test]
    fn test_parse_lch_and_back() {
        // Test lch(100 0 0) -> white
        assert_eq!(
            parse_lch(100.0, 0.0, 0.0, 1.0),
            Color::Rgba(255, 255, 255, 255)
        );
        // Test lch(0 0 0) -> black
        assert_eq!(parse_lch(0.0, 0.0, 0.0, 1.0), Color::Rgba(0, 0, 0, 255));

        // Test roundtrip
        let green = Color::Rgba(0, 255, 0, 255);
        let (l, c, h, alpha) = color_to_lch(green);
        let back = parse_lch(l, c, h, alpha);
        let Color::Rgba(r, g, b_val, a_val) = back;
        assert!((r as i32).abs() <= 1);
        assert!((255 - g as i32).abs() <= 1);
        assert!((b_val as i32).abs() <= 1);
        assert_eq!(a_val, 255);
    }

    #[test]
    fn test_parse_oklab_and_back() {
        // Test oklab(1 0 0) -> white
        assert_eq!(
            parse_oklab(1.0, 0.0, 0.0, 1.0),
            Color::Rgba(255, 255, 255, 255)
        );
        // Test oklab(0 0 0) -> black
        assert_eq!(parse_oklab(0.0, 0.0, 0.0, 1.0), Color::Rgba(0, 0, 0, 255));

        // Test roundtrip
        let blue = Color::Rgba(0, 0, 255, 255);
        let (l, a, b, alpha) = color_to_oklab(blue);
        let back = parse_oklab(l, a, b, alpha);
        let Color::Rgba(r, g, b_val, a_val) = back;
        assert!((r as i32).abs() <= 1);
        assert!((g as i32).abs() <= 1);
        assert!((255 - b_val as i32).abs() <= 1);
        assert_eq!(a_val, 255);
    }

    #[test]
    fn test_parse_oklch_and_back() {
        // Test oklch(1 0 0) -> white
        assert_eq!(
            parse_oklch(1.0, 0.0, 0.0, 1.0),
            Color::Rgba(255, 255, 255, 255)
        );
        // Test oklch(0 0 0) -> black
        assert_eq!(parse_oklch(0.0, 0.0, 0.0, 1.0), Color::Rgba(0, 0, 0, 255));

        // Test roundtrip
        let gray = Color::Rgba(128, 128, 128, 255);
        let (l, c, h, alpha) = color_to_oklch(gray);
        let back = parse_oklch(l, c, h, alpha);
        let Color::Rgba(r, g, b_val, a_val) = back;
        assert!((128 - r as i32).abs() <= 1);
        assert!((128 - g as i32).abs() <= 1);
        assert!((128 - b_val as i32).abs() <= 1);
        assert_eq!(a_val, 255);
    }
}
