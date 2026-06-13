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

/// Decodes an ICO (Windows icon / favicon) byte stream into a DecodedImage.
/// Supports both embedded PNG and uncompressed DIB (BMP) cases.
/// spec: t0507
pub fn decode_ico(bytes: &[u8]) -> Option<DecodedImage> {
    if bytes.len() < 6 {
        return None;
    }
    if bytes[0] != 0 || bytes[1] != 0 || bytes[2] != 1 || bytes[3] != 0 {
        return None;
    }

    let image_count = u16::from_le_bytes(bytes.get(4..6)?.try_into().ok()?);
    if image_count == 0 {
        return None;
    }

    let entries_end = 6_usize.checked_add((image_count as usize).checked_mul(16)?)?;
    if bytes.len() < entries_end {
        return None;
    }

    let mut best_entry: Option<(u32, u32, usize, usize)> = None; // (width, height, bytes_in_res, image_offset)
    for i in 0..image_count {
        let entry_start = 6_usize.checked_add((i as usize).checked_mul(16)?)?;
        let entry_bytes = bytes.get(entry_start..entry_start + 16)?;

        let raw_w = entry_bytes[0];
        let raw_h = entry_bytes[1];
        let w = if raw_w == 0 { 256 } else { raw_w as u32 };
        let h = if raw_h == 0 { 256 } else { raw_h as u32 };

        let bytes_in_res = u32::from_le_bytes(entry_bytes.get(8..12)?.try_into().ok()?) as usize;
        let image_offset = u32::from_le_bytes(entry_bytes.get(12..16)?.try_into().ok()?) as usize;

        let is_better = match best_entry {
            None => true,
            Some((best_w, best_h, _, _)) => {
                let area = w.checked_mul(h)?;
                let best_area = best_w.checked_mul(best_h)?;
                area > best_area
            }
        };
        if is_better {
            best_entry = Some((w, h, bytes_in_res, image_offset));
        }
    }

    let (_, _, bytes_in_res, image_offset) = best_entry?;
    let end_offset = image_offset.checked_add(bytes_in_res)?;
    let embedded = bytes.get(image_offset..end_offset)?;

    if embedded.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]) {
        decode_png(embedded)
    } else {
        // Handle BMP DIB
        if embedded.len() < 40 {
            return None;
        }

        let dib_header_size = u32::from_le_bytes(embedded.get(0..4)?.try_into().ok()?) as usize;
        if dib_header_size < 40 {
            return None;
        }

        let bi_width = i32::from_le_bytes(embedded.get(4..8)?.try_into().ok()?);
        let bi_height = i32::from_le_bytes(embedded.get(8..12)?.try_into().ok()?);

        if bi_width <= 0 || bi_height == 0 {
            return None;
        }

        // In an ICO, the DIB's biHeight is DOUBLE the real height
        let real_height = bi_height.checked_div(2)?;

        let mut modified_dib = embedded.to_vec();
        let real_height_bytes = real_height.to_le_bytes();
        modified_dib
            .get_mut(8..12)?
            .copy_from_slice(&real_height_bytes);

        // Prepend 14-byte BMP file header
        let mut bmp_bytes = Vec::with_capacity(14 + modified_dib.len());
        bmp_bytes.extend_from_slice(b"BM");

        let file_size = (14_usize.checked_add(modified_dib.len())?) as u32;
        bmp_bytes.extend_from_slice(&file_size.to_le_bytes());

        // Reserved fields (4 bytes of 0)
        bmp_bytes.extend_from_slice(&[0u8; 4]);

        let pixel_offset = (14_usize.checked_add(dib_header_size)?) as u32;
        bmp_bytes.extend_from_slice(&pixel_offset.to_le_bytes());

        bmp_bytes.extend_from_slice(&modified_dib);

        // TODO(spec): ICO AND-mask transparency and palette (<=8bpp) not yet handled
        decode_bmp(&bmp_bytes)
    }
}

