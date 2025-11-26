use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub music_volume: i32,
    pub sfx_volume: i32,
    pub music_muted: bool,
    pub sfx_muted: bool,
    pub gravity_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window_width: 1280,
            window_height: 720,
            fullscreen: false,
            music_volume: 64,
            sfx_volume: 64,
            music_muted: false,
            sfx_muted: false,
            gravity_mode: false,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        if Path::new(SETTINGS_FILE).exists() {
            match fs::read_to_string(SETTINGS_FILE) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(settings) => return settings,
                    Err(e) => eprintln!("Failed to parse settings: {}", e),
                },
                Err(e) => eprintln!("Failed to read settings file: {}", e),
            }
        }
        
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(SETTINGS_FILE, json)?;
        Ok(())
    }
}
