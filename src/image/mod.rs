use crate::raster::Canvas;
use std::io::Cursor;

/// A decoded image with RGBA8 pixels.
/// spec: S-19
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Encodes a Canvas into a PNG byte stream with compression enabled.
/// Canvas pixels are 0xAARRGGBB; converted to RGBA8 for PNG.
/// spec: S-19
pub fn encode_png(canvas: &Canvas) -> Vec<u8> {
    let mut data = Vec::with_capacity((canvas.width as usize) * (canvas.height as usize) * 4);
    for &pixel in &canvas.pixels {
        let a = ((pixel >> 24) & 0xFF) as u8;
        let r = ((pixel >> 16) & 0xFF) as u8;
        let g = ((pixel >> 8) & 0xFF) as u8;
        let b = (pixel & 0xFF) as u8;
        data.push(r);
        data.push(g);
        data.push(b);
        data.push(a);
    }

    let mut buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(Cursor::new(&mut buf), canvas.width, canvas.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        // spec: S-74 Compression is now allowed, so NoCompression is no longer needed.

        match encoder.write_header() {
            Ok(mut writer) => {
                let _ = writer.write_image_data(&data);
            }
            Err(_) => return Vec::new(),
        }
    }
    buf
}

/// Decodes a PNG byte stream into a DecodedImage.
/// PNG pixels are converted to RGBA8.
/// spec: S-49
pub fn decode_png(bytes: &[u8]) -> Option<DecodedImage> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);

    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let bytes = &buf[..info.buffer_size()];

    if info.bit_depth != png::BitDepth::Eight {
        return None;
    }

    let expected_len = match info.color_type {
        png::ColorType::Grayscale => info.width as usize * info.height as usize,
        png::ColorType::GrayscaleAlpha => info.width as usize * info.height as usize * 2,
        png::ColorType::Rgb => info.width as usize * info.height as usize * 3,
        png::ColorType::Rgba => info.width as usize * info.height as usize * 4,
        _ => return None,
    };
    if bytes.len() != expected_len {
        return None;
    }

    let mut rgba = Vec::with_capacity((info.width as usize) * (info.height as usize) * 4);
    match info.color_type {
        png::ColorType::Grayscale => {
            for &g in bytes {
                rgba.push(g);
                rgba.push(g);
                rgba.push(g);
                rgba.push(255);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for chunk in bytes.chunks_exact(2) {
                let g = chunk[0];
                let a = chunk[1];
                rgba.push(g);
                rgba.push(g);
                rgba.push(g);
                rgba.push(a);
            }
        }
        png::ColorType::Rgb => {
            for chunk in bytes.chunks_exact(3) {
                rgba.push(chunk[0]);
                rgba.push(chunk[1]);
                rgba.push(chunk[2]);
                rgba.push(255);
            }
        }
        png::ColorType::Rgba => {
            rgba.extend_from_slice(bytes);
        }
        _ => return None,
    }

    Some(DecodedImage {
        width: info.width,
        height: info.height,
        rgba,
    })
}

/// Decodes a JPEG byte stream into a DecodedImage.
/// JPEG pixels are converted to RGBA8.
pub fn decode_jpeg(bytes: &[u8]) -> Option<DecodedImage> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    decoder.read_info().ok()?;
    let info = decoder.info()?;
    let decoded_bytes = decoder.decode().ok()?;

    let expected_len = match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => info.width as usize * info.height as usize,
        jpeg_decoder::PixelFormat::RGB24 => info.width as usize * info.height as usize * 3,
        jpeg_decoder::PixelFormat::CMYK32 => info.width as usize * info.height as usize * 4,
        _ => return None,
    };

    if decoded_bytes.len() != expected_len {
        return None;
    }

    let mut rgba = Vec::with_capacity((info.width as usize) * (info.height as usize) * 4);
    match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => {
            for &g in &decoded_bytes {
                rgba.push(g);
                rgba.push(g);
                rgba.push(g);
                rgba.push(255);
            }
        }
        jpeg_decoder::PixelFormat::RGB24 => {
            for chunk in decoded_bytes.chunks_exact(3) {
                rgba.push(chunk[0]);
                rgba.push(chunk[1]);
                rgba.push(chunk[2]);
                rgba.push(255);
            }
        }
        jpeg_decoder::PixelFormat::CMYK32 => {
            for chunk in decoded_bytes.chunks_exact(4) {
                let c = chunk[0] as u32;
                let m = chunk[1] as u32;
                let y = chunk[2] as u32;
                let k = chunk[3] as u32;

                let r = ((255 - c) * (255 - k) / 255) as u8;
                let g = ((255 - m) * (255 - k) / 255) as u8;
                let b = ((255 - y) * (255 - k) / 255) as u8;

                rgba.push(r);
                rgba.push(g);
                rgba.push(b);
                rgba.push(255);
            }
        }
        _ => return None,
    }

    Some(DecodedImage {
        width: info.width as u32,
        height: info.height as u32,
        rgba,
    })
}

/// Decodes a GIF byte stream into a DecodedImage.
/// GIF pixels are converted to RGBA8.
/// Only the first frame is decoded.
/// spec: S-261
pub fn decode_gif(bytes: &[u8]) -> Option<DecodedImage> {
    // TODO(spec): only the first frame is decoded (animation/disposal/sub-frame offsets are not yet composited) and that the logical screen size may exceed the first frame's size.
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = options.read_info(Cursor::new(bytes)).ok()?;
    let frame = decoder.read_next_frame().ok()??; // returns Option<&Frame>; `??` flattens Result<Option<_>>
    let width = frame.width as u32;
    let height = frame.height as u32;
    let rgba = frame.buffer.to_vec();

    if rgba.len() != (width as usize) * (height as usize) * 4 {
        return None;
    }

    Some(DecodedImage {
        width,
        height,
        rgba,
    })
}

