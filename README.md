# Cards

[![macOS build](https://github.com/Chuntttttt/Cards/actions/workflows/macos.yaml/badge.svg)](https://github.com/Chuntttttt/Cards/actions/workflows/macos.yaml) [![Windows build](https://github.com/Chuntttttt/Cards/actions/workflows/windows.yaml/badge.svg)](https://github.com/Chuntttttt/Cards/actions/workflows/windows.yaml) [![Ubuntu](https://github.com/Chuntttttt/Cards/actions/workflows/ubuntu.yaml/badge.svg)](https://github.com/Chuntttttt/Cards/actions/workflows/ubuntu.yaml) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![Ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json)](https://github.com/astral-sh/ruff)


## Installation

### Download Pre-built Executables

Download the latest release for your platform from the [Releases](https://github.com/Chuntttttt/Cards/releases) page. No Python or dependencies required!

### Build from Source

Requires Python 3.12+ and uses [uv](https://github.com/astral-sh/uv) for dependency management:

```bash
# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Clone the repository
git clone https://github.com/Chuntttttt/Cards.git
cd Cards

# Install dependencies
uv sync --extra dev

# Run the application
uv run python -m cards --help

# Build standalone executable
uv run pyinstaller --onefile cards/__main__.py --name cards
```

Adapted from the MuPDF sample: https://github.com/pymupdf/PyMuPDF-Utilities/blob/master/examples/all-my-pics-embedded.py

## Usage

Cards is a CLI tool, you can use it to turn a folder of the structure:

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

Into a pdf that adds guidelines for cutting the cards and interleaves the front and back cards.

The card images should be 2.5x3.5 (poker card ratio). You can set the number of rows/columns by
passing the 'sides' argument (defaults to 3x3 on each page).

If there are more front cards than there are back cards the last back card will print to be the
back to the remaining unmatched front cards.

Example invocation:

`$ cards --cards-path path/to/cards --output cards.pdf --sides 5`

Example outputs:

![3x3 example](./static/pdf_example.png)

![5x5 example](./static/pdf_example_2.png)