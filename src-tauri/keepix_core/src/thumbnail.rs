use image::{DynamicImage, ImageEncoder, ImageReader};
use fast_image_resize::{self as fir};
use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use exif::{Reader as ExifReader, Tag, Value};

/// Thumbnail sizes
const GRID_THUMB_SIZE: u32 = 300;
const PREVIEW_THUMB_SIZE: u32 = 1200;

/// Result type for thumbnail operations
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThumbnailResult {
    pub media_id: String,
    pub thumbnail_path: String,
    pub preview_path: String,
    pub width: u32,
    pub height: u32,
}

/// Try to extract embedded JPEG preview from EXIF metadata offsets
fn extract_raw_jpeg(source_path: &Path) -> Option<Vec<u8>> {
    let file = File::open(source_path).ok()?;
    let mut reader = BufReader::new(file);
    let exif = ExifReader::new().read_from_container(&mut reader).ok()?;

    let mut best_offset = None;
    let mut best_length = None;

    for field in exif.fields() {
        if field.tag == Tag::JPEGInterchangeFormat {
            let offset = match &field.value {
                Value::Long(v) => v.first().copied(),
                Value::Short(v) => v.first().map(|&x| x as u32),
                _ => None,
            };
            if let Some(off) = offset {
                // Find matching length tag in same IFD
                let mut len = None;
                for f2 in exif.fields() {
                    if f2.tag == Tag::JPEGInterchangeFormatLength && f2.ifd_num == field.ifd_num {
                        len = match &f2.value {
                            Value::Long(v) => v.first().copied(),
                            Value::Short(v) => v.first().map(|&x| x as u32),
                            _ => None,
                        };
                        break;
                    }
                }
                
                // Fallback to any length tag if IFD mismatch
                if len.is_none() {
                    for f2 in exif.fields() {
                        if f2.tag == Tag::JPEGInterchangeFormatLength {
                            len = match &f2.value {
                                Value::Long(v) => v.first().copied(),
                                Value::Short(v) => v.first().map(|&x| x as u32),
                                _ => None,
                            };
                            break;
                        }
                    }
                }

                if let Some(l) = len {
                    if best_length.is_none() || l > best_length.unwrap() {
                        best_offset = Some(off);
                        best_length = Some(l);
                    }
                }
            }
        }
    }

    if let (Some(off), Some(len)) = (best_offset, best_length) {
        let mut file = File::open(source_path).ok()?;
        file.seek(SeekFrom::Start(off as u64)).ok()?;
        let mut jpeg_data = vec![0u8; len as usize];
        file.read_exact(&mut jpeg_data).ok()?;
        return Some(jpeg_data);
    }

    None
}

/// Fallback byte scanner to find embedded JPEG starts/ends in container (e.g. CR3)
fn scan_for_embedded_jpeg(source_path: &Path) -> Option<Vec<u8>> {
    let file = File::open(source_path).ok()?;
    let metadata = file.metadata().ok()?;
    let size = metadata.len();
    
    // Only scan up to 25MB to avoid excessive memory usage
    let scan_size = std::cmp::min(size, 25 * 1024 * 1024);
    let mut data = Vec::with_capacity(scan_size as usize);
    file.take(scan_size).read_to_end(&mut data).ok()?;
    
    let mut best_start = None;
    let mut best_len = 0;
    
    let len_limit = data.len();
    if len_limit < 4 {
        return None;
    }
    
    let mut i = 0;
    while i < len_limit - 4 {
        if data[i] == 0xFF && data[i+1] == 0xD8 && data[i+2] == 0xFF {
            let start = i;
            let mut j = i + 2;
            while j < len_limit - 1 {
                if data[j] == 0xFF && data[j+1] == 0xD9 {
                    let end = j + 2;
                    let len = end - start;
                    if len > best_len {
                        best_len = len;
                        best_start = Some(start);
                    }
                    break;
                }
                j += 1;
            }
            i = j;
        }
        i += 1;
    }
    
    if let Some(start) = best_start {
        if best_len > 1000 {
            return Some(data[start .. start + best_len].to_vec());
        }
    }
    
    None
}

