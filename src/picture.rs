//! Pictures in diagrams: the Image annotation keeps its picture as a data URL in `imageData`
//! ("data:image/png;base64,…"), as the original does (app-context.js:1360-1427; ImageView,
//! elements.js). This reads a file into such a URL with its natural size, and decodes one for drawing.

use std::path::Path;

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding.
pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Decode base64, skipping whitespace; None on any other character.
pub fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let (mut acc, mut bits) = (0u32, 0);
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => break,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return None,
        };
        acc = acc << 6 | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits & 0xff) as u8);
        }
    }
    Some(out)
}

/// The media type of a picture file, by its extension: the kinds the original's picker offers
/// (.png, .jpg, .jpeg, .svg).
pub fn media_type(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

/// A picture file as a data URL with its natural size in pixels.
pub fn load_file(path: &Path) -> Result<(String, u32, u32), String> {
    let mime = media_type(path).ok_or_else(|| format!("{} is not a PNG, JPEG or SVG picture.", path.display()))?;
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let (w, h) = natural_size(mime, &bytes).ok_or_else(|| format!("{} cannot be read as a picture.", path.display()))?;
    Ok((format!("data:{mime};base64,{}", base64_encode(&bytes)), w, h))
}

fn natural_size(mime: &str, bytes: &[u8]) -> Option<(u32, u32)> {
    if mime == "image/svg+xml" {
        let tree = resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options::default()).ok()?;
        let s = tree.size();
        return Some((s.width().round() as u32, s.height().round() as u32));
    }
    let img = image::load_from_memory(bytes).ok()?;
    Some((img.width(), img.height()))
}

/// The media type and bytes of a data URL.
pub fn split_data_url(url: &str) -> Option<(String, Vec<u8>)> {
    let rest = url.strip_prefix("data:")?;
    let (head, data) = rest.split_once(',')?;
    let mime = head.split(';').next().unwrap_or("").to_string();
    if head.ends_with(";base64") {
        Some((mime, base64_decode(data)?))
    } else {
        Some((mime, data.as_bytes().to_vec()))
    }
}

/// Decode a data URL into RGBA pixels (an SVG is drawn at its natural size).
pub fn decode(url: &str) -> Option<image::RgbaImage> {
    let (mime, bytes) = split_data_url(url)?;
    if mime == "image/svg+xml" {
        use resvg::{tiny_skia, usvg};
        let tree = usvg::Tree::from_data(&bytes, &usvg::Options::default()).ok()?;
        let s = tree.size();
        let (w, h) = (s.width().ceil().max(1.0) as u32, s.height().ceil().max(1.0) as u32);
        let mut pixmap = tiny_skia::Pixmap::new(w, h)?;
        resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut());
        // tiny-skia keeps premultiplied alpha
        let mut img = image::RgbaImage::new(w, h);
        for (p, c) in img.pixels_mut().zip(pixmap.pixels()) {
            let c = c.demultiply();
            *p = image::Rgba([c.red(), c.green(), c.blue(), c.alpha()]);
        }
        return Some(img);
    }
    image::load_from_memory(&bytes).ok().map(|i| i.to_rgba8())
}

/// The largest side a stereotype's icon picture keeps (image-utils.js:3-4).
pub const ICON_MAX: u32 = 500;

/// A picture file for a stereotype icon's `image`: scaled down to at most 500 pixels on its longer
/// side and stored again as PNG (JPEG for the other kinds, as the original's canvas does), with
/// its new and original sizes (image-utils.js:6-42). A square picture is scaled too, which the
/// original's two `>` tests skip.
pub fn load_icon_picture(path: &Path) -> Result<(String, u32, u32, u32, u32), String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if !matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif") {
        return Err(format!("{} is not a JPEG, PNG or GIF picture.", path.display()));
    }
    let img = image::open(path).map_err(|e| e.to_string())?;
    let (ow, oh) = (img.width(), img.height());
    let (mut w, mut h) = (ow, oh);
    if w.max(h) > ICON_MAX {
        if w >= h {
            h = ((h as f64) * ICON_MAX as f64 / w as f64).round().max(1.0) as u32;
            w = ICON_MAX;
        } else {
            w = ((w as f64) * ICON_MAX as f64 / h as f64).round().max(1.0) as u32;
            h = ICON_MAX;
        }
    }
    let img = if (w, h) == (ow, oh) { img } else { img.resize_exact(w, h, image::imageops::FilterType::Triangle) };
    let png = ext == "png";
    let mut bytes = std::io::Cursor::new(Vec::new());
    let format = if png { image::ImageFormat::Png } else { image::ImageFormat::Jpeg };
    let img = if png { img } else { image::DynamicImage::ImageRgb8(img.to_rgb8()) };
    img.write_to(&mut bytes, format).map_err(|e| e.to_string())?;
    let mime = if png { "image/png" } else { "image/jpeg" };
    Ok((format!("data:{mime};base64,{}", base64_encode(bytes.get_ref())), w, h, ow, oh))
}
