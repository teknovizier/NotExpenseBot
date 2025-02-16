use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub teloxide_token: String,
    pub notion_token: String,
    pub log_path: String,
    pub restrict_access: bool,
    pub allowed_users: Vec<u64>,
    pub categories: Vec<String>,
    pub subcategories: Vec<String>,
    pub default_currency: String,
}

pub fn load_config(file: &str) -> Config {
    let contents = match fs::read_to_string(file) {
        Ok(_a) => _a,
        Err(_) => panic!("Could not read file \"{}\"", file),
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(_b) => _b,
        Err(_) => panic!("Unable to load data from \"{}\"", file),
    };

    config
}