/// Decodes RAW image embedded JPEG or falls back to standard file image reader
fn load_image_advanced(source_path: &Path) -> Result<DynamicImage, String> {
    let ext = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
        
    let is_raw = matches!(
        ext.as_str(),
        "cr2" | "cr3" | "nef" | "arw" | "orf" | "rw2" | "dng" | "raf" | "pef"
    );

    if is_raw {
        if let Some(jpeg_bytes) = extract_raw_jpeg(source_path) {
            if let Ok(img) = image::load_from_memory_with_format(&jpeg_bytes, image::ImageFormat::Jpeg) {
                return Ok(img);
            }
        }
        if let Some(jpeg_bytes) = scan_for_embedded_jpeg(source_path) {
            if let Ok(img) = image::load_from_memory_with_format(&jpeg_bytes, image::ImageFormat::Jpeg) {
                return Ok(img);
            }
        }
    }

    // Default image loader
    let img = ImageReader::open(source_path)
        .map_err(|e| format!("Failed to open image {}: {}", source_path.display(), e))?
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess format: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image {}: {}", source_path.display(), e))?;
        
    Ok(img)
}

/// Helper to get image dimensions from EXIF or file structure
fn get_image_dimensions_advanced(path: &Path) -> (u32, u32) {
    if let Ok(file) = File::open(path) {
        let mut reader = BufReader::new(file);
        if let Ok(exif) = ExifReader::new().read_from_container(&mut reader) {
            let width = exif.get_field(Tag::PixelXDimension, exif::In::PRIMARY)
                .or_else(|| exif.get_field(Tag::ImageWidth, exif::In::PRIMARY))
                .and_then(|f| match &f.value {
                    Value::Long(v) => v.first().copied(),
                    Value::Short(v) => v.first().map(|&x| x as u32),
                    _ => None,
                });
            let height = exif.get_field(Tag::PixelYDimension, exif::In::PRIMARY)
                .or_else(|| exif.get_field(Tag::ImageLength, exif::In::PRIMARY))
                .and_then(|f| match &f.value {
                    Value::Long(v) => v.first().copied(),
                    Value::Short(v) => v.first().map(|&x| x as u32),
                    _ => None,
                });
            if let (Some(w), Some(h)) = (width, height) {
                return (w, h);
            }
        }
    }
    get_image_dimensions(path).unwrap_or((0, 0))
}

/// Generate thumbnails for a single image file (supporting RAW formats)
pub fn generate_thumbnails(
    source_path: &Path,
    output_dir: &Path,
    media_id: &str,
) -> Result<ThumbnailResult, String> {
    // Construct output paths
    let thumb_path = output_dir.join(format!("{}_thumb.jpg", media_id));
    let preview_path = output_dir.join(format!("{}_preview.jpg", media_id));

    // Skip if both already exist
    if thumb_path.exists() && preview_path.exists() {
        let (w, h) = get_image_dimensions_advanced(source_path);
        return Ok(ThumbnailResult {
            media_id: media_id.to_string(),
            thumbnail_path: thumb_path.to_string_lossy().to_string(),
            preview_path: preview_path.to_string_lossy().to_string(),
            width: w,
            height: h,
        });
    }

    // Load the source image (advanced RAW/standard loader)
    let img = load_image_advanced(source_path)?;

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

/// Resize an image to fit within max_size (preserving aspect ratio) and save as JPEG
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
        save_as_jpeg(img, output_path)?;
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

    save_as_jpeg(&resized, output_path)
}

/// Save a DynamicImage as JPEG with quality 80
fn save_as_jpeg(img: &DynamicImage, path: &Path) -> Result<(), String> {
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();

    let file = std::fs::File::create(path)
        .map_err(|e| format!("Failed to create file {}: {}", path.display(), e))?;
    let writer = std::io::BufWriter::new(file);

    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(writer, 80);
    encoder
        .write_image(rgb.as_raw(), w, h, image::ExtendedColorType::Rgb8)
        .map_err(|e| format!("Failed to encode JPEG: {}", e))?;

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
    let thumb_path = output_dir.join(format!("{}_thumb.jpg", media_id));
    let preview_path = output_dir.join(format!("{}_preview.jpg", media_id));

    // Create a simple dark placeholder image
    let placeholder = DynamicImage::new_rgba8(GRID_THUMB_SIZE, GRID_THUMB_SIZE);
    save_as_jpeg(&placeholder, &thumb_path)?;
    save_as_jpeg(&placeholder, &preview_path)?;

    Ok(ThumbnailResult {
        media_id: media_id.to_string(),
        thumbnail_path: thumb_path.to_string_lossy().to_string(),
        preview_path: preview_path.to_string_lossy().to_string(),
        width: 0,
        height: 0,
    })
}