/// Decodes a BMP byte stream into a DecodedImage.
/// Supports uncompressed 24-bit (BGR) and 32-bit (BGRA) BITMAPINFOHEADER cases.
pub fn decode_bmp(bytes: &[u8]) -> Option<DecodedImage> {
    if bytes.len() < 54 {
        return None;
    }
    if !bytes.starts_with(b"BM") {
        return None;
    }

    let pixel_offset = u32::from_le_bytes(bytes.get(10..14)?.try_into().ok()?) as usize;
    let dib_size = u32::from_le_bytes(bytes.get(14..18)?.try_into().ok()?) as usize;

    if dib_size < 40 {
        return None;
    }

    let min_file_size = 14_usize.checked_add(dib_size)?;
    if bytes.len() < min_file_size {
        return None;
    }
    if pixel_offset < min_file_size {
        return None;
    }

    let width = i32::from_le_bytes(bytes.get(18..22)?.try_into().ok()?);
    let height = i32::from_le_bytes(bytes.get(22..26)?.try_into().ok()?);

    if width <= 0 || height == 0 {
        return None;
    }

    let planes = u16::from_le_bytes(bytes.get(26..28)?.try_into().ok()?);
    if planes != 1 {
        return None;
    }

    let bpp = u16::from_le_bytes(bytes.get(28..30)?.try_into().ok()?);
    if bpp != 24 && bpp != 32 {
        return None;
    }

    let compression = u32::from_le_bytes(bytes.get(30..34)?.try_into().ok()?);
    if compression != 0 {
        return None;
    }

    let width_u32 = width as u32;
    let height_abs: u32 = height.checked_abs()?.try_into().ok()?;

    let row_bytes = width_u32.checked_mul(bpp as u32 / 8)?;
    let stride = row_bytes.checked_add(3)? / 4 * 4;

    let total_pixel_bytes = stride.checked_mul(height_abs)? as usize;
    let end_offset = pixel_offset.checked_add(total_pixel_bytes)?;
    if end_offset > bytes.len() {
        return None;
    }

    let mut rgba = vec![0u8; (width_u32 as usize) * (height_abs as usize) * 4];

    for file_row_idx in 0..height_abs {
        let target_row_idx = if height > 0 {
            height_abs - 1 - file_row_idx
        } else {
            file_row_idx
        };

        let file_row_start =
            pixel_offset.checked_add((file_row_idx as usize).checked_mul(stride as usize)?)?;
        let target_row_start = (target_row_idx as usize)
            .checked_mul(width_u32 as usize)?
            .checked_mul(4)?;

        if bpp == 24 {
            for col in 0..width_u32 as usize {
                let src_pixel_offset = file_row_start.checked_add(col.checked_mul(3)?)?;
                let b = *bytes.get(src_pixel_offset)?;
                let g = *bytes.get(src_pixel_offset.checked_add(1)?)?;
                let r = *bytes.get(src_pixel_offset.checked_add(2)?)?;

                let dst_pixel_offset = target_row_start.checked_add(col.checked_mul(4)?)?;
                let dst_slice = rgba.get_mut(dst_pixel_offset..dst_pixel_offset + 4)?;
                dst_slice[0] = r;
                dst_slice[1] = g;
                dst_slice[2] = b;
                dst_slice[3] = 255;
            }
        } else if bpp == 32 {
            for col in 0..width_u32 as usize {
                let src_pixel_offset = file_row_start.checked_add(col.checked_mul(4)?)?;
                let b = *bytes.get(src_pixel_offset)?;
                let g = *bytes.get(src_pixel_offset.checked_add(1)?)?;
                let r = *bytes.get(src_pixel_offset.checked_add(2)?)?;
                let a = *bytes.get(src_pixel_offset.checked_add(3)?)?;

                let dst_pixel_offset = target_row_start.checked_add(col.checked_mul(4)?)?;
                let dst_slice = rgba.get_mut(dst_pixel_offset..dst_pixel_offset + 4)?;
                dst_slice[0] = r;
                dst_slice[1] = g;
                dst_slice[2] = b;
                dst_slice[3] = a;
            }
        }
    }

    Some(DecodedImage {
        width: width_u32,
        height: height_abs,
        rgba,
    })
}

/// Decodes a WebP byte stream into a DecodedImage.
pub fn decode_webp(bytes: &[u8]) -> Option<DecodedImage> {
    let mut decoder = image_webp::WebPDecoder::new(Cursor::new(bytes)).ok()?;
    let (width, height) = decoder.dimensions();
    let has_alpha = decoder.has_alpha();
    let size = decoder.output_buffer_size()?;
    let mut buf = vec![0; size];
    decoder.read_image(&mut buf).ok()?;

    let mut rgba = Vec::with_capacity((width as usize) * (height as usize) * 4);
    if has_alpha {
        rgba = buf;
    } else {
        for chunk in buf.chunks_exact(3) {
            rgba.push(chunk[0]);
            rgba.push(chunk[1]);
            rgba.push(chunk[2]);
            rgba.push(255);
        }
    }

    Some(DecodedImage {
        width,
        height,
        rgba,
    })
}

struct SvgElement {
    name: String,
    attrs: std::collections::HashMap<String, String>,
}

fn parse_attributes(mut s: &str) -> std::collections::HashMap<String, String> {
    let mut attrs = std::collections::HashMap::new();
    loop {
        s = s.trim_start();
        if s.is_empty() || s.starts_with('>') || s.starts_with("/>") {
            break;
        }
        let key_end = s
            .find(|c: char| c == '=' || c.is_whitespace() || c == '>' || c == '/')
            .unwrap_or(s.len());
        if key_end == 0 {
            break;
        }
        let key = s[..key_end].to_string();
        s = &s[key_end..];
        s = s.trim_start();
        if s.starts_with('=') {
            s = &s[1..];
            s = s.trim_start();
            if s.starts_with('"') {
                s = &s[1..];
                if let Some(val_end) = s.find('"') {
                    let val = s[..val_end].to_string();
                    attrs.insert(key, val);
                    s = &s[val_end + 1..];
                } else {
                    break;
                }
            } else if s.starts_with('\'') {
                s = &s[1..];
                if let Some(val_end) = s.find('\'') {
                    let val = s[..val_end].to_string();
                    attrs.insert(key, val);
                    s = &s[val_end + 1..];
                } else {
                    break;
                }
            } else {
                let val_end = s
                    .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                    .unwrap_or(s.len());
                let val = s[..val_end].to_string();
                attrs.insert(key, val);
                s = &s[val_end..];
            }
        } else {
            attrs.insert(key, String::new());
        }
    }
    attrs
}

