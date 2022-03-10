use cached::proc_macro::cached;
use eframe::egui::RichText;
use eframe::epaint::Color32;
use eframe::{egui, epi};
use encoding::all::UTF_8;
use encoding::Encoding;
use itertools::Itertools;
use regex::Regex;
use std::fs::{read_to_string, File};
use std::io::{BufReader, Read};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const SLEEP_DURATION: u64 = 100;
const PAUSED_BY_DEFAULT: bool = true; // for release should always be true

// fn get_toml_value(key: &str) -> String {
//   todo!()
// }

#[cached(result = true)]
fn get_uuid(username: String) -> Result<String, String> {
  let response = reqwest::blocking::get(format!("https://api.mojang.com/users/profiles/minecraft/{}", username))
    .map_err(|e| e.to_string())?
    .json::<serde_json::Value>()
    .map_err(|e| e.to_string())?;

  response["id"]
    .as_str()
    .ok_or("Couldn't get uuid field".to_string())
    .map(|x| x.to_string())
}

#[cached(time = 180, result = true)]
fn get_data(username: String) -> Result<serde_json::Value, String> {
  let settings_file = read_to_string("settings.toml").expect("settings.toml file not found");
  let parsed_settings_file = settings_file
    .parse::<toml::Value>()
    .expect("settings.toml is not valid toml");
  let api_key = parsed_settings_file
    .get("api_key")
    .expect("api_key is not defined in the settings.toml file")
    .as_str()
    .expect("api_key should be a string");

  let uuid = get_uuid(username)?;

  reqwest::blocking::Client::new()
    .get("https://api.hypixel.net/player")
    .query(&[("uuid", uuid)])
    .header("API-Key", api_key)
    .send()
    .map_err(|e| e.to_string())?
    .error_for_status()
    .map_err(|e| e.to_string())?
    .json::<serde_json::Value>()
    .map_err(|e| e.to_string())
}

#[derive(Debug)]
struct Settings {
  paused: bool,
  auto_join_active: bool,
  auto_leave_active: bool,
  auto_add_on_who: bool,
  auto_clear_on_who: bool,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      paused: PAUSED_BY_DEFAULT,
      auto_join_active: true,
      auto_leave_active: true,
      auto_add_on_who: true,
      auto_clear_on_who: true,
    }
  }
}

#[derive(Debug, Default)]
struct Stats {
  username: String,

  no_data: bool, // if true, it probably means that the player is nicked

  rank: Option<String>,
  monthly_rank: Option<String>, // if its "SUPERSTAR", its mvp++, i think

  achievement_points: Option<i64>,

  karma: Option<i64>,

  beds_broken_bedwars: Option<i64>,
  beds_lost_bedwars: Option<i64>,

  final_kills_bedwars: Option<i64>,
  final_deaths_bedwars: Option<i64>,

  games_played_bedwars: Option<i64>,

  wins_bedwars: Option<i64>,
  losses_bedwars: Option<i64>,

  bedwars_level: Option<i64>,

  bedwars_winstreak: Option<i64>,
}

