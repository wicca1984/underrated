use crate::raster::Canvas;
use std::io::Cursor;

/// A decoded image with RGBA8 pixels.
/// spec: S-19
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

    let bpp = match color_type {
        0 => 1, // Grayscale
        2 => 3, // Truecolor (RGB)
        4 => 2, // Grayscale + Alpha
        6 => 4, // Truecolor + Alpha
        _ => return None,
    };

    let stride = width as usize * bpp;
    let expected_len = height as usize * (1 + stride);

    // Decompress the IDAT data
    // spec: S-74 support fully compressed DEFLATE
    let decompressed_data = inflate(&idat_data, Some(expected_len))?;

    if decompressed_data.len() != expected_len {
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

/// Bit reader for DEFLATE.
/// Reads bits LSB-first from the byte stream.
/// spec: S-74
struct BitReader<'a> {
    bytes: &'a [u8],
    byte_idx: usize,
    bit_offset: u32,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            byte_idx: 2, // Start after zlib header (CMF, FLG)
            bit_offset: 0,
        }
    }

    #[inline]
    fn read_bit(&mut self) -> Option<u8> {
        if self.byte_idx >= self.bytes.len() {
            return None;
        }
        let bit = (self.bytes[self.byte_idx] >> self.bit_offset) & 1;
        self.bit_offset += 1;
        if self.bit_offset == 8 {
            self.bit_offset = 0;
            self.byte_idx += 1;
        }
        Some(bit)
    }

    fn read_bits(&mut self, count: u32) -> Option<u32> {
        if count == 0 {
            return Some(0);
        }
        if count > 32 {
            return None;
        }
        let mut val = 0u32;
        let mut bits_read = 0;
        while bits_read < count {
            let bits_left_in_byte = 8 - self.bit_offset;
            let bits_to_take = std::cmp::min(count - bits_read, bits_left_in_byte);
            if self.byte_idx >= self.bytes.len() {
                return None;
            }
            let byte = self.bytes[self.byte_idx] as u32;
            let mask = (1 << bits_to_take) - 1;
            let part = (byte >> self.bit_offset) & mask;
            val |= part << bits_read;

            self.bit_offset += bits_to_take;
            bits_read += bits_to_take;
            if self.bit_offset == 8 {
                self.bit_offset = 0;
                self.byte_idx += 1;
            }
        }
        Some(val)
    }

    fn align_to_byte(&mut self) {
        if self.bit_offset > 0 {
            self.bit_offset = 0;
            self.byte_idx += 1;
        }
    }
}

/// Node in a Huffman tree.
/// spec: S-74
struct HuffmanNode {
    left: Option<u16>,
    right: Option<u16>,
    symbol: Option<u16>,
}

/// Huffman decoder.
/// spec: S-74
struct HuffmanDecoder {
    nodes: Vec<HuffmanNode>,
}

impl HuffmanDecoder {
    fn new(lengths: &[u8]) -> Option<Self> {
        let mut bl_count = [0u32; 16];
        for &len in lengths {
            if len > 15 {
                return None;
            }
            if len > 0 {
                bl_count[len as usize] += 1;
            }
        }

        let mut code = 0;
        let mut next_code = [0u32; 16];
        for bits in 1..=15 {
            code = (code + bl_count[bits - 1]) << 1;
            next_code[bits] = code;
        }

        let root = HuffmanNode {
            left: None,
            right: None,
            symbol: None,
        };
        let mut nodes = vec![root];

        for (symbol, &len) in lengths.iter().enumerate() {
            if len == 0 {
                continue;
            }
            let len = len as usize;
            let assigned_code = next_code[len];
            next_code[len] += 1;

            let mut node_idx = 0;
            for bit_idx in (0..len).rev() {
                let bit = (assigned_code >> bit_idx) & 1;

                if nodes[node_idx].symbol.is_some() {
                    return None;
                }

                if bit == 0 {
                    if let Some(left) = nodes[node_idx].left {
                        node_idx = left as usize;
                    } else {
                        let new_idx = nodes.len();
                        if new_idx >= 65535 {
                            return None;
                        }
                        nodes[node_idx].left = Some(new_idx as u16);
                        nodes.push(HuffmanNode {
                            left: None,
                            right: None,
                            symbol: None,
                        });
                        node_idx = new_idx;
                    }
                } else {
                    if let Some(right) = nodes[node_idx].right {
                        node_idx = right as usize;
                    } else {
                        let new_idx = nodes.len();
                        if new_idx >= 65535 {
                            return None;
                        }
                        nodes[node_idx].right = Some(new_idx as u16);
                        nodes.push(HuffmanNode {
                            left: None,
                            right: None,
                            symbol: None,
                        });
                        node_idx = new_idx;
                    }
                }
            }

            if nodes[node_idx].symbol.is_some()
                || nodes[node_idx].left.is_some()
                || nodes[node_idx].right.is_some()
            {
                return None;
            }
            nodes[node_idx].symbol = Some(symbol as u16);
        }

        Some(Self { nodes })
    }

