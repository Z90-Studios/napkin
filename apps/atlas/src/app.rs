#![deny(clippy::all)]
#![forbid(unsafe_code)]

use egui::Key;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

use crate::theme::{set_theme, LATTE, MACCHIATO};

#[derive(Clone, Serialize, Deserialize)]
pub struct NapkinService {
    host: String,
    port: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NapkinSettings {
    model: String,
    service: NapkinService,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatHistory {
    /** Used to handle multiple chats */
    instance: u32,
    /** Visual user representation */
    user: String,
    /** Message text */
    message: String,
    /** Model used, if applicable */
    model: Option<String>,
    /** Timestamp of message */
    timestamp: String,
}

impl NapkinSettings {
    pub fn default() -> Self {
        Self {
            model: "mistral".to_owned(),
            service: NapkinService {
                host: "localhost".to_owned(),
                port: "11434".to_owned(),
            },
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct OllamaRequest {
    prompt: String,
    model: String,
    stream: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct OllamaGenerateResponse {
    model: String,
    created_at: String,
    response: String,
    done: Option<bool>,
    context: Option<Vec<i64>>,
    total_duration: Option<i64>,
    load_duration: Option<i64>,
    prompt_eval_count: Option<i64>,
    prompt_eval_duration: Option<i64>,
    eval_count: Option<i64>,
    eval_duration: Option<i64>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum AsyncMessage {
    OllamaGenerateResponse(OllamaGenerateResponse),
    OllamaStatusCheck(String),
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ChatWindowState {
    pub row_sizes: Vec<f32>,
}
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct AtlasApp {
    #[serde(skip)]
    tx: Sender<AsyncMessage>,
    #[serde(skip)]
    rx: Receiver<AsyncMessage>,
    // Example stuff:
    label: String,
    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    theme: Theme,
    side_panel_open: bool,
    settings_window_open: bool,
    about_window_open: bool,
    napkin_settings: NapkinSettings,
    napkin_temp_settings: NapkinSettings,
    #[serde(skip)]
    ollama_check: String,
    current_prompt: String,
    chat_history: Vec<ChatHistory>,
    chat_window_state: ChatWindowState,
}

impl Default for AtlasApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            tx,
            rx,
            label: "Hello World!".to_owned(),
            value: 2.7,
            theme: Theme::Dark,
            side_panel_open: true,
            settings_window_open: false,
            about_window_open: false,
            napkin_settings: NapkinSettings::default(),
            napkin_temp_settings: NapkinSettings::default(),
            ollama_check: "❌".to_owned(),
            current_prompt: "".to_owned(),
            chat_history: vec![],
            chat_window_state: ChatWindowState { row_sizes: vec![] },
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

    pub fn save_settings(&mut self) {
        self.napkin_settings = self.napkin_temp_settings.clone();
    }

    pub fn revert_settings(&mut self) {
        self.napkin_temp_settings = self.napkin_settings.clone();
    }

    pub fn submit_prompt(&mut self, ctx: &egui::Context) {
        self.chat_history.push(ChatHistory {
            instance: 0,
            user: "USER".to_owned(),
            message: self.current_prompt.clone(),
            model: None,
            timestamp: "NOW".to_owned(),
        });
        ollama_send_prompt(self, self.tx.clone(), ctx.clone());
        self.current_prompt = "".to_owned();
    }
}

impl eframe::App for AtlasApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(async_message) = self.rx.try_recv() {
            match async_message {
                AsyncMessage::OllamaGenerateResponse(response) => {
                    self.chat_history.push(ChatHistory {
                        instance: 0,
                        user: "OLLAMA".to_owned(),
                        message: response.response,
                        model: Some(response.model),
                        timestamp: response.created_at,
                    });
                }
                AsyncMessage::OllamaStatusCheck(_) => {
                    self.ollama_check = "✅".to_owned();
                }
            }
        }
        set_theme(
            &ctx,
            match self.theme {
                Theme::Light => LATTE,
                Theme::Dark => MACCHIATO,
            },
        );
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Settings").clicked() {
                            self.settings_window_open = true;
                        }
                        ui.separator();
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    if ui.button("About").clicked() {
                        self.about_window_open = true;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    ui.horizontal(|ui| match self.theme {
                        Theme::Dark => {
                            if ui
                                .button("☀")
                                .on_hover_text("Switch to light mode")
                                .clicked()
                            {
                                self.theme = Theme::Light;
                            }
                        }
                        Theme::Light => {
                            if ui
                                .button("🌙")
                                .on_hover_text("Switch to dark mode")
                                .clicked()
                            {
                                self.theme = Theme::Dark;
                            }
                        }
                    });
                    ui.toggle_value(&mut self.side_panel_open, "File Browser");
                });
            });
        });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show_animated(ctx, self.side_panel_open, |ui| {
                ui.set_min_width(200.0);

                ui.horizontal(|ui| {
                    if ui
                        .button("Check Connection")
                        .on_hover_text("Check connection to Ollama server.")
                        .clicked()
                    {
                        check_ollama(self, self.tx.clone(), ctx.clone());
                    }
                    ui.label(&self.ollama_check);
                });
                ui.separator();
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
                egui::warn_if_debug_build(ui);
            });
        });
        chat_window(ctx, self);
        central_panel(ctx, self);
        settings_window(ctx, self);
        about_window(ctx, self);
    }
}