fn get_stats(username: &str) -> Stats {
  let response = get_data(username.to_string());

  if response.is_err() {
    dbg!(response.unwrap_err());

    return Stats {
      username: username.to_string(),
      no_data: true,
      ..Default::default()
    };
  }

  let player = &response.unwrap()["player"];

  Stats {
    username: username.to_string(),
    no_data: false,

    rank: player["newPackageRank"].as_str().map(|x| x.to_string()),
    monthly_rank: player["monthlyPackageRank"].as_str().map(|x| x.to_string()),

    achievement_points: player["achievementPoints"].as_i64(),

    karma: player["karma"].as_i64(),

    beds_broken_bedwars: player["stats"]["Bedwars"]["beds_broken_bedwars"].as_i64(),
    beds_lost_bedwars: player["stats"]["Bedwars"]["beds_lost_bedwars"].as_i64(),

    final_kills_bedwars: player["stats"]["Bedwars"]["final_kills_bedwars"].as_i64(),
    final_deaths_bedwars: player["stats"]["Bedwars"]["final_deaths_bedwars"].as_i64(),

    games_played_bedwars: player["stats"]["Bedwars"]["games_played_bedwars"].as_i64(),

    wins_bedwars: player["stats"]["Bedwars"]["wins_bedwars"].as_i64(),
    losses_bedwars: player["stats"]["Bedwars"]["losses_bedwars"].as_i64(),

    bedwars_level: player["achievements"]["bedwars_level"].as_i64(),

    bedwars_winstreak: player["stats"]["Bedwars"]["winstreak"].as_i64(),
  }
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

#[derive(Debug, Default)]
struct AppData {
  players: Vec<Stats>,
  settings: Settings,
}
#[derive(Debug)]
struct App {
  data: Arc<Mutex<AppData>>,

  player_add_text: String,
  font_size: f32,
}

impl Default for App {
  fn default() -> Self {
    Self {
      data: Default::default(),
      player_add_text: Default::default(),
      font_size: 14.,
    }
  }
}

impl App {
  fn small_text(&self, text: &str, color: Color32) -> RichText {
    RichText::new(text).color(color).size(self.font_size)
  }
  fn big_text(&self, text: &str, color: Color32) -> RichText {
    RichText::new(text).color(color).size(self.font_size + 4.)
  }
}

impl epi::App for App {
  fn name(&self) -> &str {
    "Jahbo"
  }

  fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
    ctx.set_visuals(egui::Visuals::dark()); // dark theme

    egui::SidePanel::left("left_panel").show(ctx, |ui| {
      ui.add_space(20.);
      ui.horizontal(|ui| {
        ui.add_space(10.);
        ui.vertical(|ui| {
          let player_add_text_response = ui.text_edit_singleline(&mut self.player_add_text);
          if ui.button("Add player").clicked()
            || (player_add_text_response.lost_focus() && ui.input().key_pressed(egui::Key::Enter))
          {
            let data = self.data.lock().unwrap();

            // dont add player that is already added
            if !data
              .players
              .iter()
              .any(|p| p.username.to_lowercase() == self.player_add_text.to_lowercase())
            {
              drop(data);

              let player = get_stats(&self.player_add_text); // takes some time

              let mut data = self.data.lock().unwrap();

              data.players.push(player);
              self.player_add_text.clear();
            }
          }

          let mut data = self.data.lock().unwrap();

          if ui.button("Remove all players").clicked() {
            data.players.clear();
          }
          ui.add_space(10.);

          ui.checkbox(&mut data.settings.paused, "Paused")
            .on_hover_text("Will not change any data automatically while paused");
          ui.checkbox(&mut data.settings.auto_join_active, "Auto join")
            .on_hover_text("If someone joins a bedwars lobby, it will add them");
          ui.checkbox(&mut data.settings.auto_leave_active, "Auto leave")
            .on_hover_text("If someone leaves a bedwars lobby, it will remove them");
          ui.checkbox(&mut data.settings.auto_add_on_who, "Auto add on who")
            .on_hover_text("On /who it will add all the players which are not already added");
          ui.checkbox(&mut data.settings.auto_clear_on_who, "Auto clear on who")
            .on_hover_text("On /who it will first remove all the players");

          ui.add_space(10.);
          ui.add(egui::Slider::new(&mut self.font_size, 6.0..=40.0).text("My value"));
        });
        ui.add_space(10.);
      });
    });

    let mut data = self.data.lock().unwrap();

    let mut players_to_remove: Vec<String> = vec![];

    egui::CentralPanel::default().show(ctx, |_| {
      for player in data.players.iter() {
        let (title_color, rank_text) = get_rank_color_and_name(
          player.rank.as_ref().unwrap_or(&"".to_string()),
          player.monthly_rank.as_ref().unwrap_or(&"".to_string()),
        );
        let title_text = format!(
          "{}{} ⭐{}",
          rank_text,
          player.username,
          player.bedwars_level.map_or("N/A".to_string(), |x| x.to_string())
        );

        let title = self.big_text(&title_text, title_color);

        let mut window_is_open = true;

        egui::Window::new(title)
          .resizable(false)
          .open(&mut window_is_open)
          .show(ctx, |ui| {
            if player.no_data {
              ui.strong(self.small_text("No data  - probably nicked", Color32::GRAY));
              return;
            }

            let mut tag = self.small_text("Tag: None", Color32::GRAY);

            if let Some(bedwars_level) = player.bedwars_level {
              let bedwars_level = bedwars_level as f64;
              if let Some(final_kills_bedwars) = player.final_kills_bedwars {
                let final_kills_bedwars = final_kills_bedwars as f64;
                if let Some(final_deaths_bedwars) = player.final_deaths_bedwars {
                  let final_deaths_bedwars = final_deaths_bedwars as f64;
                  if (bedwars_level < 15. && final_kills_bedwars / final_deaths_bedwars > 5.)
                    || (bedwars_level > 15.
                      && bedwars_level < 100.
                      && bedwars_level / (final_kills_bedwars / final_deaths_bedwars) <= 5.)
                  {
                    tag = self.small_text("Tag: ALT", Color32::YELLOW);
                  } else if let Some(losses_bedwars) = player.losses_bedwars {
                    let losses_bedwars = losses_bedwars as f64;
                    if bedwars_level < 150.
                      && final_deaths_bedwars / losses_bedwars < 0.75
                      && final_kills_bedwars / final_deaths_bedwars < 1.5
                    {
                      tag = self.small_text("Tag: SNIPER", Color32::LIGHT_RED);
                    }
                  }
                }
              }
            }

            let mut beds_ratio = 0.0;

            if let Some(beds_broken_bedwars) = player.beds_broken_bedwars {
              if let Some(beds_lost_bedwars) = player.beds_lost_bedwars {
                beds_ratio = beds_broken_bedwars as f64 / beds_lost_bedwars as f64;
              }
            }

            let mut final_ratio = 0.0;

            if let Some(final_kills_bedwars) = player.final_kills_bedwars {
              if let Some(final_deaths_bedwars) = player.final_deaths_bedwars {
                final_ratio = final_kills_bedwars as f64 / final_deaths_bedwars as f64;
              }
            }

            let mut win_ratio = 0.0;

            if let Some(wins_bedwars) = player.wins_bedwars {
              if let Some(losses_bedwars) = player.losses_bedwars {
                win_ratio = wins_bedwars as f64 / losses_bedwars as f64;
              }
            }

            ui.strong(tag);
            ui.strong(self.small_text(&format!("Final kills/deaths: {:.2}", final_ratio), Color32::GRAY));
            ui.strong(self.small_text(&format!("Wins/losses: {:.2}", win_ratio), Color32::GRAY));

            ui.add_space(15.);

            ui.strong(self.small_text(
              &format!(
                "Achievement points: {}",
                player.achievement_points.map_or("N/A".to_string(), |x| x.to_string())
              ),
              Color32::GRAY,
            ));
            ui.strong(self.small_text(
              &format!(
                "Win streak: {}",
                player.bedwars_winstreak.map_or("N/A".to_string(), |x| x.to_string())
              ),
              Color32::GRAY,
            ));
            ui.label(self.small_text(&format!("Beds broken/lost: {:.2}", beds_ratio), Color32::GRAY));
            ui.label(self.small_text(
              &format!("Karma: {}", player.karma.map_or("N/A".to_string(), |x| x.to_string())),
              Color32::GRAY,
            ));
            ui.label(self.small_text(
              &format!(
                "Beds broken: {}",
                player.beds_broken_bedwars.map_or("N/A".to_string(), |x| x.to_string())
              ),
              Color32::GRAY,
            ));
            ui.label(self.small_text(
              &format!(
                "Beds lost: {}",
                player.beds_lost_bedwars.map_or("N/A".to_string(), |x| x.to_string())
              ),
              Color32::GRAY,
            ));
            ui.label(self.small_text(
              &format!(
                "Final kills: {}",
                player.final_kills_bedwars.map_or("N/A".to_string(), |x| x.to_string())
              ),
              Color32::GRAY,
            ));
            ui.label(self.small_text(
              &format!(
                "Final deaths: {}",
                player.final_deaths_bedwars.map_or("N/A".to_string(), |x| x.to_string())
              ),
              Color32::GRAY,
            ));
            ui.label(self.small_text(
              &format!(
                "Games played: {}",
                player.games_played_bedwars.map_or("N/A".to_string(), |x| x.to_string())
              ),
              Color32::GRAY,
            ));
            ui.label(self.small_text(
              &format!(
                "Wins: {}",
                player.wins_bedwars.map_or("N/A".to_string(), |x| x.to_string())
              ),
              Color32::GRAY,
            ));
            ui.label(self.small_text(
              &format!(
                "Losses: {}",
                player.losses_bedwars.map_or("N/A".to_string(), |x| x.to_string())
              ),
              Color32::GRAY,
            ));
          });

        if !window_is_open {
          players_to_remove.push(player.username.clone());
        }
      }
    });

    for username in players_to_remove {
      if let Some((index, _)) = data.players.iter().find_position(|s| s.username == username) {
        data.players.remove(index);
        println!("Removed {}", username);
      }
    }

    frame.request_repaint();
  }

  fn setup(&mut self, _ctx: &egui::Context, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>) {
    let data_arc = self.data.clone();

    thread::spawn(move || {
      let settings_file = read_to_string("settings.toml").expect("settings.toml file not found");
      let parsed_settings_file = settings_file
        .parse::<toml::Value>()
        .expect("settings.toml is not valid toml");
      let log_file_path = parsed_settings_file
        .get("log_file")
        .expect("log_file is not defined in the settings.toml file")
        .as_str()
        .expect("log_file should be a string");

      let file = File::open(log_file_path).expect("Log file not found");
      let mut reader = BufReader::new(file);
      let mut bytes: Vec<u8> = vec![];

      loop {
        let byte_count = reader
          .read_to_end(&mut bytes)
          .expect("Reading bytes from the .log file went wrong");

        if byte_count == 0 {
          // eof
          thread::sleep(Duration::from_millis(SLEEP_DURATION));
          bytes.clear();
          continue;
        }

        let data = data_arc.lock().unwrap();

        if data.settings.paused {
          continue;
        }

        drop(data);

        let text_to_eof = UTF_8
          .decode(&bytes, encoding::DecoderTrap::Ignore)
          .expect("Decoding to UTF-8 went wrong");

        for line in text_to_eof.lines() {
          match parse_line(line) {
            ParsedLine::JoinedLobby { username } => {
              let data = data_arc.lock().unwrap();
              if !data.settings.auto_join_active {
                return;
              }

              if !data
                .players
                .iter()
                .any(|s| s.username.to_lowercase() == username.to_lowercase())
              {
                drop(data);

                let player = get_stats(&username); // takes some time

                let mut data = data_arc.lock().unwrap();

                data.players.push(player);
                println!("Added {}", username);
              }
            }
            ParsedLine::LeftLobby { username } => {
              let mut data = data_arc.lock().unwrap();
              if !data.settings.auto_leave_active {
                return;
              }

              if let Some((index, _)) = data.players.iter().find_position(|s| s.username == username) {
                data.players.remove(index);
                println!("Removed {}", username);
              }
            }
            ParsedLine::LobbyList { usernames } => {
              let mut data = data_arc.lock().unwrap();

              if data.settings.auto_clear_on_who {
                data.players.clear();
              }

              if !data.settings.auto_add_on_who {
                return;
              }

              drop(data);

              for username in usernames {
                let data = data_arc.lock().unwrap();
                if data
                  .players
                  .iter()
                  .any(|p| p.username.to_lowercase() == username.to_lowercase())
                {
                  // don't add players which are already added
                  continue;
                }

                drop(data);

                let player = get_stats(&username); // takes some time

                let mut data = data_arc.lock().unwrap();

                data.players.push(player);
                println!("Added {}", username);
              }
            }
            ParsedLine::GameStart => {
              println!("Game has started");
            }
            ParsedLine::Nothing => {}
          }
        }
        bytes.clear();
      }
    });
  }
}

fn main() {
  let app = App::default();
  let native_options = eframe::NativeOptions {
    maximized: true,
    ..Default::default()
  };
  eframe::run_native(Box::new(app), native_options);
}
