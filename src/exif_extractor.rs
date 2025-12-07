use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use exif::{In, Tag, Value};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GpsCoordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone)]
pub struct PhotoMetadata {
    pub datetime: Option<DateTime<Utc>>,
    pub gps: Option<GpsCoordinates>,
    pub datetime_source: DateTimeSource,
}

#[derive(Debug, Clone)]
pub enum DateTimeSource {
    Exif,
    Filename,
    None,
}

pub fn extract_metadata<P: AsRef<Path>>(path: P) -> Result<PhotoMetadata> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open file: {}", path.as_ref().display()))?;
    
    let mut buf_reader = BufReader::new(&file);
    let exif_reader = exif::Reader::new();
    
    let exif = exif_reader
        .read_from_container(&mut buf_reader)
        .with_context(|| "Failed to read EXIF data")?;

    let datetime = extract_datetime(&exif)?;
    let gps = extract_gps_coordinates(&exif)?;
    
    let datetime_source = if datetime.is_some() {
        DateTimeSource::Exif
    } else {
        DateTimeSource::None
    };

    Ok(PhotoMetadata { datetime, gps, datetime_source })
}

fn extract_datetime(exif_data: &exif::Exif) -> Result<Option<DateTime<Utc>>> {
    let datetime_fields = [
        Tag::DateTimeOriginal,
        Tag::DateTime,
        Tag::DateTimeDigitized,
    ];

    for &tag in &datetime_fields {
        if let Some(field) = exif_data.get_field(tag, In::PRIMARY) {
            if let Value::Ascii(ref vec) = field.value {
                if let Some(datetime_bytes) = vec.first() {
                    let datetime_str = std::str::from_utf8(datetime_bytes)
                        .context("Invalid UTF-8 in datetime field")?;
                    
                    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(
                        datetime_str.trim_end_matches('\0'),
                        "%Y:%m:%d %H:%M:%S"
                    ) {
                        return Ok(Some(DateTime::from_naive_utc_and_offset(naive_dt, Utc)));
                    }
                }
            }
        }
    }

    Ok(None)
}

fn extract_gps_coordinates(exif_data: &exif::Exif) -> Result<Option<GpsCoordinates>> {
    let lat = extract_gps_coordinate(exif_data, Tag::GPSLatitude, Tag::GPSLatitudeRef)?;
    let lon = extract_gps_coordinate(exif_data, Tag::GPSLongitude, Tag::GPSLongitudeRef)?;

    match (lat, lon) {
        (Some(latitude), Some(longitude)) => Ok(Some(GpsCoordinates { latitude, longitude })),
        _ => Ok(None),
    }
}

fn extract_gps_coordinate(
    exif_data: &exif::Exif,
    coord_tag: Tag,
    ref_tag: Tag,
) -> Result<Option<f64>> {
    let coord_field = exif_data.get_field(coord_tag, In::PRIMARY);
    let ref_field = exif_data.get_field(ref_tag, In::PRIMARY);

    if let (Some(coord), Some(ref_val)) = (coord_field, ref_field) {
        let coordinate = match coord.value {
            Value::Rational(ref rationals) if rationals.len() >= 3 => {
                let degrees = rationals[0].to_f64();
                let minutes = rationals[1].to_f64();
                let seconds = rationals[2].to_f64();
                degrees + minutes / 60.0 + seconds / 3600.0
            }
            _ => return Ok(None),
        };

        let reference = match ref_val.value {
            Value::Ascii(ref vec) if !vec.is_empty() => {
                std::str::from_utf8(&vec[0])
                    .context("Invalid UTF-8 in GPS reference")?
                    .chars()
                    .next()
            }
            _ => return Ok(None),
        };

        let signed_coordinate = match reference {
            Some('S') | Some('W') => -coordinate,
            Some('N') | Some('E') => coordinate,
            _ => return Ok(None),
        };

        Ok(Some(signed_coordinate))
    } else {
        Ok(None)
    }
}