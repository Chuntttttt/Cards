use eframe::egui;
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 780.0])
            .with_min_inner_size([700.0, 780.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Cards PDF Generator",
        options,
        Box::new(|cc| {
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

            Ok(Box::new(CardsApp::default()))
        }),
    )
}

struct CardsApp {
    cards_path: Option<PathBuf>,
    grid_size: usize,
    output_path: Option<PathBuf>,
    error_message: Option<String>,
    success_message: Option<String>,
    is_generating: bool,
    front_count: Option<usize>,
    back_count: Option<usize>,
}

impl Default for CardsApp {
    fn default() -> Self {
        Self {
            cards_path: None,
            grid_size: 3,
            output_path: Some(PathBuf::from("cards.pdf")),
            error_message: None,
            success_message: None,
            is_generating: false,
            front_count: None,
            back_count: None,
        }
    }
}

impl CardsApp {
    fn count_cards(&mut self) {
        if let Some(path) = &self.cards_path {
            let front_path = path.join("front");
            let back_path = path.join("back");

            self.front_count = Self::count_images_in_dir(&front_path);
            self.back_count = Self::count_images_in_dir(&back_path);
        }
    }

    fn count_images_in_dir(path: &PathBuf) -> Option<usize> {
        let extensions = ["png", "jpg", "jpeg"];
        std::fs::read_dir(path).ok().map(|entries| {
            entries
                .flatten()
                .filter(|entry| {
                    entry.path().is_file()
                        && entry
                            .path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| extensions.contains(&ext))
                            .unwrap_or(false)
                })
                .count()
        })
    }

    fn generate_pdf(&mut self) {
        if let (Some(cards_path), Some(output_path)) = (&self.cards_path, &self.output_path) {
            self.is_generating = true;
            self.error_message = None;
            self.success_message = None;

            match cards_core::CardWriter::new(
                cards_path.to_string_lossy().to_string(),
                self.grid_size,
            ) {
                Ok(writer) => match writer.create_pdf(&output_path.to_string_lossy()) {
                    Ok(_) => {
                        self.success_message = Some(format!(
                            "PDF generated successfully: {}",
                            output_path.display()
                        ));
                    }
                    Err(e) => {
                        self.error_message = Some(format!("PDF generation failed: {e}"));
                    }
                },
                Err(e) => {
                    self.error_message = Some(format!("Invalid configuration: {e}"));
                }
            }

            self.is_generating = false;
        }
    }
}

impl eframe::App for CardsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut folder_changed = false;
        let mut settings_changed = false;

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
                    ui.label(egui::RichText::new("📂 Input Settings").strong().size(18.0));
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Cards Folder:").size(15.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new("📁 Browse...")
                                        .min_size(egui::vec2(100.0, 30.0)),
                                )
                                .clicked()
                                && let Some(path) = rfd::FileDialog::new().pick_folder()
                            {
                                self.cards_path = Some(path);
                                self.error_message = None;
                                self.success_message = None;
                                folder_changed = true;
                            }
                            if let Some(path) = &self.cards_path {
                                ui.label(
                                    egui::RichText::new(path.display().to_string())
                                        .size(14.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new("(not selected)")
                                        .size(14.0)
                                        .italics()
                                        .color(ui.visuals().weak_text_color()),
                                );
                            }
                        });
                    });

                    ui.add_space(12.0);

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
                            settings_changed = true;
                        }
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Output PDF:").size(15.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new("💾 Save As...")
                                        .min_size(egui::vec2(100.0, 30.0)),
                                )
                                .clicked()
                                && let Some(path) = rfd::FileDialog::new()
                                    .add_filter("PDF", &["pdf"])
                                    .set_file_name("cards.pdf")
                                    .save_file()
                            {
                                self.output_path = Some(path);
                                settings_changed = true;
                            }
                            if let Some(path) = &self.output_path {
                                ui.label(
                                    egui::RichText::new(path.display().to_string())
                                        .size(14.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new("(not selected)")
                                        .size(14.0)
                                        .italics()
                                        .color(ui.visuals().weak_text_color()),
                                );
                            }
                        });
                    });
                });

            ui.add_space(20.0);

            egui::Frame::none()
                .fill(ui.visuals().faint_bg_color)
                .inner_margin(egui::Margin::symmetric(20.0, 15.0))
                .rounding(8.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("📊 Generation Info")
                            .strong()
                            .size(18.0),
                    );
                    ui.add_space(10.0);

                    if let (Some(front), Some(back)) = (self.front_count, self.back_count) {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("•")
                                    .size(20.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255)),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{front} front cards, {back} back cards"
                                ))
                                .size(15.0),
                            );
                        });

                        if front > back && back > 0 {
                            let dup_count = front - back;
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
                        let front_pages = front.div_ceil(cards_per_page);
                        let back_pages = front.div_ceil(cards_per_page);
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

                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("•")
                                    .size(20.0)
                                    .color(egui::Color32::from_rgb(100, 200, 150)),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}×{} cards per page",
                                    self.grid_size, self.grid_size
                                ))
                                .size(15.0),
                            );
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("•")
                                    .size(20.0)
                                    .color(ui.visuals().weak_text_color()),
                            );
                            ui.label(
                                egui::RichText::new("Select a cards folder to see card counts")
                                    .size(15.0)
                                    .italics()
                                    .color(ui.visuals().weak_text_color()),
                            );
                        });

                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("•")
                                    .size(20.0)
                                    .color(ui.visuals().weak_text_color()),
                            );
                            ui.label(
                                egui::RichText::new("Duplication info will appear here if needed")
                                    .size(15.0)
                                    .italics()
                                    .color(ui.visuals().weak_text_color()),
                            );
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("📄")
                                    .size(20.0)
                                    .color(ui.visuals().weak_text_color()),
                            );
                            ui.label(
                                egui::RichText::new("Page count will be calculated")
                                    .size(16.0)
                                    .italics()
                                    .color(ui.visuals().weak_text_color()),
                            );
                        });

                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("•")
                                    .size(20.0)
                                    .color(ui.visuals().weak_text_color()),
                            );
                            ui.label(
                                egui::RichText::new("Cards per page based on grid size")
                                    .size(15.0)
                                    .italics()
                                    .color(ui.visuals().weak_text_color()),
                            );
                        });
                    }
                });

            ui.add_space(20.0);

            ui.add_space(10.0);

            ui.vertical_centered(|ui| {
                let can_generate =
                    self.cards_path.is_some() && self.output_path.is_some() && !self.is_generating;
                let pdf_ready = self.success_message.is_some();

                if pdf_ready {
                    let open_button = egui::Button::new(
                        egui::RichText::new("👁 Open PDF")
                            .size(18.0)
                            .color(egui::Color32::WHITE),
                    )
                    .min_size(egui::vec2(200.0, 50.0))
                    .fill(egui::Color32::from_rgb(100, 200, 100));

                    if ui.add(open_button).clicked()
                        && let Some(output_path) = &self.output_path
                    {
                        let _ = opener::open(output_path);
                    }

                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("✓ PDF generated successfully")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(100, 200, 100)),
                    );
                } else {
                    let button_text = if self.is_generating {
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

                    if !can_generate && !self.is_generating {
                        ui.add_space(5.0);
                        ui.label(
                            egui::RichText::new("Please select cards folder and output path")
                                .size(12.0)
                                .italics()
                                .color(ui.visuals().weak_text_color()),
                        );
                    }
                }

                ui.add_space(15.0);

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

        if folder_changed {
            self.count_cards();
        }

        if settings_changed {
            self.success_message = None;
        }
    }
}
