use clap::Parser;
use printpdf::*;
use std::fs;

#[derive(Parser, Debug)]
#[command(about = "Turn directories of images into printable pdfs of card sheets")]
struct Args {
    #[arg(short = 'c', long, help = "Path to the folder containing the card images")]
    cards_path: String,

    #[arg(short = 'o', long, default_value = "cards.pdf", help = "Path and filename for the output pdf")]
    output: String,

    #[arg(short = 's', long, default_value_t = 3, help = "The number of sides in the grid (ex: 3 would produce a 3x3 grid of cards)")]
    sides: usize,

    #[arg(short = 'v', long, help = "Log actions taken at each step")]
    verbose: bool,
}

struct CardWriter {
    output: String,
    width: f32,
    height: f32,
    side_size: usize,
    horizontal_padding: f32,
    vertical_padding: f32,
    card_width: f32,
    card_height: f32,
    cards_path: String,
    verbose: bool,
}

impl CardWriter {
    fn new(cards_path: String, output: String, side_size: usize, verbose: bool) -> Self {
        // Letter size in points (72 points per inch): 8.5 x 11 inches
        let width = 612.0;  // 8.5 * 72
        let height = 792.0; // 11 * 72

        let horizontal_padding = width * (1.0 / 17.0);
        let vertical_padding = height * (1.0 / 44.0);
        let card_width = (width - (horizontal_padding * 2.0)) / side_size as f32;
        let card_height = (height - (vertical_padding * 2.0)) / side_size as f32;

        CardWriter {
            output,
            width,
            height,
            side_size,
            horizontal_padding,
            vertical_padding,
            card_width,
            card_height,
            cards_path,
            verbose,
        }
    }