fn extract_elements(mut s: &str) -> Vec<SvgElement> {
    let mut elements = Vec::new();
    loop {
        s = s.trim_start();
        if s.is_empty() {
            break;
        }

        if s.starts_with("<!--") {
            if let Some(end_idx) = s.find("-->") {
                s = &s[end_idx + 3..];
                continue;
            } else {
                break;
            }
        }

        if s.starts_with("<?") {
            if let Some(end_idx) = s.find("?>") {
                s = &s[end_idx + 2..];
                continue;
            } else {
                break;
            }
        }

        if s.starts_with("<!") {
            if let Some(end_idx) = s.find('>') {
                s = &s[end_idx + 1..];
                continue;
            } else {
                break;
            }
        }

        if s.starts_with('<') {
            s = &s[1..];
            if s.starts_with('/') {
                s = &s[1..];
                if let Some(end_idx) = s.find('>') {
                    let name = s[..end_idx].trim().to_ascii_lowercase();
                    elements.push(SvgElement {
                        name: format!("/{}", name),
                        attrs: std::collections::HashMap::new(),
                    });
                    s = &s[end_idx + 1..];
                } else {
                    break;
                }
            } else {
                let name_end = s
                    .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
                    .unwrap_or(s.len());
                let name = s[..name_end].to_ascii_lowercase();
                s = &s[name_end..];

                let attrs = parse_attributes(s);
                if let Some(tag_end) = s.find('>') {
                    let tag_content = &s[..tag_end];
                    let is_self_closing = tag_content.trim_end().ends_with('/');
                    elements.push(SvgElement {
                        name: name.clone(),
                        attrs,
                    });
                    if is_self_closing {
                        elements.push(SvgElement {
                            name: format!("/{}", name),
                            attrs: std::collections::HashMap::new(),
                        });
                    }
                    s = &s[tag_end + 1..];
                } else {
                    break;
                }
            }
        } else {
            if let Some(next_lt) = s.find('<') {
                s = &s[next_lt..];
            } else {
                break;
            }
        }
    }
    elements
}

fn parse_length(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if s.ends_with('%') {
        return None;
    }
    let s_clean = s.strip_suffix("px").unwrap_or(s);
    s_clean.trim().parse::<f64>().ok()
}

fn parse_viewbox(s: &str) -> Option<(f64, f64, f64, f64)> {
    let mut parts = Vec::new();
    for part in s.split(|c: char| c == ',' || c.is_whitespace()) {
        let part = part.trim();
        if !part.is_empty() {
            parts.push(part.parse::<f64>().ok()?);
        }
    }
    if parts.len() == 4 {
        Some((parts[0], parts[1], parts[2], parts[3]))
    } else {
        None
    }
}

fn parse_color(s: &str) -> Option<[u8; 4]> {
    let s = s.trim().to_ascii_lowercase();
    if s == "none" {
        return Some([0, 0, 0, 0]);
    }
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            return Some([r * 17, g * 17, b * 17, 255]);
        } else if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some([r, g, b, 255]);
        }
    }
    match s.as_str() {
        "black" => Some([0, 0, 0, 255]),
        "white" => Some([255, 255, 255, 255]),
        "red" => Some([255, 0, 0, 255]),
        "green" => Some([0, 128, 0, 255]),
        "lime" => Some([0, 255, 0, 255]),
        "blue" => Some([0, 0, 255, 255]),
        "yellow" => Some([255, 255, 0, 255]),
        "cyan" => Some([0, 255, 255, 255]),
        "magenta" => Some([255, 0, 255, 255]),
        "gray" | "grey" => Some([128, 128, 128, 255]),
        "silver" => Some([192, 192, 192, 255]),
        "orange" => Some([255, 165, 0, 255]),
        "purple" => Some([128, 0, 128, 255]),
        "maroon" => Some([128, 0, 0, 255]),
        "navy" => Some([0, 0, 128, 255]),
        "teal" => Some([0, 128, 128, 255]),
        "olive" => Some([128, 128, 0, 255]),
        _ => None,
    }
}

fn blend_pixel(dst: &mut [u8], src: [u8; 4]) {
    let src_a = src[3] as f64 / 255.0;
    if src_a <= 0.0 {
        return;
    }
    if src_a >= 1.0 {
        dst[0] = src[0];
        dst[1] = src[1];
        dst[2] = src[2];
        dst[3] = src[3];
        return;
    }
    let dst_a = dst[3] as f64 / 255.0;
    let out_a = src_a + dst_a * (1.0 - src_a);
    if out_a > 0.0 {
        let out_r =
            ((src[0] as f64 * src_a + dst[0] as f64 * dst_a * (1.0 - src_a)) / out_a).round() as u8;
        let out_g =
            ((src[1] as f64 * src_a + dst[1] as f64 * dst_a * (1.0 - src_a)) / out_a).round() as u8;
        let out_b =
            ((src[2] as f64 * src_a + dst[2] as f64 * dst_a * (1.0 - src_a)) / out_a).round() as u8;
        dst[0] = out_r;
        dst[1] = out_g;
        dst[2] = out_b;
        dst[3] = (out_a * 255.0).round() as u8;
    }
}

