use cards_core::{CardImage, CardWriter};
use eframe::egui;
use std::io::{Cursor, Read};
use wasm_bindgen::prelude::*;
use zip::ZipArchive;

#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    wasm_logger::init(wasm_logger::Config::default());

    let web_options = eframe::WebOptions::default();

    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let canvas = document
        .get_element_by_id("the_canvas_id")
        .ok_or("No canvas element")?;
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into()
        .map_err(|_| "Element is not a canvas")?;

    eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(|cc| Ok(Box::new(CardsWasmApp::new(cc)))),
        )
        .await
}

struct CardsWasmApp {
    grid_size: usize,
    error_message: Option<String>,
    success_message: Option<String>,
    is_processing: bool,
    front_count: usize,
    back_count: usize,
    zip_loaded: bool,
    front_images: Vec<CardImage>,
    back_images: Vec<CardImage>,
}

impl CardsWasmApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(26.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(14.0, egui::FontFamily::Monospace),
            ),
        ]
        .into();

        style.spacing.item_spacing = egui::vec2(10.0, 12.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);

        cc.egui_ctx.set_style(style);

        Self {
            grid_size: 3,
            error_message: None,
            success_message: None,
            is_processing: false,
            front_count: 0,
            back_count: 0,
            zip_loaded: false,
            front_images: Vec::new(),
            back_images: Vec::new(),
        }
    }

    fn process_zip(&mut self, zip_bytes: Vec<u8>) {
        self.is_processing = true;
        self.error_message = None;
        self.success_message = None;
        self.front_images.clear();
        self.back_images.clear();

        match self.extract_images_from_zip(zip_bytes) {
            Ok((front, back)) => {
                self.front_images = front;
                self.back_images = back;
                self.front_count = self.front_images.len();
                self.back_count = self.back_images.len();
                self.zip_loaded = true;
                log::info!(
                    "Loaded {} front cards, {} back cards",
                    self.front_count,
                    self.back_count
                );
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to process ZIP: {e}"));
                self.zip_loaded = false;
            }
        }

        self.is_processing = false;
    }

    fn extract_images_from_zip(
        &self,
        zip_bytes: Vec<u8>,
    ) -> Result<(Vec<CardImage>, Vec<CardImage>), String> {
        let cursor = Cursor::new(zip_bytes);
        let mut archive =
            ZipArchive::new(cursor).map_err(|e| format!("Invalid ZIP file: {e}"))?;

        let mut front_images = Vec::new();
        let mut back_images = Vec::new();

        let extensions = ["png", "jpg", "jpeg"];

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("ZIP error: {e}"))?;
            let name = file.name().to_string();

            if file.is_dir() {
                continue;
            }

            let path_parts: Vec<&str> = name.split('/').collect();
            if path_parts.len() < 2 {
                continue;
            }

            let folder = path_parts[path_parts.len() - 2];
            let filename = path_parts[path_parts.len() - 1];

            let ext = filename.split('.').last().unwrap_or("");
            if !extensions.contains(&ext.to_lowercase().as_str()) {
                continue;
            }

            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .map_err(|e| format!("Failed to read {name}: {e}"))?;

            let card = CardImage {
                name: filename.to_string(),
                data,
            };

            if folder == "front" {
                front_images.push(card);
            } else if folder == "back" {
                back_images.push(card);
            }
        }

        front_images.sort_by(|a, b| a.name.cmp(&b.name));
        back_images.sort_by(|a, b| a.name.cmp(&b.name));

        if front_images.is_empty() {
            return Err("No images found in front/ folder".to_string());
        }

        Ok((front_images, back_images))
    }

    fn generate_pdf(&mut self) {
        self.is_processing = true;
        self.error_message = None;
        self.success_message = None;

        match CardWriter::generate_pdf_from_images(
            self.front_images.clone(),
            self.back_images.clone(),
            self.grid_size,
        ) {
            Ok(pdf_bytes) => {
                match self.trigger_download(pdf_bytes, "cards.pdf") {
                    Ok(_) => {
                        self.success_message = Some("PDF generated successfully".to_string());
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Download failed: {e}"));
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("PDF generation failed: {e}"));
            }
        }

        self.is_processing = false;
    }

    fn trigger_download(&self, data: Vec<u8>, filename: &str) -> Result<(), String> {
        use wasm_bindgen::JsCast;
        use web_sys::{Blob, HtmlAnchorElement, Url};

        let window = web_sys::window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;

        let array = js_sys::Uint8Array::from(&data[..]);
        let array_val: &JsValue = array.as_ref();
        let parts = js_sys::Array::new();
        parts.push(array_val);

        let blob_options = web_sys::BlobPropertyBag::new();
        blob_options.set_type("application/pdf");

        let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &blob_options)
            .map_err(|_| "Failed to create blob")?;

        let url = Url::create_object_url_with_blob(&blob).map_err(|_| "Failed to create URL")?;

        let anchor: HtmlAnchorElement = document
            .create_element("a")
            .map_err(|_| "Failed to create anchor")?
            .dyn_into()
            .map_err(|_| "Failed to cast to anchor")?;

        anchor.set_href(&url);
        anchor.set_download(filename);
        anchor.click();

        Url::revoke_object_url(&url).map_err(|_| "Failed to revoke URL")?;

        Ok(())
    }
}

