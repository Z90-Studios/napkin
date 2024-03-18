#![deny(clippy::all)]
#![forbid(unsafe_code)]

use egui::Key;
use egui_dock::{DockArea, DockState, Style};
use std::collections::BTreeMap;

use crate::theme::{set_theme, MACCHIATO, ColorScheme};
use crate::types::atlas_context::NapkinGraph;
use crate::widgets::menu_bar;
use crate::types::{
    atlas_context::AtlasContext,
    napkin_settings::NapkinSettings
};




// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct OllamaRequest {
//     prompt: String,
//     model: String,
//     stream: bool,
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct OllamaGenerateResponse {
//     model: String,
//     created_at: String,
//     response: String,
//     done: Option<bool>,
//     context: Option<Vec<i64>>,
//     total_duration: Option<i64>,
//     load_duration: Option<i64>,
//     prompt_eval_count: Option<i64>,
//     prompt_eval_duration: Option<i64>,
//     eval_count: Option<i64>,
//     eval_duration: Option<i64>,
// }

// #[derive(serde::Deserialize, serde::Serialize)]
// pub enum AsyncMessage {
//     OllamaGenerateResponse(OllamaGenerateResponse),
//     OllamaStatusCheck(String),
// }

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ChatWindowState {
    pub row_sizes: Vec<f32>,
}
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct AtlasApp {
    // Application State
    pub context: AtlasContext,
    // Tree for egui_dock DockState
    pub tree: DockState<String>,
}

impl Default for AtlasApp {
    fn default() -> Self {
        // let (tx, rx) = std::sync::mpsc::channel();
        let buffers = BTreeMap::default();

        let tree = DockState::new(vec![]);

        Self {
            // chat_history: vec![],
            // chat_window_state: ChatWindowState { row_sizes: vec![] },
            context: AtlasContext {
                g: NapkinGraph::default(),
                buffers,
                color_scheme: ColorScheme::Dark { theme: MACCHIATO },
                side_panel_open: true,
                settings_window_open: false,
                about_window_open: false,
                napkin_settings: NapkinSettings::default(),
                napkin_temp_settings: NapkinSettings::default(),
                current_prompt: "".to_owned(),
            },
            tree,
        }
    }
}

impl AtlasApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    pub fn reset() -> Self {
        Default::default()
    }

    pub fn update_fps(&mut self) {
        self.frames_last_time_span += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        if elapsed.as_secs() >= 1 {
            self.last_update_time = now;
            self.fps = self.frames_last_time_span as f64 / elapsed.as_secs_f64();
            self.frames_last_time_span = 0;
        }
    }

    // pub fn submit_prompt(&mut self, ctx: &egui::Context) {
    //     self.chat_history.push(ChatHistory {
    //         instance: 0,
    //         user: "USER".to_owned(),
    //         message: self.current_prompt.clone(),
    //         model: None,
    //         timestamp: "NOW".to_owned(),
    //     });
    //     ollama_send_prompt(self, self.tx.clone(), ctx.clone());
    //     self.current_prompt = "".to_owned();
    // }
}

impl eframe::App for AtlasApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // if let Ok(async_message) = self.rx.try_recv() {
        //     match async_message {
        //         AsyncMessage::OllamaGenerateResponse(response) => {
        //             self.chat_history.push(ChatHistory {
        //                 instance: 0,
        //                 user: "OLLAMA".to_owned(),
        //                 message: response.response,
        //                 model: Some(response.model),
        //                 timestamp: response.created_at,
        //             });
        //         }
        //         AsyncMessage::OllamaStatusCheck(_) => {
        //             self.ollama_check = "✅".to_owned();
        //         }
        //     }
        // }
        set_theme(
            &ctx,
            match self.context.color_scheme {
                ColorScheme::Light { theme } => theme,
                ColorScheme::Dark { theme } => theme,
            },
        );
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        menu_bar::top_bottom_panel(self, ctx);

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show_animated(ctx, self.context.side_panel_open, |ui| {
                ui.set_min_width(200.0);

                ui.separator();

                for title in self.context.buffers.keys() {
                    let tab_location = self.tree.find_tab(title);
                    let is_open = tab_location.is_some();
                    if ui.selectable_label(is_open, title).clicked() {
                        if let Some(tab_location) = tab_location {
                            self.tree.set_active_tab(tab_location);
                        } else {
                            // Open the file for editing:
                            self.tree.push_to_focused_leaf(title.clone());
                        }
                    }
                }
            });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button(":Z90:", |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("Created by ");
                        ui.hyperlink_to("Z90 Studios", "https://github.com/Z90-Studios");
                        ui.label(".");
                    });
                });
                // ui.horizontal(|ui| {
                // if ui
                //     .button("Check Connection")
                //     .on_hover_text("Check connection to Ollama server.")
                //     .clicked()
                // {
                //     check_ollama(self, self.tx.clone(), ctx.clone());
                // }
                // ui.label(&self.ollama_check);
                // });
                egui::warn_if_debug_build(ui);
            });
        });

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.context);

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::B)) {
            self.context.side_panel_open = !self.context.side_panel_open;
        }
        // settings_window(ctx, self);
        crate::widgets::about::window(ctx, self);
    }
}