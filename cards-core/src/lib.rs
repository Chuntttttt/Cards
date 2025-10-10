use printpdf::*;
use std::fmt;
use std::fs;

/// Error types for card PDF generation
#[derive(Debug)]
pub enum CardsError {
    IoError(std::io::Error),
    ImageDecodeError(String),
    DirectoryNotFound(String),
    NoImagesFound(String),
    PdfGenerationError(String),
}

impl fmt::Display for CardsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CardsError::IoError(e) => write!(f, "IO error: {e}"),
            CardsError::ImageDecodeError(msg) => write!(f, "Image decode error: {msg}"),
            CardsError::DirectoryNotFound(path) => write!(f, "Directory not found: {path}"),
            CardsError::NoImagesFound(path) => write!(f, "No images found in: {path}"),
            CardsError::PdfGenerationError(msg) => write!(f, "PDF generation error: {msg}"),
        }
    }
}

impl std::error::Error for CardsError {}

impl From<std::io::Error> for CardsError {
    fn from(err: std::io::Error) -> Self {
        CardsError::IoError(err)
    }
}

/// Core PDF generation logic for card sheets
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
    /// * `side_size` - Grid size (e.g., 3 for 3x3 grid)
    pub fn new(cards_path: String, side_size: usize) -> Self {
        let width = 612.0; // 8.5" at 72 points per inch
        let height = 792.0; // 11" at 72 points per inch

        let horizontal_padding = width * (1.0 / 17.0);
        let vertical_padding = height * (1.0 / 44.0);
        let card_width = (width - (horizontal_padding * 2.0)) / side_size as f32;
        let card_height = (height - (vertical_padding * 2.0)) / side_size as f32;

        CardWriter {
            width,
            height,
            side_size,
            horizontal_padding,
            vertical_padding,
            card_width,
            card_height,
            cards_path,
        }
    }

    fn images_from_path(&self, images_path: &str) -> Result<Vec<String>, CardsError> {
        let mut images = Vec::new();
        let extensions = ["png", "jpg", "jpeg"];

        let entries = fs::read_dir(images_path)
            .map_err(|_| CardsError::DirectoryNotFound(images_path.to_string()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_str().unwrap_or("")) {
                        images.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        images.sort();
        Ok(images)
    }

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

    fn draw_guides(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<Op> {
        let size = 20.0;
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

        ops.push(line(x0, y0 - size, x0, y0 + size));
        ops.push(line(x0 - size, y0, x0 + size, y0));
        ops.push(line(x1, y0 - size, x1, y0 + size));
        ops.push(line(x1 - size, y0, x1 + size, y0));
        ops.push(line(x0, y1 - size, x0, y1 + size));
        ops.push(line(x0 - size, y1, x0 + size, y1));
        ops.push(line(x1, y1 - size, x1, y1 + size));
        ops.push(line(x1 - size, y1, x1 + size, y1));

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

                    // Calculate DPI to fit image to card dimensions: dpi = image_pixels * 72 / card_points
                    let dpi_x = raw_image.width as f32 * 72.0 / self.card_width;
                    let dpi_y = raw_image.height as f32 * 72.0 / self.card_height;
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

        let difference = front_cards.len() as i32 - back_cards.len() as i32;
        if difference > 0 && !back_cards.is_empty() {
            let last_back = back_cards.last().unwrap().clone();
            for _ in 0..difference {
                back_cards.push(last_back.clone());
            }
        }

        let front_pages = self.group_images(front_cards);
        let back_pages = self.align_back_cards(self.group_images(back_cards));

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
