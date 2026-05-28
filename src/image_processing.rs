use image::{imageops::FilterType, DynamicImage, ImageFormat};
use std::io::Cursor;

pub struct VariantInfo {
    pub size_name: String,
    pub filename: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

// (name, max_width, square_crop)
const SIZES: &[(&str, u32, bool)] = &[
    ("thumbnail", 150, true),
    ("medium", 300, false),
    ("large", 1024, false),
];

/// Generate resized variants (and WebP equivalents) for a given image.
/// Returns only variants whose target dimension is smaller than the original.
/// `stem` is the filename without extension (e.g. "1234567890_abc").
pub fn generate_variants(
    original_bytes: &[u8],
    mime_type: &str,
    stem: &str,
) -> Vec<VariantInfo> {
    let img = match image::load_from_memory(original_bytes) {
        Ok(img) => img,
        Err(_) => return vec![],
    };

    let orig_w = img.width();
    let orig_h = img.height();

    let (native_fmt, native_ext) = match mime_type {
        "image/jpeg" | "image/jpg" => (ImageFormat::Jpeg, "jpg"),
        "image/png" => (ImageFormat::Png, "png"),
        _ => return vec![],  // unsupported format
    };

    let mut variants = Vec::new();

    for &(size_name, max_w, crop) in SIZES {
        // Skip if original is already equal or smaller
        if orig_w <= max_w {
            continue;
        }

        let resized = if crop {
            // Square crop centred
            img.resize_to_fill(max_w, max_w, FilterType::Lanczos3)
        } else {
            // Scale to fit within max_w × (proportional height)
            let max_h = (orig_h as f32 * (max_w as f32 / orig_w as f32)) as u32;
            img.resize(max_w, max_h.max(1), FilterType::Lanczos3)
        };

        let new_w = resized.width();
        let new_h = resized.height();

        // Native format variant
        if let Some(info) = encode_variant(&resized, native_fmt, native_ext, stem, size_name, mime_type, new_w, new_h) {
            variants.push(info);
        }

        // WebP variant (best-effort — requires image-webp encoding support)
        let webp_size_name = format!("{size_name}-webp");
        if let Some(info) = encode_variant(&resized, ImageFormat::WebP, "webp", stem, &webp_size_name, "image/webp", new_w, new_h) {
            variants.push(info);
        }
    }

    variants
}

fn encode_variant(
    img: &DynamicImage,
    fmt: ImageFormat,
    ext: &str,
    stem: &str,
    size_name: &str,
    mime_type: &str,
    width: u32,
    height: u32,
) -> Option<VariantInfo> {
    let mut buf = Cursor::new(Vec::<u8>::new());
    img.write_to(&mut buf, fmt).ok()?;
    let bytes = buf.into_inner();
    let filename = format!("{stem}-{size_name}.{ext}");
    let url = format!("/static/uploads/{filename}");
    Some(VariantInfo {
        size_name: size_name.to_string(),
        filename,
        url,
        width,
        height,
        bytes,
        mime_type: mime_type.to_string(),
    })
}