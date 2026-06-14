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
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8,
        (a * 255.0).round().clamp(0.0, 255.0) as u8,
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

/// Serializes a Color into its standard CSS Color Module Level 4 string representation.
/// Opaque colors (alpha = 255) are serialized as `rgb(r, g, b)`.
/// Non-opaque colors (alpha < 255) are serialized as `rgba(r, g, b, a)`,
/// where `a` is a float in `[0.0, 1.0]` with trailing zeros omitted.
/// Spec: <https://www.w3.org/TR/css-color-4/#serializing-color-values>
pub fn serialize_color(color: Color) -> String {
    let Color::Rgba(r, g, b, a) = color;
    if a == 255 {
        format!("rgb({}, {}, {})", r, g, b)
    } else if a == 0 {
        format!("rgba({}, {}, {}, 0)", r, g, b)
    } else {
        let alpha_val = a as f32 / 255.0;
        // Format with up to 5 decimal places to be precise
        let formatted = format!("{:.5}", alpha_val);
        let mut trimmed = formatted.trim_end_matches('0');
        if trimmed.ends_with('.') {
            trimmed = &trimmed[..trimmed.len() - 1];
        }
        format!("rgba({}, {}, {}, {})", r, g, b, trimmed)
    }
}

fn extract_relative_origin_color(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let first_space = s.find(|c: char| c.is_ascii_whitespace());
    let first_paren = s.find('(');

    match (first_space, first_paren) {
        (Some(sp_idx), Some(p_idx)) if p_idx < sp_idx => {
            let mut depth = 0;
            let mut end_idx = None;
            for (i, c) in s.char_indices() {
                if c == '(' {
                    depth += 1;
                } else if c == ')' {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = Some(i + 1);
                        break;
                    }
                }
            }
            if let Some(end) = end_idx {
                Some((&s[..end], &s[end..]))
            } else {
                None
            }
        }
        _ => {
            if let Some(sp_idx) = first_space {
                Some((&s[..sp_idx], &s[sp_idx..]))
            } else {
                Some((s, ""))
            }
        }
    }
}

fn resolve_relative_tokens(func_name: &str, origin_color: Color, tokens: &[&str]) -> Vec<String> {
    match func_name {
        "rgb" | "rgba" => {
            let Color::Rgba(r, g, b, a) = origin_color;
            let r_val = r as f32;
            let g_val = g as f32;
            let b_val = b as f32;
            let a_val = a as f32 / 255.0;
            tokens
                .iter()
                .map(|&t| match t.to_ascii_lowercase().as_str() {
                    "r" => format!("{}", r_val),
                    "g" => format!("{}", g_val),
                    "b" => format!("{}", b_val),
                    "alpha" => format!("{}", a_val),
                    other => other.to_string(),
                })
                .collect()
        }
        "hsl" | "hsla" => {
            let (h, s, l, a) = color_to_hsl(origin_color);
            tokens
                .iter()
                .map(|&t| match t.to_ascii_lowercase().as_str() {
                    "h" => format!("{}", h),
                    "s" => format!("{}", s),
                    "l" => format!("{}", l),
                    "alpha" => format!("{}", a),
                    other => other.to_string(),
                })
                .collect()
        }
        "hwb" => {
            let (h, w, b, a) = color_to_hwb(origin_color);
            tokens
                .iter()
                .map(|&t| match t.to_ascii_lowercase().as_str() {
                    "h" => format!("{}", h),
                    "w" => format!("{}", w),
                    "b" => format!("{}", b),
                    "alpha" => format!("{}", a),
                    other => other.to_string(),
                })
                .collect()
        }
        "lab" => {
            let (l, a, b_val, alpha) = color_to_lab(origin_color);
            tokens
                .iter()
                .map(|&t| match t.to_ascii_lowercase().as_str() {
                    "l" => format!("{}", l),
                    "a" => format!("{}", a),
                    "b" => format!("{}", b_val),
                    "alpha" => format!("{}", alpha),
                    other => other.to_string(),
                })
                .collect()
        }
        "lch" => {
            let (l, c, h, alpha) = color_to_lch(origin_color);
            tokens
                .iter()
                .map(|&t| match t.to_ascii_lowercase().as_str() {
                    "l" => format!("{}", l),
                    "c" => format!("{}", c),
                    "h" => format!("{}", h),
                    "alpha" => format!("{}", alpha),
                    other => other.to_string(),
                })
                .collect()
        }
        "oklab" => {
            let (l, a, b_val, alpha) = color_to_oklab(origin_color);
            tokens
                .iter()
                .map(|&t| match t.to_ascii_lowercase().as_str() {
                    "l" => format!("{}", l),
                    "a" => format!("{}", a),
                    "b" => format!("{}", b_val),
                    "alpha" => format!("{}", alpha),
                    other => other.to_string(),
                })
                .collect()
        }
        "oklch" => {
            let (l, c, h, alpha) = color_to_oklch(origin_color);
            tokens
                .iter()
                .map(|&t| match t.to_ascii_lowercase().as_str() {
                    "l" => format!("{}", l),
                    "c" => format!("{}", c),
                    "h" => format!("{}", h),
                    "alpha" => format!("{}", alpha),
                    other => other.to_string(),
                })
                .collect()
        }
        _ => tokens.iter().map(|&t| t.to_string()).collect(),
    }
}

fn get_color_parts(func_name: &str, content: &str) -> Option<Vec<String>> {
    let content_trimmed = content.trim();
    if content_trimmed.to_ascii_lowercase().starts_with("from ") {
        let from_content = &content_trimmed[5..].trim();
        let (origin_color_str, rest_str) = extract_relative_origin_color(from_content)?;
        let origin_color = parse_color(origin_color_str)?;
        let rest_clean = rest_str.replace(['/', ','], " ");
        let raw_tokens: Vec<&str> = rest_clean.split_whitespace().collect();
        let resolved = resolve_relative_tokens(func_name, origin_color, &raw_tokens);
        Some(resolved)
    } else {
        let content_clean = content_trimmed.replace('/', " ");
        let parts: Vec<String> = if content_clean.contains(',') {
            content_clean
                .split(',')
                .map(|p| p.trim().to_string())
                .collect()
        } else {
            content_clean
                .split_whitespace()
                .map(|p| p.to_string())
                .collect()
        };
        Some(parts)
    }
}

