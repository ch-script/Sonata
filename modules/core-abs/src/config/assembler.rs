use crate::config::model::Config;
use crate::sys::profile::SystemProfile;
use include_dir::{include_dir, Dir};

static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

fn get_fragment(category: &str, name: &str) -> Option<String> {
    let path = format!("{}/{}.toml", category, name);
    TEMPLATES_DIR
        .get_file(&path)
        .and_then(|file| file.contents_utf8().map(|s| s.to_string()))
}

fn merge_tables(base: &mut toml::map::Map<String, toml::Value>, new: toml::map::Map<String, toml::Value>) {
    for (k, v) in new {
        match base.get_mut(&k) {
            Some(toml::Value::Table(base_table)) => {
                if let toml::Value::Table(new_table) = v {
                    merge_tables(base_table, new_table);
                } else {
                    base.insert(k, v);
                }
            }
            _ => { base.insert(k, v); }
        }
    }
}

pub fn assemble_config(profile: &SystemProfile) -> Result<String, String> {
    let mut merged_modules = toml::map::Map::new();

    let mut fragment_keys = vec![
        ("core", profile.core_utils.as_str()),
        ("pkg", profile.pkg_manager.as_str()),
        ("init", profile.init_system.as_str()),
        ("os", profile.os.as_str()),
        ("user", profile.user_manager.as_str()),
        ("net", profile.net_manager.as_str()),
        ("extrapkgs", profile.kernel.as_str()),
        ("bluetooth", "bluetoothctl"),
    ];

    if let Some(gpu) = &profile.gpu { fragment_keys.push(("gpu", gpu.as_str())); }
    if let Some(de) = &profile.de { fragment_keys.push(("de", de.as_str())); }
    if let Some(wm) = &profile.wm { fragment_keys.push(("wm", wm.as_str())); }
    if let Some(audio) = &profile.audio { fragment_keys.push(("audio", audio.as_str())); }

    for (category, name) in fragment_keys {
        if let Some(template_str) = get_fragment(category, name) {
            match toml::from_str::<toml::Value>(&template_str) {
                Ok(toml::Value::Table(parsed_modules)) => {
                    merge_tables(&mut merged_modules, parsed_modules);
                }
                Ok(_) => {
                    eprintln!("[ch] Warning: Fragment {}/{} is not a valid TOML table (missing [category] header?)", category, name);
                }
                Err(e) => {
                    eprintln!("[ch] Error parsing {}/{}.toml: {}", category, name, e);
                }
            }
        }
    }

    let final_config = Config {
        _active_distro: profile.os.clone(),
        modules: merged_modules.into_iter().collect(),
    };

    toml::to_string(&final_config).map_err(|e| format!("Failed to build final config: {}", e))
}