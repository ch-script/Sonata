use orchestrator::workspace::{create_workspace, delete_workspace, add_command, activate, deactivate, run_wp};
use core_abs::config::model::Config;
use core_abs::config::templates::{
    backup, clean_backups, clean_config, init_config, init_customs, load_profile, update_config, add_includes_to_file
};
use core_abs::sys::executor::{execute, ExecutionTarget};
use crate::ui::interactive::run_interactive_help;

use std::env;

pub fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && process_flags(&args)? {
        return Ok(());
    }

    let config = Config::load()?;

    let (module, action, actual_cmd_str, extra_args) = resolve_target(&args, &config)?;

    let mut final_args = extra_args.clone();

    if let Some(toml::Value::Table(actions)) = config.modules.get(&module) {
        if let Some(toml::Value::Table(vars_table)) = actions.get("vars") {
            if let Some(toml::Value::Table(action_vars)) = vars_table.get(&action) {
                if final_args.len() == 1 {
                    let possible_var_name = final_args[0].clone();
                    if let Some(toml::Value::Table(v)) = action_vars.get(&possible_var_name) {
                        if let Some(val) = v.get("val").and_then(|val| val.as_str()) {
                            final_args[0] = val.to_string();
                            println!("[snt] Using variable '{}': {}", possible_var_name, val);
                        }
                    }
                }
                if final_args.is_empty() {
                    for (var_name, var_data) in action_vars {
                        if let toml::Value::Table(v) = var_data {
                            if v.get("assigned").and_then(|b| b.as_bool()).unwrap_or(false) {
                                if let Some(val) = v.get("val").and_then(|s| s.as_str()) {
                                    final_args.push(val.to_string());
                                    println!(
                                        "[snt] Auto-injecting assigned variable '{}': {}",
                                        var_name, val
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut exec_target = ExecutionTarget::Local; // target resolution

    if let Some(toml::Value::Table(mod_table)) = config.modules.get(&module) {

        fn find_table_props<'a>(val: &'a toml::Value, action: &str) -> Option<&'a toml::map::Map<String, toml::Value>> {
            match val {
                toml::Value::Table(t) => {
                    if let Some(found) = t.get(action) {
                        if let toml::Value::Table(tbl) = found {
                            return Some(tbl);
                        }
                    }
                    for (_, v) in t {
                        if let Some(res) = find_table_props(v, action) {
                            return Some(res);
                        }
                    }
                    None
                }
                _ => None,
            }
        }

        if let Some(tbl) = find_table_props(&toml::Value::Table(mod_table.clone()), &action) {
            if let Some(target_val) = tbl.get("target").and_then(|v| v.as_str()) {
                if target_val == "remote" {
                    let ip = tbl.get("ip")
                        .and_then(|v| v.as_str())
                        .unwrap_or("100.102.144.38") // default
                        .to_string();

                    let port = tbl.get("port")
                        .and_then(|v| v.as_integer())
                        .unwrap_or(9999) as u16;

                    exec_target = ExecutionTarget::Remote { ip, port };
                }
            }
        }
    }
    execute(&actual_cmd_str, &final_args, exec_target)
}


fn process_flags(args: &[String]) -> Result<bool, String> {
    let mut forced_distro = None;

    for i in 1..args.len() {
        if let Some(distro) = args[i].strip_prefix("--force=") {
            forced_distro = Some(distro.to_string());
        } else if args[i] == "--force" && i + 1 < args.len() {
            forced_distro = Some(args[i + 1].clone());
        }
    }

    let mut i = 1;

    while i < args.len() {
        let arg = &args[i];

        if arg == "--create" {
            // tuf logo
            crate::ui::theme::print_logo(); // ik ik ill use "use" later i js want tuf logo

            let profile = core_abs::sys::detector::build_system_profile(forced_distro.clone());
            println!("[snt] Profile detected: {:?}", profile);
            init_config(&profile)?;
            println!("[snt] Conf created! :D all good!");
            return Ok(true);
        } else if arg == "--update" {
            let profile = core_abs::sys::detector::build_system_profile(forced_distro.clone());
            update_config(&profile)?;
            println!("[snt] Upgraded :D");
            return Ok(true);
        } else if arg == "--clean" {
            clean_config()?;
            println!("[snt] Cleaned! :c");
            return Ok(true);
        } else if arg == "--backup" {
            if i + 2 < args.len() {
                let category = &args[i + 1];
                let file_path = &args[i + 2];
                backup(category, file_path)?;
                return Ok(true);
            } else {
                return Err("Usage: snt --backup <category> <file_path>".to_string());
            }
        } else if arg == "--clean-backups" {
            let category = if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                Some(args[i + 1].as_str())
            } else {
                None
            };
            clean_backups(category)?;
            return Ok(true);
        } else if arg == "--include" {
            let mut paths = Vec::new();
            let mut origin = core_abs::config::templates::get_config_path();

            let mut j = i + 1;
            while j < args.len() {
                if args[j] == "--in" {
                    if j + 1 < args.len() {
                        let origin_str = &args[j + 1];
                        if origin_str.starts_with("~/") {
                            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                            origin = std::path::PathBuf::from(home).join(&origin_str[2..]);
                        } else if origin_str.starts_with('/') {
                            origin = std::path::PathBuf::from(origin_str);
                        } else {
                            let config_dir = core_abs::config::templates::get_config_path().parent().unwrap().to_path_buf();
                            origin = config_dir.join(origin_str);
                        }
                        break; // Stop parsing include paths after --in is found
                    } else {
                        return Err("Usage: snt --include <path1> <path2> --in <origin>".to_string());
                    }
                } else if !args[j].starts_with("--") {
                    paths.push(args[j].clone());
                    j += 1;
                } else {
                    break;
                }
            }

            if paths.is_empty() {
                return Err("Usage: snt --include <path1> <path2> [--in <origin>]".to_string());
            }

            add_includes_to_file(&origin, paths)?;
            println!("[snt] Includes successfully injected into {}", origin.display());
            return Ok(true);
        } else if arg == "--add-var" {
            if i + 5 < args.len() && args[i + 3] == "--target" {
                let var_name = &args[i + 1];
                let var_val = &args[i + 2];
                let target_mod = &args[i + 4];
                let target_act = &args[i + 5];

                let assigned = i + 6 < args.len() && args[i + 6] == "--assign";

                let mut config = core_abs::config::model::Config::load()?;

                let module_table = config
                    .modules
                    .entry(target_mod.clone())
                    .or_insert(toml::Value::Table(toml::map::Map::new()))
                    .as_table_mut()
                    .unwrap();

                let vars_table = module_table
                    .entry("vars".to_string())
                    .or_insert(toml::Value::Table(toml::map::Map::new()))
                    .as_table_mut()
                    .unwrap();

                let act_table = vars_table
                    .entry(target_act.clone())
                    .or_insert(toml::Value::Table(toml::map::Map::new()))
                    .as_table_mut()
                    .unwrap();

                let mut new_var = toml::map::Map::new();
                new_var.insert("val".to_string(), toml::Value::String(var_val.clone()));
                new_var.insert("assigned".to_string(), toml::Value::Boolean(assigned));

                act_table.insert(var_name.clone(), toml::Value::Table(new_var));

                config.save()?;
                println!(
                    "[snt] Variable '{}' saved successfully for {}:{}!",
                    var_name, target_mod, target_act
                );
                if assigned {
                    println!("[snt] It is assigned as default.");
                }
                return Ok(true);
            } else {
                return Err(
                    "Usage: snt --add-var <name> <value> --target <module> <action> [--assign]"
                        .to_string(),
                );
            }
        } else if arg == "--delete-var" {
            // snt --delete-var <name> --target <module> <action>
            if i + 4 < args.len() && args[i + 2] == "--target" {
                let var_name = &args[i + 1];
                let target_mod = &args[i + 3];
                let target_act = &args[i + 4];

                let mut config = core_abs::config::model::Config::load()?;

                if let Some(toml::Value::Table(mod_table)) = config.modules.get_mut(target_mod) {
                    if let Some(toml::Value::Table(vars_table)) = mod_table.get_mut("vars") {
                        if let Some(toml::Value::Table(act_table)) = vars_table.get_mut(target_act)
                        {
                            if act_table.remove(var_name).is_some() {
                                config.save()?;
                                println!(
                                    "[snt] Variable '{}' deleted successfully from {}:{}!",
                                    var_name, target_mod, target_act
                                );
                                return Ok(true);
                            }
                        }
                    }
                }
                return Err(format!(
                    "Variable '{}' not found in {}:{}.",
                    var_name, target_mod, target_act
                ));
            } else {
                return Err("Usage: snt --delete-var <name> --target <module> <action>".to_string());
            }
        } else if arg == "--profile" {
            let mut verbose = false;
            let mut target_field = None;

            let mut j = i + 1;
            while j < args.len() && args[j].starts_with("--") {
                if args[j] == "--verbose" {
                    verbose = true;
                } else {
                    // Most useful things of em all, there's petitions for specific things
                    target_field = Some(args[j].strip_prefix("--").unwrap().to_string());
                }
                j += 1;
            }

            match load_profile() {
                // specific fields
                Ok(prof) => {
                    if let Some(field) = target_field {
                        let val = match field.as_str() {
                            "os" => prof.os,
                            "net" | "net_manager" => prof.net_manager,
                            "pkg" | "pkg_manager" => prof.pkg_manager,
                            "init" | "init_system" => prof.init_system,
                            "wm" => prof.wm.unwrap_or_else(|| "None".to_string()),
                            "de" => prof.de.unwrap_or_else(|| "None".to_string()),
                            "gpu" => prof.gpu.unwrap_or_else(|| "None".to_string()),
                            "audio" => prof.audio.unwrap_or_else(|| "None".to_string()),
                            _ => format!("Unknown field: {}", field),
                        };

                        if verbose {
                            println!("[snt] {}: {}", field.to_uppercase(), val);
                        } else {
                            println!("{}", val);
                        }
                    } else {
                        if verbose {
                            println!("Sonata System Profile");
                            println!("{:#?}", prof);
                        } else {
                            println!(
                                "OS: {} | PKG: {} | INIT: {}",
                                prof.os, prof.pkg_manager, prof.init_system
                            );
                            println!("(Use --verbose to see all fields)");
                        }
                    }
                }
                Err(e) => println!(
                    "[snt] Could not load profile (run snt --create first?): {}",
                    e
                ),
            }
            return Ok(true);
        } else if arg == "--allow" {
            // Check if the next argument is "customs"
            if i + 1 < args.len() && args[i + 1] == "customs" {
                init_customs()?;
                return Ok(true);
            } else {
                return Err("Usage: snt --allow customs".to_string());
            }
            //return Ok(true);
        } else if arg == "init" {
            if i + 2 < args.len() {
                let ws_name = &args[i + 1];
                let action = &args[i + 2];

                if action == "--create" {
                    create_workspace(ws_name)?;
                    return Ok(true);
                } else if action == "--add" && i + 3 < args.len() {
                    let cmd_to_add = &args[i + 3];
                    add_command(ws_name, cmd_to_add)?;
                    return Ok(true);
                } else if action == "--activate" {
                    let profile = core_abs::sys::detector::build_system_profile(forced_distro.clone());
                    activate(ws_name, &profile)?;
                    return Ok(true);
                } else if action == "--deactivate" {
                    let profile = core_abs::sys::detector::build_system_profile(forced_distro.clone());
                    deactivate(ws_name, &profile)?;
                    return Ok(true);
                } else if action == "--delete" {
                    let profile = core_abs::sys::detector::build_system_profile(forced_distro.clone());
                    let _ = deactivate(ws_name, &profile);
                    delete_workspace(ws_name)?;
                    return Ok(true);
                } else if action == "--passive" {
                    let mut profile = core_abs::sys::detector::build_system_profile(forced_distro.clone());
                    profile.init_system = "passive".to_string();
                    activate(ws_name, &profile)?;
                    return Ok(true);
                } else if action == "--run" || action == "run" {
                    run_wp(ws_name)?;
                    return Ok(true);
                }
                else {
                    return Err("Usage: snt init <name> [--create | --add \"cmd\" | --activate | --passive | --run | --deactivate | --delete]".to_string());
                }
            } else {
                return Err("Usage: snt init <name> <action>".to_string());
            }
        }

        i += 1;
    }

    Ok(false)
}

fn resolve_target(
    args: &[String],
    config: &Config,
) -> Result<(String, String, String, Vec<String>), String> {
    if args.len() >= 3 {
        let module_name = &args[1];
        let mut current_val = config.modules.get(module_name)
            .ok_or_else(|| format!("Module '{}' not found in config", module_name))?;

        let mut i = 2;

        while i < args.len() {
            match current_val {
                toml::Value::Table(t) => {
                    if t.contains_key("cmd") { break; }
                    if let Some(next_val) = t.get(&args[i]) {
                        current_val = next_val;
                        i += 1;
                    } else { break; }
                }
                toml::Value::String(_) => break,
                _ => break,
            }
        }

        let action_name = args[i - 1].clone();

        let cmd_str = match current_val {
            toml::Value::String(s) => s.as_str().to_string(),
            toml::Value::Table(t) => t.get("cmd").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| {
                panic!("Error: The advanced command does not have a 'cmd' key")
            }),
            _ => return Err("Error: Invalid command format for the TOML".to_string()),
        };

        Ok((module_name.clone(), action_name, cmd_str, args[i..].to_vec()))
    } else if args.len() == 1 || (args.len() == 2 && args[1] == "help") {
        match run_interactive_help(config) {
            Some((_menu, action_key, extra_args)) => {

                fn find_cmd_str(val: &toml::Value, target_action: &str) -> Option<String> {
                    match val {
                        toml::Value::Table(t) => {
                            if let Some(found) = t.get(target_action) {
                                match found {
                                    toml::Value::String(s) => return Some(s.as_str().to_string()),
                                    toml::Value::Table(tbl) => {
                                        if let Some(cmd) = tbl.get("cmd").and_then(|v| v.as_str()) {
                                            return Some(cmd.to_string());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            for (_, v) in t {
                                if let Some(res) = find_cmd_str(v, target_action) {
                                    return Some(res);
                                }
                            }
                            None
                        }
                        _ => None,
                    }
                }
                for (root_name, root_val) in &config.modules {
                    if let Some(cmd_str) = find_cmd_str(root_val, &action_key) {
                        return Ok((root_name.clone(), action_key, cmd_str, extra_args));
                    }
                }

                Err(format!("Action '{}' not found after interactive selection", action_key))
            }
            None => Err("Canceled operation".to_string()),
        }
    } else {
        Err("Usage: snt <module> <action> [args...]\n   or: snt (for interactive mode)".to_string())
    }
}
