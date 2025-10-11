# Cards GUI

Cross-platform graphical interface for the Cards PDF generator.

## Features

- Modern, polished UI with rounded panels and color coding
- Browse and select cards directory
- Adjustable grid size (1-20) with visual slider
- Save dialog for PDF output selection
- Card count display with duplication warnings
- Page count calculation
- Large, visible Generate PDF button
- Scrollable interface - button always accessible
- Open generated PDF in system viewer

## Building

```bash
# From workspace root
cargo build --release -p cards-gui
```

## Running

```bash
# With Cargo
cargo run --release -p cards-gui

# Direct execution
./target/release/cards-gui
```

## Usage

1. Click "Browse..." to select your cards directory
   - Directory should contain `front/` and `back/` subdirectories with card images
2. Adjust grid size with the slider (default: 3x3)
3. Review the card count and page calculation
4. Select output PDF location with "Save As..."
5. Click "Generate PDF" to create the PDF
6. Click "✓ Open PDF" to view the generated PDF in your system's PDF viewer

## Distribution

The GUI is a single self-contained binary with no external dependencies. Simply distribute the compiled binary for each platform:

- macOS: `target/release/cards-gui`
- Linux: `target/release/cards-gui`
- Windows: `target/release/cards-gui.exe`

Users can download and run the binary directly - no installation required.

## Architecture

- **GUI Framework**: egui/eframe (immediate mode GUI)
- **File Dialogs**: rfd (native platform dialogs)
- **PDF Generation**: cards-core library
- **File Opener**: opener (cross-platform file/URL opener)

The app state is managed by the `CardsApp` struct which handles:
- Card counting and validation
- PDF file generation
- UI state and user interaction
