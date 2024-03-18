use std::time::Instant;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SerialInstant {
  #[serde(with = "serialize_instant")]
  pub time: Instant,
}

pub mod serialize_instant {
  use std::time::{Instant, SystemTime};
  use serde::{Serialize, Serializer, Deserialize, Deserializer, de::Error};

  pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let system_now = SystemTime::now();
    let instant_now = Instant::now();
    let approx = system_now - (instant_now - *instant);
    approx.serialize(serializer)
  }

  pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
  where
    D: Deserializer<'de>,
  {
    let de = SystemTime::deserialize(deserializer)?;
    let system_now = SystemTime::now();
    let instant_now = Instant::now();
    let duration = system_now.duration_since(de).map_err(Error::custom)?;
    let approx = instant_now - duration;
    Ok(approx)
  }
}

#[derive(Serialize, Deserialize)]
pub struct GraphSettings {
  // General Settings
  // TODO: Turn this into a list of Nodes/Edges perhaps, with the included ULID?
  pub node_count: usize,
  pub edge_count: usize,
  
  // Interaction Settings
  pub dragging_enabled: bool,
  pub node_clicking_enabled: bool,
  pub node_selection_enabled: bool,
  pub node_selection_multi_enabled: bool,
  pub edge_clicking_enabled: bool,
  pub edge_selection_enabled: bool,
  pub edge_selection_multi_enabled: bool,

  // Navigation Settings
  pub fit_to_screen_enabled: bool,
  pub zoom_and_pan_enabled: bool,
  pub screen_padding: f32,
  pub zoom_speed: f32,

  // Style Settings
  pub labels_always: bool,
}

impl Default for GraphSettings {
  fn default() -> Self {
    Self {
      node_count: 300,
      edge_count: 500,

      dragging_enabled: true,
      node_clicking_enabled: true,
      node_selection_enabled: true,
      node_selection_multi_enabled: true,
      edge_clicking_enabled: true,
      edge_selection_enabled: true,
      edge_selection_multi_enabled: true,
      
      fit_to_screen_enabled: true,
      zoom_and_pan_enabled: true,
      screen_padding: 0.3,
      zoom_speed: 0.1,

      labels_always: true,
    }
  }
}