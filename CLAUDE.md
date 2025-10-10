# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cards is a Rust CLI tool that converts directories of card images into printable PDFs with cutting guidelines. It processes front and back card images (poker card ratio: 2.5x3.5) and arranges them in grids for double-sided printing.

## Development Commands

### Setup
```bash
# Build the project
cargo build

# Build optimized release binary
cargo build --release
```

### Running the Application
```bash
# Run with Cargo (debug mode)
cargo run -- --cards-path path/to/cards --output cards.pdf --sides 3

# Run release binary directly
./target/release/cards-rust --cards-path path/to/cards --output cards.pdf --sides 3

# Run with verbose output
cargo run -- --cards-path path/to/cards --output cards.pdf --sides 3 --verbose
```

### Code Quality Tools
```bash
# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check for issues without building
cargo check
```

### Testing
```bash
# Run all tests (unit + integration)
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test

# Run a specific test
cargo test test_group_images_complete_page
```

### Building for Release
```bash
# Create optimized release binary
cargo build --release

# Binary will be at: ./target/release/cards-rust
```

## Architecture

Single-file architecture: all code is in `src/main.rs` (305 lines).

### CardWriter struct
The core PDF generation logic handles:
- Image loading from `cards_path/front/` and `cards_path/back/` directories
- Grid layout on letter-sized pages (8.5×11 inches = 612×792 points)
- Cutting guidelines with corner crosshairs and edge lines
- Back card alignment for double-sided printing (rows reversed horizontally)
- Auto-duplication of the last back card if there are more front cards than back cards
- DPI-based scaling to fit images to card dimensions

### Coordinate System
PDF uses bottom-up coordinates (origin at bottom-left), so y-coordinates are flipped when placing images and drawing guides. Image placement uses `self.height - y1` to convert from top-down layout logic to PDF coordinates.

### Image Processing
1. Images are loaded from `front/` and `back/` subdirectories
2. Files are sorted alphabetically and filtered by extension (png, jpg, jpeg)
3. Images are grouped into pages based on `side_size` (e.g., 3×3 grid)
4. Back pages have rows reversed to align with fronts for double-sided printing
5. Front and back pages are interleaved in the final PDF

## Dependencies

- `printpdf = { version = "0.8", features = ["png", "jpeg"] }` - PDF generation
- `clap = { version = "4.5", features = ["derive"] }` - CLI argument parsing

## Expected Directory Structure for Card Images

```
cards/
    front/
        card_01.png
        card_02.png
        ...
    back/
        back_01.png
        back_02.png
        ...
```

Images are sorted alphabetically and should be in PNG, JPG, or JPEG format.

## CI/CD

GitHub Actions workflows build and test on:
- macOS (`.github/workflows/macos.yaml`)
- Windows (`.github/workflows/windows.yaml`)
- Ubuntu (`.github/workflows/ubuntu.yaml`)
