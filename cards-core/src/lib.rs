//! # Cards Core Library
//!
//! A library for converting directories of card images into printable PDFs with cutting guidelines.
//!
//! ## Overview
//!
//! This library processes front and back card images from directories and arranges them in
//! configurable grids on letter-sized pages (8.5×11 inches). It handles:
//!
//! - Loading and sorting card images from `front/` and `back/` subdirectories
//! - Grouping cards into grids based on the specified side size
//! - Aligning back cards for double-sided printing (rows reversed horizontally)
//! - Generating cutting guidelines with corner crosshairs and edge lines
//! - Producing PDFs suitable for printing and manual card cutting
//!
//! ## Coordinate System
//!
//! PDFs use a bottom-up coordinate system with the origin at the bottom-left corner.
//! This library handles the conversion from top-down layout logic to PDF coordinates
//! by using `self.height - y` when placing images and drawing guides.
//!
//! ## Example
//!
//! ```no_run
//! use cards_core::CardWriter;
//!
//! let writer = CardWriter::new("path/to/cards".to_string(), 3)?;
//! writer.create_pdf("output.pdf")?;
//! # Ok::<(), cards_core::CardsError>(())
//! ```

use printpdf::*;
use std::fs;
use thiserror::Error;

/// Letter-size page width in points (8.5 inches × 72 points/inch)
const PAGE_WIDTH: f32 = 612.0;

/// Letter-size page height in points (11 inches × 72 points/inch)
const PAGE_HEIGHT: f32 = 792.0;

/// Horizontal padding as fraction of page width (approximately 0.5 inches)
const HORIZONTAL_PADDING_RATIO: f32 = 1.0 / 17.0;

/// Vertical padding as fraction of page height (approximately 0.18 inches)
const VERTICAL_PADDING_RATIO: f32 = 1.0 / 44.0;

/// Size of cutting guide crosshairs in points
const GUIDE_LINE_SIZE: f32 = 20.0;

/// DPI conversion factor (points per inch)
const POINTS_PER_INCH: f32 = 72.0;

/// Error types for card PDF generation
#[derive(Error, Debug)]
pub enum CardsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Image decode error: {0}")]
    ImageDecodeError(String),

    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("No images found in: {0}")]
    NoImagesFound(String),

    #[error("PDF generation error: {0}")]
    PdfGenerationError(String),

    #[error("Invalid grid size: {0} (must be between 1 and 20)")]
    InvalidGridSize(usize),
}

/// Core PDF generation logic for card sheets
#[derive(Debug)]
pub struct CardWriter {
    width: f32,
    height: f32,
    side_size: usize,
    horizontal_padding: f32,
    vertical_padding: f32,
    card_width: f32,
    card_height: f32,
    cards_path: String,
}

impl CardWriter {
    /// Create a new CardWriter instance
    ///
    /// # Arguments
    /// * `cards_path` - Path to directory containing front/ and back/ subdirectories
    /// * `side_size` - Grid size (e.g., 3 for 3x3 grid), must be between 1 and 20
    ///
    /// # Returns
    /// * `Ok(CardWriter)` - Successfully created writer
    /// * `Err(CardsError::InvalidGridSize)` - If side_size is 0 or greater than 20
    pub fn new(cards_path: String, side_size: usize) -> Result<Self, CardsError> {
        if side_size == 0 || side_size > 20 {
            return Err(CardsError::InvalidGridSize(side_size));
        }

        let horizontal_padding = PAGE_WIDTH * HORIZONTAL_PADDING_RATIO;
        let vertical_padding = PAGE_HEIGHT * VERTICAL_PADDING_RATIO;
        let card_width = (PAGE_WIDTH - (horizontal_padding * 2.0)) / side_size as f32;
        let card_height = (PAGE_HEIGHT - (vertical_padding * 2.0)) / side_size as f32;

        Ok(CardWriter {
            width: PAGE_WIDTH,
            height: PAGE_HEIGHT,
            side_size,
            horizontal_padding,
            vertical_padding,
            card_width,
            card_height,
            cards_path,
        })
    }

