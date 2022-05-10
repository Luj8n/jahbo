#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

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

fn main() {
  // TODO: generate the settings.toml file (if it's missing) and ask to fill it in

  let icon_bytes = include_bytes!("../assets/icon.png");
  let icon = image::load_from_memory(icon_bytes).unwrap();

  let native_options = eframe::NativeOptions {
    maximized: true,
    // transparent: true, // TODO
    icon_data: Some(eframe::IconData {
      width: icon.width(),
      height: icon.height(),
      rgba: icon.into_bytes(),
    }),
    ..Default::default()
  };
  eframe::run_native("Jahbo", native_options, Box::new(|cc| Box::new(app::App::new(cc))));
}
