use crate::types::atlas_context::AtlasContext;

pub fn display(ctx: &mut AtlasContext, ui: &mut egui::Ui) {
  ui.add(egui::Label::new("Chat"));

  ui.add(egui::Label::new("Will need to add this later, must first add graph support."));
}