use crate::sys::profile::SystemProfile;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

// helper
fn command_exists(cmd: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for dir in path.split(':') {
            let full_path = Path::new(dir).join(cmd);
            if full_path.exists() {
                return true;
            }
        }
    }
    false
}
// available support for now
fn detect_init_system() -> String {
    if command_exists("systemctl") {
        "systemd".to_string()
    } else if command_exists("rc-service") {
        "openrc".to_string()
    } else if command_exists("sv") {
        "runit".to_string()
    } else if command_exists("dinit") {
        "dinit".to_string()
    } else {
        "unknown".to_string()
    }
}
// available support for now
fn detect_pkg_manager() -> String {
    if command_exists("pacman") { "pacman".to_string() }
    else if command_exists("apt-get") { "apt".to_string() }
    else if command_exists("nix-env") { "nix".to_string() }
    else if command_exists("emerge") { "portage".to_string() }
    else if command_exists("xbps-install") { "xbps".to_string() }
    else if command_exists("dnf") { "dnf".to_string() }
    else if command_exists("zypper") { "zypper".to_string() }
    else { "fallback".to_string() }
}
// available support for now
fn detect_net_manager(os: &str) -> String {
    if os.contains("bsd") {
        return "ifconfig".to_string(); //godbsd
    }

    // for linux
    if command_exists("nmcli") {
        "networkmanager".to_string()
    } else if command_exists("iwctl") {
        "iwd".to_string()
    } else {
        "base_net".to_string() 
    }
}

// shadow-utils for Linux, pw for BSD
fn detect_user_manager(os: &str) -> String {
    if os.contains("bsd") || command_exists("pw") {
        "bsd".to_string()
    } else {
        "shadow".to_string()
    }
}


pub fn detect_os_id() -> Option<String> {
    let os = env::consts::OS;
    if os.contains("bsd") {
        return Some(os.to_string());
    }
    if let Ok(file) = File::open("/etc/os-release") {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if let Some(id) = line.strip_prefix("ID=") {
                return Some(id.trim_matches('"').to_string());
            }
        }
    }
    Some(os.to_string())
}

fn detect_de() -> Option<String> { // only DE
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let lower = desktop.to_lowercase();
        if lower.contains("kde") || lower.contains("plasma") { return Some("kde".to_string()); }
        if lower.contains("gnome") { return Some("gnome".to_string()); }
    }
    if let Ok(session) = env::var("DESKTOP_SESSION") {
        let lower = session.to_lowercase();
        if lower.contains("plasma") { return Some("kde".to_string()); }
        if lower.contains("gnome") { return Some("gnome".to_string()); }
    }
    None
}

fn detect_wm() -> Option<String> { // only WM
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() { return Some("hyprland".to_string()); }
    if env::var("NIRI_SOCKET").is_ok() { return Some("niri".to_string()); }
    if env::var("I3SOCK").is_ok() { return Some("i3wm".to_string()); } 
    if env::var("BSPWM_SOCKET").is_ok() { return Some("bspwm".to_string()); }
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let lower = desktop.to_lowercase();
        if lower.contains("i3") { return Some("i3wm".to_string()); }
        if lower.contains("bspwm") { return Some("bspwm".to_string()); }
    }
    None
}

fn detect_gpu() -> Option<String> {
    if Path::new("/sys/module/nvidia").exists() { return Some("nvidia".to_string()); }
    if Path::new("/sys/module/amdgpu").exists() { return Some("amd".to_string()); }
    if Path::new("/sys/module/i915").exists() || Path::new("/sys/module/xe").exists() { 
        return Some("intel".to_string()); 
    }
    None
}

// active audio server sockets or running processes (fallback js in case)
fn detect_audio() -> Option<String> {
    if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
        if std::path::Path::new(&format!("{}/pipewire-0", xdg_runtime)).exists() {
            return Some("pipewire".to_string());
        }
        if std::path::Path::new(&format!("{}/pulse/native", xdg_runtime)).exists() {
            return Some("pulseaudio".to_string());
        }
    }

    //fallback
    if let Ok(output) = std::process::Command::new("pgrep").arg("-x").arg("pipewire").output() {
        if output.status.success() { return Some("pipewire".to_string()); }
    }
    if let Ok(output) = std::process::Command::new("pgrep").arg("-x").arg("pulseaudio").output() {
        if output.status.success() { return Some("pulseaudio".to_string()); }
    }
    if let Ok(output) = std::process::Command::new("pgrep").arg("-x").arg("jackd").output() {
        if output.status.success() { return Some("jack".to_string()); }
    }

    None
}

// Builds the user profile based on its machine
pub fn build_system_profile(forced_distro: Option<String>) -> SystemProfile {
    let os_const = std::env::consts::OS;
    let kernel = if os_const.contains("bsd") {
        "bsd".to_string()
    } else if os_const == "linux" {
        "linux".to_string()
    } else {
        "unknown".to_string()
    };

    let os = forced_distro
        .or_else(detect_os_id)
        .unwrap_or_else(|| "unknown".to_string());

    let normalized_os: String = os
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '_')
        .collect();

    // This is a collection of defaults, if something it's not here, it must be analized thoroughly, but let's hope it reaches the point where that wouldnt be a necessity!
    let (def_pkg, def_init, def_core) = match normalized_os.as_str() {
        "arch" | "archlinux" => ("pacman", "systemd", "gnu"),
        "nixos" | "nix" => ("nix", "systemd", "gnu"),
        "ubuntu" | "debian" => ("apt", "systemd", "gnu"),
        "void" | "voidlinux" => ("xbps", "runit", "gnu"),
        "solus" | "soluslinux" => ("eopkg", "systemd", "gnu"),
        "gentoo" => ("portage", "unknown", "gnu"), // actually gentoo is a good example of something u cant simply put defaults on haha
        _ => ("unknown", "unknown", "gnu"),
    };

    let mut final_pkg = def_pkg.to_string();
    let mut final_init = def_init.to_string();

    if final_pkg == "unknown" {
        final_pkg = detect_pkg_manager();
    }
    if final_init == "unknown" {
        final_init = detect_init_system();
    }

    SystemProfile {
        os: normalized_os.clone(),
        kernel,
        pkg_manager: final_pkg,
        init_system: final_init,
        gpu: detect_gpu(),
        de: detect_de(),
        wm: detect_wm(),
        audio: detect_audio(),
        core_utils: def_core.to_string(),
        user_manager: detect_user_manager(&normalized_os),
        net_manager: detect_net_manager(&os),
    }
}