/// Parses a CSS color string into a Color.
/// Supports hex colors (#RGB, #RGBA, #RRGGBB, #RRGGBBAA), functional notations, and named/system colors.
/// Spec: <https://www.w3.org/TR/css-color-4/#color-syntax>
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(hex) = s.strip_prefix('#') {
        // A hex color is ASCII only; bail before any byte slicing so that a
        // non-ASCII character cannot panic on a char boundary (I-6).
        if !hex.is_ascii() {
            return None;
        }
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                Some(Color::Rgba(r * 17, g * 17, b * 17, 255))
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()?;
                Some(Color::Rgba(r * 17, g * 17, b * 17, a * 17))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::Rgba(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color::Rgba(r, g, b, a))
            }
            _ => None,
        }
    } else {
        let s_lower = s.to_ascii_lowercase();
        if s.ends_with(')') {
            if s_lower.starts_with("rgb(") || s_lower.starts_with("rgba(") {
                let start = if s_lower.starts_with("rgba(") { 5 } else { 4 };
                let content = s[start..s.len() - 1].trim();
                let parts_vec = get_color_parts("rgb", content)?;
                let parts: Vec<&str> = parts_vec.iter().map(|p| p.as_str()).collect();
                if parts.len() >= 3 {
                    let r = parse_rgb_component(parts[0])?;
                    let g = parse_rgb_component(parts[1])?;
                    let b = parse_rgb_component(parts[2])?;
                    let alpha = if parts.len() >= 4 {
                        parse_alpha_component(parts[3])?
                    } else {
                        1.0
                    };
                    return Some(Color::Rgba(
                        r.clamp(0.0, 255.0).round() as u8,
                        g.clamp(0.0, 255.0).round() as u8,
                        b.clamp(0.0, 255.0).round() as u8,
                        (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
                    ));
                }
            } else if s_lower.starts_with("hsl(") || s_lower.starts_with("hsla(") {
                let start = if s_lower.starts_with("hsla(") { 5 } else { 4 };
                let content = s[start..s.len() - 1].trim();
                let parts_vec = get_color_parts("hsl", content)?;
                let parts: Vec<&str> = parts_vec.iter().map(|p| p.as_str()).collect();
                if parts.len() >= 3 {
                    let h = parse_hue_angle(parts[0])?;
                    let s_val = parse_percentage_or_number(parts[1])?;
                    let l = parse_percentage_or_number(parts[2])?;
                    let alpha = if parts.len() >= 4 {
                        parse_alpha_component(parts[3])?
                    } else {
                        1.0
                    };
                    return Some(parse_hsl(h, s_val, l, alpha));
                }
            } else if s_lower.starts_with("hwb(") {
                let content = s[4..s.len() - 1].trim();
                let parts_vec = get_color_parts("hwb", content)?;
                let parts: Vec<&str> = parts_vec.iter().map(|p| p.as_str()).collect();
                if parts.len() >= 3 {
                    let h = parse_hue_angle(parts[0])?;
                    let w = parse_percentage_or_number(parts[1])?;
                    let b = parse_percentage_or_number(parts[2])?;
                    let alpha = if parts.len() >= 4 {
                        parse_alpha_component(parts[3])?
                    } else {
                        1.0
                    };
                    return Some(parse_hwb(h, w, b, alpha));
                }
            } else if s_lower.starts_with("lab(") {
                let content = s[4..s.len() - 1].trim();
                let parts_vec = get_color_parts("lab", content)?;
                let parts: Vec<&str> = parts_vec.iter().map(|p| p.as_str()).collect();
                if parts.len() >= 3 {
                    let l = parse_lab_lightness(parts[0])?;
                    let a = parse_lab_ab(parts[1])?;
                    let b = parse_lab_ab(parts[2])?;
                    let alpha = if parts.len() >= 4 {
                        parse_alpha_component(parts[3])?
                    } else {
                        1.0
                    };
                    return Some(parse_lab(l, a, b, alpha));
                }
            } else if s_lower.starts_with("lch(") {
                let content = s[4..s.len() - 1].trim();
                let parts_vec = get_color_parts("lch", content)?;
                let parts: Vec<&str> = parts_vec.iter().map(|p| p.as_str()).collect();
                if parts.len() >= 3 {
                    let l = parse_lab_lightness(parts[0])?;
                    let c = parse_lch_chroma(parts[1])?;
                    let h = parse_hue_angle(parts[2])?;
                    let alpha = if parts.len() >= 4 {
                        parse_alpha_component(parts[3])?
                    } else {
                        1.0
                    };
                    return Some(parse_lch(l, c, h, alpha));
                }
            } else if s_lower.starts_with("oklab(") {
                let content = s[6..s.len() - 1].trim();
                let parts_vec = get_color_parts("oklab", content)?;
                let parts: Vec<&str> = parts_vec.iter().map(|p| p.as_str()).collect();
                if parts.len() >= 3 {
                    let l = parse_percentage_or_number(parts[0])?;
                    let a = parse_oklab_ab(parts[1])?;
                    let b = parse_oklab_ab(parts[2])?;
                    let alpha = if parts.len() >= 4 {
                        parse_alpha_component(parts[3])?
                    } else {
                        1.0
                    };
                    return Some(parse_oklab(l, a, b, alpha));
                }
            } else if s_lower.starts_with("oklch(") {
                let content = s[6..s.len() - 1].trim();
                let parts_vec = get_color_parts("oklch", content)?;
                let parts: Vec<&str> = parts_vec.iter().map(|p| p.as_str()).collect();
                if parts.len() >= 3 {
                    let l = parse_percentage_or_number(parts[0])?;
                    let c = parse_oklch_chroma(parts[1])?;
                    let h = parse_hue_angle(parts[2])?;
                    let alpha = if parts.len() >= 4 {
                        parse_alpha_component(parts[3])?
                    } else {
                        1.0
                    };
                    return Some(parse_oklch(l, c, h, alpha));
                }
            } else if s_lower.starts_with("color(") {
                let content = s[6..s.len() - 1].trim();
                let content_clean = content.replace(['/', ','], " ");
                let parts: Vec<&str> = content_clean.split_whitespace().collect();
                if parts.len() == 4 || parts.len() == 5 {
                    let colorspace = parts[0];
                    let c1 = parse_percentage_or_number(parts[1])?;
                    let c2 = parse_percentage_or_number(parts[2])?;
                    let c3 = parse_percentage_or_number(parts[3])?;
                    let alpha = if parts.len() == 5 {
                        parse_alpha_component(parts[4])?
                    } else {
                        1.0
                    };
                    return parse_predefined_color(colorspace, c1, c2, c3, alpha);
                }
            } else if s_lower.starts_with("color-mix(") {
                let content = s[10..s.len() - 1].trim();
                let parts = split_top_level_commas(content);
                if parts.len() == 3 {
                    let space_part = parts[0].trim();
                    if let Some(stripped) = space_part.strip_prefix("in ") {
                        let space_tokens: Vec<&str> = stripped.split_whitespace().collect();
                        if !space_tokens.is_empty() {
                            let colorspace = space_tokens[0];
                            let hue_method = if space_tokens.len() >= 2 {
                                Some(space_tokens[1])
                            } else {
                                None
                            };

                            let (color1, p1) = parse_color_mix_decl(&parts[1])?;
                            let (color2, p2) = parse_color_mix_decl(&parts[2])?;

                            let (w1, w2, alpha_scale) = match (p1, p2) {
                                (None, None) => (50.0, 50.0, 1.0),
                                (Some(val), None) => (val, 100.0 - val, 1.0),
                                (None, Some(val)) => (100.0 - val, val, 1.0),
                                (Some(v1), Some(v2)) => {
                                    let sum = v1 + v2;
                                    if sum <= 0.0001
                                        || v1 < 0.0
                                        || v2 < 0.0
                                        || v1 > 100.0
                                        || v2 > 100.0
                                    {
                                        return None;
                                    }
                                    if sum > 100.0 {
                                        (v1, v2, 1.0)
                                    } else {
                                        (v1, v2, sum / 100.0)
                                    }
                                }
                            };

                            let total = w1 + w2;
                            if total <= 0.0 {
                                return None;
                            }
                            let weight = w2 / total;

                            let mut mixed =
                                mix_colors(color1, color2, weight, colorspace, hue_method)?;
                            if alpha_scale < 1.0 {
                                let Color::Rgba(r, g, b, a) = mixed;
                                let a_new = ((a as f32 / 255.0) * alpha_scale * 255.0)
                                    .round()
                                    .clamp(0.0, 255.0)
                                    as u8;
                                mixed = Color::Rgba(r, g, b, a_new);
                            }
                            return Some(mixed);
                        }
                    }
                }
            } else if s_lower.starts_with("light-dark(") {
                let content = s[11..s.len() - 1].trim();
                let parts = split_top_level_commas(content);
                if parts.len() == 2 {
                    let first_color = parse_color(&parts[0])?;
                    let _second_color = parse_color(&parts[1])?;
                    return Some(first_color);
                }
            }
        }
        named_color(s)
    }
}

