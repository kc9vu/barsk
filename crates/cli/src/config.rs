use std::fs::File;

use super::structs::Level;
use cipher::Cipher;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Conf {
    pub server: Option<String>,
    pub device_key: Option<String>,
    pub archive: Option<bool>,
    pub level: Option<Level>,
    pub group: Option<String>,
    pub icon: Option<String>,
    pub sound: Option<String>,
    pub encrypt: Option<bool>,
    pub cipher: Option<Cipher>,
    pub key: Option<String>,
    pub iv: Option<String>,
}

impl Conf {
    pub fn check(&self) -> Result<(), String> {
        if let Some(true) = self.encrypt {
            if self.key.is_some() ^ self.iv.is_some() {
                return Err(
                    "The key and iv in the configuration file must exist at the same time".into(),
                );
            }
        }
        Ok(())
    }

    pub fn file(filepath: &str) -> Result<Self, String> {
        let reader = match File::open(filepath) {
            Ok(f) => f,
            Err(error) => return Err(error.to_string()),
        };
        let config: Self = match serde_json::from_reader(reader) {
            Ok(conf) => conf,
            Err(e) => return Err(e.to_string()),
        };
        config.check()?;
        Ok(config)
    }

    pub fn default_file() -> Result<Option<Self>, String> {
        let cur_exe = match std::env::current_exe() {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };
        let cur_dir = match cur_exe.parent() {
            Some(dir) => dir,
            None => {
                return Err("Unable to locate the directory where the program is located".into())
            }
        };
        let config_file = cur_dir.join("bark.json");
        if config_file.exists() {
            Ok(Some(Self::file(config_file.to_str().unwrap())?))
        } else {
            Ok(None)
        }
    }
}
