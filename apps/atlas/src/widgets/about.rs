use crate::AtlasApp;
use egui::{RichText, FontId};

pub fn window(ctx: &egui::Context, app: &mut AtlasApp) {
  let mut should_close = false;
  egui::Window::new("About")
      .id(egui::Id::new("about_window"))
      .open(&mut app.context.about_window_open)
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
              ui.hyperlink_to(egui::RichText::new("GitHub").color(app.context.color_scheme.theme().blue), "https://github.com/Z90-Studios/napkin");
          });
          ui.add_space(12.0);
          ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.label(RichText::new(format!("Atlas Napkin Supreme ver. {}", crate::VERSION)).font(FontId::proportional(8.0)));
          });
      });

  if should_close {
      app.context.about_window_open = false;
  }
}
