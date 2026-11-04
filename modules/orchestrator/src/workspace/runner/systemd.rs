use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn activate(name: &str, script_path: &PathBuf) -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let systemd_dir = PathBuf::from(home).join(".config/systemd/user");
    fs::create_dir_all(&systemd_dir).map_err(|e| e.to_string())?;

    let service_path = systemd_dir.join(format!("paw-{}.service", name));
    let service_content = format!(
        "[Unit]\nDescription=Paw Workspace: {}\n\n[Service]\nType=simple\nExecStart={}\n\n[Install]\nWantedBy=default.target\n",
        name, script_path.display()
    );

    fs::write(&service_path, service_content).map_err(|e| e.to_string())?;

    let _ = Command::new("systemctl").args(["--user", "daemon-reload"]).output();
    let res = Command::new("systemctl").args(["--user", "enable", "--now", &format!("paw-{}.service", name)]).status();

    if res.map(|s| s.success()).unwrap_or(false) {
        Ok(())
    } else {
        Err("Failed to enable systemd service.".to_string())
    }
}

pub fn deactivate(name: &str) -> Result<(), String> {
    let svc_name = format!("paw-{}.service", name);
    let _ = Command::new("systemctl").args(["--user", "disable", "--now", &svc_name]).output();
    Ok(())
}
