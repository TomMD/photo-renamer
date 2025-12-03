use anyhow::Result;
use reverse_geocoder::{ReverseGeocoder, Locations};

use crate::exif_extractor::GpsCoordinates;

pub fn get_location_name(coords: &GpsCoordinates) -> Result<String> {
    let locations = Locations::from_memory();
    let geocoder = ReverseGeocoder::new(&locations);
    
    let search_result = geocoder.search((coords.latitude, coords.longitude));
    
    let location = if let Some(result) = search_result {
        format_location(&result.record)
    } else {
        String::new()
    };
    
    if location.is_empty() {
        Ok(format!(
            "{:08.4}{:09.4}",
            coords.latitude, coords.longitude
        ))
    } else {
        Ok(sanitize_filename(&location))
    }
}

fn format_location(record: &reverse_geocoder::Record) -> String {
    let city = if !record.name.is_empty() {
        Some(&record.name)
    } else {
        None
    };
    
    let region = if !record.admin1.is_empty() && record.admin1 != record.name {
        Some(&record.admin1)
    } else {
        None
    };
    
    let country = if !record.cc.is_empty() {
        Some(&record.cc)
    } else {
        None
    };

    match (city, region, country) {
        (Some(city), Some(region), Some(country)) => {
            format!("{}-{}-{}", city, region, country)
        }
        (Some(city), None, Some(country)) => format!("{}-{}", city, country),
        (None, Some(region), Some(country)) => format!("{}-{}", region, country),
        (Some(city), Some(region), None) => format!("{}-{}", city, region),
        (Some(city), None, None) => city.to_string(),
        (None, Some(region), None) => region.to_string(),
        (None, None, Some(country)) => country.to_string(),
        _ => String::new(),
    }
}

fn sanitize_filename(name: &str) -> String {
    name.replace([' ', '/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}