# WASM Webapp Implementation Plan

## Overview
Create a web-based version of Cards that runs entirely in the browser using WebAssembly. The webapp will be hosted on GitHub Pages at `https://chuntttttt.github.io/Cards/`.

## Architecture

### Core Library Refactor (cards-core)
**Status: In Progress**

Refactor `cards-core` to use in-memory images as the primary implementation:
- `generate_pdf_from_images(Vec<CardImage>, Vec<CardImage>, usize)` - Core implementation
- `create_pdf(&self, output_path)` - Wrapper that loads files and delegates
- `generate_pdf_bytes(&self)` - Wrapper that loads files and delegates

Benefits:
- Single source of truth for PDF generation
- Eliminates ~170 lines of duplicated code
- Makes testing easier (can test with in-memory data)
- WASM and native use same code path

### WASM Crate (cards-wasm)
**Status: Partially Complete**

A web application using:
- `eframe` with WASM target for UI rendering
- `zip` crate for extracting uploaded files
- `wasm-bindgen` for JavaScript interop
- `web-sys` for browser APIs (file download)

User workflow:
1. User creates ZIP file with structure:
   ```
   cards.zip
     front/
       card_01.png
       card_02.png
     back/
       back_01.png
       back_02.png
   ```
2. User drags/drops ZIP onto browser
3. WASM extracts ZIP, loads images into memory
4. User adjusts grid size (1-20)
5. User clicks "Generate PDF"
6. PDF downloads to browser's download folder

### Build System
**Status: Not Started**

Using `trunk` for WASM build:
- `trunk serve` - Local development server
- `trunk build --release` - Production build
- Outputs to `cards-wasm/dist/` directory
- Includes HTML, JS glue code, and WASM binary

### Deployment
**Status: Not Started**

GitHub Actions workflow (`.github/workflows/wasm-deploy.yaml`):
1. Trigger on push to trunk branch
2. Install Rust + wasm32 target + trunk
3. Build WASM with `trunk build --release`
4. Copy `cards-wasm/dist/*` to `docs/` folder
5. Commit and push to trunk (GitHub Pages serves from `docs/`)

Alternative: Deploy to `gh-pages` branch instead of `docs/` folder.

GitHub Pages configuration:
- Repository Settings → Pages → Source: Deploy from branch
- Branch: trunk, folder: /docs (or gh-pages branch)
- URL: `https://chuntttttt.github.io/Cards/`

## Implementation Status

### Completed
- [x] Research egui WASM compatibility
- [x] Create cards-wasm crate structure
- [x] Add ZIP extraction dependencies
- [x] Create initial `generate_pdf_from_images()` method
- [x] Implement WASM GUI with drag-and-drop
- [x] Implement ZIP extraction and image loading
- [x] Implement PDF download via browser API
- [x] Create index.html entry point
- [x] Create Trunk.toml configuration

### In Progress
- [ ] Refactor cards-core to eliminate duplication

### Remaining Tasks
1. **Refactor cards-core**
   - Make `generate_pdf_from_images` the core implementation
   - Convert filesystem methods to thin wrappers
   - Test that CLI and GUI still work

2. **Update workspace**
   - Add cards-wasm to workspace Cargo.toml members

3. **Local testing**
   - Install trunk: `cargo install trunk`
   - Add wasm32 target: `rustup target add wasm32-unknown-unknown`
   - Run: `cd cards-wasm && trunk serve`
   - Test in browser at `http://127.0.0.1:8080`

4. **GitHub Actions workflow**
   - Create `.github/workflows/wasm-deploy.yaml`
   - Configure to build and deploy on push to trunk
   - Decide: `docs/` folder vs `gh-pages` branch

5. **Documentation**
   - Update README with webapp link
   - Add instructions for creating compatible ZIP files
   - Document local development setup

## Technical Details

### WASM Bundle Size
Expected: 2-5 MB (includes egui + printpdf + image processing)
- First load: Downloads full bundle
- Subsequent loads: Cached by browser

### Browser Compatibility
- Modern browsers with WASM support (Chrome, Firefox, Safari, Edge)
- No IE11 support (WASM required)

### File Size Limits
- No hard limits (processing happens client-side)
- Practical limit: Browser memory (~2GB typical)
- Large card sets (1000+ images) may be slow

### Privacy
- All processing happens in browser
- No files uploaded to server
- No data leaves user's machine

## Code Sharing Summary

Shared across all platforms (CLI, GUI, WASM):
- Core PDF generation (~520 lines in cards-core)
- Card grouping and alignment logic
- Cutting guide generation
- Image scaling and DPI calculation

Platform-specific code:
- **CLI** (~40 lines): Argument parsing, file I/O
- **GUI** (~490 lines): Native file dialogs, desktop UI
- **WASM** (~400 lines): ZIP extraction, web file handling, browser download

Total code reuse: ~56% of codebase is shared library code.

## Future Enhancements

Potential improvements (not in current scope):
- Progress indicator for large ZIP files
- Image preview before generating PDF
- Drag-and-drop reordering of cards
- Support for multiple ZIP formats/structures
- Server-side option for very large card sets
- PWA support for offline use
