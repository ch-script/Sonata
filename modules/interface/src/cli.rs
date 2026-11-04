use orchestrator::workspace::{create_workspace, delete_workspace, add_command, activate, deactivate, run_wp};
use core_abs::config::model::Config;
use core_abs::config::templates::{
    backup, clean_backups, clean_config, init_config, init_customs, load_profile, update_config,
};
use core_abs::sys::executor::execute;

use crate::ui::interactive::run_interactive_help;

use std::env;

pub fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    // Process flags
    if args.len() > 1 && process_flags(&args)? {
        return Ok(());
    }

    let config = Config::load()?;

    let (module, action, extra_args) = resolve_target(&args, &config)?;

    let actions = config
        .modules
        .get(&module)
        .ok_or_else(|| format!("Module '{}' not found in config", module))?;

    let cmd_raw = actions
        .get(&action)
        .ok_or_else(|| format!("Action '{}' not found in module '{}'", action, module))?;

    let actual_cmd_str = match cmd_raw {
        toml::Value::String(s) => s.as_str(),
        toml::Value::Table(t) => t.get("cmd").and_then(|v| v.as_str()).unwrap_or_else(|| {
            panic!("Error: The advanced command does not have a cmd valid camp")
        }),
        _ => panic!("Error: Invalid command format for the TOML"),
    };

    let mut final_args = extra_args.clone(); // variables

    if let Some(toml::Value::Table(vars_table)) = actions.get("vars") {
        if let Some(toml::Value::Table(action_vars)) = vars_table.get(&action) {
            if final_args.len() == 1 {
                let possible_var_name = final_args[0].clone();

                if let Some(toml::Value::Table(v)) = action_vars.get(&possible_var_name) {
                    if let Some(val) = v.get("val").and_then(|val| val.as_str()) {
                        final_args[0] = val.to_string(); // mutable, not yet fully optimized as it should be
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
    // ----------------------------------
    execute(actual_cmd_str, &final_args)
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
) -> Result<(String, String, Vec<String>), String> {
    if args.len() >= 3 {
        Ok((args[1].clone(), args[2].clone(), args[3..].to_vec()))
    } else if args.len() == 1 || (args.len() == 2 && args[1] == "help") {
        match run_interactive_help(config) {
            Some((m, a, extra_args)) => Ok((m, a, extra_args)),
            None => Err("Canceled operation".to_string()),
        }
    } else {
        Err(
            "Usage: snt <module> <action> [args...]\n   or: snt (for interactive super easy mode :D)"
                .to_string(),
        )
    }
}