fn check_ollama(app: &AtlasApp, tx: Sender<AsyncMessage>, ctx: egui::Context) {
    let host = app.napkin_settings.service.host.clone();
    let port = app.napkin_settings.service.port.clone();
    tokio::spawn(async move {
        let body: String = Client::default()
            .get(format!("http://{}:{}/", host, port))
            .send()
            .await
            .expect("Unable to send request")
            .text()
            .await
            .expect("Unable to parse response");

        let _ = tx.send(AsyncMessage::OllamaStatusCheck(body));
        ctx.request_repaint();
    });
}

fn ollama_send_prompt(app: &AtlasApp, tx: Sender<AsyncMessage>, ctx: egui::Context) {
    let host = app.napkin_settings.service.host.clone();
    let port = app.napkin_settings.service.port.clone();
    let current_prompt = app.current_prompt.clone();
    let model = app.napkin_settings.model.clone();
    let request = OllamaRequest {
        prompt: current_prompt,
        model,
        stream: false,
    };
    println!("Request: {:?}", request);
    tokio::spawn(async move {
        match Client::default()
            .post(format!("http://{}:{}/api/generate", host, port))
            .json(&request)
            .send()
            .await
        {
            Ok(response) => {
                match response.json().await {
                    Ok(json) => {
                        let ollama_gen_response = json;
                        println!("Response: {:?}", ollama_gen_response);
                        let _ = tx.send(AsyncMessage::OllamaGenerateResponse(ollama_gen_response));
                        ctx.request_repaint();
                    }
                    Err(e) => {
                        println!("Unable to parse response: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Unable to send request: {}", e);
            }
        };
    });
}

fn central_panel(ctx: &egui::Context, app: &mut AtlasApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // The central panel the region left after adding TopPanel's and SidePanel's
        // ui.heading("eframe template");

        // ui.horizontal(|ui| {
        //     ui.label("Write something: ");
        //     ui.text_edit_singleline(&mut self.label);
        // });

        // ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
        // if ui.button("Increment").clicked() {
        //     self.value += 1.0;
        // }
    });

    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::B)) {
        app.side_panel_open = !app.side_panel_open;
    }
}

