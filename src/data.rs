use crate::fetching;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct PlayerStats {
  pub username: String,

  pub no_data: bool, // if true, it probably means that the player is nicked

  pub rank: Option<String>,
  pub donator_rank: Option<String>,
  pub monthly_rank: Option<String>, // if its "SUPERSTAR", its probably mvp++

  pub achievement_points: Option<i64>,

  pub karma: Option<i64>,

  pub beds_broken_bedwars: Option<i64>,
  pub beds_lost_bedwars: Option<i64>,

  pub final_kills_bedwars: Option<i64>,
  pub final_deaths_bedwars: Option<i64>,

  pub games_played_bedwars: Option<i64>,

  pub wins_bedwars: Option<i64>,
  pub losses_bedwars: Option<i64>,

  pub bedwars_level: Option<i64>,

  pub bedwars_winstreak: Option<i64>,

  pub guild_name: Option<String>,

  pub beds_ratio: f64,
  pub final_ratio: f64,
  pub win_ratio: f64,
}

pub fn get_stats(username: &str) -> PlayerStats {
  let game_stats_response = fetching::get_game_stats(username.to_string());
  let guild_response = fetching::get_guild(username.to_string());

  if game_stats_response.is_err() {
    dbg!(game_stats_response.unwrap_err());

    return PlayerStats {
      username: username.to_string(),
      no_data: true,
      ..Default::default()
    };
  }

  let player = &game_stats_response.unwrap()["player"];
  let guild = guild_response.map(|g| g["guild"].clone()).ok();

  let mut player = PlayerStats {
    username: username.to_string(),
    no_data: false,

    rank: player["rank"].as_str().map(|x| x.to_string()),
    donator_rank: player["newPackageRank"].as_str().map(|x| x.to_string()),
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

    guild_name: guild.map(|g| g["name"].as_str().map(|x| x.to_string())).flatten(),

    beds_ratio: 0.0,
    final_ratio: 0.0,
    win_ratio: 0.0,
  };

  if let Some(beds_broken_bedwars) = player.beds_broken_bedwars {
    if let Some(beds_lost_bedwars) = player.beds_lost_bedwars {
      player.beds_ratio = beds_broken_bedwars as f64 / beds_lost_bedwars as f64;
    }
  }

  if let Some(final_kills_bedwars) = player.final_kills_bedwars {
    if let Some(final_deaths_bedwars) = player.final_deaths_bedwars {
      player.final_ratio = final_kills_bedwars as f64 / final_deaths_bedwars as f64;
    }
  }

  if let Some(wins_bedwars) = player.wins_bedwars {
    if let Some(losses_bedwars) = player.losses_bedwars {
      player.win_ratio = wins_bedwars as f64 / losses_bedwars as f64;
    }
  }

  player
}

pub fn sort_players(data_arc: Arc<Mutex<crate::app::AppData>>) {
  let mut data = data_arc.lock().unwrap();

  data
    .players
    .sort_by(|p1, p2| p2.final_ratio.partial_cmp(&p1.final_ratio).unwrap());
}
