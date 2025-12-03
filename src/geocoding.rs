use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use crate::exif_extractor::GpsCoordinates;

#[derive(Debug, Serialize, Deserialize)]
struct NominatimResponse {
    display_name: Option<String>,
    address: Option<Address>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Address {
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    municipality: Option<String>,
    county: Option<String>,
    state: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
}

pub async fn get_location_name(coords: &GpsCoordinates) -> Result<String> {
    let client = reqwest::Client::new();
    
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}&zoom=10&addressdetails=1",
        coords.latitude, coords.longitude
    );

    sleep(Duration::from_millis(1000)).await;

    let response = client
        .get(&url)
        .header("User-Agent", "PhotoRenamer/0.1.0")
        .send()
        .await
        .context("Failed to send geocoding request")?;

    if response.status().is_success() {
        let geocoding_result: NominatimResponse = response
            .json()
            .await
            .context("Failed to parse geocoding response")?;

        if let Some(address) = geocoding_result.address {
            let location = extract_location_from_address(address);
            if !location.is_empty() {
                return Ok(sanitize_filename(&location));
            }
        }
    }

    Ok(format!(
        "{:08.4}{:09.4}",
        coords.latitude, coords.longitude
    ))
}

fn extract_location_from_address(address: Address) -> String {
    let city = address.city
        .or(address.town)
        .or(address.village)
        .or(address.municipality);
    
    let region = address.state.or(address.county);
    let country = address.country;

    match (city, region, country) {
        (Some(city), Some(region), Some(country)) => {
            if city != region {
                format!("{}-{}-{}", city, region, country)
            } else {
                format!("{}-{}", city, country)
            }
        }
        (Some(city), None, Some(country)) => format!("{}-{}", city, country),
        (None, Some(region), Some(country)) => format!("{}-{}", region, country),
        (None, None, Some(country)) => country,
        (Some(city), Some(region), None) => {
            if city != region {
                format!("{}-{}", city, region)
            } else {
                city
            }
        }
        (Some(city), None, None) => city,
        (None, Some(region), None) => region,
        _ => String::new(),
    }
}

fn sanitize_filename(name: &str) -> String {
    name.replace([' ', '/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}