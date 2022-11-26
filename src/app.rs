use std::sync::Arc;

use egui::{Key, KeyboardShortcut, Modifiers};

use crate::renderer::Renderer;

const SHORTCUT_SHOW_HIDE_UI: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::H);
const SHORTCUT_FULLSCREEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::ALT, Key::Enter);
const SHORTCUT_QUIT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Q);

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    fullscreen: bool,
    ui_shown: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for App {
    fn default() -> Self {
        Self {
            fullscreen: false,
            ui_shown: true,
        }
    }
}

impl App {
    /// Called once before the first frame to handle initializing the app.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Get the WGPU render state from the eframe creation context.
        let wgpu_render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("Wgpu isn't enabled for th eframe and/or egui libraries!");
        let wgpu_device = &wgpu_render_state.device;
        let wgpu_queue = &wgpu_render_state.queue;
        let wgpu_target_format = wgpu_render_state.target_format;

        // Create a new renderer. It is stored inside of eframe-wgpu's custom
        // renderer infrastructure via the `paint_callback_resouces` type map,
        // as it must have the same lifetime as the egui render pass.
        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(Renderer::new(wgpu_device, wgpu_queue, wgpu_target_format));

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called by eframe to save app state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs to be redrawn.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Show a top menu bar, if the UI isn't hidden
        if self.ui_shown {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    // File menu
                    ui.menu_button("File", |ui| {
                        if ui
                            .button(shortcut_text_label(ctx, "Quit", &SHORTCUT_QUIT))
                            .clicked()
                        {
                            frame.close()
                        }
                    });

                    // View menu
                    ui.menu_button("View", |ui| {
                        ui.checkbox(
                            &mut self.ui_shown,
                            shortcut_text_label(ctx, "Show UI", &SHORTCUT_SHOW_HIDE_UI),
                        );

                        if ui
                            .checkbox(
                                &mut self.fullscreen,
                                shortcut_text_label(ctx, "Fullscreen", &SHORTCUT_FULLSCREEN),
                            )
                            .clicked()
                        {
                            self.fullscreen = !self.fullscreen;
                            self.toggle_fullscreen(frame);
                        }
                    })
                });
            });
        }

        // Render the emulator in the central panel
        egui::CentralPanel::default()
            .frame(egui::Frame::canvas(&egui::Style::default()).stroke(egui::Stroke::none()))
            .show(ctx, |ui| {
                self.custom_painting(ui);
            });

        self.handle_keyboard_input(ctx, frame);
    }
}

impl App {
    /// Handle keyboard input. Returns true if input was handled by this function,
    /// and false if it was ignored.
    fn handle_keyboard_input(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) -> bool {
        let mut input_handled = false;

        if ctx.input_mut().consume_shortcut(&SHORTCUT_QUIT) {
            input_handled = true;
            frame.close();
            return input_handled;
        }

        if ctx.input_mut().consume_shortcut(&SHORTCUT_SHOW_HIDE_UI) {
            input_handled = true;
            self.toggle_ui();
        }

        if ctx.input_mut().consume_shortcut(&SHORTCUT_FULLSCREEN) {
            input_handled = true;
            self.toggle_fullscreen(frame);
        }

        input_handled
    }

    fn toggle_ui(&mut self) {
        self.ui_shown = !self.ui_shown;
    }

    fn toggle_fullscreen(&mut self, frame: &mut eframe::Frame) {
        self.fullscreen = !self.fullscreen;
        frame.set_fullscreen(self.fullscreen);
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, _) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::focusable_noninteractive());

        // Set up the egui paint callback.
        let cb = eframe::egui_wgpu::CallbackFn::new()
            .prepare(
                move |_device, queue, _egui_cmd_encoder, paint_callback_resources| {
                    let renderer = paint_callback_resources.get_mut::<Renderer>().unwrap();

                    renderer.resize((rect.width() as u32, rect.height() as u32), queue);

                    Vec::new()
                },
            )
            .paint(|_info, render_pass, paint_callback_resources| {
                let renderer = paint_callback_resources.get::<Renderer>().unwrap();

                renderer.render(render_pass);
            });

        let paint_callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(paint_callback);
    }
}

fn shortcut_text_label(ctx: &egui::Context, label: &str, shortcut: &KeyboardShortcut) -> String {
    format!("{label} ({})", ctx.format_shortcut(shortcut))
}
