#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use serde_json::from_str;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server: String,
    pub username: String,
    pub password: String,
    pub image_name: String,
}

pub fn get_config<P: AsRef<std::path::Path>>(path: P) -> Config {
    let config_file = std::fs::read_to_string(path).unwrap();
    let config: Config = from_str(&config_file).unwrap();
    config
}
