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
```

### Testing
```bash
# Run tests (when available)
cargo test

# Run with test data
cargo run -- --cards-path static/cards --output test.pdf --sides 3 --verbose
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

## Architecture

The application uses printpdf (MIT licensed) for PDF generation. The main entry point is `src/main.rs`.

Key components:
- **CardWriter struct**: Core PDF generation logic
  - Handles image loading from `cards_path/front/` and `cards_path/back/` directories
  - Creates grids of cards with cutting guidelines
  - Aligns back cards correctly for double-sided printing (reversed horizontally)
  - Automatically duplicates the last back card if there are more front cards than back cards
  - Uses DPI-based scaling to fit images to card dimensions

## Dependencies

The project uses printpdf for PDF generation with PNG and JPEG support:
- `printpdf = { version = "0.8", features = ["png", "jpeg"] }`
- `clap = { version = "4.5", features = ["derive"] }`

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
