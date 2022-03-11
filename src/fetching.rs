use cached::proc_macro::cached;

#[cached]
pub fn get_uuid(username: String) -> Result<String, String> {
  let response = reqwest::blocking::get(format!("https://api.mojang.com/users/profiles/minecraft/{}", username))
    .map_err(|e| e.to_string())?
    .json::<serde_json::Value>()
    .map_err(|e| e.to_string())?;

  response["id"]
    .as_str()
    .ok_or_else(|| "Couldn't get uuid field".to_string())
    .map(|x| x.to_string())
}

#[cached(time = 180)]
pub fn get_data(username: String) -> Result<serde_json::Value, String> {
  let api_key = crate::get_toml_value("settings.toml", "api_key")
    .as_str()
    .unwrap()
    .to_string();

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
