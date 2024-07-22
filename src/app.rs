use std::{borrow::Cow, path::PathBuf};
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
#[cfg(target_arch = "wasm32")]
use log::error;
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
mod files;
#[cfg(target_arch = "wasm32")]
mod files_wasm;
#[cfg(target_arch = "wasm32")]
use files_wasm as files;
use strum::{Display, EnumIter, IntoEnumIterator};

#[cfg(target_arch = "wasm32")]
type FilePickerSharedResult = Arc<Mutex<Option<thread::Result<Option<String>>>>>;

#[cfg(target_arch = "wasm32")]
fn input_file() -> &'static FilePickerSharedResult {
    static INPUT_FILE: OnceLock<FilePickerSharedResult> = OnceLock::new();
    INPUT_FILE.get_or_init(Default::default)
}

#[cfg(target_arch = "wasm32")]
fn output_file() -> &'static FilePickerSharedResult {
    static OUTPUT_FILE: OnceLock<FilePickerSharedResult> = OnceLock::new();
    OUTPUT_FILE.get_or_init(Default::default)
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
    #[serde(skip)]
    output_file_asked: bool,
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
            debug!("found storage");
            return eframe::get_value(storage, eframe::APP_KEY)
                .inspect(|app| {
                    debug!("found the app in the storage");
                    trace!(
                        "{}",
                        serde_json::to_string(app)
                            .map(Cow::Owned)
                            .unwrap_or(Cow::Borrowed("app serialization failed"))
                    );
                })
                .unwrap_or_default();
        }

        let app = Default::default();
        info!("app successfuly loaded");
        app
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
        debug!("storage saved");
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

            FileDirection::iter().for_each(|direction| {
                ui.label(format!("{direction} file:"));
                self.browse(ui, direction);
            });

            // ui.horizontal(|ui| {
            //     if ui.button("Alert").clicked() {
            //         self.toasts.warning("Warning!");
            //     }
            // });

            // ui.separator();

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

#[derive(Display, Clone, Copy, EnumIter)]
enum FileDirection {
    Input,
    Output,
}

impl TemplateApp {
    #[cfg(target_arch = "wasm32")]
    fn browse(&mut self, ui: &mut egui::Ui, direction: FileDirection) {
        ui.horizontal(|ui| {
            if ui.button("Browse").clicked() {
                debug!("user asked to open a file for {direction}");
                TemplateApp::prompt(direction);
                match direction {
                    FileDirection::Input => {
                        self.input_file_asked = true;
                    }
                    FileDirection::Output => {
                        self.output_file_asked = true;
                    }
                }
            }
            if match direction {
                FileDirection::Input => self.input_file_asked,
                FileDirection::Output => self.output_file_asked,
            } {
                let mut guard = match direction {
                    FileDirection::Input => input_file(),
                    FileDirection::Output => output_file(),
                }
                .lock();
                if guard.as_ref().is_some() {
                    match direction {
                        FileDirection::Input => {
                            self.input_file_asked = false;
                        }
                        FileDirection::Output => {
                            self.output_file_asked = false;
                        }
                    }
                    trace!("just got the file picker result");
                    let res = {
                        let guard = guard.as_ref();
                        unsafe { guard.unwrap_unchecked() }
                    };
                    match res {
                        Ok(maybe_path) => {
                            trace!("it performed successfuly");
                            if let Some(path) = maybe_path {
                                match ImageFile::try_new(path) {
                                    Ok(file) => {
                                        trace!("file is correct");
                                        self.files[match direction {
                                            FileDirection::Input => 0,
                                            FileDirection::Output => 1,
                                        }] = Some(file);
                                    }
                                    Err(e) => {
                                        trace!("file is incorrect, reason: {e}");
                                        self.toasts.error(e.to_string());
                                        trace!("an error toast was generated accordingly");
                                    }
                                }
                            } else {
                                trace!("however, no file was picked");
                            }
                        }
                        Err(e) => {
                            error!("the file picker just crashed with {e:?}");
                            self.toasts.error(
                                "The file picker just crashed! Can't have shit in Detroit!!!",
                            );
                        }
                    }
                    guard.take();
                }
            }
            ui.label(
                self.files[match direction {
                    FileDirection::Input => 0,
                    FileDirection::Output => 1,
                }]
                .as_ref()
                .map(|file| file.to_string())
                .unwrap_or_default(),
            );
        });
    }
    #[cfg(target_arch = "wasm32")]
    fn prompt(direction: FileDirection) {
        use std::panic::AssertUnwindSafe;

        use rfd::AsyncFileDialog;

        let fd = AsyncFileDialog::new()
            .set_title(format!("{direction} image"))
            .set_directory(working_dir())
            .add_filter(
                "images",
                &ImageFormat::all()
                    .flat_map(ImageFormat::extensions_str)
                    .collect::<Vec<&'static &'static str>>(),
            );

        match direction {
            FileDirection::Input => spawn_and_collect(
                AssertUnwindSafe(fd.pick_file().map(|x| x.map(|file| file.file_name()))),
                Arc::downgrade(match direction {
                    FileDirection::Input => input_file(),
                    FileDirection::Output => output_file(),
                }),
            ),
            FileDirection::Output => {
                spawn_and_collect(
                    AssertUnwindSafe(fd.save_file().map(|x| x.map(|file| file.file_name()))),
                    Arc::downgrade(match direction {
                        FileDirection::Input => input_file(),
                        FileDirection::Output => output_file(),
                    }),
                );
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn browse(&mut self, ui: &mut egui::Ui, direction: FileDirection) {
        use rfd::FileDialog;

        ui.horizontal(|ui| {
            if ui.button("Browse").clicked() {
                if let Some(path) = {
                    let fd = FileDialog::new()
                        .set_title(format!("{direction} image"))
                        .set_directory(working_dir())
                        .add_filter(
                            "images",
                            &ImageFormat::all()
                                .flat_map(ImageFormat::extensions_str)
                                .collect::<Vec<&'static &'static str>>(),
                        );
                    match direction {
                        FileDirection::Input => fd.pick_file(),
                        FileDirection::Output => fd.save_file(),
                    }
                } {
                    match ImageFile::try_new(&path) {
                        Ok(file) => {
                            self.files[match direction {
                                FileDirection::Input => 0,
                                FileDirection::Output => 1,
                            }] = Some(file);
                        }
                        Err(e) => {
                            self.toasts.error(e.to_string());
                        }
                    }
                }
            }
            ui.label(
                self.files[match direction {
                    FileDirection::Input => 0,
                    FileDirection::Output => 1,
                }]
                .clone()
                .map(|file| file.display().to_string())
                .unwrap_or_default(),
            );
        });
    }
}
