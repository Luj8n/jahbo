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
  let app = app::App::default();

  let icon_bytes = include_bytes!("../assets/icon.png");
  let icon = image::load_from_memory(icon_bytes).unwrap();

  let native_options = eframe::NativeOptions {
    maximized: true,
    // transparent: true, // TODO
    icon_data: Some(eframe::epi::IconData {
      width: icon.width(),
      height: icon.height(),
      rgba: icon.into_bytes(),
    }),
    ..Default::default()
  };
  eframe::run_native(Box::new(app), native_options);
}
