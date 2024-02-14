use egui_graphs::{DefaultEdgeShape, GraphView, SettingsInteraction, SettingsStyle};
use petgraph::Directed;

use crate::types::{atlas_context::{AtlasContext, NapkinGraph}, graph_view::NodeShapeAnimated};

pub fn generate_graph() -> NapkinGraph {
  let mut g = petgraph::stable_graph::StableGraph::new();

  let a = g.add_node(());
  let b = g.add_node(());
  let c = g.add_node(());

  g.add_edge(a, b, ());
  g.add_edge(b, c, ());
  g.add_edge(c, a, ());

  NapkinGraph((&g).into())
}

pub fn display(ctx: &mut AtlasContext, ui: &mut egui::Ui) {
  let interaction_settings = &SettingsInteraction::new()
    .with_dragging_enabled(true)
    .with_node_clicking_enabled(true)
    .with_node_selection_enabled(true)
    .with_node_selection_multi_enabled(true)
    .with_edge_clicking_enabled(true)
    .with_edge_selection_enabled(true)
    .with_edge_selection_multi_enabled(true);
  let style_settings = &SettingsStyle::new()
    .with_labels_always(true);
  let napkin_graph = &mut ctx.g.0;
  ui.add(
    &mut GraphView::<(), (), Directed, u32, NodeShapeAnimated, DefaultEdgeShape>::new(napkin_graph)
      .with_styles(style_settings)
      .with_interactions(interaction_settings),
  );
}