fn parse_predefined_color(
    colorspace: &str,
    c1: f32,
    c2: f32,
    c3: f32,
    alpha: f32,
) -> Option<Color> {
    let alpha_u8 = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    let c1 = c1 as f64;
    let c2 = c2 as f64;
    let c3 = c3 as f64;
    match colorspace.to_ascii_lowercase().as_str() {
        "srgb" => {
            let r = (c1 * 255.0).round().clamp(0.0, 255.0) as u8;
            let g = (c2 * 255.0).round().clamp(0.0, 255.0) as u8;
            let b = (c3 * 255.0).round().clamp(0.0, 255.0) as u8;
            Some(Color::Rgba(r, g, b, alpha_u8))
        }
        "srgb-linear" => {
            let r = linear_to_srgb(c1 as f32);
            let g = linear_to_srgb(c2 as f32);
            let b = linear_to_srgb(c3 as f32);
            Some(Color::Rgba(r, g, b, alpha_u8))
        }
        "display-p3" => {
            let r_lin = 1.2249401 * c1 - 0.2249404 * c2 + 0.0 * c3;
            let g_lin = -0.0420569 * c1 + 1.0420571 * c2 + 0.0 * c3;
            let b_lin = -0.0197376 * c1 - 0.0786361 * c2 + 1.0983735 * c3;
            let r = linear_to_srgb(r_lin as f32);
            let g = linear_to_srgb(g_lin as f32);
            let b = linear_to_srgb(b_lin as f32);
            Some(Color::Rgba(r, g, b, alpha_u8))
        }
        "xyz" | "xyz-d65" => {
            let r_lin = 3.24096994 * c1 - 1.53738318 * c2 - 0.49861076 * c3;
            let g_lin = -0.96924364 * c1 + 1.87596750 * c2 + 0.04155506 * c3;
            let b_lin = 0.05563008 * c1 - 0.20397696 * c2 + 1.05697151 * c3;
            let r = linear_to_srgb(r_lin as f32);
            let g = linear_to_srgb(g_lin as f32);
            let b = linear_to_srgb(b_lin as f32);
            Some(Color::Rgba(r, g, b, alpha_u8))
        }
        "xyz-d50" => {
            let r_lin = 3.1338561 * c1 - 1.6168667 * c2 - 0.4906146 * c3;
            let g_lin = -0.9787684 * c1 + 1.9161415 * c2 + 0.0334540 * c3;
            let b_lin = 0.0719453 * c1 - 0.2289914 * c2 + 1.4052427 * c3;
            let r = linear_to_srgb(r_lin as f32);
            let g = linear_to_srgb(g_lin as f32);
            let b = linear_to_srgb(b_lin as f32);
            Some(Color::Rgba(r, g, b, alpha_u8))
        }
        "a98-rgb" => {
            let lin = |val: f64| {
                let sign = if val < 0.0 { -1.0 } else { 1.0 };
                sign * val.abs().powf(563.0 / 256.0)
            };
            let r_lin = lin(c1);
            let g_lin = lin(c2);
            let b_lin = lin(c3);

            let x = (573536.0 / 994567.0) * r_lin
                + (263643.0 / 1420810.0) * g_lin
                + (187206.0 / 994567.0) * b_lin;
            let y = (591459.0 / 1989134.0) * r_lin
                + (6239551.0 / 9945670.0) * g_lin
                + (374412.0 / 4972835.0) * b_lin;
            let z = (53769.0 / 1989134.0) * r_lin
                + (351524.0 / 4972835.0) * g_lin
                + (4929758.0 / 4972835.0) * b_lin;

            let r_lin_srgb = 3.24096994 * x - 1.53738318 * y - 0.49861076 * z;
            let g_lin_srgb = -0.96924364 * x + 1.87596750 * y + 0.04155506 * z;
            let b_lin_srgb = 0.05563008 * x - 0.20397696 * y + 1.05697151 * z;

            let r = linear_to_srgb(r_lin_srgb as f32);
            let g = linear_to_srgb(g_lin_srgb as f32);
            let b = linear_to_srgb(b_lin_srgb as f32);
            Some(Color::Rgba(r, g, b, alpha_u8))
        }
        "prophoto-rgb" => {
            let lin = |val: f64| {
                let et = 1.0 / 512.0;
                let sign = if val < 0.0 { -1.0 } else { 1.0 };
                let abs = val.abs();
                if abs >= et {
                    sign * abs.powf(1.8)
                } else {
                    val / 16.0
                }
            };
            let r_lin = lin(c1);
            let g_lin = lin(c2);
            let b_lin = lin(c3);

            let x = 0.7977666449006423 * r_lin
                + 0.13518129740053308 * g_lin
                + 0.0313477341283922 * b_lin;
            let y = 0.2880748288194013 * r_lin
                + 0.711835234241873 * g_lin
                + 0.00008993693872564 * b_lin;
            let z = 0.0 * r_lin + 0.0 * g_lin + 0.8251046025104602 * b_lin;

            let r_lin_srgb = 3.1338561 * x - 1.6168667 * y - 0.4906146 * z;
            let g_lin_srgb = -0.9787684 * x + 1.9161415 * y + 0.0334540 * z;
            let b_lin_srgb = 0.0719453 * x - 0.2289914 * y + 1.4052427 * z;

            let r = linear_to_srgb(r_lin_srgb as f32);
            let g = linear_to_srgb(g_lin_srgb as f32);
            let b = linear_to_srgb(b_lin_srgb as f32);
            Some(Color::Rgba(r, g, b, alpha_u8))
        }
        "rec2020" => {
            let lin = |val: f64| {
                let sign = if val < 0.0 { -1.0 } else { 1.0 };
                sign * val.abs().powf(2.4)
            };
            let r_lin = lin(c1);
            let g_lin = lin(c2);
            let b_lin = lin(c3);

            let x = 0.6369580483012914 * r_lin
                + 0.14461690358620832 * g_lin
                + 0.1688809751641721 * b_lin;
            let y = 0.2627002120112671 * r_lin
                + 0.6779980715188708 * g_lin
                + 0.05930171646986196 * b_lin;
            let z = 0.0 * r_lin + 0.028072693049087428 * g_lin + 1.060985057710791 * b_lin;

            let r_lin_srgb = 3.24096994 * x - 1.53738318 * y - 0.49861076 * z;
            let g_lin_srgb = -0.96924364 * x + 1.87596750 * y + 0.04155506 * z;
            let b_lin_srgb = 0.05563008 * x - 0.20397696 * y + 1.05697151 * z;

            let r = linear_to_srgb(r_lin_srgb as f32);
            let g = linear_to_srgb(g_lin_srgb as f32);
            let b = linear_to_srgb(b_lin_srgb as f32);
            Some(Color::Rgba(r, g, b, alpha_u8))
        }
        _ => None,
    }
}

fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    for c in s.chars() {
        if c == '(' {
            depth += 1;
            current.push(c);
        } else if c == ')' {
            if depth > 0 {
                depth -= 1;
            }
            current.push(c);
        } else if c == ',' && depth == 0 {
            parts.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(c);
        }
    }
    parts.push(current.trim().to_string());
    parts
}

fn parse_color_mix_decl(decl: &str) -> Option<(Color, Option<f32>)> {
    let decl = decl.trim();
    if decl.starts_with(|c: char| c.is_ascii_digit() || c == '.' || c == '-') {
        let first_space_idx = decl.find(char::is_whitespace)?;
        let pct_str = decl[..first_space_idx].trim();
        let stripped = pct_str.strip_suffix('%')?;
        let pct = stripped
            .parse::<f32>()
            .ok()
            .filter(|&p| (0.0..=100.0).contains(&p))?;
        let color_str = decl[first_space_idx..].trim();
        let color = parse_color(color_str)?;
        return Some((color, Some(pct)));
    }

    if let Some(last_space_idx) = decl.rfind(char::is_whitespace) {
        let pct_str = decl[last_space_idx..].trim();
        if let Some(stripped) = pct_str.strip_suffix('%') {
            let pct = stripped
                .parse::<f32>()
                .ok()
                .filter(|&p| (0.0..=100.0).contains(&p))?;
            let color_str = decl[..last_space_idx].trim();
            let color = parse_color(color_str)?;
            return Some((color, Some(pct)));
        }
    }

    let color = parse_color(decl)?;
    Some((color, None))
}

fn parse_hue_angle(part: &str) -> Option<f32> {
    let part = part.trim().to_ascii_lowercase();
    if part == "none" {
        return Some(0.0);
    }
    if let Some(stripped) = part.strip_suffix("deg") {
        stripped.parse::<f32>().ok()
    } else if let Some(stripped) = part.strip_suffix("rad") {
        let rad = stripped.parse::<f32>().ok()?;
        Some(rad.to_degrees())
    } else if let Some(stripped) = part.strip_suffix("grad") {
        let grad = stripped.parse::<f32>().ok()?;
        Some(grad * 0.9)
    } else if let Some(stripped) = part.strip_suffix("turn") {
        let turn = stripped.parse::<f32>().ok()?;
        Some(turn * 360.0)
    } else {
        part.parse::<f32>().ok()
    }
}

fn parse_rgb_component(part: &str) -> Option<f32> {
    let part = part.trim();
    if part.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    if let Some(stripped) = part.strip_suffix('%') {
        let p = stripped.parse::<f32>().ok()?;
        Some((p / 100.0) * 255.0)
    } else {
        part.parse::<f32>().ok()
    }
}

