use eframe;
use eframe::epaint::Color32;
use regex::Regex;
use std::fs::read_to_string;

mod app;
mod data;
mod fetching;

fn get_toml_value(file_name: &str, key: &str) -> toml::Value {
  let file = read_to_string(file_name).expect(&format!("{} file not found", file_name));
  let parsed_file = file
    .parse::<toml::Value>()
    .expect(&format!("{} is not valid toml", file_name));

  parsed_file
    .get(key)
    .expect(&format!("{} is not defined in the settings.toml file", key))
    .clone()
}

enum ParsedLine {
  JoinedLobby { username: String },
  LeftLobby { username: String },
  LobbyList { usernames: Vec<String> },
  GameStart,
  Nothing,
}

fn parse_line(line: &str) -> ParsedLine {
  let joined_lobby_re = Regex::new(r"\[CHAT\] ([^ ]+) has joined").unwrap();
  let left_lobby_re = Regex::new(r"\[CHAT\] ([^ ]+) has quit").unwrap();
  let who_lobby_re = Regex::new(r"\[CHAT\] ONLINE: (.+)").unwrap();
  // TODO: maybe there is better way of checking if a game has started
  let game_start_re = Regex::new(r"\[CHAT\] The game starts in 1 seconds!").unwrap();

  if let Some(captures) = joined_lobby_re.captures(line) {
    ParsedLine::JoinedLobby {
      username: captures[1].to_string(),
    }
  } else if let Some(captures) = left_lobby_re.captures(line) {
    ParsedLine::LeftLobby {
      username: captures[1].to_string(),
    }
  } else if let Some(captures) = who_lobby_re.captures(line) {
    ParsedLine::LobbyList {
      usernames: captures[1].split(", ").map(|x| x.to_string()).collect(),
    }
  } else if game_start_re.is_match(line) {
    ParsedLine::GameStart
  } else {
    ParsedLine::Nothing
  }
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
