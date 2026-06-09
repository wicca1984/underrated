use crate::raster::Canvas;
use std::io::Cursor;

/// A decoded image with RGBA8 pixels.
/// spec: S-19
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Encodes a Canvas into a PNG byte stream without compression to enable decoding.
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
        encoder.set_deflate_compression(png::DeflateCompression::NoCompression);

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
    if bytes.len() < 8 {
        return None;
    }
    // 1. Signature Check
    let signature = &bytes[0..8];
    if signature != [137, 80, 78, 71, 13, 10, 26, 10] {
        return None;
    }

    let mut pos = 8;
    let mut ihdr_parsed = false;
    let mut width = 0u32;
    let mut height = 0u32;
    let mut color_type_opt = None;
    let mut idat_data = Vec::new();

    // Loop through chunks
    while pos < bytes.len() {
        if pos + 8 > bytes.len() {
            // Truncated chunk header
            return None;
        }
        let length =
            u32::from_be_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]])
                as usize;
        let chunk_type = &bytes[pos + 4..pos + 8];
        pos += 8;

        if pos + length + 4 > bytes.len() {
            // Truncated chunk data or CRC
            return None;
        }

        let chunk_data = &bytes[pos..pos + length];
        // CRC is 4 bytes at pos + length, skip it safely
        pos += length + 4;

        match chunk_type {
            b"IHDR" => {
                if ihdr_parsed {
                    // IHDR can only appear once
                    return None;
                }
                if length != 13 {
                    return None;
                }
                width = u32::from_be_bytes([
                    chunk_data[0],
                    chunk_data[1],
                    chunk_data[2],
                    chunk_data[3],
                ]);
                height = u32::from_be_bytes([
                    chunk_data[4],
                    chunk_data[5],
                    chunk_data[6],
                    chunk_data[7],
                ]);
                let bit_depth = chunk_data[8];
                let col_type = chunk_data[9];
                let compression_method = chunk_data[10];
                let filter_method = chunk_data[11];
                let interlace_method = chunk_data[12];

                if width == 0 || height == 0 {
                    return None;
                }
                if bit_depth != 8 {
                    // spec: only 8-bit supported
                    return None;
                }
                if col_type != 0 && col_type != 2 && col_type != 4 && col_type != 6 {
                    // spec: only grayscale, truecolor, grayscale+alpha, truecolor+alpha
                    return None;
                }
                if compression_method != 0 {
                    return None;
                }
                if filter_method != 0 {
                    return None;
                }
                if interlace_method != 0 {
                    // We only support non-interlaced
                    return None;
                }
                color_type_opt = Some(col_type);
                ihdr_parsed = true;
            }
            b"IDAT" => {
                if !ihdr_parsed {
                    return None;
                }
                idat_data.extend_from_slice(chunk_data);
            }
            b"IEND" => {
                break;
            }
            _ => {
                // Ignore other ancillary chunks
            }
        }
    }

    if !ihdr_parsed || idat_data.is_empty() {
        return None;
    }

    let color_type = color_type_opt?;

    // Decompress the IDAT data
    let decompressed_data = inflate_stored(&idat_data)?;

    let bpp = match color_type {
        0 => 1, // Grayscale
        2 => 3, // Truecolor (RGB)
        4 => 2, // Grayscale + Alpha
        6 => 4, // Truecolor + Alpha
        _ => return None,
    };

    let stride = width as usize * bpp;
    if decompressed_data.len() != height as usize * (1 + stride) {
        return None;
    }

    let mut recon = vec![0u8; height as usize * stride];

    for r in 0..height as usize {
        let filter_pos = r * (1 + stride);
        let filter_type = decompressed_data[filter_pos];
        for c in 0..stride {
            let x = decompressed_data[filter_pos + 1 + c];
            let a = if c >= bpp {
                recon[r * stride + c - bpp]
            } else {
                0
            };
            let b = if r > 0 {
                recon[(r - 1) * stride + c]
            } else {
                0
            };
            let c_prev = if r > 0 && c >= bpp {
                recon[(r - 1) * stride + c - bpp]
            } else {
                0
            };

            let recon_val = match filter_type {
                0 => x,
                1 => x.wrapping_add(a),
                2 => x.wrapping_add(b),
                3 => x.wrapping_add(((a as u32 + b as u32) / 2) as u8),
                4 => x.wrapping_add(paeth_predictor(a, b, c_prev)),
                _ => return None, // Invalid filter type
            };
            recon[r * stride + c] = recon_val;
        }
    }

    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for r in 0..height as usize {
        for col in 0..width as usize {
            let pixel_pos = r * stride + col * bpp;
            let (r_val, g_val, b_val, a_val) = match color_type {
                0 => {
                    let g = recon[pixel_pos];
                    (g, g, g, 255)
                }
                2 => (
                    recon[pixel_pos],
                    recon[pixel_pos + 1],
                    recon[pixel_pos + 2],
                    255,
                ),
                4 => {
                    let g = recon[pixel_pos];
                    (g, g, g, recon[pixel_pos + 1])
                }
                6 => (
                    recon[pixel_pos],
                    recon[pixel_pos + 1],
                    recon[pixel_pos + 2],
                    recon[pixel_pos + 3],
                ),
                _ => return None,
            };
            rgba.push(r_val);
            rgba.push(g_val);
            rgba.push(b_val);
            rgba.push(a_val);
        }
    }

    Some(DecodedImage {
        width,
        height,
        rgba,
    })
}

