use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn execute(cmd_raw: &str, extra_args: &[String]) -> Result<(), String> {
    let (formatted_cmd, unused_args) = interpolate_tokens(cmd_raw, extra_args);
    let full_command = if unused_args.is_empty() {
        formatted_cmd
    } else {
        format!("{} {}", formatted_cmd, unused_args.join(" "))
    };
    let err = Command::new("fish") // the command is sent to fish, SOOO, its needed to get the system working
        .arg("-c")
        .arg(&full_command)
        .exec();

    Err(format!("Failed to execute command via fish: {}", err))
}

fn interpolate_tokens(cmd_raw: &str, extra_args: &[String]) -> (String, Vec<String>) {
    let mut formatted = cmd_raw.to_string();
    let mut used_indices = Vec::new();

    for (i, arg) in extra_args.iter().enumerate() {
        let token = format!("{{{}}}", i + 1);
        if formatted.contains(&token) {
            formatted = formatted.replace(&token, arg);
            used_indices.push(i);
        }
    }

    let unused_args: Vec<String> = extra_args
        .iter()
        .enumerate()
        .filter(|(i, _)| !used_indices.contains(i))
        .map(|(_, arg)| arg.clone())
        .collect();

    (formatted, unused_args)
}