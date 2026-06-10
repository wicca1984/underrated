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

/// Decodes an image byte stream (PNG or JPEG) into a DecodedImage by sniffing the format.
pub fn decode_image(bytes: &[u8]) -> Option<DecodedImage> {
    if bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]) {
        decode_png(bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        decode_jpeg(bytes)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JPEG_BASE64_1: &str = "/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////wgALCAABAAEBAREA/8QAFBABAAAAAAAAAAAAAAAAAAAAAP/aAAgBAQABPxA=";
    const JPEG_BASE64_2: &str = "/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/wAALCAABAAEBAREA/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEAAD8AKp//2Q==";

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
    fn test_decode_garbage() {
        assert!(decode_png(b"not a png").is_none());
        assert!(decode_png(&[]).is_none());
        assert!(decode_jpeg(b"not a jpeg").is_none());
        assert!(decode_jpeg(&[]).is_none());
    }

    #[test]
    fn test_truncated_input() {
        let mut canvas = Canvas::new(2, 2);
        canvas.pixels[0] = 0xFFFF0000;
        let png_bytes = encode_png(&canvas);
        assert!(decode_png(&png_bytes[0..png_bytes.len() - 10]).is_none());

        let jpeg_bytes = crate::loader::decode_base64(JPEG_BASE64_2).unwrap();
        assert!(decode_jpeg(&jpeg_bytes[0..jpeg_bytes.len() - 10]).is_none());
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

        // Test garbage rejected by decode_image
        assert!(decode_image(b"neither png nor jpeg").is_none());
    }
}
