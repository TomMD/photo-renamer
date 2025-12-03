# Photo Renamer

Just a claude-built tool to rename photos based on JFIF dates.

Quickstart:

```
nix run github:tommd/photo-renamer
```

# AI Garbage below

A Rust tool to rename photos based on EXIF metadata. Automatically renames photos to:
- `YYYYMMDD` format if only date information is available
- `YYYYMMDD-LOCATION` format if both date and GPS location are available

## Features

- Extracts date/time from EXIF data (DateTimeOriginal, DateTime, DateTimeDigitized)
- Converts GPS coordinates to location names using OpenStreetMap Nominatim API
- Supports recursive directory processing
- Dry-run mode to preview changes
- Handles duplicate filenames automatically
- Supports common image formats: JPEG, TIFF, RAW files (CR2, NEF, ARW, DNG)

## Usage

```bash
# Rename a single photo
photo-renamer photo.jpg

# Process all photos in a directory
photo-renamer /path/to/photos

# Process directories recursively
photo-renamer -r /path/to/photos

# Dry run to see what would be renamed
photo-renamer -n /path/to/photos

# Show help
photo-renamer --help
```

## Building with Nix

```bash
# Enter development shell
nix develop

# Build the project
nix build

# Run directly
nix run
```

## Building with Cargo

```bash
# Build
cargo build --release

# Run
cargo run -- --help
```

## Examples

Original filename: `IMG_1234.jpg`
- With date only: `20231225.jpg`
- With date and GPS: `20231225-Paris-France.jpg`
- Fallback GPS format: `20231225-48.856613002.3522.jpg`

Files without EXIF data are left unchanged.

## Dependencies

- Network access for geocoding (optional - falls back to coordinates)
- OpenStreetMap Nominatim API for location resolution
