use crate::config::assembler::assemble_config;
use crate::sys::backup::BackupManager;
use crate::sys::profile::SystemProfile;
use include_dir::{Dir, include_dir};
use std::fs;
use std::path::{Path, PathBuf};

static SCRIPTS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/scripts");

pub fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/snt/config.toml")
}

fn extract_scripts_helper(category: &str, service_name: &str) -> Result<(), String> {
    let mut target_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    target_dir.push("snt");
    target_dir.push("scripts");
    target_dir.push(category);
    target_dir.push(service_name);

    let full_path = format!("{}/{}", category, service_name);

    if let Some(service_scripts) = SCRIPTS_DIR.get_dir(&full_path) {
        // now we have modular scripts
        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create scripts dir: {}", e))?;

        for file in service_scripts.files() {
            if let Some(file_name) = file.path().file_name() {
                let dest_path = target_dir.join(file_name);
                fs::write(&dest_path, file.contents())
                    .map_err(|e| format!("Failed to write script: {}", e))?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = fs::metadata(&dest_path) {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o755);
                        let _ = fs::set_permissions(&dest_path, perms);
                    }
                }
            }
        }
        println!(
            "[snt] Scripts for '{}' extracted and made executable.", // Debug for each
            full_path
        );
    }
    Ok(())
}

fn extract_scripts(profile: &SystemProfile) -> Result<(), String> {
    // Being mandatory or optional will make sense in future updates, stay tuned
    let mandatory: Vec<(&str, &str)> = vec![
        ("os", profile.os.as_str()),
        ("pkg", profile.pkg_manager.as_str()),
        ("init", profile.init_system.as_str()),
        ("core", profile.core_utils.as_str()),
        ("user", profile.user_manager.as_str()),
        ("net", profile.net_manager.as_str()),
    ];

    for (category, service) in mandatory {
        extract_scripts_helper(category, service)?;
    }

    let optional: Vec<(&str, &Option<String>)> = vec![
        ("gpu", &profile.gpu),
        ("de", &profile.de),
        ("wm", &profile.wm),
        ("audio", &profile.audio),
    ];

    for (category, opt) in optional {
        if let Some(service) = opt {
            extract_scripts_helper(category, service.as_str())?;
        }
    }

    Ok(())
}

pub fn clean_config() -> Result<(), String> {
    let path = get_config_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to remove config file: {}", e))?;
        println!("[snt] Configuration file was removed successfully");
    } else {
        println!(
            "[snt] There's no configuration file found to be cleaned, like there isn't at all"
        );
    }
    Ok(())
}

pub fn update_config(profile: &SystemProfile) -> Result<PathBuf, String> {
    let path = get_config_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to remove old config file: {}", e))?;
    }
    init_config(profile)
}

pub fn backup(category: &str, file_path: &str) -> Result<(), String> {
    let manager = BackupManager::new();
    let dest = manager.backup(category, file_path)?;
    println!("[snt] Backup successful.");
    println!("Saved at: {}", dest.display());
    Ok(())
}

pub fn clean_backups(category: Option<&str>) -> Result<(), String> {
    let manager = BackupManager::new();
    manager.clean(category)?;
    println!("[ch] Backups cleaned successfully.");
    Ok(())
}
pub fn get_profile_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/snt/profile.toml")
}

pub fn save_profile(profile: &SystemProfile) -> Result<(), String> {
    let path = get_profile_path();

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create profile directory: {}", e))?;
        }
    }

    let toml_string =
        toml::to_string(profile).map_err(|e| format!("Failed to serialize profile: {}", e))?;

    fs::write(&path, toml_string).map_err(|e| format!("Failed to write profile file: {}", e))?;

    Ok(())
}

pub fn load_profile() -> Result<SystemProfile, String> {
    let path = get_profile_path();
    let contents =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read profile: {}", e))?;

    toml::from_str(&contents).map_err(|e| format!("Failed to parse profile: {}", e))
}

pub fn get_customs_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/snt/customs.toml")
}

pub fn init_customs() -> Result<(), String> {
    let path = get_customs_path();

    if path.exists() {
        return Err("customs.toml already exists! You can edit it directly at ~/.config/snt/customs.toml".to_string());
    }

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Boilerplate for the user to understand how it works
    let boilerplate = r#"# Sonata Customs Configuration
# Here you can define your own modules or extend existing ones!
# These commands will seamlessly integrate into the TUI and CLI.

[my_scripts]
hello = "echo 'Hello!'"

# You can also extend existing ones! (uncomment to test)
# [pkg]
# custom_update = "echo 'updating or sm like that'"
"#;

    std::fs::write(&path, boilerplate)
        .map_err(|e| format!("Failed to create customs file: {}", e))?;

    // Inject the include directly into config.toml
    let config_path = get_config_path();
    if config_path.exists() {
        let _ = add_includes_to_file(&config_path, vec!["./customs.toml".to_string()]);
    }

    println!("[snt] customs.toml initialized successfully and linked in config.toml!");
    Ok(())
}

pub fn init_config(profile: &SystemProfile) -> Result<PathBuf, String> {
    let config_path = get_config_path();

    if config_path.exists() {
        return Err("Config file already exists. Use '--update' to... update it.".to_string());
    }

    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Inject assembled fragments
    let assembled_toml = assemble_config(profile)?;
    fs::write(&config_path, assembled_toml.trim())
        .map_err(|e| format!("Failed to write config file hmph!: {}", e))?;

    extract_scripts(profile)?;

    save_profile(profile)?; // necessary for command "--profile --verbose", it also works as cache for other things...

    Ok(config_path)
}

pub fn add_includes_to_file(file_path: &Path, new_includes: Vec<String>) -> Result<(), String> {
    if !file_path.exists() {
        return Err(format!("File {} does not exist.", file_path.display()));
    }

    let contents = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let mut table: toml::map::Map<String, toml::Value> = toml::from_str(&contents).map_err(|e| e.to_string())?;

    let mut current_includes = Vec::new();
    if let Some(inc) = table.remove("include") {
        match inc {
            toml::Value::String(s) => current_includes.push(s),
            toml::Value::Array(arr) => {
                for item in arr {
                    if let toml::Value::String(s) = item {
                        current_includes.push(s);
                    }
                }
            }
            _ => {}
        }
    }

    for ni in new_includes {
        if !current_includes.contains(&ni) {
            current_includes.push(ni);
        }
    }

    if current_includes.len() == 1 {
        table.insert("include".to_string(), toml::Value::String(current_includes[0].clone()));
    } else if current_includes.len() > 1 {
        let arr = current_includes.into_iter().map(toml::Value::String).collect();
        table.insert("include".to_string(), toml::Value::Array(arr));
    }

    let new_contents = toml::to_string(&table).map_err(|e| e.to_string())?;
    std::fs::write(file_path, new_contents).map_err(|e| e.to_string())?;

    Ok(())
}
