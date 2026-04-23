use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Command {
    Unlock,
    KeepUnlocked,
    Lock,
    String(String),
}