/// spec: S-49
/// Decompresses a zlib stream containing only uncompressed/stored blocks.
/// Marks other block types as TODO(spec).
fn inflate_stored(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 6 {
        return None;
    }
    // Read zlib header
    let cmf = data[0];
    let flg = data[1];

    // Check zlib header integrity
    if (cmf & 0x0F) != 8 {
        // Only Method 8 (DEFLATE) is supported by zlib
        return None;
    }
    if !(cmf as u32 * 256 + flg as u32).is_multiple_of(31) {
        return None;
    }
    if (flg & 0x20) != 0 {
        // Preset dictionary is not supported
        return None;
    }

    let mut out = Vec::new();
    let mut pos = 2;

    loop {
        if pos >= data.len() - 4 {
            return None;
        }
        let header = data[pos];
        pos += 1;

        let bfinal = (header & 0x01) != 0;
        let btype = (header >> 1) & 0x03;

        if btype != 0 {
            // TODO(spec): Support compressed DEFLATE blocks (BTYPE 01 and 10)
            return None;
        }

        // For BTYPE 00, read LEN and NLEN after skipping remaining bits to align with byte boundary
        if pos + 4 > data.len() - 4 {
            return None;
        }
        let len = (data[pos] as u16) | ((data[pos + 1] as u16) << 8);
        let nlen = (data[pos + 2] as u16) | ((data[pos + 3] as u16) << 8);
        pos += 4;

        if len != !nlen {
            return None;
        }

        if pos + len as usize > data.len() - 4 {
            return None;
        }

        out.extend_from_slice(&data[pos..pos + len as usize]);
        pos += len as usize;

        if bfinal {
            break;
        }
    }

    // The remaining 4 bytes represent the Adler-32 checksum.
    // We can verify that we read the entire stream up to the 4-byte checksum.
    if pos != data.len() - 4 {
        return None;
    }

    Some(out)
}

/// spec: S-49
/// Reconstructs a pixel byte using the Paeth predictor.
fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let a = a as i32;
    let b = b as i32;
    let c = c as i32;
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_truncated_input() {
        let mut canvas = Canvas::new(2, 2);
        canvas.pixels[0] = 0xFFFF0000;
        let png_bytes = encode_png(&canvas);
        assert!(decode_png(&png_bytes[0..png_bytes.len() - 10]).is_none());
    }
}
