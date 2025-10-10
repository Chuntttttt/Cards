# Cards

[![macOS build](https://github.com/Chuntttttt/Cards/actions/workflows/macos.yaml/badge.svg)](https://github.com/Chuntttttt/Cards/actions/workflows/macos.yaml) [![Windows build](https://github.com/Chuntttttt/Cards/actions/workflows/windows.yaml/badge.svg)](https://github.com/Chuntttttt/Cards/actions/workflows/windows.yaml) [![Ubuntu](https://github.com/Chuntttttt/Cards/actions/workflows/ubuntu.yaml/badge.svg)](https://github.com/Chuntttttt/Cards/actions/workflows/ubuntu.yaml) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A CLI tool that converts directories of card images into printable PDFs with cutting guidelines. Written in Rust using MIT-licensed [printpdf](https://github.com/fschutt/printpdf) for PDF generation.

## Installation

### Download Pre-built Executables

Download the latest release for your platform from the [Releases](https://github.com/Chuntttttt/Cards/releases) page. No dependencies required!

### Build from Source

Requires Rust 1.70+ and Cargo:

```bash
# Clone the repository
git clone https://github.com/Chuntttttt/Cards.git
cd Cards

# Build release binary
cargo build --release

# The executable will be at ./target/release/cards-rust
# Or run directly with:
cargo run --release -- --help
```

## Usage

Cards is a CLI tool that turns a folder structure like this:

```
cards/
      front/
            card_01.png
            card_02.png
            ...
            card_50.png
      back/
            back_01.png
            back_02.png
            ...
            back_50.png
```

Into a PDF with cutting guidelines and interleaved front/back cards for double-sided printing.

The card images should be 2.5x3.5 (poker card ratio). You can set the number of rows/columns by
passing the `--sides` argument (defaults to 3x3 on each page).

If there are more front cards than back cards, the last back card will be duplicated for the
remaining unmatched front cards.

### Command Line Options

```bash
cards-rust --cards-path <PATH> [OPTIONS]

Options:
  -c, --cards-path <PATH>   Path to the folder containing card images (required)
  -o, --output <FILE>       Output PDF filename [default: cards.pdf]
  -s, --sides <N>           Grid size (e.g., 3 for 3x3) [default: 3]
  -v, --verbose             Show progress messages
  -h, --help                Print help
```

### Example Invocations

```bash
# Generate 3x3 grid PDF
./target/release/cards-rust --cards-path path/to/cards --output cards.pdf

# Generate 5x5 grid PDF with verbose output
./target/release/cards-rust --cards-path path/to/cards --output cards.pdf --sides 5 --verbose
```

### Example Outputs

![3x3 example](./static/pdf_example.png)

![5x5 example](./static/pdf_example_2.png)