fn parse_alpha_component(part: &str) -> Option<f32> {
    let part = part.trim();
    if part.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    if let Some(stripped) = part.strip_suffix('%') {
        let p = stripped.parse::<f32>().ok()?;
        Some(p / 100.0)
    } else {
        part.parse::<f32>().ok()
    }
}

fn parse_percentage_or_number(part: &str) -> Option<f32> {
    let part = part.trim();
    if part.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    if let Some(stripped) = part.strip_suffix('%') {
        let p = stripped.parse::<f32>().ok()?;
        Some(p / 100.0)
    } else {
        part.parse::<f32>().ok()
    }
}

fn parse_lab_lightness(part: &str) -> Option<f32> {
    let part = part.trim();
    if part.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    if let Some(stripped) = part.strip_suffix('%') {
        stripped.parse::<f32>().ok()
    } else {
        part.parse::<f32>().ok()
    }
}

fn parse_oklab_ab(part: &str) -> Option<f32> {
    let part = part.trim();
    if part.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    if let Some(stripped) = part.strip_suffix('%') {
        let p = stripped.parse::<f32>().ok()?;
        Some((p / 100.0) * 0.4)
    } else {
        part.parse::<f32>().ok()
    }
}

fn parse_lab_ab(part: &str) -> Option<f32> {
    let part = part.trim();
    if part.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    if let Some(stripped) = part.strip_suffix('%') {
        let p = stripped.parse::<f32>().ok()?;
        Some((p / 100.0) * 125.0)
    } else {
        part.parse::<f32>().ok()
    }
}

fn parse_lch_chroma(part: &str) -> Option<f32> {
    let part = part.trim();
    if part.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    if let Some(stripped) = part.strip_suffix('%') {
        let p = stripped.parse::<f32>().ok()?;
        Some((p / 100.0) * 150.0)
    } else {
        part.parse::<f32>().ok()
    }
}

fn parse_oklch_chroma(part: &str) -> Option<f32> {
    let part = part.trim();
    if part.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    if let Some(stripped) = part.strip_suffix('%') {
        let p = stripped.parse::<f32>().ok()?;
        Some((p / 100.0) * 0.4)
    } else {
        part.parse::<f32>().ok()
    }
}

fn interpolate_hue(h1: f32, h2: f32, t: f32, method: &str) -> f32 {
    let h1 = if h1.is_finite() { h1 } else { 0.0 };
    let h2 = if h2.is_finite() { h2 } else { 0.0 };

    let mut theta1 = h1 % 360.0;
    if theta1 < 0.0 {
        theta1 += 360.0;
    }
    let mut theta2 = h2 % 360.0;
    if theta2 < 0.0 {
        theta2 += 360.0;
    }

    match method.to_ascii_lowercase().as_str() {
        "shorter" => {
            let diff = theta2 - theta1;
            if diff > 180.0 {
                theta2 -= 360.0;
            } else if diff < -180.0 {
                theta2 += 360.0;
            }
        }
        "longer" => {
            let diff = theta2 - theta1;
            if diff > 0.0 && diff < 180.0 {
                theta2 -= 360.0;
            } else if diff <= 0.0 && diff > -180.0 {
                theta2 += 360.0;
            }
        }
        "increasing" if theta2 < theta1 => {
            theta2 += 360.0;
        }
        "decreasing" if theta2 > theta1 => {
            theta2 -= 360.0;
        }
        _ => {} // "specified"
    }

    let mut h = theta1 * (1.0 - t) + theta2 * t;
    h %= 360.0;
    if h < 0.0 {
        h += 360.0;
    }
    h
}

