use crate::data;
use crate::data::PlayerStats;
use eframe::egui::RichText;
use eframe::epaint::Color32;
use eframe::{egui, epi};
use itertools::Itertools;
use std::sync::{Arc, Mutex};
use std::thread;

const PAUSED_BY_DEFAULT: bool = false; // for release should always be true

fn get_rank_color_and_name(rank: &str, donator_rank: &str, monthly_rank: &str) -> (Color32, String) {
  match (rank, donator_rank, monthly_rank) {
    ("ADMIN", _, _) => (Color32::RED, "[ADMIN] ".to_string()),
    ("GAME_MASTER", _, _) => (Color32::GREEN, "[GM] ".to_string()),
    ("YOUTUBER", _, _) => (Color32::LIGHT_RED, "[YOUTUBE] ".to_string()),
    (_, "MVP_PLUS", "SUPERSTAR") => (Color32::GOLD, "[MVP++] ".to_string()),
    (_, "MVP_PLUS", _) => (Color32::LIGHT_BLUE, "[MVP+] ".to_string()),
    (_, "MVP", _) => (Color32::LIGHT_BLUE, "[MVP] ".to_string()),
    (_, "VIP_PLUS", _) => (Color32::LIGHT_GREEN, "[VIP+] ".to_string()),
    (_, "VIP", _) => (Color32::LIGHT_GREEN, "[VIP] ".to_string()),
    _ => (Color32::GRAY, "".to_string()),
  }
}

#[derive(Debug)]
pub struct AppSettings {
  pub paused: bool,
  pub auto_join_active: bool,
  pub auto_leave_active: bool,
  pub auto_add_on_who: bool,
  pub auto_clear_on_who: bool,
}

impl Default for AppSettings {
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
pub struct AppData {
  pub players: Vec<PlayerStats>,
  pub settings: AppSettings,
}

#[derive(Debug)]
pub struct App {
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
    // self.clear_color();
    ctx.set_visuals(egui::Visuals::dark()); // dark theme

    egui::SidePanel::left("left_panel").show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.add_space(10.);
        ui.vertical(|ui| {
          ui.add_space(20.);

          let player_add_text_response = ui.text_edit_singleline(&mut self.player_add_text);

          ui.add_space(5.);
          ui.horizontal(|ui| {
            if ui.button("Add player").clicked()
              || (player_add_text_response.lost_focus() && ui.input().key_pressed(egui::Key::Enter))
            {
              player_add_text_response.request_focus();

              let data = self.data.lock().unwrap();

              let username = self.player_add_text.trim();
              // dont add a player that is an empty string or is already added
              if !username.is_empty()
                && !data
                  .players
                  .iter()
                  .any(|p| p.username.to_lowercase() == username.to_lowercase())
              {
                drop(data);

                let player = data::get_stats(username); // takes some time

                let mut data = self.data.lock().unwrap();

                data.players.push(player);
                self.player_add_text.clear();
              }
            }

            let mut data = self.data.lock().unwrap();

            if ui.button("Remove all players").clicked() {
              data.players.clear();
            }
          });
          ui.add_space(10.);

          let mut data = self.data.lock().unwrap();

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
          ui.add(egui::Slider::new(&mut self.font_size, 6.0..=40.0).text("Font size"));
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
          player.donator_rank.as_ref().unwrap_or(&"".to_string()),
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
              ui.strong(self.small_text("Not a real minecraft username. Could be nicked", Color32::GRAY));
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

    thread::spawn(|| crate::parsing::start_parsing_logs(data_arc));
  }
}
