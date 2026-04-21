use crate::{commands::Command, distance_rssi};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub connection: Vec<Connection>,
}

impl Config {
    pub fn connections(&self) -> Vec<&Connection> {
        self.connection.iter().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.connection.is_empty()
    }

    pub fn update_rssi(&mut self, mac: &str, rssi: i16) {
        for connection in self.connection.iter_mut() {
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
        self.connections()
            .iter()
            .filter_map(|c| c.get_ble())
            .any(|ble| ble.should_lock())
    }

    pub fn can_unlock(&self) -> bool {
        self.connections()
            .iter()
            .filter_map(|c| c.get_ble())
            .any(|ble| ble.can_unlock())
    }

    pub fn keep_unlocked(&self) -> bool {
        self.connections()
            .iter()
            .filter_map(|c| c.get_ble())
            .any(|ble| ble.keep_unlocked())
    }
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Nearby(ProximityAction),
    Away(ProximityAction),
}

#[derive(Deserialize, Debug, Clone)]
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

    let config: Config = Figment::new()
        .merge(Toml::file(config_file))
        .merge(Env::prefixed(&format!("{APP_NAME}_")))
        .extract()?;

    Ok(config)
}
