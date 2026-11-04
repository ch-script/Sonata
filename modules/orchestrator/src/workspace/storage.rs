use std::fs;
use std::path::PathBuf;
use super::model::InitWorkspace;

pub fn get_init_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/snt/init")
}

pub fn get_workspace_path(name: &str) -> PathBuf {
    get_init_dir().join("user").join(format!("{}.toml", name))
}

pub fn create_workspace(name: &str) -> Result<(), String> {
    let path = get_workspace_path(name);
    if path.exists() {
        return Err(format!("Workspace '{}' already exists.", name));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let default_ws = InitWorkspace::default();
    let toml_str = toml::to_string(&default_ws).map_err(|e| e.to_string())?;

    fs::write(&path, toml_str).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_workspace(name: &str) -> Result<(), String> {
    let path = get_workspace_path(name);

    if !path.exists() {
        return Err(format!("Workspace '{}' does not exist.", name));
    }
    fs::remove_file(&path).map_err(|e| e.to_string())?;
    let script_path = get_init_dir().join("run").join(format!("{}.sh", name));
    if script_path.exists() {
        let _ = fs::remove_file(script_path);
    }
    println!("[snt] Init workspace '{}' deleted successfully.", name);
    Ok(())
}

pub fn add_command(name: &str, cmd: &str) -> Result<(), String> {
    let path = get_workspace_path(name);
    if !path.exists() {
        return Err(format!("Workspace '{}' does not exist.", name));
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut ws: InitWorkspace = toml::from_str(&content).map_err(|e| e.to_string())?;

    ws.commands.push(cmd.to_string());

    let toml_str = toml::to_string(&ws).map_err(|e| e.to_string())?;
    fs::write(&path, toml_str).map_err(|e| e.to_string())?;
    Ok(())
}
