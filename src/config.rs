use std::fs::{ self, OpenOptions };
use std::io::Write;
use serde::{ Serialize, Deserialize };
use serde_json;

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Clone)]
pub struct Preferences {
    pub theme: String,
    pub word_count: u8,
    pub letter_count: u8,
    pub format: String,
    pub save_directory: String
}

pub fn get_preferences() -> Preferences {
    let data = match fs::read_to_string("./data/preferences.json") {
        Ok(val) => val,
        Err(_) => return Preferences::default()
    };

    match serde_json::from_str(&data) {
        Ok(val) => val,
        Err(_) => Preferences::default()
    }
}

pub fn save_preferences(prefs: Preferences) -> bool {
    let data = match serde_json::to_string_pretty(&prefs) {
        Ok(val) => val,
        Err(_) => return false
    };

    let mut file = match OpenOptions::new().write(true).create(true).truncate(true).open("./data/preferences.json") {
        Ok(val) => val,
        Err(_) => return false
    };
    
    match file.write_all(&data.as_bytes()) {
        Ok(_) => true,
        Err(_) => false
    }
}

impl Preferences {
    pub fn default() -> Preferences {
        Preferences {
            theme: String::from("Dark"),
            word_count: 15,
            letter_count: 11,
            format: String::from("Letter"),
            save_directory: String::from("./out")
        }
    }
}