fn chat_window(ctx: &egui::Context, app: &mut AtlasApp) {
    egui::Window::new("Chat")
        .open(&mut true)
        .resizable(true)
        .constrain(true)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::right_to_left(egui::Align::TOP),
                    |ui| {
                        if ui.button("Clear").clicked() {
                            app.chat_history.clear();
                        }
                        ui.add_space(8.0);
                        if ui.button("Configure").clicked() {
                            app.settings_window_open = true;
                        }
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&app.napkin_settings.model).color(
                                match app.theme {
                                    Theme::Light => LATTE.sapphire,
                                    Theme::Dark => MACCHIATO.sapphire,
                                },
                            ));
                            ui.label("Model: ");
                        });
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&app.napkin_settings.service.port).color(
                                    match app.theme {
                                        Theme::Light => LATTE.sapphire,
                                        Theme::Dark => MACCHIATO.sapphire,
                                    },
                                ),
                            );
                            ui.label("Port: ");
                        });
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&app.napkin_settings.service.host).color(
                                    match app.theme {
                                        Theme::Light => LATTE.sapphire,
                                        Theme::Dark => MACCHIATO.sapphire,
                                    },
                                ),
                            );
                            ui.label("Host: ");
                        });
                    },
                );
                let user_color = match app.theme {
                    Theme::Light => LATTE.flamingo,
                    Theme::Dark => MACCHIATO.flamingo,
                };
                let ai_color = match app.theme {
                    Theme::Light => LATTE.sapphire,
                    Theme::Dark => MACCHIATO.sapphire,
                };
                egui::Frame::none()
                    .rounding(4.0)
                    .inner_margin(egui::vec2(4.0, 4.0))
                    .stroke(egui::Stroke::new(
                        1.0,
                        match app.theme {
                            Theme::Light => LATTE.surface1,
                            Theme::Dark => MACCHIATO.surface1,
                        },
                    ))
                    .show(ui, |ui| {
                        let available_width = ui.available_width();
                        let available_height = ui.available_height();
                        // ui.set_width(available_width);
                        egui_extras::TableBuilder::new(ui)
                            .stick_to_bottom(true)
                            .striped(true)
                            .column(egui_extras::Column::exact(50.0))
                            .column(egui_extras::Column::exact(240.0))
                            .column(egui_extras::Column::remainder().at_least(240.0).clip(true))
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.label(egui::RichText::new("User").color(match app.theme {
                                        Theme::Light => LATTE.sapphire,
                                        Theme::Dark => MACCHIATO.sapphire,
                                    }));
                                });
                                header.col(|ui| {
                                    ui.label(egui::RichText::new("Message").color(
                                        match app.theme {
                                            Theme::Light => LATTE.sapphire,
                                            Theme::Dark => MACCHIATO.sapphire,
                                        },
                                    ));
                                });
                                header.col(|ui| {
                                    ui.label(egui::RichText::new("Timestamp").color(
                                        match app.theme {
                                            Theme::Light => LATTE.sapphire,
                                            Theme::Dark => MACCHIATO.sapphire,
                                        },
                                    ));
                                });
                            })
                            .body(|mut body| {
                                if app.chat_window_state.row_sizes.is_empty() {
                                    app.chat_window_state.row_sizes =
                                        vec![30.0; app.chat_history.len()];
                                }
                                if app.chat_window_state.row_sizes.len() != app.chat_history.len() {
                                    app.chat_window_state.row_sizes =
                                        vec![30.0; app.chat_history.len()];
                                }
                                // for prompt in app.chat_history.iter() {
                                // }
                                let row_sizes = app.chat_window_state.row_sizes.clone();
                                body.heterogeneous_rows(row_sizes.into_iter(), |mut row| {
                                    let row_index = row.index();
                                    let prompt = &app.chat_history[row_index];
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(&prompt.user).color(
                                                        if &prompt.user == "USER" {
                                                            user_color
                                                        } else {
                                                            ai_color
                                                        },
                                                    ),
                                                )
                                                .wrap(false),
                                            );
                                            ui.add_space(8.0);
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(&prompt.timestamp).color(
                                                    match app.theme {
                                                        Theme::Light => LATTE.yellow,
                                                        Theme::Dark => MACCHIATO.yellow,
                                                    },
                                                ),
                                            )
                                            .wrap(false),
                                        );
                                    });
                                    row.col(|ui| {
                                        // let text_frame = egui::Frame::none()
                                        //     .rounding(2.0)
                                        //     .inner_margin(egui::vec2(4.0, 4.0))
                                        //     .outer_margin(egui::vec2(8.0, 0.0))
                                        //     .stroke(egui::Stroke::new(
                                        //         1.0,
                                        //         match app.theme {
                                        //             Theme::Light => LATTE.surface0,
                                        //             Theme::Dark => MACCHIATO.surface0,
                                        //         },
                                        //     ))
                                        //     .show(ui, |ui| {
                                        //     });
                                        let message_label = ui.add(
                                            egui::Label::new(egui::RichText::new(&prompt.message))
                                                .wrap(true),
                                        );
                                        // let current_height = ui.min_rect().height();
                                        let current_height = message_label.rect.height();
                                        if current_height > 30.0 {
                                            app.chat_window_state.row_sizes[row_index] =
                                                current_height;
                                        }
                                    });
                                });
                            });
                        // egui::Grid::new("some_unique_id").show(ui, |ui| {
                        //     ui.label("First row, first column");

                        //     let text_frame = egui::Frame::none()
                        //         .rounding(2.0)
                        //         .inner_margin(egui::vec2(4.0, 4.0))
                        //         .outer_margin(egui::vec2(8.0, 0.0))
                        //         .stroke(egui::Stroke::new(
                        //             1.0,
                        //             match app.theme {
                        //                 Theme::Light => LATTE.surface0,
                        //                 Theme::Dark => MACCHIATO.surface0,
                        //             },
                        //         ))
                        //         .show(ui, |ui| {
                        //             ui.label(&prompt.message);
                        //         });
                        //     ui.label("First row, third column");
                        //     ui.end_row();

                        //     ui.label("Second row, first column");
                        //     ui.label("Second row, second column");
                        //     ui.label("Second row, third column");
                        //     ui.end_row();

                        //     ui.horizontal(|ui| {
                        //         ui.label("Same");
                        //         ui.label("cell");
                        //     });
                        //     ui.label("Third row, second column");
                        //     ui.end_row();
                        // });
                    });
            });

            ui.add_space(4.0);

            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if ui.button("Send").clicked() {
                        app.submit_prompt(&ctx);
                    }
                    let edit_response = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::singleline(&mut app.current_prompt),
                    );
                    if edit_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        app.submit_prompt(&ctx);
                        edit_response.request_focus();
                    }
                });
            });
        });
}