/// Decodes a QOI (Quite OK Image) byte stream into a DecodedImage.
/// spec: S-19
pub fn decode_qoi(bytes: &[u8]) -> Option<DecodedImage> {
    if bytes.len() < 22 {
        return None;
    }

    if bytes.get(0..4)? != b"qoif" {
        return None;
    }

    let width = u32::from_be_bytes(bytes.get(4..8)?.try_into().ok()?);
    let height = u32::from_be_bytes(bytes.get(8..12)?.try_into().ok()?);
    let channels = *bytes.get(12)?;
    let colorspace = *bytes.get(13)?;

    if width == 0 || height == 0 {
        return None;
    }
    if channels != 3 && channels != 4 {
        return None;
    }
    if colorspace != 0 && colorspace != 1 {
        return None;
    }

    let total_pixels = (width as usize).checked_mul(height as usize)?;
    let total_bytes = total_pixels.checked_mul(4)?;
    let mut rgba = vec![0u8; total_bytes];

    let mut px_prev = [0u8, 0u8, 0u8, 255u8];
    let mut index = [[0u8; 4]; 64];

    let mut p_in = 14;
    let mut p_out = 0;

    while p_out < total_bytes {
        // Stop at the 8-byte end marker
        if bytes.get(p_in..p_in + 8) == Some(&[0, 0, 0, 0, 0, 0, 0, 1]) {
            break;
        }

        let tag = match bytes.get(p_in) {
            Some(&t) => t,
            None => return None,
        };
        p_in += 1;

        if tag == 0xFE {
            // QOI_OP_RGB
            let r = *bytes.get(p_in)?;
            let g = *bytes.get(p_in + 1)?;
            let b = *bytes.get(p_in + 2)?;
            p_in += 3;

            px_prev = [r, g, b, px_prev[3]];
            let hash = ((px_prev[0] as usize * 3)
                + (px_prev[1] as usize * 5)
                + (px_prev[2] as usize * 7)
                + (px_prev[3] as usize * 11))
                % 64;
            index[hash] = px_prev;

            if p_out + 4 <= total_bytes {
                rgba[p_out] = px_prev[0];
                rgba[p_out + 1] = px_prev[1];
                rgba[p_out + 2] = px_prev[2];
                rgba[p_out + 3] = px_prev[3];
                p_out += 4;
            } else {
                break;
            }
        } else if tag == 0xFF {
            // QOI_OP_RGBA
            let r = *bytes.get(p_in)?;
            let g = *bytes.get(p_in + 1)?;
            let b = *bytes.get(p_in + 2)?;
            let a = *bytes.get(p_in + 3)?;
            p_in += 4;

            px_prev = [r, g, b, a];
            let hash = ((px_prev[0] as usize * 3)
                + (px_prev[1] as usize * 5)
                + (px_prev[2] as usize * 7)
                + (px_prev[3] as usize * 11))
                % 64;
            index[hash] = px_prev;

            if p_out + 4 <= total_bytes {
                rgba[p_out] = px_prev[0];
                rgba[p_out + 1] = px_prev[1];
                rgba[p_out + 2] = px_prev[2];
                rgba[p_out + 3] = px_prev[3];
                p_out += 4;
            } else {
                break;
            }
        } else {
            let op = tag & 0xC0;
            if op == 0x00 {
                // QOI_OP_INDEX
                let idx = (tag & 0x3F) as usize;
                px_prev = index[idx];

                if p_out + 4 <= total_bytes {
                    rgba[p_out] = px_prev[0];
                    rgba[p_out + 1] = px_prev[1];
                    rgba[p_out + 2] = px_prev[2];
                    rgba[p_out + 3] = px_prev[3];
                    p_out += 4;
                } else {
                    break;
                }
            } else if op == 0x40 {
                // QOI_OP_DIFF
                let dr = ((tag >> 4) & 0x03).wrapping_sub(2);
                let dg = ((tag >> 2) & 0x03).wrapping_sub(2);
                let db = (tag & 0x03).wrapping_sub(2);

                let r = px_prev[0].wrapping_add(dr);
                let g = px_prev[1].wrapping_add(dg);
                let b = px_prev[2].wrapping_add(db);

                px_prev = [r, g, b, px_prev[3]];
                let hash = ((px_prev[0] as usize * 3)
                    + (px_prev[1] as usize * 5)
                    + (px_prev[2] as usize * 7)
                    + (px_prev[3] as usize * 11))
                    % 64;
                index[hash] = px_prev;

                if p_out + 4 <= total_bytes {
                    rgba[p_out] = px_prev[0];
                    rgba[p_out + 1] = px_prev[1];
                    rgba[p_out + 2] = px_prev[2];
                    rgba[p_out + 3] = px_prev[3];
                    p_out += 4;
                } else {
                    break;
                }
            } else if op == 0x80 {
                // QOI_OP_LUMA
                let byte2 = match bytes.get(p_in) {
                    Some(&b) => b,
                    None => return None,
                };
                p_in += 1;

                let dg = (tag & 0x3F).wrapping_sub(32);
                let dr_minus_dg = ((byte2 >> 4) & 0x0F).wrapping_sub(8);
                let db_minus_dg = (byte2 & 0x0F).wrapping_sub(8);

                let dr = dr_minus_dg.wrapping_add(dg);
                let db = db_minus_dg.wrapping_add(dg);

                let r = px_prev[0].wrapping_add(dr);
                let g = px_prev[1].wrapping_add(dg);
                let b = px_prev[2].wrapping_add(db);

                px_prev = [r, g, b, px_prev[3]];
                let hash = ((px_prev[0] as usize * 3)
                    + (px_prev[1] as usize * 5)
                    + (px_prev[2] as usize * 7)
                    + (px_prev[3] as usize * 11))
                    % 64;
                index[hash] = px_prev;

                if p_out + 4 <= total_bytes {
                    rgba[p_out] = px_prev[0];
                    rgba[p_out + 1] = px_prev[1];
                    rgba[p_out + 2] = px_prev[2];
                    rgba[p_out + 3] = px_prev[3];
                    p_out += 4;
                } else {
                    break;
                }
            } else if op == 0xC0 {
                // QOI_OP_RUN
                let run_len = ((tag & 0x3F) + 1) as usize;
                for _ in 0..run_len {
                    if p_out + 4 <= total_bytes {
                        rgba[p_out] = px_prev[0];
                        rgba[p_out + 1] = px_prev[1];
                        rgba[p_out + 2] = px_prev[2];
                        rgba[p_out + 3] = px_prev[3];
                        p_out += 4;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    if p_out == total_bytes {
        Some(DecodedImage {
            width,
            height,
            rgba,
        })
    } else {
        None
    }
}

/// Decodes a Netpbm (PNM: PBM/PGM/PPM) byte stream into a DecodedImage.
/// Supports ASCII variants (P1, P2, P3) and binary variants (P4, P5, P6).
/// spec: S-19
pub fn decode_pnm(bytes: &[u8]) -> Option<DecodedImage> {
    struct PnmParser<'a> {
        bytes: &'a [u8],
        pos: usize,
    }

    impl<'a> PnmParser<'a> {
        fn new(bytes: &'a [u8]) -> Self {
            Self { bytes, pos: 0 }
        }

        fn skip_whitespace_and_comments(&mut self) {
            while self.pos < self.bytes.len() {
                let b = self.bytes[self.pos];
                if b.is_ascii_whitespace() {
                    self.pos += 1;
                } else if b == b'#' {
                    self.pos += 1;
                    while self.pos < self.bytes.len() {
                        let next_b = self.bytes[self.pos];
                        self.pos += 1;
                        if next_b == b'\n' || next_b == b'\r' {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        }

        fn next_u32(&mut self) -> Option<u32> {
            self.skip_whitespace_and_comments();
            let start = self.pos;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            if start == self.pos {
                return None;
            }
            let s = std::str::from_utf8(&self.bytes[start..self.pos]).ok()?;
            s.parse::<u32>().ok()
        }

        fn next_p1_bit(&mut self) -> Option<u8> {
            self.skip_whitespace_and_comments();
            if self.pos < self.bytes.len() {
                let b = self.bytes[self.pos];
                if b == b'0' || b == b'1' {
                    self.pos += 1;
                    return Some(b - b'0');
                }
            }
            None
        }
    }

    if bytes.len() < 2 || bytes[0] != b'P' {
        return None;
    }
    let variant = bytes[1];
    if !((b'1'..=b'6').contains(&variant)) {
        return None;
    }

    let mut parser = PnmParser::new(bytes);
    parser.pos = 2; // Move past the magic "Px"

    let width = parser.next_u32()?;
    let height = parser.next_u32()?;

    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        return None;
    }

    let total_pixels = width.checked_mul(height)?;
    let total_bytes = total_pixels.checked_mul(4)?;

    let maxval = if variant == b'1' || variant == b'4' {
        0
    } else {
        let mv = parser.next_u32()?;
        if mv == 0 || mv > 65535 {
            return None;
        }
        mv
    };

    let is_binary = variant == b'4' || variant == b'5' || variant == b'6';

    let rgba = if is_binary {
        // Exactly one whitespace byte separates the last header token from the binary raster.
        let separator_byte = *parser.bytes.get(parser.pos)?;
        if !separator_byte.is_ascii_whitespace() {
            return None;
        }
        parser.pos += 1;
        let raster_start = parser.pos;
        let raster_bytes = &parser.bytes[raster_start..];

        match variant {
            b'4' => {
                let bytes_per_row = (width as usize).div_ceil(8);
                let expected_raster_len = bytes_per_row.checked_mul(height as usize)?;
                if raster_bytes.len() < expected_raster_len {
                    return None;
                }
                let mut rgba = Vec::with_capacity(total_bytes as usize);
                for y in 0..height {
                    let row_start = (y as usize).checked_mul(bytes_per_row)?;
                    let row_data = raster_bytes.get(row_start..row_start + bytes_per_row)?;
                    for x in 0..width {
                        let byte_idx = (x as usize) / 8;
                        let bit_idx = 7 - ((x as usize) % 8);
                        let byte_val = row_data[byte_idx];
                        let bit = (byte_val >> bit_idx) & 1;
                        if bit == 1 {
                            rgba.extend_from_slice(&[0, 0, 0, 255]);
                        } else {
                            rgba.extend_from_slice(&[255, 255, 255, 255]);
                        }
                    }
                }
                rgba
            }
            b'5' => {
                let mut rgba = Vec::with_capacity(total_bytes as usize);
                if maxval < 256 {
                    let expected_raster_len = total_pixels as usize;
                    if raster_bytes.len() < expected_raster_len {
                        return None;
                    }
                    for &value_byte in raster_bytes.iter().take(expected_raster_len) {
                        let value = value_byte as u32;
                        let value = value.min(maxval);
                        let gray = (value * 255 / maxval) as u8;
                        rgba.extend_from_slice(&[gray, gray, gray, 255]);
                    }
                } else {
                    let expected_raster_len = (total_pixels as usize).checked_mul(2)?;
                    if raster_bytes.len() < expected_raster_len {
                        return None;
                    }
                    for chunk in raster_bytes.chunks_exact(2).take(total_pixels as usize) {
                        let value = u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
                        let value = value.min(maxval);
                        let gray = (value * 255 / maxval) as u8;
                        rgba.extend_from_slice(&[gray, gray, gray, 255]);
                    }
                }
                rgba
            }
            b'6' => {
                let mut rgba = Vec::with_capacity(total_bytes as usize);
                if maxval < 256 {
                    let expected_raster_len = (total_pixels as usize).checked_mul(3)?;
                    if raster_bytes.len() < expected_raster_len {
                        return None;
                    }
                    for chunk in raster_bytes.chunks_exact(3).take(total_pixels as usize) {
                        let r_val = chunk[0] as u32;
                        let g_val = chunk[1] as u32;
                        let b_val = chunk[2] as u32;

                        let r_val = r_val.min(maxval);
                        let g_val = g_val.min(maxval);
                        let b_val = b_val.min(maxval);

                        let r = (r_val * 255 / maxval) as u8;
                        let g = (g_val * 255 / maxval) as u8;
                        let b = (b_val * 255 / maxval) as u8;

                        rgba.extend_from_slice(&[r, g, b, 255]);
                    }
                } else {
                    let expected_raster_len = (total_pixels as usize).checked_mul(6)?;
                    if raster_bytes.len() < expected_raster_len {
                        return None;
                    }
                    for chunk in raster_bytes.chunks_exact(6).take(total_pixels as usize) {
                        let r_val = u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
                        let g_val = u16::from_be_bytes([chunk[2], chunk[3]]) as u32;
                        let b_val = u16::from_be_bytes([chunk[4], chunk[5]]) as u32;

                        let r_val = r_val.min(maxval);
                        let g_val = g_val.min(maxval);
                        let b_val = b_val.min(maxval);

                        let r = (r_val * 255 / maxval) as u8;
                        let g = (g_val * 255 / maxval) as u8;
                        let b = (b_val * 255 / maxval) as u8;

                        rgba.extend_from_slice(&[r, g, b, 255]);
                    }
                }
                rgba
            }
            _ => return None,
        }
    } else {
        match variant {
            b'1' => {
                let mut rgba = Vec::with_capacity(total_bytes as usize);
                for _ in 0..total_pixels {
                    let bit = parser.next_p1_bit()?;
                    if bit == 1 {
                        rgba.extend_from_slice(&[0, 0, 0, 255]);
                    } else {
                        rgba.extend_from_slice(&[255, 255, 255, 255]);
                    }
                }
                rgba
            }
            b'2' => {
                let mut rgba = Vec::with_capacity(total_bytes as usize);
                for _ in 0..total_pixels {
                    let value = parser.next_u32()?;
                    let value = value.min(maxval);
                    let gray = (value * 255 / maxval) as u8;
                    rgba.extend_from_slice(&[gray, gray, gray, 255]);
                }
                rgba
            }
            b'3' => {
                let mut rgba = Vec::with_capacity(total_bytes as usize);
                for _ in 0..total_pixels {
                    let r_val = parser.next_u32()?;
                    let g_val = parser.next_u32()?;
                    let b_val = parser.next_u32()?;

                    let r_val = r_val.min(maxval);
                    let g_val = g_val.min(maxval);
                    let b_val = b_val.min(maxval);

                    let r = (r_val * 255 / maxval) as u8;
                    let g = (g_val * 255 / maxval) as u8;
                    let b = (b_val * 255 / maxval) as u8;

                    rgba.extend_from_slice(&[r, g, b, 255]);
                }
                rgba
            }
            _ => return None,
        }
    };

    Some(DecodedImage {
        width,
        height,
        rgba,
    })
}

fn is_conservative_tga(bytes: &[u8]) -> bool {
    if bytes.len() < 18 {
        return false;
    }
    let color_map_type = bytes[1];
    if color_map_type > 1 {
        return false;
    }
    let image_type = bytes[2];
    if image_type != 2 && image_type != 3 && image_type != 10 && image_type != 11 {
        return false;
    }
    let width = u16::from_le_bytes([bytes[12], bytes[13]]) as u32;
    let height = u16::from_le_bytes([bytes[14], bytes[15]]) as u32;
    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        return false;
    }
    let bpp = bytes[16];
    if bpp != 8 && bpp != 24 && bpp != 32 {
        return false;
    }

    let descriptor = bytes[17];
    if (descriptor & 0xC0) != 0 {
        return false;
    }

    true
}

/// Decodes a TGA (Truevision Targa) image byte stream into a DecodedImage.
/// Supports uncompressed true-color (type 2) at 24bpp and 32bpp,
/// uncompressed grayscale (type 3) at 8bpp, and RLE versions (types 10 and 11).
pub fn decode_tga(bytes: &[u8]) -> Option<DecodedImage> {
    if bytes.len() < 18 {
        return None;
    }

    let id_length = bytes[0] as usize;
    let color_map_type = bytes[1];
    if color_map_type > 1 {
        return None;
    }
    let image_type = bytes[2];

    if image_type != 2 && image_type != 3 && image_type != 10 && image_type != 11 {
        return None;
    }

    let color_map_len = u16::from_le_bytes([*bytes.get(5)?, *bytes.get(6)?]) as usize;
    let color_map_entry_size = *bytes.get(7)? as usize;

    let color_map_bytes = if color_map_type == 1 {
        color_map_len.checked_mul(color_map_entry_size.div_ceil(8))?
    } else {
        0
    };

    let data_offset = 18_usize
        .checked_add(id_length)?
        .checked_add(color_map_bytes)?;

    if data_offset > bytes.len() {
        return None;
    }

    let width = u16::from_le_bytes([*bytes.get(12)?, *bytes.get(13)?]) as u32;
    let height = u16::from_le_bytes([*bytes.get(14)?, *bytes.get(15)?]) as u32;
    let bpp = *bytes.get(16)?;
    let descriptor = *bytes.get(17)?;

    if width == 0 || height == 0 {
        return None;
    }

    if (image_type == 2 || image_type == 10) && bpp != 24 && bpp != 32 {
        return None;
    }
    if (image_type == 3 || image_type == 11) && bpp != 8 {
        return None;
    }

    let has_alpha = bpp == 32 && (descriptor & 0x0F) == 8;
    let total_pixels = (width as usize).checked_mul(height as usize)?;
    let mut rgba = vec![0u8; total_pixels.checked_mul(4)?];

    let mut p = data_offset;
    let mut pixel_index = 0_usize;

    if image_type == 2 || image_type == 3 {
        let bytes_per_pixel = (bpp as usize) / 8;
        let expected_bytes = total_pixels.checked_mul(bytes_per_pixel)?;
        if bytes.len().checked_sub(data_offset)? < expected_bytes {
            return None;
        }

        for _ in 0..total_pixels {
            let col = pixel_index % (width as usize);
            let row = pixel_index / (width as usize);

            let dest_y = if (descriptor & 0x20) != 0 {
                row
            } else {
                (height as usize) - 1 - row
            };
            let dest_x = if (descriptor & 0x10) != 0 {
                (width as usize) - 1 - col
            } else {
                col
            };

            let dest_idx = (dest_y * (width as usize) + dest_x) * 4;

            if bpp == 24 {
                let b = *bytes.get(p)?;
                let g = *bytes.get(p + 1)?;
                let r = *bytes.get(p + 2)?;
                p += 3;

                rgba[dest_idx] = r;
                rgba[dest_idx + 1] = g;
                rgba[dest_idx + 2] = b;
                rgba[dest_idx + 3] = 255;
            } else if bpp == 32 {
                let b = *bytes.get(p)?;
                let g = *bytes.get(p + 1)?;
                let r = *bytes.get(p + 2)?;
                let a = *bytes.get(p + 3)?;
                p += 4;

                rgba[dest_idx] = r;
                rgba[dest_idx + 1] = g;
                rgba[dest_idx + 2] = b;
                rgba[dest_idx + 3] = if has_alpha { a } else { 255 };
            } else if bpp == 8 {
                let g_val = *bytes.get(p)?;
                p += 1;

                rgba[dest_idx] = g_val;
                rgba[dest_idx + 1] = g_val;
                rgba[dest_idx + 2] = g_val;
                rgba[dest_idx + 3] = 255;
            }

            pixel_index += 1;
        }
    } else if image_type == 10 || image_type == 11 {
        let bytes_per_pixel = (bpp as usize) / 8;
        while pixel_index < total_pixels {
            let rle_header = *bytes.get(p)?;
            p += 1;

            let count = ((rle_header & 0x7F) as usize) + 1;
            if pixel_index + count > total_pixels {
                return None;
            }

            if (rle_header & 0x80) != 0 {
                // RLE packet
                let pixel_bytes = bytes.get(p..p + bytes_per_pixel)?;
                p += bytes_per_pixel;

                let (r, g, b, a) = if bpp == 8 {
                    (pixel_bytes[0], pixel_bytes[0], pixel_bytes[0], 255)
                } else if bpp == 24 {
                    (pixel_bytes[2], pixel_bytes[1], pixel_bytes[0], 255)
                } else {
                    // bpp == 32
                    (
                        pixel_bytes[2],
                        pixel_bytes[1],
                        pixel_bytes[0],
                        if has_alpha { pixel_bytes[3] } else { 255 },
                    )
                };

                for _ in 0..count {
                    let col = pixel_index % (width as usize);
                    let row = pixel_index / (width as usize);

                    let dest_y = if (descriptor & 0x20) != 0 {
                        row
                    } else {
                        (height as usize) - 1 - row
                    };
                    let dest_x = if (descriptor & 0x10) != 0 {
                        (width as usize) - 1 - col
                    } else {
                        col
                    };

                    let dest_idx = (dest_y * (width as usize) + dest_x) * 4;
                    rgba[dest_idx] = r;
                    rgba[dest_idx + 1] = g;
                    rgba[dest_idx + 2] = b;
                    rgba[dest_idx + 3] = a;

                    pixel_index += 1;
                }
            } else {
                // Raw packet
                for _ in 0..count {
                    let pixel_bytes = bytes.get(p..p + bytes_per_pixel)?;
                    p += bytes_per_pixel;

                    let (r, g, b, a) = if bpp == 8 {
                        (pixel_bytes[0], pixel_bytes[0], pixel_bytes[0], 255)
                    } else if bpp == 24 {
                        (pixel_bytes[2], pixel_bytes[1], pixel_bytes[0], 255)
                    } else {
                        // bpp == 32
                        (
                            pixel_bytes[2],
                            pixel_bytes[1],
                            pixel_bytes[0],
                            if has_alpha { pixel_bytes[3] } else { 255 },
                        )
                    };

                    let col = pixel_index % (width as usize);
                    let row = pixel_index / (width as usize);

                    let dest_y = if (descriptor & 0x20) != 0 {
                        row
                    } else {
                        (height as usize) - 1 - row
                    };
                    let dest_x = if (descriptor & 0x10) != 0 {
                        (width as usize) - 1 - col
                    } else {
                        col
                    };

                    let dest_idx = (dest_y * (width as usize) + dest_x) * 4;
                    rgba[dest_idx] = r;
                    rgba[dest_idx + 1] = g;
                    rgba[dest_idx + 2] = b;
                    rgba[dest_idx + 3] = a;

                    pixel_index += 1;
                }
            }
        }
    }

    Some(DecodedImage {
        width,
        height,
        rgba,
    })
}

/// Decodes a PCX (ZSoft Paintbrush) image byte stream into a DecodedImage.
/// Supports 24-bit RGB (8 bits/plane x 3 planes) and 256-color indexed (8 bits/plane x 1 plane with VGA palette).
pub fn decode_pcx(bytes: &[u8]) -> Option<DecodedImage> {
    if bytes.len() < 128 {
        return None;
    }

    let manufacturer = bytes[0];
    if manufacturer != 0x0A {
        return None;
    }

    let _version = bytes[1];
    let encoding = bytes[2];
    if encoding != 1 {
        return None;
    }

    let bits_per_pixel = bytes[3];
    if bits_per_pixel != 8 {
        return None;
    }

    let xmin = u16::from_le_bytes([*bytes.get(4)?, *bytes.get(5)?]);
    let ymin = u16::from_le_bytes([*bytes.get(6)?, *bytes.get(7)?]);
    let xmax = u16::from_le_bytes([*bytes.get(8)?, *bytes.get(9)?]);
    let ymax = u16::from_le_bytes([*bytes.get(10)?, *bytes.get(11)?]);

    if xmax < xmin || ymax < ymin {
        return None;
    }

    let width = (xmax as u32) - (xmin as u32) + 1;
    let height = (ymax as u32) - (ymin as u32) + 1;

    // Guard against unbounded allocation from hostile dimensions (matches the
    // 16384 cap used by the other decoders in this module).
    if width > 16384 || height > 16384 {
        return None;
    }

    let nplanes = bytes[65];
    if nplanes != 1 && nplanes != 3 {
        return None;
    }

    let bytes_per_line = u16::from_le_bytes([*bytes.get(66)?, *bytes.get(67)?]) as usize;
    if bytes_per_line < width as usize {
        return None;
    }

    let total_scanline_bytes = (nplanes as usize) * bytes_per_line;

    let rle_limit = if nplanes == 1 {
        if bytes.len() < 128 + 769 {
            return None;
        }
        let pal_start = bytes.len() - 769;
        if bytes[pal_start] != 0x0C {
            return None;
        }
        pal_start
    } else {
        bytes.len()
    };

    let palette = if nplanes == 1 {
        let pal_start = bytes.len() - 769;
        &bytes[pal_start + 1..]
    } else {
        &[]
    };

    struct RleDecoder<'a> {
        bytes: &'a [u8],
        offset: usize,
        run_count: usize,
        run_val: u8,
    }

    impl<'a> RleDecoder<'a> {
        fn new(bytes: &'a [u8]) -> Self {
            Self {
                bytes,
                offset: 128,
                run_count: 0,
                run_val: 0,
            }
        }

        fn next_byte(&mut self) -> Option<u8> {
            if self.run_count > 0 {
                self.run_count -= 1;
                return Some(self.run_val);
            }
            let b = *self.bytes.get(self.offset)?;
            self.offset += 1;
            if (b & 0xC0) == 0xC0 {
                let count = (b & 0x3F) as usize;
                let val = *self.bytes.get(self.offset)?;
                self.offset += 1;
                if count == 0 {
                    return None;
                }
                self.run_count = count - 1;
                self.run_val = val;
                Some(val)
            } else {
                Some(b)
            }
        }
    }

    let mut rle_decoder = RleDecoder::new(&bytes[..rle_limit]);
    let total_bytes = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(4)?;
    let mut rgba = vec![0u8; total_bytes];

    // Allocated once and reused across scanlines; every iteration fully
    // overwrites the buffer below before reading it.
    let mut scanline_buf = vec![0u8; total_scanline_bytes];

    for y in 0..height {
        for b in &mut scanline_buf {
            *b = rle_decoder.next_byte()?;
        }

        if nplanes == 3 {
            for (x, &r) in scanline_buf.iter().enumerate().take(width as usize) {
                let g = scanline_buf[bytes_per_line + x];
                let b = scanline_buf[2 * bytes_per_line + x];
                let dest_idx = (y as usize * width as usize + x) * 4;
                rgba[dest_idx] = r;
                rgba[dest_idx + 1] = g;
                rgba[dest_idx + 2] = b;
                rgba[dest_idx + 3] = 255;
            }
        } else {
            // nplanes == 1
            for (x, &index_val) in scanline_buf.iter().enumerate().take(width as usize) {
                let index = index_val as usize;
                let r = *palette.get(index * 3)?;
                let g = *palette.get(index * 3 + 1)?;
                let b = *palette.get(index * 3 + 2)?;
                let dest_idx = (y as usize * width as usize + x) * 4;
                rgba[dest_idx] = r;
                rgba[dest_idx + 1] = g;
                rgba[dest_idx + 2] = b;
                rgba[dest_idx + 3] = 255;
            }
        }
    }

    Some(DecodedImage {
        width,
        height,
        rgba,
    })
}

/// Decodes an image byte stream (PNG, JPEG, GIF, BMP, WebP, SVG, ICO, QOI, PCX, or TGA) into a DecodedImage by sniffing the format.
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
    } else if bytes.len() >= 6 && bytes[0] == 0 && bytes[1] == 0 && bytes[2] == 1 && bytes[3] == 0 {
        decode_ico(bytes)
    } else if bytes.starts_with(b"qoif") {
        decode_qoi(bytes)
    } else if let Some(true) = is_svg_sniff(bytes) {
        decode_svg(bytes)
    } else if bytes.len() >= 2 && bytes[0] == b'P' && (b'1'..=b'6').contains(&bytes[1]) {
        decode_pnm(bytes)
    } else if bytes.first() == Some(&0x0A)
        && bytes.get(2) == Some(&1)
        && matches!(bytes.get(1), Some(0 | 2 | 3 | 4 | 5))
    {
        decode_pcx(bytes)
    } else {
        // Detect TGA as a last resort
        let is_tga_footer =
            bytes.len() >= 26 && bytes.get(bytes.len() - 18..) == Some(b"TRUEVISION-XFILE.\0");
        if is_tga_footer || is_conservative_tga(bytes) {
            decode_tga(bytes)
        } else {
            None
        }
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

    #[test]
    fn test_ico_png_embedded_t0507() {
        let mut canvas = Canvas::new(2, 2);
        canvas.pixels[0] = 0xFFFF0000; // Red
        canvas.pixels[1] = 0xFF00FF00; // Green
        canvas.pixels[2] = 0xFF0000FF; // Blue
        canvas.pixels[3] = 0x80FFFFFF; // Semi-transparent White
        let png_bytes = encode_png(&canvas);

        let mut ico_bytes = Vec::new();
        // ICONDIR
        ico_bytes.extend_from_slice(&[0x00, 0x00]); // Reserved
        ico_bytes.extend_from_slice(&[0x01, 0x00]); // Type (1)
        ico_bytes.extend_from_slice(&[0x01, 0x00]); // Image count (1)

        // ICONDIRENTRY
        ico_bytes.push(2); // Width
        ico_bytes.push(2); // Height
        ico_bytes.push(0); // Color count
        ico_bytes.push(0); // Reserved
        ico_bytes.extend_from_slice(&[1, 0]); // Planes (1)
        ico_bytes.extend_from_slice(&[32, 0]); // Bit count (32)

        let bytes_in_res = png_bytes.len() as u32;
        ico_bytes.extend_from_slice(&bytes_in_res.to_le_bytes());

        let image_offset = 22_u32;
        ico_bytes.extend_from_slice(&image_offset.to_le_bytes());

        // Append PNG bytes
        ico_bytes.extend_from_slice(&png_bytes);

        let decoded = decode_ico(&ico_bytes).expect("Should decode ICO PNG successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 255]);
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 255]);
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 255]);
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 128]);

        // Test via decode_image dispatcher
        let decoded_img = decode_image(&ico_bytes).expect("Should decode via decode_image");
        assert_eq!(decoded_img.width, 2);
    }

    #[test]
    fn test_ico_bmp_embedded_t0507() {
        let mut dib_bytes = Vec::new();
        // biSize
        dib_bytes.extend_from_slice(&[40, 0, 0, 0]);
        // biWidth
        dib_bytes.extend_from_slice(&[2, 0, 0, 0]);
        // biHeight (doubled to 4)
        dib_bytes.extend_from_slice(&[4, 0, 0, 0]);
        // biPlanes
        dib_bytes.extend_from_slice(&[1, 0]);
        // biBitCount
        dib_bytes.extend_from_slice(&[32, 0]);
        // biCompression
        dib_bytes.extend_from_slice(&[0, 0, 0, 0]);
        // biSizeImage
        dib_bytes.extend_from_slice(&[16, 0, 0, 0]);
        // biXPelsPerMeter, biYPelsPerMeter, biClrUsed, biClrImportant
        dib_bytes.extend_from_slice(&[0u8; 16]);

        // Pixel data: Bottom row first, then top row
        // Bottom row: Blue [255, 0, 0, 255], White [255, 255, 255, 128]
        dib_bytes.extend_from_slice(&[255, 0, 0, 255, 255, 255, 255, 128]);
        // Top row: Red [0, 0, 255, 255], Green [0, 255, 0, 255]
        dib_bytes.extend_from_slice(&[0, 0, 255, 255, 0, 255, 0, 255]);

        // AND mask (e.g. 2 bytes)
        dib_bytes.extend_from_slice(&[0, 0]);

        let mut ico_bytes = Vec::new();
        // ICONDIR
        ico_bytes.extend_from_slice(&[0x00, 0x00]); // Reserved
        ico_bytes.extend_from_slice(&[0x01, 0x00]); // Type (1)
        ico_bytes.extend_from_slice(&[0x01, 0x00]); // Image count (1)

        // ICONDIRENTRY
        ico_bytes.push(2); // Width
        ico_bytes.push(2); // Height
        ico_bytes.push(0); // Color count
        ico_bytes.push(0); // Reserved
        ico_bytes.extend_from_slice(&[1, 0]); // Planes (1)
        ico_bytes.extend_from_slice(&[32, 0]); // Bit count (32)

        let bytes_in_res = dib_bytes.len() as u32;
        ico_bytes.extend_from_slice(&bytes_in_res.to_le_bytes());

        let image_offset = 22_u32;
        ico_bytes.extend_from_slice(&image_offset.to_le_bytes());

        // Append DIB bytes
        ico_bytes.extend_from_slice(&dib_bytes);

        let decoded = decode_ico(&ico_bytes).expect("Should decode ICO DIB successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba.len(), 16);
        // RGBA
        // Top-left: Red
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 255]);
        // Top-right: Green
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 255]);
        // Bottom-left: Blue
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 255]);
        // Bottom-right: White (semi-transparent)
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 128]);

        // Test via decode_image dispatcher
        let decoded_img = decode_image(&ico_bytes).expect("Should decode via decode_image");
        assert_eq!(decoded_img.width, 2);
    }

    #[test]
    fn test_ico_sniff_rejects_non_ico_t0507() {
        // Assert decode_image still routes PNG, BMP, etc. to their own decoders.
        let mut canvas = Canvas::new(1, 1);
        canvas.pixels[0] = 0xFFFF0000;
        let png_bytes = encode_png(&canvas);
        let decoded_png = decode_image(&png_bytes).expect("Should decode PNG via decode_image");
        assert_eq!(decoded_png.width, 1);

        // Assert random/short bytes return None without panicking
        assert!(decode_image(&[]).is_none());
        assert!(decode_image(&[0, 0, 1]).is_none());
        assert!(decode_image(&[0, 0, 1, 0]).is_none());
        assert!(decode_image(&[0, 0, 1, 0, 0, 0]).is_none());
        assert!(decode_ico(&[]).is_none());
        assert!(decode_ico(&[0, 0, 1]).is_none());
        assert!(decode_ico(&[0, 0, 1, 0, 0, 0]).is_none());
    }

    #[test]
    fn test_decode_qoi_run() {
        let qoi_bytes = vec![
            // Header
            0x71, 0x6F, 0x69, 0x66, // magic: qoif
            0, 0, 0, 2, // width: 2
            0, 0, 0, 2, // height: 2
            4, // channels: 4
            0, // colorspace: 0
            // Chunks
            0xFF, 255, 0, 0, 255,  // QOI_OP_RGBA: [255, 0, 0, 255]
            0xC2, // QOI_OP_RUN: repeat 3 times
            // End marker
            0, 0, 0, 0, 0, 0, 0, 1,
        ];
        let decoded = decode_qoi(&qoi_bytes).expect("Should decode QOI run successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba.len(), 16);
        for i in 0..4 {
            assert_eq!(&decoded.rgba[i * 4..(i + 1) * 4], &[255, 0, 0, 255]);
        }
    }

    #[test]
    fn test_decode_qoi_diff() {
        let qoi_bytes = vec![
            // Header
            0x71, 0x6F, 0x69, 0x66, // magic: qoif
            0, 0, 0, 2, // width: 2
            0, 0, 0, 1, // height: 1
            4, // channels: 4
            0, // colorspace: 0
            // Chunks
            0xFF, 0, 255, 0, 255,  // QOI_OP_RGBA: [0, 255, 0, 255]
            0x76, // QOI_OP_DIFF: dr=1, dg=-1, db=0 -> [1, 254, 0, 255]
            // End marker
            0, 0, 0, 0, 0, 0, 0, 1,
        ];
        let decoded = decode_qoi(&qoi_bytes).expect("Should decode QOI diff successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 1);
        assert_eq!(decoded.rgba.len(), 8);
        assert_eq!(&decoded.rgba[0..4], &[0, 255, 0, 255]);
        assert_eq!(&decoded.rgba[4..8], &[1, 254, 0, 255]);
    }

    #[test]
    fn test_decode_qoi_rgb_and_index() {
        let qoi_bytes = vec![
            // Header
            0x71, 0x6F, 0x69, 0x66, // magic: qoif
            0, 0, 0, 2, // width: 2
            0, 0, 0, 1, // height: 1
            4, // channels: 4
            0, // colorspace: 0
            // Chunks
            0xFE, 100, 150, 200,  // QOI_OP_RGB: [100, 150, 200, 255]
            0x00, // QOI_OP_INDEX: load from index 0 -> [0, 0, 0, 0]
            // End marker
            0, 0, 0, 0, 0, 0, 0, 1,
        ];
        let decoded = decode_qoi(&qoi_bytes).expect("Should decode QOI RGB and INDEX successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 1);
        assert_eq!(decoded.rgba.len(), 8);
        assert_eq!(&decoded.rgba[0..4], &[100, 150, 200, 255]);
        assert_eq!(&decoded.rgba[4..8], &[0, 0, 0, 0]);
    }

    #[test]
    fn test_decode_qoi_luma() {
        let qoi_bytes = vec![
            // Header
            0x71, 0x6F, 0x69, 0x66, // magic: qoif
            0, 0, 0, 2, // width: 2
            0, 0, 0, 1, // height: 1
            4, // channels: 4
            0, // colorspace: 0
            // Chunks
            0xFF, 50, 100, 150, 255, // QOI_OP_RGBA: [50, 100, 150, 255]
            0xA3, 0x69, // QOI_OP_LUMA: dg=3, dr_minus_dg=-2, db_minus_dg=1
            // End marker
            0, 0, 0, 0, 0, 0, 0, 1,
        ];
        let decoded = decode_qoi(&qoi_bytes).expect("Should decode QOI luma successfully");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 1);
        assert_eq!(decoded.rgba.len(), 8);
        assert_eq!(&decoded.rgba[0..4], &[50, 100, 150, 255]);
        assert_eq!(&decoded.rgba[4..8], &[51, 103, 154, 255]);
    }

    #[test]
    fn test_decode_qoi_malformed_and_routing() {
        // 1. Assert decode_qoi on a too-short / wrong-magic buffer returns None (no panic).
        assert!(decode_qoi(&[]).is_none());
        assert!(decode_qoi(b"qoif").is_none());
        assert!(decode_qoi(b"wrongmagic_with_plenty_of_bytes_to_exceed_header").is_none());

        // 2. Assert decode_image routes a qoif-prefixed buffer to the QOI path.
        let qoi_bytes = vec![
            0x71, 0x6F, 0x69, 0x66, // magic: qoif
            0, 0, 0, 2, // width: 2
            0, 0, 0, 1, // height: 1
            4, // channels: 4
            0, // colorspace: 0
            // Chunks
            0xFF, 50, 100, 150, 255,  // QOI_OP_RGBA
            0xC0, // QOI_OP_RUN: 1 repeat
            // End marker
            0, 0, 0, 0, 0, 0, 0, 1,
        ];
        let decoded = decode_image(&qoi_bytes).expect("Should route to QOI decoder");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 1);
        assert_eq!(&decoded.rgba[0..4], &[50, 100, 150, 255]);
        assert_eq!(&decoded.rgba[4..8], &[50, 100, 150, 255]);
    }

    #[test]
    fn test_decode_pnm_all() {
        // P1: ASCII bitmap
        let p1_data = b"P1\n# Comment\n2 1\n1 0\n";
        let img = decode_pnm(p1_data).expect("Should decode P1");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 1);
        assert_eq!(&img.rgba, &[0, 0, 0, 255, 255, 255, 255, 255]);

        // P2: ASCII graymap
        let p2_data = b"P2\n2 1\n100\n50 100";
        let img = decode_pnm(p2_data).expect("Should decode P2");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 1);
        assert_eq!(&img.rgba, &[127, 127, 127, 255, 255, 255, 255, 255]);

        // P3: ASCII pixmap
        let p3_data = b"P3\n# Another comment\n2 1\n255\n10 20 30 40 50 60\n";
        let img = decode_pnm(p3_data).expect("Should decode P3");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 1);
        assert_eq!(&img.rgba, &[10, 20, 30, 255, 40, 50, 60, 255]);

        // P4: Binary bitmap
        let p4_data = b"P4\n2 1\n\x80";
        let img = decode_pnm(p4_data).expect("Should decode P4");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 1);
        assert_eq!(&img.rgba, &[0, 0, 0, 255, 255, 255, 255, 255]);

        // P5: Binary graymap (maxval < 256)
        let p5_data = b"P5\n2 1\n255\n\x0a\x14";
        let img = decode_pnm(p5_data).expect("Should decode P5 < 256");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 1);
        assert_eq!(&img.rgba, &[10, 10, 10, 255, 20, 20, 20, 255]);

        // P5: Binary graymap (maxval >= 256)
        let p5_data_16 = b"P5\n2 1\n1000\n\x01\xf4\x03\xe8";
        let img = decode_pnm(p5_data_16).expect("Should decode P5 >= 256");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 1);
        assert_eq!(&img.rgba, &[127, 127, 127, 255, 255, 255, 255, 255]);

        // P6: Binary pixmap (maxval < 256)
        let p6_data = b"P6\n1 1\n255\n\x0a\x14\x1e";
        let img = decode_pnm(p6_data).expect("Should decode P6 < 256");
        assert_eq!(img.width, 1);
        assert_eq!(img.height, 1);
        assert_eq!(&img.rgba, &[10, 20, 30, 255]);

        // P6: Binary pixmap (maxval >= 256)
        let p6_data_16 = b"P6\n1 1\n1000\n\x01\xf4\x01\xf4\x01\xf4";
        let img = decode_pnm(p6_data_16).expect("Should decode P6 >= 256");
        assert_eq!(img.width, 1);
        assert_eq!(img.height, 1);
        assert_eq!(&img.rgba, &[127, 127, 127, 255]);

        // Check decode_image routing
        let routed = decode_image(p6_data).expect("Should route P6 to decode_image");
        assert_eq!(routed.width, 1);
        assert_eq!(routed.height, 1);
        assert_eq!(&routed.rgba, &[10, 20, 30, 255]);

        // Malformed and invalid checks
        assert!(decode_pnm(&[]).is_none());
        assert!(decode_pnm(b"P").is_none());
        assert!(decode_pnm(b"P7\n1 1\n255\n").is_none());
        assert!(decode_pnm(b"P6\n0 1\n255\n").is_none());
        assert!(decode_pnm(b"P6\n1 1\n0\n").is_none());
        assert!(decode_pnm(b"P6\n1 1\n65536\n").is_none());
    }

    #[test]
    fn test_decode_tga_24bpp_bottom_up() {
        // Uncompressed true-color (type 2), 2x2, 24bpp, descriptor=0 (bottom-up)
        let mut tga = vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 24, 0];
        // Bottom-up: Row 1 stored first, then Row 0.
        // Row 1: col 0 = Blue, col 1 = White
        // Row 0: col 0 = Red, col 1 = Green
        // Pixels are BGR
        tga.extend_from_slice(&[
            255, 0, 0, // Blue: B=255, G=0, R=0
            255, 255, 255, // White: B=255, G=255, R=255
            0, 0, 255, // Red: B=0, G=0, R=255
            0, 255, 0, // Green: B=0, G=255, R=0
        ]);

        let decoded = decode_tga(&tga).expect("Should decode TGA");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba.len(), 16);

        // Decoded top-left (Row 0, col 0): Red
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 255]);
        // Decoded top-right (Row 0, col 1): Green
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 255]);
        // Decoded bottom-left (Row 1, col 0): Blue
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 255]);
        // Decoded bottom-right (Row 1, col 1): White
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 255]);
    }

    #[test]
    fn test_decode_tga_24bpp_top_down() {
        // Uncompressed true-color (type 2), 2x2, 24bpp, descriptor=0x20 (top-down)
        let mut tga = vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 24, 0x20];
        // Top-down: Row 0 stored first, then Row 1.
        // Row 0: col 0 = Red, col 1 = Green
        // Row 1: col 0 = Blue, col 1 = White
        tga.extend_from_slice(&[
            0, 0, 255, // Red: B=0, G=0, R=255
            0, 255, 0, // Green: B=0, G=255, R=0
            255, 0, 0, // Blue: B=255, G=0, R=0
            255, 255, 255, // White: B=255, G=255, R=255
        ]);

        let decoded = decode_tga(&tga).expect("Should decode TGA");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);

        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 255]);
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 255]);
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 255]);
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 255]);
    }

    #[test]
    fn test_decode_tga_32bpp_with_alpha() {
        // Uncompressed true-color (type 2), 2x2, 32bpp, descriptor=8 (bottom-up, 8 alpha bits)
        let mut tga = vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 32, 8];
        // Pixels are BGRA
        // Row 1 (bottom): col 0 = Blue with alpha 128, col 1 = White with alpha 255
        // Row 0 (top): col 0 = Red with alpha 64, col 1 = Green with alpha 192
        tga.extend_from_slice(&[
            255, 0, 0, 128, 255, 255, 255, 255, 0, 0, 255, 64, 0, 255, 0, 192,
        ]);

        let decoded = decode_tga(&tga).expect("Should decode TGA");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);

        // Row 0
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 64]);
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 192]);
        // Row 1
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 128]);
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 255]);
    }

    #[test]
    fn test_decode_tga_32bpp_no_alpha() {
        // Uncompressed true-color (type 2), 2x2, 32bpp, descriptor=0 (bottom-up, 0 alpha bits -> treat A as 255)
        let mut tga = vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 32, 0];
        tga.extend_from_slice(&[
            255, 0, 0, 128, 255, 255, 255, 255, 0, 0, 255, 64, 0, 255, 0, 192,
        ]);

        let decoded = decode_tga(&tga).expect("Should decode TGA");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);

        // All A must be 255
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 255]);
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 255]);
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 255]);
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 255, 255]);
    }

    #[test]
    fn test_decode_tga_8bpp_grayscale() {
        // Uncompressed black-and-white (type 3), 2x2, 8bpp, descriptor=0
        let mut tga = vec![0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 8, 0];
        // Bottom-up: Row 1 = [100, 200], Row 0 = [50, 150]
        tga.extend_from_slice(&[100, 200, 50, 150]);

        let decoded = decode_tga(&tga).expect("Should decode TGA");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);

        assert_eq!(&decoded.rgba[0..4], &[50, 50, 50, 255]);
        assert_eq!(&decoded.rgba[4..8], &[150, 150, 150, 255]);
        assert_eq!(&decoded.rgba[8..12], &[100, 100, 100, 255]);
        assert_eq!(&decoded.rgba[12..16], &[200, 200, 200, 255]);
    }

    #[test]
    fn test_decode_tga_rle_24bpp() {
        // RLE true-color (type 10), 2x2, 24bpp, descriptor=0 (bottom-up)
        let mut tga = vec![0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 24, 0];
        // Raw packet of count 2: Red, Green
        // RLE packet of count 2: Blue
        tga.extend_from_slice(&[
            0x01, // Raw, count 2
            0, 0, 255, // Red
            0, 255, 0,    // Green
            0x81, // RLE, count 2
            255, 0, 0, // Blue
        ]);

        let decoded = decode_tga(&tga).expect("Should decode RLE TGA");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);

        // Row 0: Blue, Blue
        assert_eq!(&decoded.rgba[0..4], &[0, 0, 255, 255]);
        assert_eq!(&decoded.rgba[4..8], &[0, 0, 255, 255]);
        // Row 1: Red, Green
        assert_eq!(&decoded.rgba[8..12], &[255, 0, 0, 255]);
        assert_eq!(&decoded.rgba[12..16], &[0, 255, 0, 255]);
    }

    #[test]
    fn test_decode_tga_rle_32bpp() {
        // RLE true-color (type 10), 2x2, 32bpp, descriptor=8 (with alpha)
        let mut tga = vec![0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 32, 8];
        // RLE packet of count 4: Red (alpha 128)
        tga.extend_from_slice(&[
            0x83, // RLE, count 4
            0, 0, 255, 128, // Red (alpha 128)
        ]);

        let decoded = decode_tga(&tga).expect("Should decode RLE TGA");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);

        for i in 0..4 {
            assert_eq!(&decoded.rgba[i * 4..(i + 1) * 4], &[255, 0, 0, 128]);
        }
    }

    #[test]
    fn test_decode_tga_rle_8bpp_grayscale() {
        // RLE grayscale (type 11), 2x2, 8bpp, descriptor=0
        let mut tga = vec![0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 8, 0];
        // RLE packet of count 4: Gray 128
        tga.extend_from_slice(&[
            0x83, // RLE, count 4
            128,
        ]);

        let decoded = decode_tga(&tga).expect("Should decode RLE grayscale TGA");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);

        for i in 0..4 {
            assert_eq!(&decoded.rgba[i * 4..(i + 1) * 4], &[128, 128, 128, 255]);
        }
    }

    #[test]
    fn test_decode_tga_malformed() {
        assert!(decode_tga(&[]).is_none());
        assert!(decode_tga(&[0; 10]).is_none());

        // Incorrect image type
        let bad_type = vec![0, 0, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 24, 0];
        assert!(decode_tga(&bad_type).is_none());

        // Zero width/height
        let zero_width = vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 24, 0];
        assert!(decode_tga(&zero_width).is_none());

        // Missing pixel bytes
        let truncated = vec![
            0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 24, 0, 255, 0,
            0, // only 1 pixel instead of 4
        ];
        assert!(decode_tga(&truncated).is_none());
    }

    #[test]
    fn test_decode_image_routing_tga() {
        // TGA with version 2.0 footer TRUEVISION-XFILE.\0
        let mut tga = vec![
            0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 24, 0, 255, 0, 0, 255, 255, 255, 0, 0,
            255, 0, 255, 0,
        ];
        // 26-byte TGA footer (last 18 bytes is TRUEVISION-XFILE.\0)
        tga.extend_from_slice(&[0; 8]); // Extension and developer offsets
        tga.extend_from_slice(b"TRUEVISION-XFILE.\0");

        let decoded = decode_image(&tga).expect("Should route and decode TGA");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);

        // TGA without footer but matching conservatively
        let tga_no_footer = vec![
            0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 24, 0, 255, 0, 0, 255, 255, 255, 0, 0,
            255, 0, 255, 0,
        ];
        let decoded_no_footer =
            decode_image(&tga_no_footer).expect("Should route and decode TGA without footer");
        assert_eq!(decoded_no_footer.width, 2);
    }

    #[test]
    fn test_decode_pcx_24bit_rle() {
        let mut pcx = vec![0u8; 128];
        pcx[0] = 0x0A; // Manufacturer
        pcx[1] = 5; // Version
        pcx[2] = 1; // Encoding
        pcx[3] = 8; // BitsPerPixel
        pcx[4..12].copy_from_slice(&[0, 0, 0, 0, 1, 0, 1, 0]); // Xmin, Ymin, Xmax, Ymax
        pcx[65] = 3; // NPlanes
        pcx[66..68].copy_from_slice(&[2, 0]); // BytesPerLine (2)

        // RLE stream
        // Row 0: R plane run of 2 val 100, G plane run of 2 val 150, B plane run of 2 val 200
        pcx.extend_from_slice(&[0xC2, 100, 0xC2, 150, 0xC2, 200]);
        // Row 1: R plane [10, 20], G plane [30, 40], B plane [50, 60] (literals)
        pcx.extend_from_slice(&[10, 20, 30, 40, 50, 60]);

        let decoded = decode_pcx(&pcx).expect("Should decode 24-bit PCX");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba.len(), 16);

        // Row 0
        assert_eq!(&decoded.rgba[0..4], &[100, 150, 200, 255]);
        assert_eq!(&decoded.rgba[4..8], &[100, 150, 200, 255]);
        // Row 1
        assert_eq!(&decoded.rgba[8..12], &[10, 30, 50, 255]);
        assert_eq!(&decoded.rgba[12..16], &[20, 40, 60, 255]);

        // Test routing in decode_image
        let routed = decode_image(&pcx).expect("Should route and decode 24-bit PCX");
        assert_eq!(routed.width, 2);
        assert_eq!(routed.rgba, decoded.rgba);
    }

    #[test]
    fn test_decode_pcx_indexed_palette() {
        let mut pcx = vec![0u8; 128];
        pcx[0] = 0x0A; // Manufacturer
        pcx[1] = 5; // Version
        pcx[2] = 1; // Encoding
        pcx[3] = 8; // BitsPerPixel
        pcx[4..12].copy_from_slice(&[0, 0, 0, 0, 1, 0, 1, 0]); // Xmin, Ymin, Xmax, Ymax
        pcx[65] = 1; // NPlanes
        pcx[66..68].copy_from_slice(&[2, 0]); // BytesPerLine (2)

        // RLE stream
        // Row 0: literals [1, 2]
        pcx.extend_from_slice(&[1, 2]);
        // Row 1: RLE run of 1 val 3, literal 4
        pcx.extend_from_slice(&[0xC1, 3, 4]);

        // Palette marker
        pcx.push(0x0C);

        // 256 RGB triples (768 bytes)
        let mut palette = vec![0u8; 768];
        palette[3..6].copy_from_slice(&[255, 0, 0]);
        palette[6..9].copy_from_slice(&[0, 255, 0]);
        palette[9..12].copy_from_slice(&[0, 0, 255]);
        palette[12..15].copy_from_slice(&[255, 255, 0]);
        pcx.extend_from_slice(&palette);

        let decoded = decode_pcx(&pcx).expect("Should decode indexed PCX");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba.len(), 16);

        // Row 0
        assert_eq!(&decoded.rgba[0..4], &[255, 0, 0, 255]);
        assert_eq!(&decoded.rgba[4..8], &[0, 255, 0, 255]);
        // Row 1
        assert_eq!(&decoded.rgba[8..12], &[0, 0, 255, 255]);
        assert_eq!(&decoded.rgba[12..16], &[255, 255, 0, 255]);

        // Test routing in decode_image
        let routed = decode_image(&pcx).expect("Should route and decode indexed PCX");
        assert_eq!(routed.width, 2);
        assert_eq!(routed.rgba, decoded.rgba);
    }
}
