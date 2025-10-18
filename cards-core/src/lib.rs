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
    side_size: usize,
    cards_path: String,
}

/// Represents a card image with its data in memory
#[derive(Debug, Clone)]
pub struct CardImage {
    pub name: String,
    pub data: Vec<u8>,
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

        Ok(CardWriter {
            side_size,
            cards_path,
        })
    }

    /// Load images from a directory into memory as CardImage structs.
    ///
    /// Scans the specified directory for image files (png, jpg, jpeg), loads them into
    /// memory, and returns them sorted alphabetically by filename.
    ///
    /// # Arguments
    /// * `images_path` - Directory path to scan for image files
    ///
    /// # Returns
    /// * `Ok(Vec<CardImage>)` - Sorted list of card images with data loaded
    /// * `Err(CardsError::DirectoryNotFound)` - If the directory doesn't exist or can't be read
    /// * `Err(CardsError::NoImagesFound)` - If no valid images found in directory
    fn load_images_from_directory(&self, images_path: &str) -> Result<Vec<CardImage>, CardsError> {
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
                        let name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let data = fs::read(&path)?;
                        images.push(CardImage { name, data });
                    }
                }
                Err(e) => {
                    log::warn!("Skipping entry due to filesystem error: {e}");
                }
            }
        }

        images.sort_by(|a, b| a.name.cmp(&b.name));

        if images.is_empty() {
            return Err(CardsError::NoImagesFound(images_path.to_string()));
        }

        Ok(images)
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
    /// Loads images from the configured cards_path and delegates to generate_pdf_from_images.
    ///
    /// # Returns
    /// Vec<u8> containing the complete PDF document
    pub fn generate_pdf_bytes(&self) -> Result<Vec<u8>, CardsError> {
        let front_images =
            self.load_images_from_directory(&format!("{}/front", self.cards_path))?;
        let back_images = self
            .load_images_from_directory(&format!("{}/back", self.cards_path))
            .unwrap_or_else(|_| Vec::new());

        log::info!(
            "Loaded {} front cards, {} back cards",
            front_images.len(),
            back_images.len()
        );

        Self::generate_pdf_from_images(front_images, back_images, self.side_size)
    }

    /// Generate PDF from in-memory card images (for WASM/web use)
    ///
    /// # Arguments
    /// * `front_images` - Vector of front card images with data
    /// * `back_images` - Vector of back card images with data
    /// * `side_size` - Grid size (e.g., 3 for 3x3 grid)
    ///
    /// # Returns
    /// Vec<u8> containing the complete PDF document
    pub fn generate_pdf_from_images(
        front_images: Vec<CardImage>,
        mut back_images: Vec<CardImage>,
        side_size: usize,
    ) -> Result<Vec<u8>, CardsError> {
        if side_size == 0 || side_size > 20 {
            return Err(CardsError::InvalidGridSize(side_size));
        }

        let horizontal_padding = PAGE_WIDTH * HORIZONTAL_PADDING_RATIO;
        let vertical_padding = PAGE_HEIGHT * VERTICAL_PADDING_RATIO;
        let card_width = (PAGE_WIDTH - (horizontal_padding * 2.0)) / side_size as f32;
        let card_height = (PAGE_HEIGHT - (vertical_padding * 2.0)) / side_size as f32;

        let mut doc = PdfDocument::new("Cards PDF");

        log::info!(
            "Processing {} front cards, {} back cards",
            front_images.len(),
            back_images.len()
        );

        let difference = front_images.len() as i32 - back_images.len() as i32;
        if difference > 0
            && let Some(last_back) = back_images.last()
        {
            log::debug!("Duplicating last back card {difference} times");
            let last_back = last_back.clone();
            for _ in 0..difference {
                back_images.push(last_back.clone());
            }
        }

        let front_pages = Self::group_images_from_memory(&front_images, side_size);
        let back_pages = Self::align_back_cards_from_memory(Self::group_images_from_memory(
            &back_images,
            side_size,
        ));

        log::debug!(
            "Generated {} front pages, {} back pages",
            front_pages.len(),
            back_pages.len()
        );

        let mut pages = Vec::new();

        for (front_page, back_page) in front_pages.iter().zip(back_pages.iter()) {
            let front_ops = Self::create_page_ops_from_memory(
                &mut doc,
                front_page,
                PAGE_WIDTH,
                PAGE_HEIGHT,
                card_width,
                card_height,
                horizontal_padding,
                vertical_padding,
            )?;
            pages.push(PdfPage::new(
                Mm::from(Pt(PAGE_WIDTH)),
                Mm::from(Pt(PAGE_HEIGHT)),
                front_ops,
            ));

            let back_ops = Self::create_page_ops_from_memory(
                &mut doc,
                back_page,
                PAGE_WIDTH,
                PAGE_HEIGHT,
                card_width,
                card_height,
                horizontal_padding,
                vertical_padding,
            )?;
            pages.push(PdfPage::new(
                Mm::from(Pt(PAGE_WIDTH)),
                Mm::from(Pt(PAGE_HEIGHT)),
                back_ops,
            ));
        }

        if front_pages.len() > back_pages.len() {
            for front_page in front_pages.iter().skip(back_pages.len()) {
                let front_ops = Self::create_page_ops_from_memory(
                    &mut doc,
                    front_page,
                    PAGE_WIDTH,
                    PAGE_HEIGHT,
                    card_width,
                    card_height,
                    horizontal_padding,
                    vertical_padding,
                )?;
                pages.push(PdfPage::new(
                    Mm::from(Pt(PAGE_WIDTH)),
                    Mm::from(Pt(PAGE_HEIGHT)),
                    front_ops,
                ));
            }
        }

        let bytes = doc
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut Vec::new());

        Ok(bytes)
    }

    fn group_images_from_memory(
        images: &[CardImage],
        side_size: usize,
    ) -> Vec<Vec<Vec<Option<CardImage>>>> {
        let mut pages = Vec::new();
        let mut current_page = Vec::new();
        let mut current_row = Vec::new();

        for image in images {
            current_row.push(Some(image.clone()));

            if current_row.len() == side_size {
                current_page.push(current_row);
                current_row = Vec::new();

                if current_page.len() == side_size {
                    pages.push(current_page);
                    current_page = Vec::new();
                }
            }
        }

        if !current_row.is_empty() {
            while current_row.len() < side_size {
                current_row.push(None);
            }
            current_page.push(current_row);
        }

        if !current_page.is_empty() {
            while current_page.len() < side_size {
                current_page.push(vec![None; side_size]);
            }
            pages.push(current_page);
        }

        pages
    }

    fn align_back_cards_from_memory(
        back_pages: Vec<Vec<Vec<Option<CardImage>>>>,
    ) -> Vec<Vec<Vec<Option<CardImage>>>> {
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

    fn create_page_ops_from_memory(
        doc: &mut PdfDocument,
        images: &[Vec<Option<CardImage>>],
        width: f32,
        height: f32,
        card_width: f32,
        card_height: f32,
        horizontal_padding: f32,
        vertical_padding: f32,
    ) -> Result<Vec<Op>, CardsError> {
        let mut ops = Vec::new();

        // Draw complete grid first, regardless of whether cards fill all positions
        let side_size = images.len();
        ops.extend(Self::draw_complete_grid_static(
            side_size,
            card_width,
            card_height,
            horizontal_padding,
            vertical_padding,
            width,
            height,
        ));

        // Then place cards
        for (row_index, row) in images.iter().enumerate() {
            for (image_index, image_opt) in row.iter().enumerate() {
                if let Some(image) = image_opt {
                    let x0 = image_index as f32 * card_width + horizontal_padding;
                    let y0 = row_index as f32 * card_height + vertical_padding;
                    let y1 = y0 + card_height;

                    let raw_image = RawImage::decode_from_bytes(&image.data, &mut Vec::new())
                        .map_err(|e| {
                            CardsError::ImageDecodeError(format!(
                                "Failed to decode {}: {e:?}",
                                image.name
                            ))
                        })?;

                    let image_id = doc.add_image(&raw_image);

                    let dpi_x = raw_image.width as f32 * POINTS_PER_INCH / card_width;
                    let dpi_y = raw_image.height as f32 * POINTS_PER_INCH / card_height;
                    let dpi = dpi_x.max(dpi_y);

                    ops.push(Op::UseXobject {
                        id: image_id,
                        transform: XObjectTransform {
                            translate_x: Some(Pt(x0)),
                            translate_y: Some(Pt(height - y1)),
                            scale_x: None,
                            scale_y: None,
                            dpi: Some(dpi),
                            ..Default::default()
                        },
                    });

                    log::debug!("Added image: {} at ({x0}, {y0})", image.name);
                }
            }
        }

        Ok(ops)
    }

    fn draw_complete_grid_static(
        side_size: usize,
        card_width: f32,
        card_height: f32,
        horizontal_padding: f32,
        vertical_padding: f32,
        width: f32,
        height: f32,
    ) -> Vec<Op> {
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

        // Draw vertical lines (columns)
        for col in 0..=side_size {
            let x = col as f32 * card_width + horizontal_padding;

            // Draw vertical line from top edge to bottom edge of page
            ops.push(line(x, 0.0, x, height));

            // Draw crosshairs at each intersection
            for row in 0..=side_size {
                let y = row as f32 * card_height + vertical_padding;
                let pdf_y = height - y;

                // Horizontal crosshair
                ops.push(line(x - GUIDE_LINE_SIZE, pdf_y, x + GUIDE_LINE_SIZE, pdf_y));
            }
        }

        // Draw horizontal lines (rows)
        for row in 0..=side_size {
            let y = row as f32 * card_height + vertical_padding;

            // Convert to PDF coordinates (bottom-up)
            let pdf_y = height - y;

            // Draw horizontal line from left edge to right edge of page
            ops.push(line(0.0, pdf_y, width, pdf_y));

            // Vertical crosshairs are already drawn by the vertical lines loop
        }

        ops
    }
}
