# Cards

Fork from https://github.com/Chuntttttt/Cards

## Requirements

Requires MuPDF, available via pip:

`pip install PyMuPDF`

Adapted the MuPDF sample: https://github.com/pymupdf/PyMuPDF-Utilities/blob/master/examples/all-my-pics-embedded.py

Uses [poetry](https://python-poetry.org) for dependency management.

## Usage

Cards is a python based tool, you can use it to turn a folder of the structure:
<br> Any Directory in `Input` Will be converted into a seperate pdf.

```
input/
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

