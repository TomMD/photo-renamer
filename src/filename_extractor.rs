use anyhow::Result;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use regex::Regex;
use std::path::Path;

pub fn extract_date_from_filename<P: AsRef<Path>>(path: P) -> Result<Option<DateTime<Utc>>> {
    let filename = path.as_ref()
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("");

    // Common patterns for filename dates
    let patterns = [
        // IMG_20240827_123456.jpg -> 20240827
        r"[A-Z]*_?(\d{8})_?\d*",
        // IMG-20240827-123456.jpg -> 20240827  
        r"[A-Z]*-?(\d{8})-?\d*",
        // 20240827_123456.jpg -> 20240827
        r"^(\d{8})_?\d*",
        // DSC_20240827.jpg -> 20240827
        r"[A-Z]*_(\d{8})",
        // Various other common patterns
        r"(\d{4})(\d{2})(\d{2})",
    ];

    for pattern_str in &patterns {
        if let Ok(regex) = Regex::new(pattern_str) {
            if let Some(captures) = regex.captures(filename) {
                if let Some(date_match) = captures.get(1) {
                    let date_str = date_match.as_str();
                    
                    // Try to parse as YYYYMMDD
                    if date_str.len() == 8 {
                        if let Ok(naive_date) = NaiveDate::parse_from_str(date_str, "%Y%m%d") {
                            // Default to noon if no time information
                            let naive_datetime = naive_date.and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap());
                            return Ok(Some(DateTime::from_naive_utc_and_offset(naive_datetime, Utc)));
                        }
                    }
                }
                
                // Try to extract separate year, month, day if pattern has multiple groups
                if captures.len() >= 4 {
                    if let (Some(year), Some(month), Some(day)) = 
                        (captures.get(1), captures.get(2), captures.get(3)) {
                        
                        if let (Ok(y), Ok(m), Ok(d)) = (
                            year.as_str().parse::<i32>(),
                            month.as_str().parse::<u32>(),
                            day.as_str().parse::<u32>()
                        ) {
                            if let Some(naive_date) = NaiveDate::from_ymd_opt(y, m, d) {
                                let naive_datetime = naive_date.and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap());
                                return Ok(Some(DateTime::from_naive_utc_and_offset(naive_datetime, Utc)));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_patterns() {
        // Test IMG_20240827_384785.jpg
        let date = extract_date_from_filename("IMG_20240827_384785.jpg").unwrap();
        assert!(date.is_some());
        let dt = date.unwrap();
        assert_eq!(dt.format("%Y%m%d").to_string(), "20240827");

        // Test DSC_20240827.jpg
        let date = extract_date_from_filename("DSC_20240827.jpg").unwrap();
        assert!(date.is_some());

        // Test 20240827_123456.jpg
        let date = extract_date_from_filename("20240827_123456.jpg").unwrap();
        assert!(date.is_some());

        // Test no match
        let date = extract_date_from_filename("random_file.jpg").unwrap();
        assert!(date.is_none());
    }
}