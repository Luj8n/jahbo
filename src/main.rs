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
  let native_options = eframe::NativeOptions {
    maximized: true,
    ..Default::default()
  };
  eframe::run_native(Box::new(app), native_options);
}
