use egui_dock::TabViewer;
use egui_graphs::Graph;
use petgraph::{stable_graph::DefaultIx, Directed};
use crate::{theme::ColorScheme, widgets::tabs::graph_view::generate_graph};

use super::{
  graph_view::NodeShapeAnimated, napkin_settings::NapkinSettings, panel_tab::{PanelTab, Title}, panel_type::PanelType
};
use crate::widgets::tabs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub struct NapkinGraph(pub Graph<(), (), Directed, DefaultIx, NodeShapeAnimated>);

impl Default for NapkinGraph {
  fn default() -> Self {
    generate_graph()
  }
}

#[derive(Serialize, Deserialize)]
pub struct AtlasContext {
    #[serde(skip)]
    pub g: NapkinGraph,
    pub sim: Simulation<(), f32>,

    pub graph_sim_stopped: bool,

    pub graph_settings: GraphSettings,

    pub last_events: Vec<String>,

    pub fps: f64,
    pub last_update_time: Instant,
    pub frames_last_time_span: usize,

    pub event_publisher: Sender<Event>,
    pub event_consumer: Receiver<Event>,

    pub pan: Option<[f32; 2]>,
    pub zoom: Option<f32>,

    pub color_scheme: ColorScheme,
    pub side_panel_open: bool,
    pub settings_window_open: bool,
    pub about_window_open: bool,
    pub napkin_settings: NapkinSettings,
    pub napkin_temp_settings: NapkinSettings,
    pub current_prompt: String,
    pub buffers: BTreeMap<Title, PanelTab>,
}

impl AtlasContext {
  pub fn save_settings(&mut self) {
      self.napkin_settings = self.napkin_temp_settings.clone();
  }

  pub fn revert_settings(&mut self) {
      self.napkin_temp_settings = self.napkin_settings.clone();
  }
  fn chat_window(&mut self, ui: &mut egui::Ui) {
      tabs::chat::display(self, ui)
  }
  fn graph_window(&mut self, ui: &mut egui::Ui) {
    tabs::graph_view::display(self, ui)
  }
  fn settings_window(&mut self, ui: &mut egui::Ui) {
      tabs::settings::display(self, ui)
  }
}


impl TabViewer for AtlasContext {
  type Tab = Title;

  fn title(&mut self, title: &mut Title) -> egui::WidgetText {
      egui::WidgetText::from(&*title)
  }

  fn ui(&mut self, ui: &mut egui::Ui, title: &mut Title) {
      let panel_tab: &mut PanelTab = self.buffers.entry(title.clone()).or_default();
      // let panel_app = &mut panel_tab.app;

      match &panel_tab.panel_type {
          PanelType::Text => {
              if let Some(text) = &mut panel_tab.text {
                  let _ = egui::TextEdit::multiline(text)
                      .desired_width(f32::INFINITY)
                      .show(ui);
              } else {
                  ui.add(egui::Label::new("Invalid text buffer"));
              }
          }
          PanelType::Chat {history, row_sizes} => self.chat_window(ui),
          PanelType::Settings => self.settings_window(ui),
          PanelType::Graph => self.graph_window(ui),
          // PanelType::Chat => chat_window(ui.ctx(), app),
          // PanelType::Graph => central_panel(ui.ctx(), app),
      }
  }
}