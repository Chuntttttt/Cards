# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cards is a Python CLI tool that converts directories of card images into printable PDFs with cutting guidelines. It processes front and back card images (poker card ratio: 2.5x3.5) and arranges them in grids for double-sided printing.

## Development Commands

### Setup
```bash
# Install dependencies using Poetry
poetry install
```

### Running the Application
```bash
# Run the CLI tool directly
poetry run python -m cards --cards-path path/to/cards --output cards.pdf --sides 3

# Or activate the virtual environment first
poetry shell
python -m cards --cards-path path/to/cards --output cards.pdf --sides 3
```

### Testing
```bash
# Run tests with pytest
poetry run pytest

# Run a specific test
poetry run pytest tests/test_cards.py::test_version
```

### Code Quality Tools
```bash
# Type checking with mypy
poetry run mypy cards/

# Code formatting with black
poetry run black cards/ tests/

# Linting with flake8
poetry run flake8 cards/ tests/
```

### Building Standalone Executable
```bash
# Create standalone executable with PyInstaller
poetry run pyinstaller --onefile cards/cards.py
```

## Architecture

The application uses PyMuPDF (fitz) for PDF generation. The main entry point is `cards/__main__.py` which calls the `main()` function from `cards/cards.py`.

Key components:
- **CardWriter class**: Core PDF generation logic in `cards/cards.py`
  - Handles image loading from `cards_path/front/` and `cards_path/back/` directories
  - Creates grids of cards with cutting guidelines
  - Aligns back cards correctly for double-sided printing (reversed horizontally)
  - Automatically duplicates the last back card if there are more front cards than back cards

## Dependencies

The project requires PyMuPDF which depends on system libraries:
- macOS: `brew install mupdf swig freetype`
- Other platforms: See PyMuPDF documentation for installation requirements

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