fn skip_metadata(mut s: &str) -> &str {
    loop {
        s = s.trim_start();
        if s.starts_with("<?") {
            let end_idx = s.find("?>");
            if let Some(end) = end_idx {
                s = &s[end + 2..];
                continue;
            }
        }
        if s.starts_with("<!--") {
            let end_idx = s.find("-->");
            if let Some(end) = end_idx {
                s = &s[end + 3..];
                continue;
            }
        }
        if s.starts_with("<!") {
            let end_idx = s.find('>');
            if let Some(end) = end_idx {
                s = &s[end + 1..];
                continue;
            }
        }
        break;
    }
    s
}

fn is_svg_sniff(bytes: &[u8]) -> Option<bool> {
    let mut data = bytes;
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        data = &data[3..];
    }
    let s = std::str::from_utf8(data).ok()?;
    let s = skip_metadata(s);
    Some(s.starts_with("<svg"))
}

/// Decodes an SVG byte stream into a DecodedImage.
pub fn decode_svg(bytes: &[u8]) -> Option<DecodedImage> {
    let mut data = bytes;
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        data = &data[3..];
    }
    let s = std::str::from_utf8(data).ok()?;
    let elements = extract_elements(s);

    let svg_elem = elements.iter().find(|e| e.name == "svg")?;

    let explicit_w = svg_elem.attrs.get("width").and_then(|w| parse_length(w));
    let explicit_h = svg_elem.attrs.get("height").and_then(|h| parse_length(h));

    let vb = svg_elem.attrs.get("viewBox").and_then(|s| parse_viewbox(s));

    let final_w = if let Some(w) = explicit_w {
        w
    } else if let Some((_, _, vb_w, _)) = vb {
        vb_w
    } else {
        16.0
    };

    let final_h = if let Some(h) = explicit_h {
        h
    } else if let Some((_, _, _, vb_h)) = vb {
        vb_h
    } else {
        16.0
    };

    if final_w <= 0.0 || final_h <= 0.0 {
        return None;
    }

    let out_w = (final_w.round() as i32).clamp(1, 1024) as u32;
    let out_h = (final_h.round() as i32).clamp(1, 1024) as u32;

    let (vb_min_x, vb_min_y, vb_w, vb_h) = if let Some((vx, vy, vw, vh)) = vb {
        if vw > 0.0 && vh > 0.0 {
            (vx, vy, vw, vh)
        } else {
            (0.0, 0.0, final_w, final_h)
        }
    } else {
        (0.0, 0.0, final_w, final_h)
    };

    let scale_x = out_w as f64 / vb_w;
    let scale_y = out_h as f64 / vb_h;

    let transform_x = |x: f64| -> f64 { (x - vb_min_x) * scale_x };
    let transform_y = |y: f64| -> f64 { (y - vb_min_y) * scale_y };

    let mut rgba = vec![0u8; (out_w as usize) * (out_h as usize) * 4];

    let mut in_defs = false;
    let mut depth = 0;

    for elem in &elements {
        if elem.name == "defs" {
            in_defs = true;
            depth += 1;
            continue;
        } else if elem.name == "/defs" {
            depth -= 1;
            if depth <= 0 {
                in_defs = false;
                depth = 0;
            }
            continue;
        } else if elem.name.starts_with('/') {
            continue;
        }

        if in_defs {
            continue;
        }

        match elem.name.as_str() {
            "rect" => {
                let x = elem
                    .attrs
                    .get("x")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let y = elem
                    .attrs
                    .get("y")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let w = match elem.attrs.get("width").and_then(|v| v.parse::<f64>().ok()) {
                    Some(v) => v,
                    None => continue,
                };
                let h = match elem.attrs.get("height").and_then(|v| v.parse::<f64>().ok()) {
                    Some(v) => v,
                    None => continue,
                };
                if w <= 0.0 || h <= 0.0 {
                    continue;
                }

                let fill_str = elem
                    .attrs
                    .get("fill")
                    .map(|s| s.as_str())
                    .unwrap_or("black");
                let Some(mut fill_color) = parse_color(fill_str) else {
                    continue;
                };

                let opacity = elem
                    .attrs
                    .get("fill-opacity")
                    .and_then(|o| o.parse::<f64>().ok())
                    .unwrap_or(1.0);
                fill_color[3] = (fill_color[3] as f64 * opacity).round().clamp(0.0, 255.0) as u8;

                let px_x1 = transform_x(x);
                let px_y1 = transform_y(y);
                let px_x2 = transform_x(x + w);
                let px_y2 = transform_y(y + h);

                let x_start = px_x1.min(px_x2);
                let x_end = px_x1.max(px_x2);
                let y_start = px_y1.min(px_y2);
                let y_end = px_y1.max(px_y2);

                let min_px = (x_start.floor() as i32).max(0).min(out_w as i32) as u32;
                let max_px = (x_end.ceil() as i32).max(0).min(out_w as i32) as u32;
                let min_py = (y_start.floor() as i32).max(0).min(out_h as i32) as u32;
                let max_py = (y_end.ceil() as i32).max(0).min(out_h as i32) as u32;

                for py in min_py..max_py {
                    let cy = py as f64 + 0.5;
                    if cy >= y_start && cy <= y_end {
                        for px in min_px..max_px {
                            let cx = px as f64 + 0.5;
                            if cx >= x_start && cx <= x_end {
                                let idx = ((py * out_w + px) * 4) as usize;
                                if let Some(slice) = rgba.get_mut(idx..idx + 4) {
                                    blend_pixel(slice, fill_color);
                                }
                            }
                        }
                    }
                }
            }
            "circle" => {
                let cx_val = elem
                    .attrs
                    .get("cx")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let cy_val = elem
                    .attrs
                    .get("cy")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let r_val = match elem.attrs.get("r").and_then(|v| v.parse::<f64>().ok()) {
                    Some(v) => v,
                    None => continue,
                };
                if r_val <= 0.0 {
                    continue;
                }

                let rx = r_val * scale_x;
                let ry = r_val * scale_y;
                if rx <= 0.0 || ry <= 0.0 {
                    continue;
                }

                let fill_str = elem
                    .attrs
                    .get("fill")
                    .map(|s| s.as_str())
                    .unwrap_or("black");
                let Some(mut fill_color) = parse_color(fill_str) else {
                    continue;
                };

                let opacity = elem
                    .attrs
                    .get("fill-opacity")
                    .and_then(|o| o.parse::<f64>().ok())
                    .unwrap_or(1.0);
                fill_color[3] = (fill_color[3] as f64 * opacity).round().clamp(0.0, 255.0) as u8;

                let px_cx = transform_x(cx_val);
                let px_cy = transform_y(cy_val);

                let x_start = px_cx - rx;
                let x_end = px_cx + rx;
                let y_start = px_cy - ry;
                let y_end = px_cy + ry;

                let min_px = (x_start.floor() as i32).max(0).min(out_w as i32) as u32;
                let max_px = (x_end.ceil() as i32).max(0).min(out_w as i32) as u32;
                let min_py = (y_start.floor() as i32).max(0).min(out_h as i32) as u32;
                let max_py = (y_end.ceil() as i32).max(0).min(out_h as i32) as u32;

                for py in min_py..max_py {
                    let cy = py as f64 + 0.5;
                    let dy = (cy - px_cy) / ry;
                    let dy_sq = dy * dy;
                    if dy_sq <= 1.0 {
                        for px in min_px..max_px {
                            let cx = px as f64 + 0.5;
                            let dx = (cx - px_cx) / rx;
                            if dx * dx + dy_sq <= 1.0 {
                                let idx = ((py * out_w + px) * 4) as usize;
                                if let Some(slice) = rgba.get_mut(idx..idx + 4) {
                                    blend_pixel(slice, fill_color);
                                }
                            }
                        }
                    }
                }
            }
            "ellipse" => {
                let cx_val = elem
                    .attrs
                    .get("cx")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let cy_val = elem
                    .attrs
                    .get("cy")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let rx_val = match elem.attrs.get("rx").and_then(|v| v.parse::<f64>().ok()) {
                    Some(v) => v,
                    None => continue,
                };
                let ry_val = match elem.attrs.get("ry").and_then(|v| v.parse::<f64>().ok()) {
                    Some(v) => v,
                    None => continue,
                };
                if rx_val <= 0.0 || ry_val <= 0.0 {
                    continue;
                }

                let rx = rx_val * scale_x;
                let ry = ry_val * scale_y;
                if rx <= 0.0 || ry <= 0.0 {
                    continue;
                }

                let fill_str = elem
                    .attrs
                    .get("fill")
                    .map(|s| s.as_str())
                    .unwrap_or("black");
                let Some(mut fill_color) = parse_color(fill_str) else {
                    continue;
                };

                let opacity = elem
                    .attrs
                    .get("fill-opacity")
                    .and_then(|o| o.parse::<f64>().ok())
                    .unwrap_or(1.0);
                fill_color[3] = (fill_color[3] as f64 * opacity).round().clamp(0.0, 255.0) as u8;

                let px_cx = transform_x(cx_val);
                let px_cy = transform_y(cy_val);

                let x_start = px_cx - rx;
                let x_end = px_cx + rx;
                let y_start = px_cy - ry;
                let y_end = px_cy + ry;

                let min_px = (x_start.floor() as i32).max(0).min(out_w as i32) as u32;
                let max_px = (x_end.ceil() as i32).max(0).min(out_w as i32) as u32;
                let min_py = (y_start.floor() as i32).max(0).min(out_h as i32) as u32;
                let max_py = (y_end.ceil() as i32).max(0).min(out_h as i32) as u32;

                for py in min_py..max_py {
                    let cy = py as f64 + 0.5;
                    let dy = (cy - px_cy) / ry;
                    let dy_sq = dy * dy;
                    if dy_sq <= 1.0 {
                        for px in min_px..max_px {
                            let cx = px as f64 + 0.5;
                            let dx = (cx - px_cx) / rx;
                            if dx * dx + dy_sq <= 1.0 {
                                let idx = ((py * out_w + px) * 4) as usize;
                                if let Some(slice) = rgba.get_mut(idx..idx + 4) {
                                    blend_pixel(slice, fill_color);
                                }
                            }
                        }
                    }
                }
            }
            "path" => {
                // TODO(spec): SVG <path> bezier rasterization is a separate wave
            }
            _ => {}
        }
    }

    Some(DecodedImage {
        width: out_w,
        height: out_h,
        rgba,
    })
}

