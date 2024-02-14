use crate::types::atlas_context::AtlasContext;
use egui::{FontId, RichText};

pub fn display(ctx: &mut AtlasContext, ui: &mut egui::Ui) {
    let mut should_close = false;
    let mut should_save = false;

    // egui::Window::new("Settings")
    //     .open(&mut app.settings_window_open)
    //     .resizable(false)
    //     .show(ctx, |ui| {

    ui.heading("LLM Settings");
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
        ui.text_edit_singleline(&mut ctx.napkin_temp_settings.model);
        ui.label("Model: ");
    });
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
        ui.text_edit_singleline(&mut ctx.napkin_temp_settings.service.host);
        ui.label("Host: ").rect.set_width(80.0);
    });
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
        ui.text_edit_singleline(&mut ctx.napkin_temp_settings.service.port);
        ui.label("Port: ").rect.set_width(80.0);
    });
    ui.add_space(42.0);
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
    });
    ui.add_space(12.0);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
      ui.label(RichText::new(format!("Atlas Napkin Supreme ver. {}", crate::VERSION)).font(FontId::proportional(8.0)));
    });

    if should_close {
        // app.settings_window_open = false;

        if should_save {
            ctx.save_settings();
        } else {
            ctx.revert_settings();
        }
    }
}
