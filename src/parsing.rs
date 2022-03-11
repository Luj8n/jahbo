use crate::data;
use encoding::all::UTF_8;
use encoding::Encoding;
use itertools::Itertools;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const SLEEP_DURATION: u64 = 100;

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

pub fn start_parsing_logs(data_arc: Arc<Mutex<crate::app::AppData>>) {
  let log_file_path = crate::get_toml_value("settings.toml", "log_file")
    .as_str()
    .unwrap()
    .to_string();

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

            let player = data::get_stats(&username); // takes some time

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

          usernames.par_iter().for_each(|username| {
            let data = data_arc.lock().unwrap();
            if data
              .players
              .iter()
              .any(|p| p.username.to_lowercase() == username.to_lowercase())
            {
              // don't add players which are already added
              return;
            }

            drop(data);

            let player = data::get_stats(username); // takes some time

            let mut data = data_arc.lock().unwrap();

            data.players.push(player);
            println!("Added {}", username);
          });
        }
        ParsedLine::GameStart => {
          println!("Game has started");
        }
        ParsedLine::Nothing => {}
      }
    }
    bytes.clear();
  }
}