    /// Load image file paths from a directory, filtered by extension and sorted alphabetically.
    ///
    /// Scans the specified directory for image files (png, jpg, jpeg) and returns their
    /// absolute paths in sorted order. This ensures consistent ordering of cards when
    /// generating PDFs.
    ///
    /// # Arguments
    /// * `images_path` - Directory path to scan for image files
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - Sorted list of image file paths
    /// * `Err(CardsError::DirectoryNotFound)` - If the directory doesn't exist or can't be read
    ///
    /// # Error Handling
    /// Uses `.flatten()` to silently skip entries that produce filesystem errors (e.g.,
    /// permission issues). Invalid UTF-8 in file extensions is handled with `unwrap_or("")`.
    fn images_from_path(&self, images_path: &str) -> Result<Vec<String>, CardsError> {
        let mut images = Vec::new();
        let extensions = ["png", "jpg", "jpeg"];

        let entries = fs::read_dir(images_path)
            .map_err(|_| CardsError::DirectoryNotFound(images_path.to_string()))?;

        for entry in entries {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file()
                        && let Some(ext) = path.extension()
                        && extensions.contains(&ext.to_str().unwrap_or(""))
                    {
                        images.push(path.to_string_lossy().to_string());
                    }
                }
                Err(e) => {
                    log::warn!("Skipping entry due to filesystem error: {e}");
                }
            }
        }

        images.sort();
        Ok(images)
    }

    /// Group card images into pages of grids, padding incomplete rows and pages with None.
    ///
    /// Takes a flat list of image paths and organizes them into a 3D structure:
    /// `Vec<Page<Vec<Row<Vec<Option<Image>>>>>`. Each page is a `side_size × side_size` grid
    /// of card positions.
    ///
    /// # Arguments
    /// * `images` - Flat list of image file paths in the order they should appear
    ///
    /// # Returns
    /// A 3D structure where:
    /// - Outer Vec: pages
    /// - Middle Vec: rows within a page
    /// - Inner Vec: card positions within a row
    /// - Option<String>: Some(path) for cards, None for empty positions
    ///
    /// # Padding Behavior
    /// - Incomplete rows are padded with `None` to reach `side_size` width
    /// - Incomplete pages are padded with empty rows (all `None`) to reach `side_size` height
    /// - This ensures all pages have consistent dimensions for PDF layout
    ///
    /// # Example
    /// For `side_size = 3` and 5 images:
    /// ```text
    /// Page 1:
    ///   Row 1: [img1, img2, img3]
    ///   Row 2: [img4, img5, None]
    ///   Row 3: [None, None, None]
    /// ```
    fn group_images(&self, images: Vec<String>) -> Vec<Vec<Vec<Option<String>>>> {
        let mut pages = Vec::new();
        let mut current_page = Vec::new();
        let mut current_row = Vec::new();

        for image in images {
            current_row.push(Some(image));

            if current_row.len() == self.side_size {
                current_page.push(current_row);
                current_row = Vec::new();

                if current_page.len() == self.side_size {
                    pages.push(current_page);
                    current_page = Vec::new();
                }
            }
        }

        if !current_row.is_empty() {
            while current_row.len() < self.side_size {
                current_row.push(None);
            }
            current_page.push(current_row);
        }

        if !current_page.is_empty() {
            while current_page.len() < self.side_size {
                current_page.push(vec![None; self.side_size]);
            }
            pages.push(current_page);
        }

        pages
    }

    /// Reverse each row horizontally to align back cards for double-sided printing.
    ///
    /// When printing double-sided cards, the paper is flipped along the long edge.
    /// This means the back side needs to be horizontally mirrored so that each back
    /// aligns with its corresponding front when viewed through the paper.
    ///
    /// # Arguments
    /// * `back_pages` - Grouped back card pages from `group_images()`
    ///
    /// # Returns
    /// The same page structure with each row reversed
    ///
    /// # Example
    /// ```text
    /// Before (back cards as they appear in directory):
    ///   [back1, back2, back3]
    ///   [back4, back5, back6]
    ///
    /// After (how they should be printed):
    ///   [back3, back2, back1]  <- will align with [front1, front2, front3]
    ///   [back6, back5, back4]  <- will align with [front4, front5, front6]
    /// ```
    fn align_back_cards(
        &self,
        back_pages: Vec<Vec<Vec<Option<String>>>>,
    ) -> Vec<Vec<Vec<Option<String>>>> {
        back_pages
            .into_iter()
            .map(|page| {
                page.into_iter()
                    .map(|row| {
                        let mut reversed: Vec<_> = row.into_iter().collect();
                        reversed.reverse();
                        reversed
                    })
                    .collect()
            })
            .collect()
    }

    /// Generate cutting guide lines for a single card position.
    ///
    /// Creates crosshair marks at the four corners of a card and extends lines to the
    /// page edges for cards positioned near the margins. This helps with precise cutting
    /// when creating physical cards from the printed PDF.
    ///
    /// # Arguments
    /// * `x0`, `y0` - Bottom-left corner coordinates in PDF points (bottom-up coordinate system)
    /// * `x1`, `y1` - Top-right corner coordinates in PDF points
    ///
    /// # Returns
    /// Vector of PDF drawing operations (lines) for the cutting guides
    ///
    /// # Guide Layout
    /// - **Crosshairs**: 20pt cross marks at each of the four corners
    /// - **Edge extensions**: Lines from corners to page edges for marginal cards
    ///   - Bottom edge: if y0 < 20pt
    ///   - Top edge: if y0 > 500pt
    ///   - Left edge: if x0 < 40pt
    ///   - Right edge: if x0 > 390pt
    ///
    /// # Coordinate System
    /// Note that PDF coordinates have origin at bottom-left, so y0 < y1 and higher y
    /// values are toward the top of the page.
    fn draw_guides(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<Op> {
        let mut ops = Vec::new();

        let line = |x0: f32, y0: f32, x1: f32, y1: f32| Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point {
                            x: Pt(x0),
                            y: Pt(y0),
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Pt(x1),
                            y: Pt(y1),
                        },
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        };

        ops.push(line(x0, y0 - GUIDE_LINE_SIZE, x0, y0 + GUIDE_LINE_SIZE));
        ops.push(line(x0 - GUIDE_LINE_SIZE, y0, x0 + GUIDE_LINE_SIZE, y0));
        ops.push(line(x1, y0 - GUIDE_LINE_SIZE, x1, y0 + GUIDE_LINE_SIZE));
        ops.push(line(x1 - GUIDE_LINE_SIZE, y0, x1 + GUIDE_LINE_SIZE, y0));
        ops.push(line(x0, y1 - GUIDE_LINE_SIZE, x0, y1 + GUIDE_LINE_SIZE));
        ops.push(line(x0 - GUIDE_LINE_SIZE, y1, x0 + GUIDE_LINE_SIZE, y1));
        ops.push(line(x1, y1 - GUIDE_LINE_SIZE, x1, y1 + GUIDE_LINE_SIZE));
        ops.push(line(x1 - GUIDE_LINE_SIZE, y1, x1 + GUIDE_LINE_SIZE, y1));

        if y0 < 20.0 {
            ops.push(line(x0, 0.0, x0, y0));
            ops.push(line(x1, 0.0, x1, y0));
        }

        if y0 > 500.0 {
            ops.push(line(x0, self.height, x0, y1));
            ops.push(line(x1, self.height, x1, y1));
        }

        if x0 < 40.0 {
            ops.push(line(0.0, y0, x0, y0));
            ops.push(line(0.0, y1, x0, y1));
        }

        if x0 > 390.0 {
            ops.push(line(x1, y0, self.width, y0));
            ops.push(line(x1, y1, self.width, y1));
        }

        ops
    }

    /// Create PDF drawing operations for a single page of cards.
    ///
    /// Processes a page grid (from `group_images`) and generates the PDF operations needed
    /// to place images and draw cutting guides. Handles coordinate conversion from top-down
    /// layout logic to PDF's bottom-up coordinate system.
    ///
    /// # Arguments
    /// * `doc` - Mutable reference to the PDF document (for adding image resources)
    /// * `images` - 2D grid of card positions (rows × columns), with Some(path) or None
    ///
    /// # Returns
    /// * `Ok(Vec<Op>)` - Vector of PDF operations to render the page
    /// * `Err(CardsError)` - If image reading or decoding fails
    ///
    /// # DPI Calculation
    /// Images are scaled to fit card dimensions using DPI:
    /// ```text
    /// dpi = (image_pixels * 72) / card_points
    /// ```
    /// Uses the maximum of horizontal and vertical DPI to maintain aspect ratio while
    /// ensuring the image covers the full card area.
    ///
    /// # Coordinate Conversion
    /// Layout logic uses top-down coordinates (row 0 at top), but PDF uses bottom-up
    /// coordinates (origin at bottom-left). Conversion: `pdf_y = page_height - layout_y`
    fn create_page_ops(
        &self,
        doc: &mut PdfDocument,
        images: &[Vec<Option<String>>],
    ) -> Result<Vec<Op>, CardsError> {
        let mut ops = Vec::new();

        for (row_index, row) in images.iter().enumerate() {
            for (image_index, image_opt) in row.iter().enumerate() {
                if let Some(image_path) = image_opt {
                    let x0 = image_index as f32 * self.card_width + self.horizontal_padding;
                    let x1 = x0 + self.card_width;
                    let y0 = row_index as f32 * self.card_height + self.vertical_padding;
                    let y1 = y0 + self.card_height;

                    let image_bytes = fs::read(image_path)?;
                    let raw_image = RawImage::decode_from_bytes(&image_bytes, &mut Vec::new())
                        .map_err(|e| {
                            CardsError::ImageDecodeError(format!(
                                "Failed to decode {image_path}: {e:?}"
                            ))
                        })?;

                    let image_id = doc.add_image(&raw_image);

                    let dpi_x = raw_image.width as f32 * POINTS_PER_INCH / self.card_width;
                    let dpi_y = raw_image.height as f32 * POINTS_PER_INCH / self.card_height;
                    let dpi = dpi_x.max(dpi_y);

                    ops.push(Op::UseXobject {
                        id: image_id,
                        transform: XObjectTransform {
                            translate_x: Some(Pt(x0)),
                            translate_y: Some(Pt(self.height - y1)),
                            scale_x: None,
                            scale_y: None,
                            dpi: Some(dpi),
                            ..Default::default()
                        },
                    });

                    log::debug!("Added image: {image_path} at ({x0}, {y0})");

                    ops.extend(self.draw_guides(x0, self.height - y1, x1, self.height - y0));
                }
            }
        }

        Ok(ops)
    }

    /// Generate PDF and write to file
    ///
    /// # Arguments
    /// * `output_path` - File path where PDF should be written
    ///
    /// # Returns
    /// Result indicating success or CardsError
    pub fn create_pdf(&self, output_path: &str) -> Result<(), CardsError> {
        let bytes = self.generate_pdf_bytes()?;
        std::fs::write(output_path, bytes)?;
        log::info!("PDF saved to: {output_path}");
        Ok(())
    }

    /// Generate PDF and return as bytes (for web services)
    ///
    /// # Returns
    /// Vec<u8> containing the complete PDF document
    pub fn generate_pdf_bytes(&self) -> Result<Vec<u8>, CardsError> {
        let mut doc = PdfDocument::new("Cards PDF");

        let front_cards = self.images_from_path(&format!("{}/front", self.cards_path))?;
        let mut back_cards = self.images_from_path(&format!("{}/back", self.cards_path))?;

        log::info!(
            "Found {} front cards, {} back cards",
            front_cards.len(),
            back_cards.len()
        );

        let difference = front_cards.len() as i32 - back_cards.len() as i32;
        if difference > 0
            && let Some(last_back) = back_cards.last()
        {
            log::debug!("Duplicating last back card {difference} times");
            let last_back = last_back.clone();
            for _ in 0..difference {
                back_cards.push(last_back.clone());
            }
        }

        let front_pages = self.group_images(front_cards);
        let back_pages = self.align_back_cards(self.group_images(back_cards));

        log::debug!(
            "Generated {} front pages, {} back pages",
            front_pages.len(),
            back_pages.len()
        );

        let mut pages = Vec::new();

        for (front_page, back_page) in front_pages.iter().zip(back_pages.iter()) {
            let front_ops = self.create_page_ops(&mut doc, front_page)?;
            pages.push(PdfPage::new(
                Mm::from(Pt(self.width)),
                Mm::from(Pt(self.height)),
                front_ops,
            ));

            let back_ops = self.create_page_ops(&mut doc, back_page)?;
            pages.push(PdfPage::new(
                Mm::from(Pt(self.width)),
                Mm::from(Pt(self.height)),
                back_ops,
            ));
        }

        if front_pages.len() > back_pages.len() {
            for front_page in front_pages.iter().skip(back_pages.len()) {
                let front_ops = self.create_page_ops(&mut doc, front_page)?;
                pages.push(PdfPage::new(
                    Mm::from(Pt(self.width)),
                    Mm::from(Pt(self.height)),
                    front_ops,
                ));
            }
        }

        let bytes = doc
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut Vec::new());

        Ok(bytes)
    }
}