    fn decode(&self, reader: &mut BitReader) -> Option<u16> {
        let mut node_idx = 0;
        loop {
            let node = &self.nodes[node_idx];
            if let Some(sym) = node.symbol {
                return Some(sym);
            }
            let bit = reader.read_bit()?;
            let next = if bit == 0 { node.left } else { node.right };
            if let Some(next_idx) = next {
                node_idx = next_idx as usize;
            } else {
                return None;
            }
        }
    }
}

/// Builds the fixed literal/length decoder.
/// spec: S-74
fn get_fixed_literal_decoder() -> HuffmanDecoder {
    let mut lengths = vec![0u8; 288];
    lengths[0..144].fill(8);
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    lengths[280..288].fill(8);
    match HuffmanDecoder::new(&lengths) {
        Some(decoder) => decoder,
        None => HuffmanDecoder { nodes: Vec::new() },
    }
}

/// Builds the fixed distance decoder.
/// spec: S-74
fn get_fixed_distance_decoder() -> HuffmanDecoder {
    let lengths = vec![5u8; 32];
    match HuffmanDecoder::new(&lengths) {
        Some(decoder) => decoder,
        None => HuffmanDecoder { nodes: Vec::new() },
    }
}

/// Mapping of length code to base length and extra bits.
/// spec: S-74
fn get_length_info(code: u16) -> Option<(u32, u32)> {
    match code {
        257 => Some((3, 0)),
        258 => Some((4, 0)),
        259 => Some((5, 0)),
        260 => Some((6, 0)),
        261 => Some((7, 0)),
        262 => Some((8, 0)),
        263 => Some((9, 0)),
        264 => Some((10, 0)),
        265 => Some((11, 1)),
        266 => Some((13, 1)),
        267 => Some((15, 1)),
        268 => Some((17, 1)),
        269 => Some((19, 2)),
        270 => Some((23, 2)),
        271 => Some((27, 2)),
        272 => Some((31, 2)),
        273 => Some((35, 3)),
        274 => Some((43, 3)),
        275 => Some((51, 3)),
        276 => Some((59, 3)),
        277 => Some((67, 4)),
        278 => Some((83, 4)),
        279 => Some((99, 4)),
        280 => Some((115, 4)),
        281 => Some((131, 5)),
        282 => Some((163, 5)),
        283 => Some((195, 5)),
        284 => Some((227, 5)),
        285 => Some((258, 0)),
        _ => None,
    }
}

/// Mapping of distance code to base distance and extra bits.
/// spec: S-74
fn get_distance_info(code: u16) -> Option<(u32, u32)> {
    match code {
        0 => Some((1, 0)),
        1 => Some((2, 0)),
        2 => Some((3, 0)),
        3 => Some((4, 0)),
        4 => Some((5, 1)),
        5 => Some((7, 1)),
        6 => Some((9, 2)),
        7 => Some((13, 2)),
        8 => Some((17, 3)),
        9 => Some((25, 3)),
        10 => Some((33, 4)),
        11 => Some((49, 4)),
        12 => Some((65, 5)),
        13 => Some((97, 5)),
        14 => Some((129, 6)),
        15 => Some((193, 6)),
        16 => Some((257, 7)),
        17 => Some((385, 7)),
        18 => Some((513, 8)),
        19 => Some((769, 8)),
        20 => Some((1025, 9)),
        21 => Some((1537, 9)),
        22 => Some((2049, 10)),
        23 => Some((3073, 10)),
        24 => Some((4097, 11)),
        25 => Some((6145, 11)),
        26 => Some((8193, 12)),
        27 => Some((12289, 12)),
        28 => Some((16385, 13)),
        29 => Some((24577, 13)),
        _ => None,
    }
}

/// Sequence of index permutation for the code lengths of the code length alphabet.
/// spec: S-74
const CODE_LEN_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// Helper to decode a single Huffman-coded DEFLATE block.
/// spec: S-74
fn decode_huffman_block(
    reader: &mut BitReader,
    lit_decoder: &HuffmanDecoder,
    dist_decoder: &HuffmanDecoder,
    out: &mut Vec<u8>,
    limit: Option<usize>,
) -> Option<()> {
    loop {
        let sym = lit_decoder.decode(reader)?;
        match sym {
            0..=255 => {
                if limit.is_some_and(|max_len| out.len() >= max_len) {
                    return None;
                }
                out.push(sym as u8);
            }
            256 => {
                break;
            }
            257..=285 => {
                let (base_len, extra_len_bits) = get_length_info(sym)?;
                let extra_len = reader.read_bits(extra_len_bits)?;
                let length = base_len + extra_len;

                let dist_sym = dist_decoder.decode(reader)?;
                let (base_dist, extra_dist_bits) = get_distance_info(dist_sym)?;
                let extra_dist = reader.read_bits(extra_dist_bits)?;
                let distance = base_dist + extra_dist;

                if distance == 0 || (distance as usize) > out.len() {
                    return None;
                }

                if limit.is_some_and(|max_len| out.len() + length as usize > max_len) {
                    return None;
                }

                let start_idx = out.len() - distance as usize;
                for i in 0..length as usize {
                    let b = out[start_idx + i];
                    out.push(b);
                }
            }
            _ => {
                return None;
            }
        }
    }
    Some(())
}

