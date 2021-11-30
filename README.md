# Cards

[![macOS build](https://github.com/Chuntttttt/Cards/actions/workflows/macos.yaml/badge.svg)](https://github.com/Chuntttttt/Cards/actions/workflows/macos.yaml) [![Windows build](https://github.com/Chuntttttt/Cards/actions/workflows/windows.yaml/badge.svg)](https://github.com/Chuntttttt/Cards/actions/workflows/windows.yaml) [![Ubuntu](https://github.com/Chuntttttt/Cards/actions/workflows/ubuntu.yaml/badge.svg)](https://github.com/Chuntttttt/Cards/actions/workflows/ubuntu.yaml) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![Code style: black](https://img.shields.io/badge/code%20style-black-000000.svg)](https://github.com/psf/black)


## Requirements

Requires MuPDF, available on macOS via homebrew:

`$ brew install mupdf swig freetype`

Adapted the MuPDF sample: https://github.com/pymupdf/PyMuPDF-Utilities/blob/master/examples/all-my-pics-embedded.py

Uses [poetry](https://python-poetry.org) for dependency management.

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

Example:

`$ cards --cards-path path/to/cards --output cards.py --sides 5`
