mod exif_extractor;
mod geocoding;
mod renamer;

use anyhow::Result;
use clap::{Arg, Command};
use std::path::Path;
use walkdir::WalkDir;

use renamer::{is_image_file, rename_photo};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("photo-renamer")
        .version("0.1.0")
        .about("Rename photos based on EXIF date and GPS location data")
        .arg(
            Arg::new("path")
                .help("Path to photo or directory containing photos")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("Process directories recursively")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("dry-run")
                .short('n')
                .long("dry-run")
                .help("Show what would be renamed without actually renaming")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let path_str = matches.get_one::<String>("path").unwrap();
    let recursive = matches.get_flag("recursive");
    let dry_run = matches.get_flag("dry-run");

    let path = Path::new(path_str);

    if !path.exists() {
        eprintln!("Error: Path '{}' does not exist", path_str);
        std::process::exit(1);
    }

    if dry_run {
        println!("DRY RUN MODE - No files will be renamed");
        println!("=====================================");
    }

    let mut processed = 0;
    let mut renamed = 0;

    if path.is_file() {
        if is_image_file(path) {
            processed += 1;
            if !dry_run {
                if let Ok(Some(_)) = rename_photo(path).await {
                    renamed += 1;
                }
            } else {
                println!("Would process: {}", path.display());
            }
        } else {
            eprintln!("Error: '{}' is not a supported image file", path_str);
            std::process::exit(1);
        }
    } else if path.is_dir() {
        let walker = if recursive {
            WalkDir::new(path)
        } else {
            WalkDir::new(path).max_depth(1)
        };

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();
            
            if file_path.is_file() && is_image_file(file_path) {
                processed += 1;
                
                if !dry_run {
                    match rename_photo(file_path).await {
                        Ok(Some(_)) => renamed += 1,
                        Ok(None) => {},
                        Err(e) => eprintln!("Error processing {}: {}", file_path.display(), e),
                    }
                } else {
                    println!("Would process: {}", file_path.display());
                }
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Processed: {} files", processed);
    if !dry_run {
        println!("Renamed: {} files", renamed);
        println!("Skipped: {} files", processed - renamed);
    } else {
        println!("(Dry run - no files were actually renamed)");
    }

    Ok(())
}