/// Interpolates between two colors in a given color space.
/// weight is the weight of color2 in the range [0.0, 1.0].
/// Spec: <https://www.w3.org/TR/css-color-4/#interpolation>
pub fn mix_colors(
    color1: Color,
    color2: Color,
    weight: f32,
    colorspace: &str,
    hue_method: Option<&str>,
) -> Option<Color> {
    let t = if weight.is_finite() { weight } else { 0.5 }.clamp(0.0, 1.0);
    let hue_method = hue_method.unwrap_or("shorter");

    match colorspace.to_ascii_lowercase().as_str() {
        "srgb" => {
            let Color::Rgba(r1, g1, b1, a1) = color1;
            let Color::Rgba(r2, g2, b2, a2) = color2;

            let r1 = r1 as f32 / 255.0;
            let g1 = g1 as f32 / 255.0;
            let b1 = b1 as f32 / 255.0;
            let a1 = a1 as f32 / 255.0;

            let r2 = r2 as f32 / 255.0;
            let g2 = g2 as f32 / 255.0;
            let b2 = b2 as f32 / 255.0;
            let a2 = a2 as f32 / 255.0;

            let pr1 = r1 * a1;
            let pg1 = g1 * a1;
            let pb1 = b1 * a1;

            let pr2 = r2 * a2;
            let pg2 = g2 * a2;
            let pb2 = b2 * a2;

            let a_mix = a1 * (1.0 - t) + a2 * t;

            let pr_mix = pr1 * (1.0 - t) + pr2 * t;
            let pg_mix = pg1 * (1.0 - t) + pg2 * t;
            let pb_mix = pb1 * (1.0 - t) + pb2 * t;

            let (r, g, b) = if a_mix > 0.0 {
                (pr_mix / a_mix, pg_mix / a_mix, pb_mix / a_mix)
            } else {
                (0.0, 0.0, 0.0)
            };

            Some(Color::Rgba(
                (r * 255.0).round().clamp(0.0, 255.0) as u8,
                (g * 255.0).round().clamp(0.0, 255.0) as u8,
                (b * 255.0).round().clamp(0.0, 255.0) as u8,
                (a_mix * 255.0).round().clamp(0.0, 255.0) as u8,
            ))
        }
        "srgb-linear" | "linear-srgb" => {
            let Color::Rgba(r1, g1, b1, a1) = color1;
            let Color::Rgba(r2, g2, b2, a2) = color2;

            let r1 = srgb_to_linear(r1);
            let g1 = srgb_to_linear(g1);
            let b1 = srgb_to_linear(b1);
            let a1 = a1 as f32 / 255.0;

            let r2 = srgb_to_linear(r2);
            let g2 = srgb_to_linear(g2);
            let b2 = srgb_to_linear(b2);
            let a2 = a2 as f32 / 255.0;

            let pr1 = r1 * a1;
            let pg1 = g1 * a1;
            let pb1 = b1 * a1;

            let pr2 = r2 * a2;
            let pg2 = g2 * a2;
            let pb2 = b2 * a2;

            let a_mix = a1 * (1.0 - t) + a2 * t;

            let pr_mix = pr1 * (1.0 - t) + pr2 * t;
            let pg_mix = pg1 * (1.0 - t) + pg2 * t;
            let pb_mix = pb1 * (1.0 - t) + pb2 * t;

            let (r_lin, g_lin, b_lin) = if a_mix > 0.0 {
                (pr_mix / a_mix, pg_mix / a_mix, pb_mix / a_mix)
            } else {
                (0.0, 0.0, 0.0)
            };

            Some(Color::Rgba(
                linear_to_srgb(r_lin),
                linear_to_srgb(g_lin),
                linear_to_srgb(b_lin),
                (a_mix * 255.0).round().clamp(0.0, 255.0) as u8,
            ))
        }
        "lab" => {
            let (l1, a1_val, b1_val, alpha1) = color_to_lab(color1);
            let (l2, a2_val, b2_val, alpha2) = color_to_lab(color2);

            let pl1 = l1 * alpha1;
            let pa1 = a1_val * alpha1;
            let pb1 = b1_val * alpha1;

            let pl2 = l2 * alpha2;
            let pa2 = a2_val * alpha2;
            let pb2 = b2_val * alpha2;

            let a_mix = alpha1 * (1.0 - t) + alpha2 * t;

            let (l, a, b) = if a_mix > 0.0 {
                (
                    pl1 * (1.0 - t) + pl2 * t,
                    pa1 * (1.0 - t) + pa2 * t,
                    pb1 * (1.0 - t) + pb2 * t,
                )
            } else {
                (0.0, 0.0, 0.0)
            };

            let (l_unpre, a_unpre, b_unpre) = if a_mix > 0.0 {
                (l / a_mix, a / a_mix, b / a_mix)
            } else {
                (0.0, 0.0, 0.0)
            };

            Some(parse_lab(l_unpre, a_unpre, b_unpre, a_mix))
        }
        "oklab" => {
            let (l1, a1_val, b1_val, alpha1) = color_to_oklab(color1);
            let (l2, a2_val, b2_val, alpha2) = color_to_oklab(color2);

            let pl1 = l1 * alpha1;
            let pa1 = a1_val * alpha1;
            let pb1 = b1_val * alpha1;

            let pl2 = l2 * alpha2;
            let pa2 = a2_val * alpha2;
            let pb2 = b2_val * alpha2;

            let a_mix = alpha1 * (1.0 - t) + alpha2 * t;

            let (l, a, b) = if a_mix > 0.0 {
                (
                    pl1 * (1.0 - t) + pl2 * t,
                    pa1 * (1.0 - t) + pa2 * t,
                    pb1 * (1.0 - t) + pb2 * t,
                )
            } else {
                (0.0, 0.0, 0.0)
            };

            let (l_unpre, a_unpre, b_unpre) = if a_mix > 0.0 {
                (l / a_mix, a / a_mix, b / a_mix)
            } else {
                (0.0, 0.0, 0.0)
            };

            Some(parse_oklab(l_unpre, a_unpre, b_unpre, a_mix))
        }
        "lch" => {
            let (l1, c1, mut h1, alpha1) = color_to_lch(color1);
            let (l2, c2, mut h2, alpha2) = color_to_lch(color2);

            if c1 < 0.0001 {
                h1 = h2;
            }
            if c2 < 0.0001 {
                h2 = h1;
            }

            let pl1 = l1 * alpha1;
            let pc1 = c1 * alpha1;

            let pl2 = l2 * alpha2;
            let pc2 = c2 * alpha2;

            let a_mix = alpha1 * (1.0 - t) + alpha2 * t;

            let l_unpre = if a_mix > 0.0 {
                (pl1 * (1.0 - t) + pl2 * t) / a_mix
            } else {
                l1 * (1.0 - t) + l2 * t
            };

            let c_unpre = if a_mix > 0.0 {
                (pc1 * (1.0 - t) + pc2 * t) / a_mix
            } else {
                c1 * (1.0 - t) + c2 * t
            };

            let h_mix = interpolate_hue(h1, h2, t, hue_method);

            Some(parse_lch(l_unpre, c_unpre, h_mix, a_mix))
        }
        "oklch" => {
            let (l1, c1, mut h1, alpha1) = color_to_oklch(color1);
            let (l2, c2, mut h2, alpha2) = color_to_oklch(color2);

            if c1 < 0.0001 {
                h1 = h2;
            }
            if c2 < 0.0001 {
                h2 = h1;
            }

            let pl1 = l1 * alpha1;
            let pc1 = c1 * alpha1;

            let pl2 = l2 * alpha2;
            let pc2 = c2 * alpha2;

            let a_mix = alpha1 * (1.0 - t) + alpha2 * t;

            let l_unpre = if a_mix > 0.0 {
                (pl1 * (1.0 - t) + pl2 * t) / a_mix
            } else {
                l1 * (1.0 - t) + l2 * t
            };

            let c_unpre = if a_mix > 0.0 {
                (pc1 * (1.0 - t) + pc2 * t) / a_mix
            } else {
                c1 * (1.0 - t) + c2 * t
            };

            let h_mix = interpolate_hue(h1, h2, t, hue_method);

            Some(parse_oklch(l_unpre, c_unpre, h_mix, a_mix))
        }
        "hsl" => {
            let (mut h1, s1, l1, alpha1) = color_to_hsl(color1);
            let (mut h2, s2, l2, alpha2) = color_to_hsl(color2);

            if s1 < 0.0001 {
                h1 = h2;
            }
            if s2 < 0.0001 {
                h2 = h1;
            }

            let ps1 = s1 * alpha1;
            let pl1 = l1 * alpha1;

            let ps2 = s2 * alpha2;
            let pl2 = l2 * alpha2;

            let a_mix = alpha1 * (1.0 - t) + alpha2 * t;

            let s_unpre = if a_mix > 0.0 {
                (ps1 * (1.0 - t) + ps2 * t) / a_mix
            } else {
                s1 * (1.0 - t) + s2 * t
            };

            let l_unpre = if a_mix > 0.0 {
                (pl1 * (1.0 - t) + pl2 * t) / a_mix
            } else {
                l1 * (1.0 - t) + l2 * t
            };

            let h_mix = interpolate_hue(h1, h2, t, hue_method);

            Some(parse_hsl(h_mix, s_unpre, l_unpre, a_mix))
        }
        "hwb" => {
            let (mut h1, w1, b1, alpha1) = color_to_hwb(color1);
            let (mut h2, w2, b2, alpha2) = color_to_hwb(color2);

            if w1 + b1 >= 0.9999 {
                h1 = h2;
            }
            if w2 + b2 >= 0.9999 {
                h2 = h1;
            }

            let pw1 = w1 * alpha1;
            let pb1 = b1 * alpha1;

            let pw2 = w2 * alpha2;
            let pb2 = b2 * alpha2;

            let a_mix = alpha1 * (1.0 - t) + alpha2 * t;

            let w_unpre = if a_mix > 0.0 {
                (pw1 * (1.0 - t) + pw2 * t) / a_mix
            } else {
                w1 * (1.0 - t) + w2 * t
            };

            let b_unpre = if a_mix > 0.0 {
                (pb1 * (1.0 - t) + pb2 * t) / a_mix
            } else {
                b1 * (1.0 - t) + b2 * t
            };

            let h_mix = interpolate_hue(h1, h2, t, hue_method);

            Some(parse_hwb(h_mix, w_unpre, b_unpre, a_mix))
        }
        _ => None,
    }
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

    #[test]
    fn test_parse_color() {
        // Hex colors without alpha
        assert_eq!(parse_color("#fff"), Some(Color::Rgba(255, 255, 255, 255)));
        assert_eq!(parse_color("#ff0000"), Some(Color::Rgba(255, 0, 0, 255)));

        // Hex colors with alpha (#RGBA, #RRGGBBAA)
        assert_eq!(parse_color("#0000"), Some(Color::Rgba(0, 0, 0, 0)));
        assert_eq!(parse_color("#ff000080"), Some(Color::Rgba(255, 0, 0, 128)));
        assert_eq!(
            parse_color("#ffffff1a"),
            Some(Color::Rgba(255, 255, 255, 26))
        );

        // Invalid hex colors
        assert_eq!(parse_color("#fffg"), None);
        assert_eq!(parse_color("#ff00000"), None);
        assert_eq!(parse_color("#"), None);
        assert_eq!(parse_color("#🌍"), None);

        // Named colors
        assert_eq!(parse_color("red"), Some(Color::Rgba(255, 0, 0, 255)));
        assert_eq!(
            parse_color("ReBeCcApUrPlE"),
            Some(Color::Rgba(102, 51, 153, 255))
        );
        assert_eq!(parse_color("transparent"), Some(Color::Rgba(0, 0, 0, 0)));

        // System colors
        assert_eq!(parse_color("Canvas"), Some(Color::Rgba(255, 255, 255, 255)));
        assert_eq!(parse_color("canvastext"), Some(Color::Rgba(0, 0, 0, 255)));

        // Invalid inputs
        assert_eq!(parse_color(""), None);
        assert_eq!(parse_color("   "), None);
        assert_eq!(parse_color("not-a-color"), None);
    }

    #[test]
    fn test_parse_color_functional() {
        // RGB & RGBA
        assert_eq!(
            parse_color("rgb(255, 0, 0)"),
            Some(Color::Rgba(255, 0, 0, 255))
        );
        assert_eq!(
            parse_color("rgba(0 255 0 / 0.5)"),
            Some(Color::Rgba(0, 255, 0, 128))
        );
        assert_eq!(
            parse_color("rgb(100% 100% 100%)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(
            parse_color("rgba(0, 0, 255, 20%)"),
            Some(Color::Rgba(0, 0, 255, 51))
        );

        // HSL & HSLA
        assert_eq!(
            parse_color("hsl(0, 100%, 50%)"),
            Some(Color::Rgba(255, 0, 0, 255))
        );
        assert_eq!(
            parse_color("hsl(120deg 100% 50% / 50%)"),
            Some(Color::Rgba(0, 255, 0, 128))
        );
        assert_eq!(
            parse_color("hsla(240, 100%, 50%, 0.2)"),
            Some(Color::Rgba(0, 0, 255, 51))
        );

        // HWB
        assert_eq!(
            parse_color("hwb(0, 0%, 0%)"),
            Some(Color::Rgba(255, 0, 0, 255))
        );
        assert_eq!(
            parse_color("hwb(120deg 0% 0% / 0.5)"),
            Some(Color::Rgba(0, 255, 0, 128))
        );

        // LAB
        assert_eq!(
            parse_color("lab(100% 0 0)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(parse_color("lab(0, 0, 0)"), Some(Color::Rgba(0, 0, 0, 255)));

        // LCH
        assert_eq!(
            parse_color("lch(100% 0 0)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(parse_color("lch(0, 0, 0)"), Some(Color::Rgba(0, 0, 0, 255)));

        // OKLAB
        assert_eq!(
            parse_color("oklab(1 0 0)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(parse_color("oklab(0 0 0)"), Some(Color::Rgba(0, 0, 0, 255)));

        // OKLCH
        assert_eq!(
            parse_color("oklch(1 0 0)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(parse_color("oklch(0 0 0)"), Some(Color::Rgba(0, 0, 0, 255)));

        // none keyword handling
        assert_eq!(
            parse_color("rgb(none none none / none)"),
            Some(Color::Rgba(0, 0, 0, 0))
        );
        assert_eq!(
            parse_color("hsl(none none none / none)"),
            Some(Color::Rgba(0, 0, 0, 0))
        );
        assert_eq!(
            parse_color("hwb(none none none / none)"),
            Some(Color::Rgba(255, 0, 0, 0))
        );
        assert_eq!(
            parse_color("lab(none none none / none)"),
            Some(Color::Rgba(0, 0, 0, 0))
        );
        assert_eq!(
            parse_color("oklab(none none none / none)"),
            Some(Color::Rgba(0, 0, 0, 0))
        );

        // color() predefined color space parsing
        assert_eq!(
            parse_color("color(srgb 1 1 1)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(
            parse_color("color(srgb 0.5 0 0 / 0.5)"),
            Some(Color::Rgba(128, 0, 0, 128))
        );
        assert_eq!(
            parse_color("color(srgb-linear 1 0 0)"),
            Some(Color::Rgba(255, 0, 0, 255))
        );
        assert_eq!(
            parse_color("color(display-p3 1 1 1)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(
            parse_color("color(xyz-d50 0 0 0)"),
            Some(Color::Rgba(0, 0, 0, 255))
        );
        assert_eq!(
            parse_color("color(xyz-d65 1 1 1)"),
            Some(Color::Rgba(255, 249, 244, 255))
        );
        assert_eq!(
            parse_color("color(xyz-d65 0.95047 1.0 1.08883)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        // a98-rgb, prophoto-rgb, and rec2020 predefined color spaces
        assert_eq!(
            parse_color("color(a98-rgb 1 1 1)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(
            parse_color("color(a98-rgb 0 0 0)"),
            Some(Color::Rgba(0, 0, 0, 255))
        );
        assert_eq!(
            parse_color("color(prophoto-rgb 1 1 1)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(
            parse_color("color(prophoto-rgb 0 0 0)"),
            Some(Color::Rgba(0, 0, 0, 255))
        );
        assert_eq!(
            parse_color("color(rec2020 1 1 1)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(
            parse_color("color(rec2020 0 0 0)"),
            Some(Color::Rgba(0, 0, 0, 255))
        );
        assert_eq!(
            parse_color("color(srgb none none none / none)"),
            Some(Color::Rgba(0, 0, 0, 0))
        );
        // commas/slashes inside color()
        assert_eq!(
            parse_color("color(srgb, 0.5, 0, 0 / 0.5)"),
            Some(Color::Rgba(128, 0, 0, 128))
        );
    }

    #[test]
    fn test_mix_colors() {
        let red = Color::Rgba(255, 0, 0, 255);
        let blue = Color::Rgba(0, 0, 255, 255);

        // sRGB mix at 50%
        let mixed_srgb = mix_colors(red.clone(), blue.clone(), 0.5, "srgb", None).unwrap();
        assert_eq!(mixed_srgb, Color::Rgba(128, 0, 128, 255));

        // sRGB-linear mix
        let mixed_lin = mix_colors(red.clone(), blue.clone(), 0.5, "srgb-linear", None).unwrap();
        assert!(mixed_lin != mixed_srgb);

        // Polar hue interpolation (shorter)
        let mixed_hsl = mix_colors(red.clone(), blue.clone(), 0.5, "hsl", Some("shorter")).unwrap();
        let (h, _, _, _) = color_to_hsl(mixed_hsl);
        assert!((h - 300.0).abs() < 1.0 || (h - 60.0).abs() < 1.0);

        // Longer arc
        let mixed_hsl_longer =
            mix_colors(red.clone(), blue.clone(), 0.5, "hsl", Some("longer")).unwrap();
        let (h_longer, _, _, _) = color_to_hsl(mixed_hsl_longer);
        assert!((h_longer - 120.0).abs() < 1.0);
    }

    #[test]
    fn test_serialize_color() {
        // Opaque color
        assert_eq!(
            serialize_color(Color::Rgba(255, 0, 0, 255)),
            "rgb(255, 0, 0)"
        );
        assert_eq!(
            serialize_color(Color::Rgba(0, 255, 128, 255)),
            "rgb(0, 255, 128)"
        );

        // Transparent color
        assert_eq!(serialize_color(Color::Rgba(0, 0, 0, 0)), "rgba(0, 0, 0, 0)");
        assert_eq!(
            serialize_color(Color::Rgba(255, 100, 50, 0)),
            "rgba(255, 100, 50, 0)"
        );

        // Semi-transparent colors with fractional alphas
        assert_eq!(
            serialize_color(Color::Rgba(0, 0, 255, 128)),
            "rgba(0, 0, 255, 0.50196)"
        );
        assert_eq!(
            serialize_color(Color::Rgba(100, 150, 200, 26)),
            "rgba(100, 150, 200, 0.10196)"
        );
        assert_eq!(
            serialize_color(Color::Rgba(255, 255, 255, 51)),
            "rgba(255, 255, 255, 0.2)"
        );
    }

    #[test]
    fn test_relative_color_syntax() {
        // rgb / rgba relative syntax
        assert_eq!(
            parse_color("rgb(from red r g b)"),
            Some(Color::Rgba(255, 0, 0, 255))
        );
        assert_eq!(
            parse_color("rgba(from #0000ff r g b / 0.5)"),
            Some(Color::Rgba(0, 0, 255, 128))
        );
        assert_eq!(
            parse_color("rgb(FROM red R G B / ALPHA)"),
            Some(Color::Rgba(255, 0, 0, 255))
        );

        // hsl / hsla relative syntax
        assert_eq!(
            parse_color("hsl(from hsl(120, 100%, 50%) h s l)"),
            Some(Color::Rgba(0, 255, 0, 255))
        );
        assert_eq!(
            parse_color("hsla(from rgb(255, 0, 0) h s l / alpha)"),
            Some(Color::Rgba(255, 0, 0, 255))
        );

        // hwb relative syntax
        assert_eq!(
            parse_color("hwb(from red h w b)"),
            Some(Color::Rgba(255, 0, 0, 255))
        );

        // lab / lch relative syntax
        assert_eq!(
            parse_color("lab(from white l a b)"),
            Some(Color::Rgba(255, 255, 255, 255))
        );
        assert_eq!(
            parse_color("lch(from black l c h)"),
            Some(Color::Rgba(0, 0, 0, 255))
        );

        // oklab / oklch relative syntax
        assert_eq!(
            parse_color("oklab(from red l a b)"),
            Some(Color::Rgba(255, 0, 0, 255))
        );
        assert_eq!(
            parse_color("oklch(from blue l c h)"),
            Some(Color::Rgba(0, 0, 255, 255))
        );
    }
}
