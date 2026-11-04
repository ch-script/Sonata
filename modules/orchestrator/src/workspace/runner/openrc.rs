use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn activate(name: &str, script_path: &PathBuf) -> Result<(), String> {
    let rc_content = format!(
        "#!/sbin/openrc-run\n\nname=\"snt-{}\"\ncommand=\"{}\"\ncommand_background=true\npidfile=\"/run/snt-{}.pid\"\n",
        name, script_path.display(), name
    );

    let tmp_path = format!("/tmp/snt-{}", name);
    fs::write(&tmp_path, rc_content).map_err(|e| e.to_string())?;

    let dest = format!("/etc/init.d/snt-{}", name);

    let _ = Command::new("sudo").args(["mv", &tmp_path, &dest]).status();
    let _ = Command::new("sudo").args(["chmod", "+x", &dest]).status();
    let res = Command::new("sudo").args(["rc-update", "add", &format!("snt-{}", name), "default"]).status();

    if res.map(|s| s.success()).unwrap_or(false) {
        Ok(())
    } else {
        Err("Failed to enable OpenRC service.".to_string())
    }
}

pub fn deactivate(name: &str) -> Result<(), String> {
    let _ = Command::new("sudo").args(["rc-update", "del", &format!("snt-{}", name)]).output();
    Ok(())
}
