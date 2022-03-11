use crate::fetching;

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
}

pub fn get_stats(username: &str) -> PlayerStats {
  let response = fetching::get_data(username.to_string());

  if response.is_err() {
    dbg!(response.unwrap_err());

    return PlayerStats {
      username: username.to_string(),
      no_data: true,
      ..Default::default()
    };
  }

  let player = &response.unwrap()["player"];

  PlayerStats {
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
  }
}
