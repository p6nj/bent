use std::path::PathBuf;

use dirs::{download_dir, home_dir, picture_dir};
use egui_notify::Toasts;
use files::ImageFile;
use image::ImageFormat;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
mod files;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    files: [Option<ImageFile>; 2],

    #[serde(skip)]
    toasts: Toasts,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_zoom_factor(1.1);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     // The top panel is often a good place for a menu bar:

        //     egui::menu::bar(ui, |ui| {
        //         // NOTE: no File->Quit on web pages!
        //         if cfg!(not(target_arch = "wasm32")) {
        //             ui.menu_button("File", |ui| {
        //                 if ui.button("Quit").clicked() {
        //                     ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        //                 }
        //             });
        //             ui.add_space(16.0);
        //         }
        //         ui.with_layout(
        //             egui::Layout::right_to_left(egui::Align::Center),
        //             egui::widgets::global_dark_light_mode_buttons,
        //         )
        //     });
        // });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Bent");
            ui.separator();

            ui.label("Input file:");
            ui.horizontal(|ui| {
                if ui.button("Browse").clicked() {
                    if let Some(path) = FileDialog::new()
                        .set_title("Input image")
                        .set_directory(working_dir())
                        .add_filter(
                            "images",
                            &ImageFormat::all()
                                .map(ImageFormat::extensions_str)
                                .flatten()
                                .collect::<Vec<&'static &'static str>>(),
                        )
                        .pick_file()
                    {
                        match ImageFile::try_new(&path) {
                            Ok(file) => {
                                self.files[0] = Some(file);
                            }
                            Err(e) => {
                                self.toasts.error(e.to_string());
                            }
                        }
                    }
                }
                ui.label(
                    self.files[0]
                        .clone()
                        .map(|file| file.display().to_string())
                        .unwrap_or_default(),
                );
            });

            ui.horizontal(|ui| {
                if ui.button("Alert").clicked() {
                    self.toasts.warning("Warning!");
                }
            });

            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add(egui::github_link_file!(
                    "https://github.com/p6nj/bent/blob/master/",
                    "Source code"
                ));
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
        self.toasts.show(ctx);
    }
}

fn working_dir() -> PathBuf {
    picture_dir()
        .or(download_dir())
        .or(home_dir())
        .unwrap_or("/".into())
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