impl eframe::App for CardsWasmApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);

            ui.vertical_centered(|ui| {
                ui.heading("🎴 Cards PDF Generator");
                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new("Create printable card PDFs with cutting guides")
                        .size(14.0)
                        .color(ui.visuals().weak_text_color()),
                );
            });

            ui.add_space(25.0);

            egui::Frame::none()
                .fill(ui.visuals().faint_bg_color)
                .inner_margin(egui::Margin::symmetric(20.0, 15.0))
                .rounding(8.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("📦 Upload ZIP File").strong().size(18.0));
                    ui.add_space(10.0);

                    ui.label(
                        egui::RichText::new(
                            "ZIP should contain front/ and back/ folders with card images",
                        )
                        .size(14.0)
                        .color(ui.visuals().weak_text_color()),
                    );

                    ui.add_space(10.0);

                    let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());

                    if !dropped_files.is_empty() {
                        if let Some(file) = dropped_files.first() {
                            if let Some(bytes) = &file.bytes {
                                self.process_zip(bytes.to_vec());
                            }
                        }
                    }

                    let drop_text = if self.zip_loaded {
                        "✓ ZIP loaded - Drop new file to replace"
                    } else {
                        "Drag and drop ZIP file here"
                    };

                    let drop_color = if self.zip_loaded {
                        egui::Color32::from_rgb(100, 200, 100)
                    } else {
                        ui.visuals().text_color()
                    };

                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(drop_text)
                                .size(16.0)
                                .color(drop_color),
                        );
                    });
                });

            ui.add_space(20.0);

            egui::Frame::none()
                .fill(ui.visuals().faint_bg_color)
                .inner_margin(egui::Margin::symmetric(20.0, 15.0))
                .rounding(8.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("⚙️ Settings").strong().size(18.0));
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Grid Size:").size(15.0));
                        ui.add_space(10.0);
                        let old_grid_size = self.grid_size;
                        let grid_text = format!("{}×{}", self.grid_size, self.grid_size);
                        ui.add(
                            egui::Slider::new(&mut self.grid_size, 1..=20)
                                .text(grid_text)
                                .min_decimals(0),
                        );
                        if self.grid_size != old_grid_size {
                            self.success_message = None;
                        }
                    });
                });

            ui.add_space(20.0);

            if self.zip_loaded {
                egui::Frame::none()
                    .fill(ui.visuals().faint_bg_color)
                    .inner_margin(egui::Margin::symmetric(20.0, 15.0))
                    .rounding(8.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("📊 Card Info")
                                .strong()
                                .size(18.0),
                        );
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("•")
                                    .size(20.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255)),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} front cards, {} back cards",
                                    self.front_count, self.back_count
                                ))
                                .size(15.0),
                            );
                        });

                        if self.front_count > self.back_count && self.back_count > 0 {
                            let dup_count = self.front_count - self.back_count;
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("•")
                                        .size(20.0)
                                        .color(egui::Color32::from_rgb(255, 180, 100)),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Last back card will be duplicated {dup_count} times"
                                    ))
                                    .size(15.0)
                                    .color(ui.visuals().warn_fg_color),
                                );
                            });
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        let cards_per_page = self.grid_size * self.grid_size;
                        let front_pages = self.front_count.div_ceil(cards_per_page);
                        let back_pages = self.front_count.div_ceil(cards_per_page);
                        let total_pages = front_pages + back_pages;

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📄").size(20.0));
                            ui.label(
                                egui::RichText::new(format!("{total_pages} pages total"))
                                    .strong()
                                    .size(16.0),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "({front_pages} front + {back_pages} back)"
                                ))
                                .size(14.0)
                                .color(ui.visuals().weak_text_color()),
                            );
                        });
                    });

                ui.add_space(20.0);
            }

            ui.vertical_centered(|ui| {
                let can_generate = self.zip_loaded && !self.is_processing;

                let button_text = if self.is_processing {
                    "⏳ Generating..."
                } else {
                    "✨ Generate PDF"
                };

                let button_color = if can_generate {
                    egui::Color32::from_rgb(80, 140, 255)
                } else {
                    egui::Color32::from_rgb(100, 100, 120)
                };

                let button = egui::Button::new(
                    egui::RichText::new(button_text)
                        .size(18.0)
                        .color(egui::Color32::WHITE),
                )
                .min_size(egui::vec2(200.0, 50.0))
                .fill(button_color);

                if ui.add_enabled(can_generate, button).clicked() {
                    self.generate_pdf();
                }

                if !can_generate && !self.is_processing {
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new("Please upload a ZIP file first")
                            .size(12.0)
                            .italics()
                            .color(ui.visuals().weak_text_color()),
                    );
                }

                ui.add_space(15.0);

                if let Some(success) = &self.success_message {
                    ui.label(
                        egui::RichText::new(format!("✓ {success}"))
                            .size(15.0)
                            .color(egui::Color32::from_rgb(100, 200, 100)),
                    );
                }

                if let Some(error) = &self.error_message {
                    ui.label(
                        egui::RichText::new(format!("❌ {error}"))
                            .size(15.0)
                            .color(egui::Color32::from_rgb(255, 100, 100)),
                    );
                }
            });

            ui.add_space(20.0);
        });
    }
}
