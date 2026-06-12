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

/// Decodes an image byte stream (PNG, JPEG, GIF, BMP, or WebP) into a DecodedImage by sniffing the format.
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
}
