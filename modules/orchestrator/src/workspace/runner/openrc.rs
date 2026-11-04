use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn activate(name: &str, script_path: &PathBuf) -> Result<(), String> {
    let rc_content = format!(
        "#!/sbin/openrc-run\n\nname=\"paw-{}\"\ncommand=\"{}\"\ncommand_background=true\npidfile=\"/run/paw-{}.pid\"\n",
        name, script_path.display(), name
    );

    let tmp_path = format!("/tmp/paw-{}", name);
    fs::write(&tmp_path, rc_content).map_err(|e| e.to_string())?;

    let dest = format!("/etc/init.d/paw-{}", name);

    let _ = Command::new("sudo").args(["mv", &tmp_path, &dest]).status();
    let _ = Command::new("sudo").args(["chmod", "+x", &dest]).status();
    let res = Command::new("sudo").args(["rc-update", "add", &format!("paw-{}", name), "default"]).status();

    if res.map(|s| s.success()).unwrap_or(false) {
        Ok(())
    } else {
        Err("Failed to enable OpenRC service.".to_string())
    }
}

pub fn deactivate(name: &str) -> Result<(), String> {
    let _ = Command::new("sudo").args(["rc-update", "del", &format!("paw-{}", name)]).output();
    Ok(())
}
