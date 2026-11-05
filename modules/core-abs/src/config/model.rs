use super::templates::get_config_path;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use toml::Value;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(rename = "active_distro")]
    pub _active_distro: String,

    #[serde(flatten)]
    pub modules: toml::map::Map<String, Value>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let root_path = get_config_path();

        let root_contents = std::fs::read_to_string(&root_path)
            .map_err(|e| format!("Failed to read config (did you run snt --create?): {}", e))?;

        let mut base_config: Config = toml::from_str(&root_contents)
            .map_err(|e| format!("Failed to parse config.toml: {}", e))?;

        let base_dir = root_path.parent().unwrap_or(Path::new("")).to_path_buf();

        let mut visited = Vec::new();
        Self::resolve_includes(&mut base_config.modules, &base_dir, &mut visited)?;

        Ok(base_config)
    }

    fn resolve_includes(
        base_modules: &mut toml::map::Map<String, Value>,
        base_dir: &PathBuf,
        visited: &mut Vec<PathBuf>,
    ) -> Result<(), String> {
        let mut paths_to_include = Vec::new();

        if let Some(include_val) = base_modules.remove("include") {
            match include_val {
                Value::String(s) => paths_to_include.push(s),
                Value::Array(arr) => {
                    for item in arr {
                        if let Value::String(s) = item {
                            paths_to_include.push(s);
                        }
                    }
                }
                _ => println!("[snt] Warning: 'include' key must be a string or an array of strings."),
            }
        }

        for path_str in paths_to_include {
            let expanded_path = expand_path(&path_str, base_dir);

            if visited.contains(&expanded_path) {
                println!("[snt] Warning: Circular include detected for {}. Skipping...", expanded_path.display());
                continue;
            }

            visited.push(expanded_path.clone());

            if expanded_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&expanded_path) {
                    if let Ok(Value::Table(mut included_table)) = toml::from_str(&contents) {

                        // RECURSIVE
                        let new_base_dir = expanded_path.parent().unwrap_or(Path::new("")).to_path_buf();
                        Self::resolve_includes(&mut included_table, &new_base_dir, visited)?;
                        merge_tables(base_modules, included_table);
                    } else {
                        println!("[snt] Warning: {} is incorrectly formatted.", expanded_path.display());
                    }
                } else {
                    println!("[snt] Warning: Failed to read included file {}.", expanded_path.display());
                }
            } else {
                println!("[snt] Warning: Included file {} does not exist.", expanded_path.display());
            }
        }

        Ok(())
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

// HELPERS

/// `~/` o `./` paths
fn expand_path(path_str: &str, base_dir: &PathBuf) -> PathBuf {
    if path_str.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(&path_str[2..])
    } else if path_str.starts_with("./") {
        base_dir.join(&path_str[2..])
    } else if path_str.starts_with('/') {
        PathBuf::from(path_str)
    } else {
        base_dir.join(path_str)
    }
}

fn merge_tables(base: &mut toml::map::Map<String, Value>, new: toml::map::Map<String, Value>) {
    for (k, v) in new {
        match base.get_mut(&k) {
            Some(Value::Table(base_table)) => {
                if let Value::Table(new_table) = v {
                    merge_tables(base_table, new_table);
                } else {
                    base.insert(k, v);
                }
            }
            _ => {
                base.insert(k, v);
            }
        }
    }
}
