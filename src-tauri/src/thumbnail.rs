use image::codecs::webp::WebPEncoder;
use image::{DynamicImage, ImageEncoder, ImageReader};
use fast_image_resize::{self as fir};
use std::num::NonZeroU32;
use std::path::Path;

/// Thumbnail sizes
const GRID_THUMB_SIZE: u32 = 300;
const PREVIEW_THUMB_SIZE: u32 = 1200;

/// Result type for thumbnail operations
#[derive(Debug, Clone)]
pub struct ThumbnailResult {
    pub media_id: String,
    pub thumbnail_path: String,
    pub preview_path: String,
    pub width: u32,
    pub height: u32,
}

/// Generate thumbnails for a single image file
/// Returns paths to the generated grid and preview thumbnails
pub fn generate_thumbnails(
    source_path: &Path,
    output_dir: &Path,
    media_id: &str,
) -> Result<ThumbnailResult, String> {
    // Construct output paths
    let thumb_path = output_dir.join(format!("{}_thumb.webp", media_id));
    let preview_path = output_dir.join(format!("{}_preview.webp", media_id));

    // Skip if both already exist
    if thumb_path.exists() && preview_path.exists() {
        let (w, h) = get_image_dimensions(source_path).unwrap_or((0, 0));
        return Ok(ThumbnailResult {
            media_id: media_id.to_string(),
            thumbnail_path: thumb_path.to_string_lossy().to_string(),
            preview_path: preview_path.to_string_lossy().to_string(),
            width: w,
            height: h,
        });
    }

    // Load the source image
    let img = ImageReader::open(source_path)
        .map_err(|e| format!("Failed to open image {}: {}", source_path.display(), e))?
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess format: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image {}: {}", source_path.display(), e))?;

    let orig_width = img.width();
    let orig_height = img.height();

    // Generate grid thumbnail
    if !thumb_path.exists() {
        resize_and_save(&img, GRID_THUMB_SIZE, &thumb_path)?;
    }

    // Generate preview thumbnail
    if !preview_path.exists() {
        resize_and_save(&img, PREVIEW_THUMB_SIZE, &preview_path)?;
    }

    Ok(ThumbnailResult {
        media_id: media_id.to_string(),
        thumbnail_path: thumb_path.to_string_lossy().to_string(),
        preview_path: preview_path.to_string_lossy().to_string(),
        width: orig_width,
        height: orig_height,
    })
}

/// Resize an image to fit within max_size (preserving aspect ratio) and save as WebP
fn resize_and_save(img: &DynamicImage, max_size: u32, output_path: &Path) -> Result<(), String> {
    let (orig_w, orig_h) = (img.width(), img.height());

    // Calculate target dimensions preserving aspect ratio
    let (target_w, target_h) = if orig_w > orig_h {
        let h = (max_size as f64 * orig_h as f64 / orig_w as f64) as u32;
        (max_size, h.max(1))
    } else {
        let w = (max_size as f64 * orig_w as f64 / orig_h as f64) as u32;
        (w.max(1), max_size)
    };

    // Don't upscale — if image is smaller than target, just copy
    if orig_w <= target_w && orig_h <= target_h {
        save_as_webp(img, output_path)?;
        return Ok(());
    }

    // Use fast_image_resize for SIMD-accelerated resizing
    let src_image = fir::images::Image::from_vec_u8(
        orig_w,
        orig_h,
        img.to_rgba8().into_raw(),
        fir::PixelType::U8x4,
    )
    .map_err(|e| format!("Failed to create source image: {}", e))?;

    let mut dst_image = fir::images::Image::new(
        target_w,
        target_h,
        fir::PixelType::U8x4,
    );

    let mut resizer = fir::Resizer::new();
    resizer
        .resize(
            &src_image,
            &mut dst_image,
            Some(&fir::ResizeOptions::new().resize_alg(fir::ResizeAlg::Convolution(
                fir::FilterType::Lanczos3,
            ))),
        )
        .map_err(|e| format!("Failed to resize: {}", e))?;

    // Convert back to DynamicImage and save
    let resized = DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(target_w, target_h, dst_image.into_vec())
            .ok_or("Failed to create resized image buffer")?,
    );

    save_as_webp(&resized, output_path)
}

/// Save a DynamicImage as WebP
fn save_as_webp(img: &DynamicImage, path: &Path) -> Result<(), String> {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    let file = std::fs::File::create(path)
        .map_err(|e| format!("Failed to create file {}: {}", path.display(), e))?;
    let writer = std::io::BufWriter::new(file);

    let encoder = WebPEncoder::new_lossless(writer);
    encoder
        .write_image(rgba.as_raw(), w, h, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("Failed to encode WebP: {}", e))?;

    Ok(())
}

/// Get image dimensions without fully decoding
fn get_image_dimensions(path: &Path) -> Result<(u32, u32), String> {
    let reader = ImageReader::open(path)
        .map_err(|e| format!("Failed to open: {}", e))?
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess format: {}", e))?;

    let dims = reader
        .into_dimensions()
        .map_err(|e| format!("Failed to read dimensions: {}", e))?;

    Ok(dims)
}

/// Generate a placeholder thumbnail for video files
pub fn generate_video_placeholder(
    output_dir: &Path,
    media_id: &str,
) -> Result<ThumbnailResult, String> {
    let thumb_path = output_dir.join(format!("{}_thumb.webp", media_id));
    let preview_path = output_dir.join(format!("{}_preview.webp", media_id));

    // Create a simple dark placeholder image
    let placeholder = DynamicImage::new_rgba8(GRID_THUMB_SIZE, GRID_THUMB_SIZE);
    save_as_webp(&placeholder, &thumb_path)?;
    save_as_webp(&placeholder, &preview_path)?;

    Ok(ThumbnailResult {
        media_id: media_id.to_string(),
        thumbnail_path: thumb_path.to_string_lossy().to_string(),
        preview_path: preview_path.to_string_lossy().to_string(),
        width: 0,
        height: 0,
    })
}