    fn images_from_path(&self, images_path: &str) -> Vec<String> {
        let mut images = Vec::new();
        let extensions = ["png", "jpg", "jpeg"];

        if let Ok(entries) = fs::read_dir(images_path) {
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
        }

        images.sort();
        images
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

        // Handle remaining images
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

    fn align_back_cards(&self, back_pages: Vec<Vec<Vec<Option<String>>>>) -> Vec<Vec<Vec<Option<String>>>> {
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

        // Helper to create a line
        let line = |x0: f32, y0: f32, x1: f32, y1: f32| Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point { x: Pt(x0), y: Pt(y0) },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point { x: Pt(x1), y: Pt(y1) },
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        };

        // Corner crosshairs
        ops.push(line(x0, y0 - size, x0, y0 + size));
        ops.push(line(x0 - size, y0, x0 + size, y0));
        ops.push(line(x1, y0 - size, x1, y0 + size));
        ops.push(line(x1 - size, y0, x1 + size, y0));
        ops.push(line(x0, y1 - size, x0, y1 + size));
        ops.push(line(x0 - size, y1, x0 + size, y1));
        ops.push(line(x1, y1 - size, x1, y1 + size));
        ops.push(line(x1 - size, y1, x1 + size, y1));

        // Edge lines based on position
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

    fn create_page_ops(&self, doc: &mut PdfDocument, images: &[Vec<Option<String>>]) -> Vec<Op> {
        let mut ops = Vec::new();

        for (row_index, row) in images.iter().enumerate() {
            for (image_index, image_opt) in row.iter().enumerate() {
                if let Some(image_path) = image_opt {
                    let x0 = image_index as f32 * self.card_width + self.horizontal_padding;
                    let x1 = x0 + self.card_width;
                    let y0 = row_index as f32 * self.card_height + self.vertical_padding;
                    let y1 = y0 + self.card_height;

                    // Load and add image (PDF coordinates are bottom-up, so flip y)
                    if let Ok(image_bytes) = fs::read(image_path) {
                        if let Ok(raw_image) = RawImage::decode_from_bytes(&image_bytes, &mut Vec::new()) {
                            let image_id = doc.add_image(&raw_image);

                            // printpdf defaults to 300 DPI for images
                            // At 300 DPI: 1px = 72/300 = 0.24 pt
                            // Calculate DPI to make image fit card dimensions
                            // We want: image_pixels * (72/dpi) = card_points
                            // So: dpi = image_pixels * 72 / card_points
                            let dpi_x = raw_image.width as f32 * 72.0 / self.card_width;
                            let dpi_y = raw_image.height as f32 * 72.0 / self.card_height;
                            // Use the larger DPI to ensure image fits without distortion
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
                                }
                            });

                            if self.verbose {
                                println!("Added image: {} at ({}, {})", image_path, x0, y0);
                            }
                        }
                    }

                    // Draw cutting guides (convert y to PDF coordinates)
                    ops.extend(self.draw_guides(x0, self.height - y1, x1, self.height - y0));
                }
            }
        }

        ops
    }

    fn create_pdf(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut doc = PdfDocument::new("Cards PDF");

        let front_cards = self.images_from_path(&format!("{}/front", self.cards_path));
        let mut back_cards = self.images_from_path(&format!("{}/back", self.cards_path));

        // Duplicate last back card if needed
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

        // Interleave front and back pages
        for (front_page, back_page) in front_pages.iter().zip(back_pages.iter()) {
            let front_ops = self.create_page_ops(&mut doc, front_page);
            pages.push(PdfPage::new(Mm::from(Pt(self.width)), Mm::from(Pt(self.height)), front_ops));

            let back_ops = self.create_page_ops(&mut doc, back_page);
            pages.push(PdfPage::new(Mm::from(Pt(self.width)), Mm::from(Pt(self.height)), back_ops));
        }

        // Handle remaining front pages if any
        if front_pages.len() > back_pages.len() {
            for front_page in front_pages.iter().skip(back_pages.len()) {
                let front_ops = self.create_page_ops(&mut doc, front_page);
                pages.push(PdfPage::new(Mm::from(Pt(self.width)), Mm::from(Pt(self.height)), front_ops));
            }
        }

        let bytes = doc
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut Vec::new());

        std::fs::write(&self.output, bytes)?;

        if self.verbose {
            println!("PDF saved to: {}", self.output);
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let writer = CardWriter::new(
        args.cards_path,
        args.output,
        args.sides,
        args.verbose,
    );

    writer.create_pdf()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardwriter_dimensions() {
        let writer = CardWriter::new(
            "test_path".to_string(),
            "output.pdf".to_string(),
            3,
            false,
        );

        // Letter size in points
        assert_eq!(writer.width, 612.0);
        assert_eq!(writer.height, 792.0);

        // Verify padding calculations
        assert_eq!(writer.horizontal_padding, 612.0 * (1.0 / 17.0));
        assert_eq!(writer.vertical_padding, 792.0 * (1.0 / 44.0));

        // Verify card dimensions
        let expected_card_width = (612.0 - (612.0 * (1.0 / 17.0) * 2.0)) / 3.0;
        let expected_card_height = (792.0 - (792.0 * (1.0 / 44.0) * 2.0)) / 3.0;
        assert_eq!(writer.card_width, expected_card_width);
        assert_eq!(writer.card_height, expected_card_height);
    }

    #[test]
    fn test_cardwriter_different_grid_sizes() {
        let writer_5x5 = CardWriter::new(
            "test_path".to_string(),
            "output.pdf".to_string(),
            5,
            false,
        );

        assert_eq!(writer_5x5.side_size, 5);

        // Card dimensions should be smaller with more cards per page
        let expected_card_width = (612.0 - (612.0 * (1.0 / 17.0) * 2.0)) / 5.0;
        assert_eq!(writer_5x5.card_width, expected_card_width);
    }

    #[test]
    fn test_images_from_path_sorting() {
        // This tests the sorting behavior - images should be alphabetically sorted
        let writer = CardWriter::new(
            "static".to_string(),
            "output.pdf".to_string(),
            3,
            false,
        );

        let images = writer.images_from_path("static/cards/front");

        // Verify images are sorted (if directory exists and has images)
        if images.len() > 1 {
            for i in 0..images.len() - 1 {
                assert!(images[i] <= images[i + 1], "Images should be sorted alphabetically");
            }
        }
    }

    #[test]
    fn test_group_images_complete_page() {
        let writer = CardWriter::new(
            "test_path".to_string(),
            "output.pdf".to_string(),
            3,
            false,
        );

        // Create exactly 9 images (3x3 = one complete page)
        let images: Vec<String> = (0..9).map(|i| format!("image_{}.png", i)).collect();
        let pages = writer.group_images(images);

        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].len(), 3); // 3 rows
        assert_eq!(pages[0][0].len(), 3); // 3 columns
    }

    #[test]
    fn test_group_images_partial_page() {
        let writer = CardWriter::new(
            "test_path".to_string(),
            "output.pdf".to_string(),
            3,
            false,
        );

        // Create 5 images (should pad to fill 3x3)
        let images: Vec<String> = (0..5).map(|i| format!("image_{}.png", i)).collect();
        let pages = writer.group_images(images);

        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].len(), 3); // Should have 3 rows (padded)

        // Check that we have 5 Some values and 4 None values
        let mut some_count = 0;
        let mut none_count = 0;
        for row in &pages[0] {
            for cell in row {
                if cell.is_some() {
                    some_count += 1;
                } else {
                    none_count += 1;
                }
            }
        }
        assert_eq!(some_count, 5);
        assert_eq!(none_count, 4);
    }

    #[test]
    fn test_group_images_multiple_pages() {
        let writer = CardWriter::new(
            "test_path".to_string(),
            "output.pdf".to_string(),
            3,
            false,
        );

        // Create 15 images (should create 2 pages: one full, one partial)
        let images: Vec<String> = (0..15).map(|i| format!("image_{}.png", i)).collect();
        let pages = writer.group_images(images);

        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].len(), 3); // First page: 3 rows
        assert_eq!(pages[1].len(), 3); // Second page: 3 rows (padded)
    }

    #[test]
    fn test_align_back_cards() {
        let writer = CardWriter::new(
            "test_path".to_string(),
            "output.pdf".to_string(),
            3,
            false,
        );

        // Create a simple page structure
        let pages = vec![
            vec![
                vec![Some("1".to_string()), Some("2".to_string()), Some("3".to_string())],
                vec![Some("4".to_string()), Some("5".to_string()), Some("6".to_string())],
                vec![Some("7".to_string()), Some("8".to_string()), Some("9".to_string())],
            ]
        ];

        let aligned = writer.align_back_cards(pages);

        // Check that rows are reversed
        assert_eq!(aligned[0][0][0], Some("3".to_string()));
        assert_eq!(aligned[0][0][1], Some("2".to_string()));
        assert_eq!(aligned[0][0][2], Some("1".to_string()));

        assert_eq!(aligned[0][1][0], Some("6".to_string()));
        assert_eq!(aligned[0][1][1], Some("5".to_string()));
        assert_eq!(aligned[0][1][2], Some("4".to_string()));
    }

    #[test]
    fn test_draw_guides_returns_ops() {
        let writer = CardWriter::new(
            "test_path".to_string(),
            "output.pdf".to_string(),
            3,
            false,
        );

        let ops = writer.draw_guides(100.0, 100.0, 200.0, 200.0);

        // Should return multiple drawing operations (at least corner crosshairs)
        assert!(!ops.is_empty(), "Should generate drawing operations for guides");
        // 8 crosshair lines minimum (4 corners × 2 lines each)
        assert!(ops.len() >= 8, "Should have at least 8 ops for corner crosshairs");
    }
}
