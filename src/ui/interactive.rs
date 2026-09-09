use crate::config::model::Config;
//use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use inquire::{Confirm, Select, Text};
use std::io::{Write, stdout};

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

pub fn run_interactive_help(config: &Config) -> Option<(String, String, Vec<String>)> {
    if config.modules.is_empty() {
        println!(
            "No modules available in your config file, that's strange... Maybe we couldn't detect your OS :c"
        );
        return None;
    }
    navigate_menu("Main Menu", &config.modules, vec![])
}

pub fn manage_variables_tui(module: &str, action: &str) -> Result<(), String> {
    let mut stdout = stdout();

    loop {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0)).unwrap();
        println!("Variable Manager for {}:{}", module, action);
        println!("1. Add new variable");
        println!("2. Delete variable");
        println!("3. View my variables");
        println!("4. Return");
        print!("\nSelect an option: ");
        stdout.flush().unwrap();

        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                print!("Variable name (e.g. path): ");
                stdout.flush().unwrap();
                let mut v_name = String::new();
                std::io::stdin().read_line(&mut v_name).unwrap();

                print!("Variable value (e.g. /etc/nixos#paw): ");
                stdout.flush().unwrap();
                let mut v_val = String::new();
                std::io::stdin().read_line(&mut v_val).unwrap();

                print!("Set as default assigned? (y/n): ");
                stdout.flush().unwrap();
                let mut v_assign = String::new();
                std::io::stdin().read_line(&mut v_assign).unwrap();

                let is_assigned = v_assign.trim().to_lowercase() == "y";

                let mut config = Config::load()?;
                let act_table = config
                    .modules
                    .entry(module.to_string())
                    .or_insert(toml::Value::Table(toml::map::Map::new()))
                    .as_table_mut()
                    .unwrap()
                    .entry("vars".to_string())
                    .or_insert(toml::Value::Table(toml::map::Map::new()))
                    .as_table_mut()
                    .unwrap()
                    .entry(action.to_string())
                    .or_insert(toml::Value::Table(toml::map::Map::new()))
                    .as_table_mut()
                    .unwrap();

                let mut new_var = toml::map::Map::new();
                new_var.insert(
                    "val".to_string(),
                    toml::Value::String(v_val.trim().to_string()),
                );
                new_var.insert("assigned".to_string(), toml::Value::Boolean(is_assigned));

                act_table.insert(v_name.trim().to_string(), toml::Value::Table(new_var));
                config.save()?;

                println!("Saved! Press Enter to continue.");
                std::io::stdin().read_line(&mut String::new()).unwrap();
            }
            "2" => {
                print!("Enter variable name to delete: ");
                stdout.flush().unwrap();
                let mut v_name = String::new();
                std::io::stdin().read_line(&mut v_name).unwrap();

                let mut config = Config::load()?;
                let mut deleted = false;

                if let Some(toml::Value::Table(mod_table)) = config.modules.get_mut(module) {
                    if let Some(toml::Value::Table(vars_table)) = mod_table.get_mut("vars") {
                        if let Some(toml::Value::Table(act_table)) = vars_table.get_mut(action) {
                            if act_table.remove(v_name.trim()).is_some() {
                                config.save()?;
                                deleted = true;
                            }
                        }
                    }
                }

                if deleted {
                    println!("Deleted successfully! Press Enter to continue.");
                } else {
                    println!("Variable not found. Press Enter to continue.");
                }
                std::io::stdin().read_line(&mut String::new()).unwrap();
            }
            "3" => {
                let config = Config::load()?;
                println!("\nCurrent Variables");
                if let Some(toml::Value::Table(mod_table)) = config.modules.get(module) {
                    if let Some(toml::Value::Table(vars_table)) = mod_table.get("vars") {
                        if let Some(toml::Value::Table(act_table)) = vars_table.get(action) {
                            for (k, v) in act_table {
                                if let toml::Value::Table(v_data) = v {
                                    let val = v_data
                                        .get("val")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown");
                                    let assigned = v_data
                                        .get("assigned")
                                        .and_then(|b| b.as_bool())
                                        .unwrap_or(false);
                                    let status = if assigned {
                                        "[Assigned]"
                                    } else {
                                        "[Unassigned]"
                                    };
                                    println!("- {} {} -> {}", k, status, val);
                                }
                            }
                        } else {
                            println!("No variables for this action.");
                        }
                    } else {
                        println!("No variables for this module.");
                    }
                }
                println!("-------------------------");
                println!("Press Enter to continue...");
                std::io::stdin().read_line(&mut String::new()).unwrap();
            }
            "4" | "q" => break,
            _ => {}
        }
    }

    Ok(())
}

