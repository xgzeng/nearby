use crate::{commands::Command, distance_rssi};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::Path;

use std::sync::Mutex;

pub fn save_config(path: &Path, data: &ConfigData) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(parent)?;
        }
    }

    let toml_string = toml::to_string_pretty(data)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;

    use std::io::Write;
    file.write_all(toml_string.as_bytes())?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigData {
    #[serde(default)]
    pub connection: Vec<Connection>,
}

#[derive(Debug)]
pub struct Config {
    inner: Mutex<ConfigData>,
}

impl Config {
    pub fn new(data: ConfigData) -> Self {
        Self {
            inner: Mutex::new(data),
        }
    }

    pub fn connections(&self) -> Vec<Connection> {
        self.inner.lock().unwrap().connection.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().connection.is_empty()
    }

    pub fn contains(&self, mac: &str) -> bool {
        self.inner.lock().unwrap().connection.iter().any(|c| {
            c.get_ble()
                .map(|ble| ble.mac == mac)
                .unwrap_or(false)
        })
    }

    pub fn update_rssi(&self, mac: &str, rssi: i16) {
        let mut inner = self.inner.lock().unwrap();
        for connection in inner.connection.iter_mut() {
            match connection {
                Connection::Ble(ble) => {
                    if ble.mac == mac {
                        ble.rssi = Some(rssi);
                    }
                }
            }
        }
    }

    pub fn should_lock(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner
            .connection
            .iter()
            .filter_map(|c| c.get_ble())
            .any(|ble| ble.should_lock())
    }

    pub fn can_unlock(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner
            .connection
            .iter()
            .filter_map(|c| c.get_ble())
            .any(|ble| ble.can_unlock())
    }

    pub fn keep_unlocked(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner
            .connection
            .iter()
            .filter_map(|c| c.get_ble())
            .any(|ble| ble.keep_unlocked())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Connection {
    Ble(BLEConnection),
}

impl Connection {
    pub fn get_ble(&self) -> Option<&BLEConnection> {
        match self {
            Connection::Ble(ble) => Some(ble),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BLEConnection {
    pub mac: String,
    pub rssi: Option<i16>,
    pub actions: Option<Vec<Action>>,
}

impl BLEConnection {
    pub fn can_unlock(&self) -> bool {
        let distance = self.rssi.map(distance_rssi).unwrap_or(1000.0);
        self.actions
            .as_ref()
            .map(|actions| {
                actions.iter().any(|a| match a {
                    Action::Nearby(action) => {
                        distance < action.threshold && action.command == Command::Unlock
                    }
                    Action::Away(_) => false,
                })
            })
            .unwrap_or(false)
    }

    pub fn keep_unlocked(&self) -> bool {
        let distance = self.rssi.map(distance_rssi).unwrap_or(1000.0);
        self.actions
            .as_ref()
            .map(|actions| {
                actions.iter().any(|a| match a {
                    Action::Nearby(action) => {
                        distance < action.threshold && action.command == Command::KeepUnlocked
                    }
                    Action::Away(_) => false,
                })
            })
            .unwrap_or(false)
    }

    pub fn should_lock(&self) -> bool {
        let distance = self.rssi.map(distance_rssi).unwrap_or(1000.0);
        self.actions
            .as_ref()
            .map(|actions| {
                actions.iter().any(|a| match a {
                    Action::Nearby(_) => false,
                    Action::Away(action) => {
                        distance > action.threshold && action.command == Command::Lock
                    }
                })
            })
            .unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Nearby(ProximityAction),
    Away(ProximityAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProximityAction {
    #[serde(default)]
    pub threshold: f32,
    pub command: Command,
}

const APP_NAME: &str = "nearby";

pub fn default_config_dir() -> std::path::PathBuf {
    let config_dir = dirs::config_dir().expect("Could not find config directory");
    config_dir.join(APP_NAME)
}

pub fn get_config(cfg_file: Option<&Path>) -> anyhow::Result<Config> {
    let config_file = if let Some(cfg_file) = cfg_file {
        cfg_file.to_path_buf()
    } else {
        default_config_dir().join("config.toml")
    };

    let data: ConfigData = Figment::new()
        .merge(Toml::file(config_file))
        .merge(Env::prefixed(&format!("{APP_NAME}_")))
        .extract()?;

    Ok(Config::new(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::MetadataExt;
    use tempfile::tempdir;

    #[test]
    fn test_save_config_permissions_and_content() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let config_path = dir.path().join("nearby").join("config.toml");
        
        let data = ConfigData {
            connection: vec![Connection::Ble(BLEConnection {
                mac: "AA:BB:CC:DD:EE:FF".to_string(),
                rssi: None,
                actions: Some(vec![Action::Nearby(ProximityAction {
                    threshold: 2.0,
                    command: Command::Unlock,
                })]),
            })],
        };

        save_config(&config_path, &data)?;

        // Check directory permissions
        let parent = config_path.parent().unwrap();
        let parent_metadata = fs::metadata(parent)?;
        assert_eq!(parent_metadata.mode() & 0o777, 0o700);

        // Check file permissions
        let file_metadata = fs::metadata(&config_path)?;
        assert_eq!(file_metadata.mode() & 0o777, 0o600);

        // Check content (round-trip)
        let content = fs::read_to_string(&config_path)?;
        let loaded: ConfigData = toml::from_str(&content)?;
        assert_eq!(loaded.connection.len(), 1);
        let Connection::Ble(ble) = &loaded.connection[0];
        assert_eq!(ble.mac, "AA:BB:CC:DD:EE:FF");

        Ok(())
    }
}
