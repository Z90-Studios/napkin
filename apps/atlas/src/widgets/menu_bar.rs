#![deny(clippy::all)]
#![forbid(unsafe_code)]
use egui::Context;

use crate::AtlasApp;
use crate::types::{panel_tab::PanelTab, panel_type::PanelType};
use crate::theme::{ColorScheme, LATTE, MACCHIATO};


pub fn top_bottom_panel(atlas_app: &mut AtlasApp, ctx: &Context) {
  egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
    // The top panel is often a good place for a menu bar:

    egui::menu::bar(ui, |ui| {
        // NOTE: no File->Quit on web pages!
        let is_web = cfg!(target_arch = "wasm32");
        ui.menu_button("File", |ui| {
            if ui.button("Reset").clicked() {
                *atlas_app = AtlasApp::reset();
            }
            if ui.button("Settings").clicked() {
                // atlas_app.settings_window_open = true;
                atlas_app.tree.push_to_focused_leaf("Settings".to_owned());
                atlas_app.context.buffers.insert(
                    "Settings".to_owned(),
                    PanelTab {
                        title: "Settings".to_owned(),
                        text: None,
                        panel_type: PanelType::Settings,
                    },
                );
            }
            if !is_web {
                ui.separator();
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });
        ui.menu_button("Window", |ui| {
            if ui.button("Project Tree").clicked() {
                atlas_app.context.side_panel_open = !atlas_app.context.side_panel_open;
            }
            if ui.button("Graph").clicked() {
                atlas_app.tree.push_to_focused_leaf("Graph View".to_owned());
                atlas_app.context.buffers.insert(
                    "Graph View".to_owned(),
                    PanelTab {
                        title: "Graph View".to_owned(),
                        text: None,
                        panel_type: PanelType::Graph,
                    },
                );
            }
            if ui.button("Chat").clicked() {
                atlas_app.tree.push_to_focused_leaf("Chat".to_owned());
                // atlas_app.tabs.buffers.insert(
                //     "Chat".to_owned(),
                //     PanelTab {
                //         title: "Chat".to_owned(),
                //         text: None,
                //         panel_type: PanelType::Chat {
                //             history: atlas_app.chat_history.clone(),
                //             row_sizes: atlas_app.chat_window_state.row_sizes.clone(),
                //         },
                //     },
                // );
            }
        });
        if ui.button("About").clicked() {
            atlas_app.context.about_window_open = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            ui.horizontal(|ui| match atlas_app.context.color_scheme {
                ColorScheme::Dark { .. } => {
                    if ui
                        .button("☀")
                        .on_hover_text("Switch to light mode")
                        .clicked()
                    {
                        atlas_app.context.color_scheme = ColorScheme::Light {
                            theme: LATTE
                        };
                    }
                }
                ColorScheme::Light { .. } => {
                    if ui
                        .button("🌙")
                        .on_hover_text("Switch to dark mode")
                        .clicked()
                    {
                        atlas_app.context.color_scheme = ColorScheme::Dark {
                            theme: MACCHIATO
                        };
                    }
                }
            });
            ui.toggle_value(&mut atlas_app.context.side_panel_open, "File Browser");
        });
    });
  });
}