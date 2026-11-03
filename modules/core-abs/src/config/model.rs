use super::templates::get_config_path;
use serde::{Deserialize, Serialize};
// use std::fs;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[serde(rename = "active_distro")]
    pub _active_distro: String,

    #[serde(flatten)]
    pub modules: toml::map::Map<String, toml::Value>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let path = crate::config::templates::get_config_path();
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config (did you run paw --create?): {}", e))?;

        let mut config: Config =
            toml::from_str(&contents).map_err(|e| format!("Failed to parse config.toml: {}", e))?;

        let customs_path = crate::config::templates::get_customs_path(); // this will MERGE the customs file if there is one
        if customs_path.exists() {
            if let Ok(custom_contents) = std::fs::read_to_string(&customs_path) {
                // parse customs as a generic table
                if let Ok(toml::Value::Table(custom_modules)) = toml::from_str(&custom_contents) {
                    // append new modules or inject into existing ones
                    for (mod_name, mod_data) in custom_modules {
                        match config.modules.get_mut(&mod_name) {
                            Some(toml::Value::Table(base_table)) => {
                                // merge actions inside
                                if let toml::Value::Table(new_table) = mod_data {
                                    for (act_name, act_data) in new_table {
                                        base_table.insert(act_name, act_data);
                                    }
                                }
                            }
                            _ => {
                                config.modules.insert(mod_name, mod_data);
                            }
                        }
                    }
                } else {
                    println!(
                        "[paw] Warning: customs.toml is incorrectly formatted. Skipping customs..."
                    );
                    println!(
                        "[paw] We recommend you to read the documentation for this part on the REPO."
                    );
                }
            }
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = get_config_path();
        let toml_string =
            toml::to_string(self).map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&path, toml_string)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }
}
