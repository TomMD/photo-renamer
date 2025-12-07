use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::exif_extractor::{extract_metadata, PhotoMetadata, DateTimeSource};
use crate::filename_extractor::extract_date_from_filename;
use crate::geocoding::get_location_name;

pub async fn rename_photo<P: AsRef<Path>>(file_path: P) -> Result<Option<PathBuf>> {
    let path = file_path.as_ref();
    
    let mut metadata = match extract_metadata(path) {
        Ok(meta) => meta,
        Err(_) => {
            // Create empty metadata for filename fallback
            PhotoMetadata {
                datetime: None,
                gps: None,
                datetime_source: DateTimeSource::None,
            }
        }
    };

    // If no EXIF datetime, try extracting from filename
    if metadata.datetime.is_none() {
        if let Ok(Some(filename_date)) = extract_date_from_filename(path) {
            metadata.datetime = Some(filename_date);
            metadata.datetime_source = DateTimeSource::Filename;
            println!("Extracted date from filename for: {}", path.display());
        }
    }

    let new_name = generate_new_filename(&metadata)?;
    
    if let Some(new_filename) = new_name {
        let parent_dir = path.parent().unwrap_or(Path::new("."));
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        
        let new_path = if extension.is_empty() {
            parent_dir.join(new_filename)
        } else {
            parent_dir.join(format!("{}.{}", new_filename, extension))
        };

        if new_path == path {
            println!("File already has correct name: {}", path.display());
            return Ok(None);
        }

        let final_path = ensure_unique_filename(new_path).await?;
        
        fs::rename(path, &final_path)
            .await
            .with_context(|| format!("Failed to rename {} to {}", path.display(), final_path.display()))?;
        
        println!("Renamed: {} -> {}", 
                path.file_name().unwrap().to_string_lossy(),
                final_path.file_name().unwrap().to_string_lossy());
        
        Ok(Some(final_path))
    } else {
        println!("No datetime information found in EXIF or filename for: {}", path.display());
        Ok(None)
    }
}

fn generate_new_filename(metadata: &PhotoMetadata) -> Result<Option<String>> {
    if let Some(datetime) = &metadata.datetime {
        let datetime_str = datetime.format("%Y%m%d%H%M%S").to_string();
        
        if let Some(gps) = &metadata.gps {
            let location = get_location_name(gps)
                .unwrap_or_else(|_| format!("{:08.4}{:09.4}", gps.latitude, gps.longitude));
            Ok(Some(format!("{}-{}", datetime_str, location)))
        } else {
            Ok(Some(datetime_str))
        }
    } else {
        Ok(None)
    }
}

async fn ensure_unique_filename(path: PathBuf) -> Result<PathBuf> {
    let mut counter = 1;
    let original_stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("photo")
        .to_string();
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());
    
    let mut current_path = path;
    
    while current_path.exists() {
        let new_stem = format!("{}-{}", original_stem, counter);
        current_path.set_file_name(&new_stem);
        
        if let Some(ref ext) = extension {
            current_path.set_extension(ext);
        }
        
        counter += 1;
        
        if counter > 1000 {
            anyhow::bail!("Too many files with similar names");
        }
    }
    
    Ok(current_path)
}

pub fn is_image_file<P: AsRef<Path>>(path: P) -> bool {
    if let Some(extension) = path.as_ref().extension() {
        if let Some(ext_str) = extension.to_str() {
            let ext_lower = ext_str.to_lowercase();
            matches!(ext_lower.as_str(), 
                "jpg" | "jpeg" | "tiff" | "tif" | "raw" | "cr2" | "nef" | "arw" | "dng")
        } else {
            false
        }
    } else {
        false
    }
}