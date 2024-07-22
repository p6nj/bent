use std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
use std::{
    panic::UnwindSafe,
    sync::{Arc, OnceLock, Weak},
    thread,
};

use dirs::{download_dir, home_dir, picture_dir};
#[cfg(target_arch = "wasm32")]
use egui::mutex::Mutex;
use egui_notify::Toasts;
use files::ImageFile;
#[cfg(target_arch = "wasm32")]
use futures::{Future, FutureExt};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
mod files;
#[cfg(target_arch = "wasm32")]
mod files_wasm;
#[cfg(target_arch = "wasm32")]
use files_wasm as files;

#[cfg(target_arch = "wasm32")]
type FilePickerSharedResult = Arc<Mutex<Option<thread::Result<Option<String>>>>>;

#[cfg(target_arch = "wasm32")]
fn input_file() -> &'static FilePickerSharedResult {
    static INPUT_FILE: OnceLock<FilePickerSharedResult> = OnceLock::new();
    INPUT_FILE.get_or_init(Default::default)
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    files: [Option<ImageFile>; 2],

    #[serde(skip)]
    toasts: Toasts,

    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    input_file_asked: bool,
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
            self.browse(ui);

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

#[cfg(target_arch = "wasm32")]
pub fn spawn_and_collect<F: 'static + Future<Output = Option<String>> + UnwindSafe>(
    future: F,
    arc: Weak<Mutex<Option<thread::Result<Option<String>>>>>,
) {
    wasm_bindgen_futures::spawn_local(async move {
        let future = future.catch_unwind();
        future
            .then(|result| async move {
                if let Some(arc) = arc.upgrade() {
                    arc.lock().replace(result);
                }
            })
            .await;
    });
}

impl TemplateApp {
    #[cfg(target_arch = "wasm32")]
    fn browse(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Browse").clicked() {
                TemplateApp::prompt();
                self.input_file_asked = true;
            }
            if self.input_file_asked {
                if let Some(result) = input_file().lock().as_ref() {
                    self.input_file_asked = false;
                    match result {
                        Ok(maybe_path) => {
                            if let Some(path) = maybe_path {
                                match ImageFile::try_new(path) {
                                    Ok(file) => {
                                        self.files[0] = Some(file);
                                    }
                                    Err(e) => {
                                        self.toasts.error(e.to_string());
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            self.toasts.error(
                                "The file picker just crashed! Can't have shit in Detroit!!!",
                            );
                        }
                    }
                }
            }
            ui.label(
                self.files[0]
                    .clone()
                    .map(|file| file.to_string())
                    .unwrap_or_default(),
            );
        });
    }
    #[cfg(target_arch = "wasm32")]
    fn prompt() {
        use std::panic::AssertUnwindSafe;

        use rfd::AsyncFileDialog;

        spawn_and_collect(
            AssertUnwindSafe(
                AsyncFileDialog::new()
                    .set_title("Input image")
                    .set_directory(working_dir())
                    .add_filter(
                        "images",
                        &ImageFormat::all()
                            .flat_map(ImageFormat::extensions_str)
                            .collect::<Vec<&'static &'static str>>(),
                    )
                    .pick_file()
                    .map(|x| x.map(|file| file.file_name())),
            ),
            Arc::<
                egui::mutex::Mutex<
                    std::option::Option<
                        Result<
                            std::option::Option<std::string::String>,
                            Box<(dyn std::any::Any + std::marker::Send + 'static)>,
                        >,
                    >,
                >,
            >::downgrade(&input_file().clone()),
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn browse(&mut self, ui: &mut egui::Ui) {
        use rfd::FileDialog;

        ui.horizontal(|ui| {
            if ui.button("Browse").clicked() {
                if let Some(path) = FileDialog::new()
                    .set_title("Input image")
                    .set_directory(working_dir())
                    .add_filter(
                        "images",
                        &ImageFormat::all()
                            .flat_map(ImageFormat::extensions_str)
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
    }
}