/// Decodes an image byte stream (PNG, JPEG, GIF, BMP, WebP, or SVG) into a DecodedImage by sniffing the format.
pub fn decode_image(bytes: &[u8]) -> Option<DecodedImage> {
    if bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]) {
        decode_png(bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        decode_jpeg(bytes)
    } else if bytes.starts_with(b"GIF8") {
        decode_gif(bytes)
    } else if bytes.starts_with(b"BM") {
        decode_bmp(bytes)
    } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        decode_webp(bytes)
    } else if let Some(true) = is_svg_sniff(bytes) {
        decode_svg(bytes)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JPEG_BASE64_1: &str = "/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////wgALCAABAAEBAREA/8QAFBABAAAAAAAAAAAAAAAAAAAAAP/aAAgBAQABPxA=";
    const JPEG_BASE64_2: &str = "/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/wAALCAABAAEBAREA/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEAAD8AKp//2Q==";
    const GIF_BASE64: &str = "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7";

    #[test]
    fn test_round_trip() {
        let mut canvas = Canvas::new(2, 2);
        // 0xAARRGGBB
        canvas.pixels[0] = 0xFFFF0000; // Opaque Red
        canvas.pixels[1] = 0xFF00FF00; // Opaque Green
        canvas.pixels[2] = 0xFF0000FF; // Opaque Blue
        canvas.pixels[3] = 0x80FFFFFF; // Semi-transparent White

        let png_bytes = encode_png(&canvas);
        assert!(!png_bytes.is_empty());

        let decoded = decode_png(&png_bytes).expect("Should decode successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba.len(), 2 * 2 * 4);

        // RGBA8
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 255]);
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 255]);
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 255]);
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 128]);
    }

    #[test]
    fn test_webp_round_trip() {
        let mut buf = Vec::new();
        let encoder = image_webp::WebPEncoder::new(&mut buf);
        let pixels = vec![
            255, 0, 0, 255, // Red
            0, 255, 0, 255, // Green
            0, 0, 255, 255, // Blue
            255, 255, 255, 128, // Semi-transparent White
        ];
        encoder
            .encode(&pixels, 2, 2, image_webp::ColorType::Rgba8)
            .expect("Should encode successfully");

        let decoded = decode_webp(&buf).expect("Should decode successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba.len(), 2 * 2 * 4);
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 255]);
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 255]);
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 255]);
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 128]);
    }

    #[test]
    fn test_webp_round_trip_rgb() {
        let mut buf = Vec::new();
        let encoder = image_webp::WebPEncoder::new(&mut buf);
        let pixels = vec![
            255, 0, 0, // Red
            0, 255, 0, // Green
            0, 0, 255, // Blue
            255, 255, 255, // White
        ];
        encoder
            .encode(&pixels, 2, 2, image_webp::ColorType::Rgb8)
            .expect("Should encode successfully");

        let decoded = decode_webp(&buf).expect("Should decode successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba.len(), 2 * 2 * 4);
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 255]);
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 255]);
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 255]);
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 255]);
    }

    #[test]
    fn test_decode_garbage() {
        assert!(decode_png(b"not a png").is_none());
        assert!(decode_png(&[]).is_none());
        assert!(decode_jpeg(b"not a jpeg").is_none());
        assert!(decode_jpeg(&[]).is_none());
        assert!(decode_gif(b"not a gif").is_none());
        assert!(decode_gif(&[]).is_none());
        assert!(decode_webp(b"not a webp").is_none());
        assert!(decode_webp(&[]).is_none());
    }

    #[test]
    fn test_truncated_input() {
        let mut canvas = Canvas::new(2, 2);
        canvas.pixels[0] = 0xFFFF0000;
        let png_bytes = encode_png(&canvas);
        assert!(decode_png(&png_bytes[0..png_bytes.len() - 10]).is_none());

        let jpeg_bytes = crate::loader::decode_base64(JPEG_BASE64_2).unwrap();
        assert!(decode_jpeg(&jpeg_bytes[0..jpeg_bytes.len() - 10]).is_none());

        let gif_bytes = crate::loader::decode_base64(GIF_BASE64).unwrap();
        assert!(decode_gif(&gif_bytes[0..gif_bytes.len() / 2]).is_none());

        let mut webp_buf = Vec::new();
        let encoder = image_webp::WebPEncoder::new(&mut webp_buf);
        let pixels = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 128,
        ];
        encoder
            .encode(&pixels, 2, 2, image_webp::ColorType::Rgba8)
            .unwrap();
        assert!(decode_webp(&webp_buf[0..webp_buf.len() - 10]).is_none());
    }

    #[test]
    fn test_decode_jpeg_minimal() {
        let jpeg_bytes_1 = crate::loader::decode_base64(JPEG_BASE64_1).unwrap();
        if let Some(decoded) = decode_jpeg(&jpeg_bytes_1) {
            assert_eq!(decoded.width, 1);
            assert_eq!(decoded.height, 1);
            assert_eq!(decoded.rgba.len(), 4);
            return;
        }

        let jpeg_bytes_2 = crate::loader::decode_base64(JPEG_BASE64_2).unwrap();
        let decoded =
            decode_jpeg(&jpeg_bytes_2).expect("Should decode successfully with second fallback");
        assert_eq!(decoded.width, 1);
        assert_eq!(decoded.height, 1);
        assert_eq!(decoded.rgba.len(), 4);
    }

    #[test]
    fn test_decode_gif_minimal() {
        let gif_bytes = crate::loader::decode_base64(GIF_BASE64).unwrap();
        let decoded = decode_gif(&gif_bytes).expect("Should decode successfully");
        assert_eq!(decoded.width, 1);
        assert_eq!(decoded.height, 1);
        assert_eq!(decoded.rgba.len(), 4);
    }

    #[test]
    fn test_decode_image_sniffing() {
        // Test PNG decoding via decode_image
        let mut canvas = Canvas::new(1, 1);
        canvas.pixels[0] = 0xFFFF0000;
        let png_bytes = encode_png(&canvas);
        let decoded_png = decode_image(&png_bytes).expect("Should sniff and decode PNG");
        assert_eq!(decoded_png.width, 1);
        assert_eq!(decoded_png.height, 1);
        assert_eq!(&decoded_png.rgba[..], &[255, 0, 0, 255]);

        // Test JPEG decoding via decode_image
        let jpeg_bytes = crate::loader::decode_base64(JPEG_BASE64_2).unwrap();
        let decoded_jpeg = decode_image(&jpeg_bytes).expect("Should sniff and decode JPEG");
        assert_eq!(decoded_jpeg.width, 1);
        assert_eq!(decoded_jpeg.height, 1);
        assert_eq!(decoded_jpeg.rgba.len(), 4);

        // Test GIF decoding via decode_image
        let gif_bytes = crate::loader::decode_base64(GIF_BASE64).unwrap();
        let decoded_gif = decode_image(&gif_bytes).expect("Should sniff and decode GIF");
        assert_eq!(decoded_gif.width, 1);
        assert_eq!(decoded_gif.height, 1);
        assert_eq!(decoded_gif.rgba.len(), 4);

        // Test WebP decoding via decode_image
        let mut webp_buf = Vec::new();
        let encoder = image_webp::WebPEncoder::new(&mut webp_buf);
        let pixels = vec![255, 0, 0, 255];
        encoder
            .encode(&pixels, 1, 1, image_webp::ColorType::Rgba8)
            .expect("Should encode successfully");
        let decoded_webp = decode_image(&webp_buf).expect("Should sniff and decode WebP");
        assert_eq!(decoded_webp.width, 1);
        assert_eq!(decoded_webp.height, 1);
        assert_eq!(&decoded_webp.rgba[..], &[255, 0, 0, 255]);

        // Test garbage rejected by decode_image
        assert!(decode_image(b"neither png nor jpeg nor gif nor bmp").is_none());
    }

    #[test]
    fn test_decode_bmp_24_bottom_up() {
        let mut bmp = vec![
            // --- FILE HEADER (14 bytes) ---
            0x42, 0x4D, // Signature "BM"
            70, 0, 0, 0, // File size (70 bytes)
            0, 0, 0, 0, // Reserved
            54, 0, 0, 0, // Pixel data offset (54)
            // --- DIB HEADER (40 bytes) ---
            40, 0, 0, 0, // Header size (40)
            2, 0, 0, 0, // Width (2)
            2, 0, 0, 0, // Height (2)
            1, 0, // Planes (1)
            24, 0, // BPP (24)
            0, 0, 0, 0, // Compression (0 = BI_RGB)
            16, 0, 0, 0, // Image size (16 bytes of pixel data)
            0, 0, 0, 0, // X pixels per meter (0)
            0, 0, 0, 0, // Y pixels per meter (0)
            0, 0, 0, 0, // Total colors (0)
            0, 0, 0, 0, // Important colors (0)
        ];

        bmp.extend_from_slice(&[
            // Row 1 (bottom row)
            255, 0, 0, // col 0 (Blue): B=255, G=0, R=0
            255, 255, 255, // col 1 (White): B=255, G=255, R=255
            0, 0, // Padding to 8 bytes
            // Row 0 (top row)
            0, 0, 255, // col 0 (Red): B=0, G=0, R=255
            0, 255, 0, // col 1 (Green): B=0, G=255, R=0
            0, 0, // Padding to 8 bytes
        ]);

        let decoded = decode_bmp(&bmp).expect("Should decode BMP successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba.len(), 2 * 2 * 4);

        // Top-left pixel (row 0, col 0): Red
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 255]);
        // Top-right pixel (row 0, col 1): Green
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 255]);
        // Bottom-left pixel (row 1, col 0): Blue
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 255]);
        // Bottom-right pixel (row 1, col 1): White
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 255]);

        // Test sniff and decode via decode_image
        let decoded_sniffed = decode_image(&bmp).expect("Should sniff and decode BMP");
        assert_eq!(decoded_sniffed.width, 2);
        assert_eq!(decoded_sniffed.height, 2);
        assert_eq!(decoded_sniffed.rgba, decoded.rgba);
    }

    #[test]
    fn test_decode_bmp_24_top_down() {
        let mut bmp = vec![
            // --- FILE HEADER (14 bytes) ---
            0x42, 0x4D, // Signature "BM"
            70, 0, 0, 0, // File size (70 bytes)
            0, 0, 0, 0, // Reserved
            54, 0, 0, 0, // Pixel data offset (54)
            // --- DIB HEADER (40 bytes) ---
            40, 0, 0, 0, // Header size (40)
            2, 0, 0, 0, // Width (2)
            254, 255, 255, 255, // Height (-2) -> top-down
            1, 0, // Planes (1)
            24, 0, // BPP (24)
            0, 0, 0, 0, // Compression (0 = BI_RGB)
            16, 0, 0, 0, // Image size (16 bytes of pixel data)
            0, 0, 0, 0, // X pixels per meter (0)
            0, 0, 0, 0, // Y pixels per meter (0)
            0, 0, 0, 0, // Total colors (0)
            0, 0, 0, 0, // Important colors (0)
        ];

        bmp.extend_from_slice(&[
            // Row 0 (top row)
            0, 0, 255, // col 0 (Red): B=0, G=0, R=255
            0, 255, 0, // col 1 (Green): B=0, G=255, R=0
            0, 0, // Padding to 8 bytes
            // Row 1 (bottom row)
            255, 0, 0, // col 0 (Blue): B=255, G=0, R=0
            255, 255, 255, // col 1 (White): B=255, G=255, R=255
            0, 0, // Padding to 8 bytes
        ]);

        let decoded = decode_bmp(&bmp).expect("Should decode BMP successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba.len(), 2 * 2 * 4);

        // Top-left pixel (row 0, col 0): Red
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 255]);
        // Top-right pixel (row 0, col 1): Green
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 255]);
        // Bottom-left pixel (row 1, col 0): Blue
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 255]);
        // Bottom-right pixel (row 1, col 1): White
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 255]);
    }

    #[test]
    fn test_decode_bmp_32_bottom_up() {
        let mut bmp32 = vec![
            // --- FILE HEADER (14 bytes) ---
            0x42, 0x4D, // Signature "BM"
            70, 0, 0, 0, // File size (70)
            0, 0, 0, 0, // Reserved
            54, 0, 0, 0, // Pixel data offset (54)
            // --- DIB HEADER (40 bytes) ---
            40, 0, 0, 0, // Header size (40)
            2, 0, 0, 0, // Width (2)
            2, 0, 0, 0, // Height (2)
            1, 0, // Planes (1)
            32, 0, // BPP (32)
            0, 0, 0, 0, // Compression (0 = BI_RGB)
            16, 0, 0, 0, // Image size (16)
            0, 0, 0, 0, // X pixels per meter
            0, 0, 0, 0, // Y pixels per meter
            0, 0, 0, 0, // Total colors
            0, 0, 0, 0, // Important colors
        ];

        bmp32.extend_from_slice(&[
            // Row 1 (bottom row)
            255, 0, 0, 128, // col 0: B=255, G=0, R=0, A=128
            255, 255, 255, 255, // col 1: B=255, G=255, R=255, A=255
            // Row 0 (top row)
            0, 0, 255, 64, // col 0: B=0, G=0, R=255, A=64
            0, 255, 0, 192, // col 1: B=0, G=255, R=0, A=192
        ]);

        let decoded = decode_bmp(&bmp32).expect("Should decode 32-bit BMP successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba.len(), 2 * 2 * 4);

        // Top-left (row 0, col 0): Red with Alpha 64
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 64]);
        // Top-right (row 0, col 1): Green with Alpha 192
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 192]);
        // Bottom-left (row 1, col 0): Blue with Alpha 128
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 128]);
        // Bottom-right (row 1, col 1): White with Alpha 255
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 255]);
    }

    #[test]
    fn test_decode_bmp_malformed() {
        // Truncated bytes
        assert!(decode_bmp(&[]).is_none());
        assert!(decode_bmp(b"BM").is_none());
        assert!(decode_bmp(&[0x42, 0x4D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).is_none());

        // Incorrect magic
        let mut bad_magic = vec![0; 70];
        bad_magic[0] = b'A';
        bad_magic[1] = b'B';
        assert!(decode_bmp(&bad_magic).is_none());

        // Header size too small (< 40)
        let mut small_header = vec![0; 70];
        small_header[0] = b'B';
        small_header[1] = b'M';
        small_header[10] = 54;
        small_header[14] = 12; // OS2 DIB header size (unsupported)
        assert!(decode_bmp(&small_header).is_none());

        // Unsupported BPP (e.g. 8-bit)
        let bad_bpp = vec![
            0x42, 0x4D, 70, 0, 0, 0, 0, 0, 0, 0, 54, 0, 0, 0, 40, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0,
            1, 0, 8, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert!(decode_bmp(&bad_bpp).is_none());

        // Compression BI_BITFIELDS without implemented masks (we require compression == 0)
        let bad_comp = vec![
            0x42, 0x4D, 70, 0, 0, 0, 0, 0, 0, 0, 54, 0, 0, 0, 40, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0,
            1, 0, 32, 0, 3, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert!(decode_bmp(&bad_comp).is_none());

        // Invalid dimensions
        let bad_dim = vec![
            0x42, 0x4D, 70, 0, 0, 0, 0, 0, 0, 0, 54, 0, 0, 0, 40, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0,
            1, 0, 24, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert!(decode_bmp(&bad_dim).is_none());
    }

    #[test]
    fn test_svg_decode_rect() {
        let svg = r##"
            <svg width="4" height="4">
                <rect x="0" y="0" width="4" height="4" fill="#ff0000" />
            </svg>
        "##;
        let decoded = decode_svg(svg.as_bytes()).expect("Should decode successfully");
        assert_eq!(decoded.width, 4);
        assert_eq!(decoded.height, 4);
        assert_eq!(decoded.rgba.len(), 4 * 4 * 4);
        for i in 0..16 {
            assert_eq!(&decoded.rgba[i * 4..(i + 1) * 4], &[255, 0, 0, 255]);
        }
    }

    #[test]
    fn test_svg_decode_viewbox_only() {
        let svg = r#"
            <svg viewBox="0 0 8 8">
                <rect x="0" y="0" width="8" height="8" fill="blue" />
            </svg>
        "#;
        let decoded = decode_svg(svg.as_bytes()).expect("Should decode successfully");
        assert_eq!(decoded.width, 8);
        assert_eq!(decoded.height, 8);
        assert_eq!(decoded.rgba.len(), 8 * 8 * 4);
        assert_eq!(&decoded.rgba[0..4], &[0, 0, 255, 255]);
    }

    #[test]
    fn test_svg_decode_circle() {
        let svg = r#"
            <svg width="3" height="3" viewBox="0 0 3 3">
                <circle cx="1.5" cy="1.5" r="1.0" fill="green" />
            </svg>
        "#;
        let decoded = decode_svg(svg.as_bytes()).expect("Should decode successfully");
        assert_eq!(decoded.width, 3);
        assert_eq!(decoded.height, 3);

        let idx_center = (3 + 1) * 4;
        assert_eq!(&decoded.rgba[idx_center..idx_center + 4], &[0, 128, 0, 255]);

        let idx_corner = 0;
        assert_eq!(&decoded.rgba[idx_corner..idx_corner + 4], &[0, 0, 0, 0]);
    }

    #[test]
    fn test_svg_decode_ellipse() {
        let svg = r#"
            <svg width="5" height="5" viewBox="0 0 5 5">
                <ellipse cx="2.5" cy="2.5" rx="2.0" ry="1.0" fill="yellow" />
            </svg>
        "#;
        let decoded = decode_svg(svg.as_bytes()).expect("Should decode successfully");
        assert_eq!(decoded.width, 5);
        assert_eq!(decoded.height, 5);

        let idx_center = (2 * 5 + 2) * 4;
        assert_eq!(
            &decoded.rgba[idx_center..idx_center + 4],
            &[255, 255, 0, 255]
        );
    }

    #[test]
    fn test_svg_sniff_and_prolog() {
        let svg_with_prolog = r#"<?xml version="1.0" encoding="utf-8"?>
            <!-- SVG comments here -->
            <!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
            <svg width="2" height="2">
                <rect width="2" height="2" fill="white" />
            </svg>
        "#;
        let decoded =
            decode_image(svg_with_prolog.as_bytes()).expect("Should sniff and decode SVG");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(&decoded.rgba[0..4], &[255, 255, 255, 255]);
    }

    #[test]
    fn test_svg_invalid_rejected() {
        let bad_svg1 = r#"<rect width="10" height="10" fill="red" />"#;
        assert!(decode_image(bad_svg1.as_bytes()).is_none());

        let bad_svg2 = "not an xml at all";
        assert!(decode_image(bad_svg2.as_bytes()).is_none());

        assert!(decode_image(&[]).is_none());
    }
}