fn settings_window(ctx: &egui::Context, app: &mut AtlasApp) {
    let mut should_close = false;
    let mut should_save = false;

    egui::Window::new("Settings")
        .open(&mut app.settings_window_open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("LLM Settings");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.text_edit_singleline(&mut app.napkin_temp_settings.model);
                ui.label("Model: ");
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.text_edit_singleline(&mut app.napkin_temp_settings.service.host);
                ui.label("Host: ").rect.set_width(80.0);
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.text_edit_singleline(&mut app.napkin_temp_settings.service.port);
                ui.label("Port: ").rect.set_width(80.0);
            });
            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }
                    ui.separator();
                    if ui.button("Save").clicked() {
                        should_save = true;
                        should_close = true;
                    }
                });
            })
        });

    if should_close {
        app.settings_window_open = false;

        if should_save {
            app.save_settings();
        } else {
            app.revert_settings();
        }
    }
}

fn about_window(ctx: &egui::Context, app: &mut AtlasApp) {
    let mut should_close = false;
    egui::Window::new("About")
        .id(egui::Id::new("about_window"))
        .open(&mut app.about_window_open)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(ctx, |ui| {
            if ui.interact(ui.max_rect(), egui::Id::new("about_window"), egui::Sense::click()).clicked() {
                should_close = true;
            }
            ui.heading("Project: Atlas Napkin");
            ui.label("So here's the plan:\n\nThe purpose of this application is to serve as the frontend to a locally run AI agent. This agent will do the following:\n\n1. Parse a codebase, or other information.\n2. Map the data into a network graph with vector database.\n3. Use the data in prompting along with multiple other elements to create a cohesive change to codebases.");

            ui.separator();
            ui.label("This is a project by Z90 Studios.");
            ui.label("This project is licensed under the MIT license.");
            ui.horizontal(|ui| {
                ui.label("This project is open source and can be found on");
                ui.hyperlink_to(egui::RichText::new("GitHub").color(MACCHIATO.blue), "https://github.com/Z90-Studios/napkin");
            });
        });

    if should_close {
        app.about_window_open = false;
    }
}
