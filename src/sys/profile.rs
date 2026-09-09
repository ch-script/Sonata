use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemProfile {
    pub os: String,
    pub kernel: String,
    pub pkg_manager: String,
    pub init_system: String,
    pub gpu: Option<String>,
    pub de: Option<String>,
    pub wm: Option<String>,
    pub audio: Option<String>,
    pub core_utils: String,
    pub user_manager: String,
    pub net_manager: String,
}
