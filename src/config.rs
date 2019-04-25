use std::fs;
use std::path::Path;

use serde::{Serialize, Deserialize};
use toml;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub respect_vars: bool,
    pub respect_path: bool,
    pub bin_dirs: Vec<String>,
    pub prompt: String,
}

impl Config {
    pub fn load(path: &Path) -> Option<Config> {
        if let Ok(config) = fs::read_to_string(path) {
            if let Ok(config) = toml::from_str(&config) {
                return Some(config);
            }
        }
        None
    }

    pub fn to_string(&self) -> String {
        toml::to_string(&self).unwrap()
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            respect_vars: true,
            respect_path: true,
            bin_dirs: vec![String::from("/bin"), String::from("/usr/bin")],
            prompt: String::from("» "),
        }
    }
}