fn navigate_menu(
    menu_name: &str,
    current_table: &toml::map::Map<String, toml::Value>,
    path: Vec<String>,
) -> Option<(String, String, Vec<String>)> {
    loop {
        clear_screen();
        crate::ui::theme::print_logo();

        let mut keys: Vec<String> = current_table.keys().cloned().collect();
        keys.sort();

        if keys.is_empty() {
            println!("Under construction '{}'.", menu_name);
            return None;
        }

        let mut options: Vec<String> = vec!["Go back".to_string()];

        if !path.is_empty() {
            options.push("Manage Variables".to_string());
        }

        let vars_table_opt = current_table.get("vars").and_then(|v| v.as_table());

        for key in &keys {
            if key == "vars" {
                continue;
            }

            let mut auto_val = None; // see if there is something assigned by default
            if let Some(vars_table) = vars_table_opt {
                if let Some(toml::Value::Table(action_vars)) = vars_table.get(key) {
                    for (_v_name, v_data) in action_vars {
                        if let toml::Value::Table(v_props) = v_data {
                            if v_props
                                .get("assigned")
                                .and_then(|b| b.as_bool())
                                .unwrap_or(false)
                            {
                                if let Some(val) = v_props.get("val").and_then(|s| s.as_str()) {
                                    auto_val = Some(val.to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            match current_table.get(key).unwrap() {
                toml::Value::Table(t) => {
                    if !t.contains_key("cmd") {
                        options.push(format!("{:<18}", key));
                    } else {
                        let mut cmd_str = t
                            .get("cmd")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if let Some(val) = &auto_val {
                            cmd_str = cmd_str.replace("{1}", val);
                        }
                        options.push(format!("{:<18} - ({})", key, cmd_str));
                    }
                }
                toml::Value::String(s) => {
                    let mut cmd_str = s.to_string();
                    if let Some(val) = &auto_val {
                        cmd_str = cmd_str.replace("{1}", val);
                    }
                    options.push(format!("{:<18} - ({})", key, cmd_str));
                }

                _ => {}
            }
        }

        let prompt_msg = if path.is_empty() {
            "Select a module to explore:".to_string()
        } else {
            format!("Path: {} > Select an option:", path.join(" > "))
        };

        let selected_option = match Select::new(&prompt_msg, options).prompt() {
            Ok(opt) => opt,
            Err(_) => return None,
        };

        if selected_option == "Go back" {
            return None;
        }

        if selected_option == "Manage Variables" {
            let mut action_keys: Vec<String> = current_table
                .iter()
                .filter(|(k, v)| {
                    if *k == "vars" {
                        return false;
                    }
                    match v {
                        toml::Value::Table(t) => t.contains_key("cmd"),
                        toml::Value::String(_) => true,
                        _ => false,
                    }
                })
                .map(|(k, _)| k.clone())
                .collect();

            if action_keys.is_empty() {
                let _ =
                    Text::new("No actions available to manage variables. Press Enter...").prompt();
                continue;
            }

            action_keys.sort();
            action_keys.push("Cancel".to_string());

            if let Ok(act) = Select::new("Manage variables for which action?", action_keys).prompt()
            {
                if act != "Cancel" {
                    let _ = manage_variables_tui(&path[0], &act); // path[0] is the actual module .-.
                }
            }
            continue;
        }

        let selected_key = selected_option
            .split_whitespace()
            .next()
            .unwrap()
            .to_string();
        let value = current_table.get(&selected_key).unwrap();

        match value {
            // submenu or advanced command
            toml::Value::Table(t) => {
                if let Some(cmd_val) = t.get("cmd").and_then(|v| v.as_str()) {
                    // advanced comm
                    if let Some(confirm_type) = t.get("confirm").and_then(|v| v.as_str()) {
                        let msg = t
                            .get("msg")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Are you sure?");

                        if confirm_type == "yesorno" {
                            let ans = Confirm::new(msg).with_default(false).prompt();
                            if ans.is_err() || !ans.unwrap() {
                                continue;
                            }
                        } else if confirm_type.starts_with("match:") {
                            let required = confirm_type.strip_prefix("match:").unwrap();
                            let ans = Text::new(msg).prompt();
                            if ans.is_err() || ans.unwrap() != required {
                                println!("Confirmation failed. Press Enter to continue...");
                                let _ = Text::new("").prompt();
                                continue;
                            }
                        }
                    }

                    // advance
                    if let Some(extra_args) = prompt_required_tokens(cmd_val) {
                        return Some((menu_name.to_string(), selected_key, extra_args));
                    }
                } else {
                    // submenu
                    let mut new_path = path.clone();
                    new_path.push(selected_key.clone());

                    if let Some(result) = navigate_menu(&selected_key, t, new_path) {
                        return Some(result);
                    }
                }
            }
            // normal command
            toml::Value::String(cmd_raw) => {
                if let Some(extra_args) = prompt_required_tokens(cmd_raw) {
                    return Some((menu_name.to_string(), selected_key, extra_args));
                }
            }
            _ => continue,
        }
    }
}

fn prompt_required_tokens(cmd_raw: &str) -> Option<Vec<String>> {
    let mut extra_args = Vec::new();
    let mut i = 1;

    loop {
        let token = format!("{{{}}}", i);
        if cmd_raw.contains(&token) {
            let input = match Text::new(&format!("Enter value for {}:", token)).prompt() {
                Ok(val) => val,
                Err(_) => return None,
            };

            if input.trim().is_empty() {
                println!("[paw] The argument cannot be empty.");
                return None;
            }

            extra_args.push(input.trim().to_string());
            i += 1;
        } else {
            break;
        }
    }
    Some(extra_args)
}
