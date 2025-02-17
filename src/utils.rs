use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub teloxide_token: String,
    pub notion_token: String,
    pub notion_parent_page_id: String,
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

pub fn round_to_two_digits(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub fn get_month_number(month_name: &str) -> Option<u32> {
    match month_name.to_lowercase().as_str() {
        "january" => Some(1),
        "february" => Some(2),
        "march" => Some(3),
        "april" => Some(4),
        "may" => Some(5),
        "june" => Some(6),
        "july" => Some(7),
        "august" => Some(8),
        "september" => Some(9),
        "october" => Some(10),
        "november" => Some(11),
        "december" => Some(12),
        _ => None,
    }
}
