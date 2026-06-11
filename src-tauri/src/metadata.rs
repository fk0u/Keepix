use exif::{In, Reader as ExifReader, Tag, Value};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Parsed EXIF metadata for display in the UI
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExifData {
    // Camera
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub software: Option<String>,

    // Exposure
    pub focal_length: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub iso: Option<String>,
    pub exposure_compensation: Option<String>,
    pub flash: Option<String>,
    pub metering_mode: Option<String>,
    pub white_balance: Option<String>,

    // Image
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<u16>,
    pub color_space: Option<String>,

    // Date & Time
    pub date_taken: Option<String>,
    pub date_digitized: Option<String>,

    // GPS
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub gps_altitude: Option<f64>,

    // File info (set by caller)
    pub file_size: Option<i64>,
    pub file_format: Option<String>,
}

/// Read EXIF data from an image file
pub fn read_exif(path: &Path) -> ExifData {
    let mut data = ExifData::default();

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return data,
    };

    let reader = BufReader::new(file);
    let exif = match ExifReader::new().read_from_container(&mut std::io::BufReader::new(reader)) {
        Ok(e) => e,
        Err(_) => return data,
    };

    // Camera info
    data.camera_make = get_string_tag(&exif, Tag::Make);
    data.camera_model = get_string_tag(&exif, Tag::Model);
    data.lens_model = get_string_tag(&exif, Tag::LensModel);
    data.software = get_string_tag(&exif, Tag::Software);

    // Exposure settings
    data.focal_length = get_rational_tag_formatted(&exif, Tag::FocalLength, "mm");
    data.aperture = get_fnumber(&exif);
    data.shutter_speed = get_exposure_time(&exif);
    data.iso = get_string_tag(&exif, Tag::PhotographicSensitivity);
    data.exposure_compensation = get_string_tag(&exif, Tag::ExposureBiasValue);
    data.flash = get_string_tag(&exif, Tag::Flash);
    data.metering_mode = get_string_tag(&exif, Tag::MeteringMode);
    data.white_balance = get_string_tag(&exif, Tag::WhiteBalance);

    // Image dimensions from EXIF
    data.width = get_u32_tag(&exif, Tag::PixelXDimension)
        .or_else(|| get_u32_tag(&exif, Tag::ImageWidth));
    data.height = get_u32_tag(&exif, Tag::PixelYDimension)
        .or_else(|| get_u32_tag(&exif, Tag::ImageLength));
    data.orientation = get_u16_tag(&exif, Tag::Orientation);
    data.color_space = get_string_tag(&exif, Tag::ColorSpace);

    // Date/time
    data.date_taken = get_string_tag(&exif, Tag::DateTimeOriginal);
    data.date_digitized = get_string_tag(&exif, Tag::DateTimeDigitized);

    // GPS
    data.gps_latitude = get_gps_coord(&exif, Tag::GPSLatitude, Tag::GPSLatitudeRef);
    data.gps_longitude = get_gps_coord(&exif, Tag::GPSLongitude, Tag::GPSLongitudeRef);
    data.gps_altitude = get_gps_altitude(&exif);

    data
}

/// Extract a string value from an EXIF tag
fn get_string_tag(exif: &exif::Exif, tag: Tag) -> Option<String> {
    exif.get_field(tag, In::PRIMARY)
        .map(|f| f.display_value().with_unit(exif).to_string())
}

/// Extract a u32 value from an EXIF tag
fn get_u32_tag(exif: &exif::Exif, tag: Tag) -> Option<u32> {
    exif.get_field(tag, In::PRIMARY).and_then(|f| {
        match &f.value {
            Value::Long(v) => v.first().copied(),
            Value::Short(v) => v.first().map(|&x| x as u32),
            _ => None,
        }
    })
}

/// Extract a u16 value from an EXIF tag
fn get_u16_tag(exif: &exif::Exif, tag: Tag) -> Option<u16> {
    exif.get_field(tag, In::PRIMARY).and_then(|f| {
        match &f.value {
            Value::Short(v) => v.first().copied(),
            _ => None,
        }
    })
}

/// Format a rational tag with a unit
fn get_rational_tag_formatted(exif: &exif::Exif, tag: Tag, unit: &str) -> Option<String> {
    exif.get_field(tag, In::PRIMARY).and_then(|f| {
        match &f.value {
            Value::Rational(v) => v.first().map(|r| {
                let val = r.num as f64 / r.denom as f64;
                format!("{:.1}{}", val, unit)
            }),
            _ => Some(f.display_value().with_unit(exif).to_string()),
        }
    })
}

/// Get f-number formatted as "f/X.X"
fn get_fnumber(exif: &exif::Exif) -> Option<String> {
    exif.get_field(Tag::FNumber, In::PRIMARY).and_then(|f| {
        match &f.value {
            Value::Rational(v) => v.first().map(|r| {
                let val = r.num as f64 / r.denom as f64;
                format!("f/{:.1}", val)
            }),
            _ => Some(f.display_value().with_unit(exif).to_string()),
        }
    })
}

/// Get exposure time formatted as "1/Xs" or "Xs"
fn get_exposure_time(exif: &exif::Exif) -> Option<String> {
    exif.get_field(Tag::ExposureTime, In::PRIMARY).and_then(|f| {
        match &f.value {
            Value::Rational(v) => v.first().map(|r| {
                if r.num < r.denom {
                    format!("{}/{}s", r.num, r.denom)
                } else {
                    let val = r.num as f64 / r.denom as f64;
                    format!("{:.1}s", val)
                }
            }),
            _ => Some(f.display_value().with_unit(exif).to_string()),
        }
    })
}

/// Parse GPS coordinates from EXIF
fn get_gps_coord(exif: &exif::Exif, coord_tag: Tag, ref_tag: Tag) -> Option<f64> {
    let field = exif.get_field(coord_tag, In::PRIMARY)?;
    let ref_field = exif.get_field(ref_tag, In::PRIMARY)?;

    if let Value::Rational(ref rationals) = field.value {
        if rationals.len() >= 3 {
            let degrees = rationals[0].num as f64 / rationals[0].denom as f64;
            let minutes = rationals[1].num as f64 / rationals[1].denom as f64;
            let seconds = rationals[2].num as f64 / rationals[2].denom as f64;

            let mut coord = degrees + minutes / 60.0 + seconds / 3600.0;

            // Check reference direction (S or W means negative)
            let ref_str = ref_field.display_value().to_string();
            if ref_str.contains('S') || ref_str.contains('W') {
                coord = -coord;
            }

            return Some(coord);
        }
    }

    None
}

/// Parse GPS altitude from EXIF
fn get_gps_altitude(exif: &exif::Exif) -> Option<f64> {
    let field = exif.get_field(Tag::GPSAltitude, In::PRIMARY)?;

    if let Value::Rational(ref rationals) = field.value {
        if let Some(r) = rationals.first() {
            let alt = r.num as f64 / r.denom as f64;

            // Check altitude reference (1 = below sea level)
            let below = exif
                .get_field(Tag::GPSAltitudeRef, In::PRIMARY)
                .and_then(|f| {
                    if let Value::Byte(ref v) = f.value {
                        v.first().map(|&b| b == 1)
                    } else {
                        None
                    }
                })
                .unwrap_or(false);

            return Some(if below { -alt } else { alt });
        }
    }

    None
}