/// Decompresses a zlib stream containing DEFLATE blocks.
/// spec: S-74
fn inflate(data: &[u8], limit: Option<usize>) -> Option<Vec<u8>> {
    if data.len() < 6 {
        return None;
    }
    // Read zlib header
    let cmf = data[0];
    let flg = data[1];

    // Check zlib header integrity
    if (cmf & 0x0F) != 8 {
        return None;
    }
    if !(cmf as u32 * 256 + flg as u32).is_multiple_of(31) {
        return None;
    }
    if (flg & 0x20) != 0 {
        return None;
    }

    let mut reader = BitReader::new(data);
    let mut out = Vec::new();

    loop {
        let bfinal = reader.read_bit()? != 0;
        let btype = reader.read_bits(2)?;

        match btype {
            0 => {
                // Stored block
                reader.align_to_byte();
                if reader.byte_idx + 4 > data.len() - 4 {
                    return None;
                }
                let len = (reader.bytes[reader.byte_idx] as u16)
                    | ((reader.bytes[reader.byte_idx + 1] as u16) << 8);
                let nlen = (reader.bytes[reader.byte_idx + 2] as u16)
                    | ((reader.bytes[reader.byte_idx + 3] as u16) << 8);
                reader.byte_idx += 4;

                if len != !nlen {
                    return None;
                }
                if reader.byte_idx + len as usize > data.len() - 4 {
                    return None;
                }
                if limit.is_some_and(|max_len| out.len() + len as usize > max_len) {
                    return None;
                }
                out.extend_from_slice(
                    &reader.bytes[reader.byte_idx..reader.byte_idx + len as usize],
                );
                reader.byte_idx += len as usize;
            }
            1 => {
                // Fixed Huffman
                let lit_decoder = get_fixed_literal_decoder();
                let dist_decoder = get_fixed_distance_decoder();
                decode_huffman_block(&mut reader, &lit_decoder, &dist_decoder, &mut out, limit)?;
            }
            2 => {
                // Dynamic Huffman
                let hlit = reader.read_bits(5)? + 257;
                let hdist = reader.read_bits(5)? + 1;
                let hclen = reader.read_bits(4)? + 4;

                if hlit > 286 || hdist > 32 || hclen > 19 {
                    return None;
                }

                let mut cl_lengths = vec![0u8; 19];
                for i in 0..hclen as usize {
                    cl_lengths[CODE_LEN_ORDER[i]] = reader.read_bits(3)? as u8;
                }

                let cl_decoder = HuffmanDecoder::new(&cl_lengths)?;

                let total_codes = (hlit + hdist) as usize;
                let mut lengths = Vec::with_capacity(total_codes);

                while lengths.len() < total_codes {
                    let sym = cl_decoder.decode(&mut reader)?;
                    match sym {
                        0..=15 => {
                            lengths.push(sym as u8);
                        }
                        16 => {
                            if lengths.is_empty() {
                                return None;
                            }
                            let last = *lengths.last()?;
                            let extra = reader.read_bits(2)?;
                            let count = 3 + extra as usize;
                            if lengths.len() + count > total_codes {
                                return None;
                            }
                            lengths.resize(lengths.len() + count, last);
                        }
                        17 => {
                            let extra = reader.read_bits(3)?;
                            let count = 3 + extra as usize;
                            if lengths.len() + count > total_codes {
                                return None;
                            }
                            lengths.resize(lengths.len() + count, 0);
                        }
                        18 => {
                            let extra = reader.read_bits(7)?;
                            let count = 11 + extra as usize;
                            if lengths.len() + count > total_codes {
                                return None;
                            }
                            lengths.resize(lengths.len() + count, 0);
                        }
                        _ => return None,
                    }
                }

                if lengths.len() != total_codes {
                    return None;
                }

                let lit_lengths = &lengths[0..hlit as usize];
                let dist_lengths = &lengths[hlit as usize..];

                let lit_decoder = HuffmanDecoder::new(lit_lengths)?;
                let dist_decoder = HuffmanDecoder::new(dist_lengths)?;

                decode_huffman_block(&mut reader, &lit_decoder, &dist_decoder, &mut out, limit)?;
            }
            _ => {
                return None;
            }
        }

        if bfinal {
            break;
        }
    }

    reader.align_to_byte();
    if reader.byte_idx != data.len() - 4 {
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
