# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cards is a Rust workspace containing a library and CLI tool for converting directories of card images into printable PDFs with cutting guidelines. It processes front and back card images (poker card ratio: 2.5x3.5) and arranges them in grids for double-sided printing.

### Workspace Structure

- **cards-core**: Library crate containing core PDF generation logic
- **cards-cli**: Binary crate providing the CLI interface

## Development Commands

### Setup
```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p cards-core
cargo build -p cards-cli

# Build optimized release binary
cargo build --release
# Binary will be at: ./target/release/cards
```

### Running the Application
```bash
# Run with Cargo (debug mode)
cargo run -p cards-cli -- --cards-path path/to/cards --output cards.pdf --sides 3

# Run release binary directly
./target/release/cards --cards-path path/to/cards --output cards.pdf --sides 3

# Run with verbose output (enables debug logging)
cargo run -p cards-cli -- --cards-path path/to/cards --output cards.pdf --sides 3 --verbose
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
# Run all tests in workspace
cargo test --workspace

# Test specific crate
cargo test -p cards-core
cargo test -p cards-cli

# Run only library unit tests
cargo test -p cards-core --lib

# Run integration tests
cargo test -p cards-cli --test integration_test

# Run a specific test
cargo test test_generate_pdf_bytes_returns_data
```

### Building for Release
```bash
# Create optimized release binary
cargo build --release

# Binary will be at: ./target/release/cards
```

## Architecture

Workspace architecture with two crates:

### cards-core (Library)
Location: `cards-core/src/lib.rs`

**Public API:**
- `CardsError`: Error enum for library operations (IoError, ImageDecodeError, DirectoryNotFound, NoImagesFound, PdfGenerationError)
- `CardWriter`: Core PDF generation struct

**CardWriter methods:**
- `new(cards_path: String, side_size: usize) -> Self`: Create instance
- `create_pdf(&self, output_path: &str) -> Result<(), CardsError>`: Generate PDF file
- `generate_pdf_bytes(&self) -> Result<Vec<u8>, CardsError>`: Generate PDF as bytes

The library handles:
- Image loading from `cards_path/front/` and `cards_path/back/` directories
- Grid layout on letter-sized pages (8.5×11 inches = 612×792 points)
- Cutting guidelines with corner crosshairs and edge lines
- Back card alignment for double-sided printing (rows reversed horizontally)
- Auto-duplication of the last back card if there are more front cards than back cards
- DPI-based scaling to fit images to card dimensions
- Logging via the `log` crate facade

### cards-cli (Binary)
Location: `cards-cli/src/main.rs`

Provides CLI interface using:
- `clap` for argument parsing
- `env_logger` for logging output
- `cards-core` for PDF generation

The CLI configures logging based on `--verbose` flag and delegates to the library.

### Coordinate System
PDF uses bottom-up coordinates (origin at bottom-left), so y-coordinates are flipped when placing images and drawing guides. Image placement uses `self.height - y1` to convert from top-down layout logic to PDF coordinates.

### Image Processing
1. Images are loaded from `front/` and `back/` subdirectories
2. Files are sorted alphabetically and filtered by extension (png, jpg, jpeg)
3. Images are grouped into pages based on `side_size` (e.g., 3×3 grid)
4. Back pages have rows reversed to align with fronts for double-sided printing
5. Front and back pages are interleaved in the final PDF

## Dependencies

### cards-core
- `printpdf = { version = "0.8", features = ["png", "jpeg"] }` - PDF generation
- `log = "0.4"` - Logging facade

### cards-cli
- `cards-core = { path = "../cards-core" }` - Core library
- `clap = { version = "4.5", features = ["derive"] }` - CLI argument parsing
- `env_logger = "0.11"` - Logging implementation
- `log = "0.4"` - Logging facade

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
