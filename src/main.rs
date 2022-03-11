use eframe::epaint::Color32;
use std::fs::read_to_string;

mod app;
mod data;
mod fetching;
mod parsing;

fn get_toml_value(file_name: &str, key: &str) -> toml::Value {
  let file = read_to_string(file_name).unwrap_or_else(|_| panic!("{} file not found", file_name));
  let parsed_file = file
    .parse::<toml::Value>()
    .unwrap_or_else(|_| panic!("{} is not valid toml", file_name));

  parsed_file
    .get(key)
    .unwrap_or_else(|| panic!("{} is not defined in the settings.toml file", key))
    .clone()
}

fn get_rank_color_and_name(rank: &str, monthly_rank: &str) -> (Color32, String) {
  match (rank, monthly_rank) {
    (_, "SUPERSTAR") => (Color32::GOLD, "MVP++ ".to_string()),
    ("MVP_PLUS", _) => (Color32::LIGHT_BLUE, "MVP+ ".to_string()),
    ("MVP", _) => (Color32::LIGHT_BLUE, "MVP ".to_string()),
    ("VIP_PLUS", _) => (Color32::LIGHT_GREEN, "VIP+ ".to_string()),
    ("VIP", _) => (Color32::LIGHT_GREEN, "VIP ".to_string()),
    _ => (Color32::GRAY, "".to_string()),
  }
}

fn main() {
  let app = app::App::default();
  let native_options = eframe::NativeOptions {
    maximized: true,
    ..Default::default()
  };
  eframe::run_native(Box::new(app), native_options);
}
