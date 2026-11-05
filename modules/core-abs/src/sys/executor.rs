use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub enum ExecutionTarget {
    Local,
    Remote { ip: String, port: u16 },
}

pub fn execute(
    cmd_raw: &str,
    extra_args: &[String],
    target: ExecutionTarget,
) -> Result<(), String> {
    let (formatted_cmd, unused_args) = interpolate_tokens(cmd_raw, extra_args);
    let full_command = if unused_args.is_empty() {
        formatted_cmd
    } else {
        format!("{} {}", formatted_cmd, unused_args.join(" "))
    };

    match target {
        ExecutionTarget::Local => execute_local(&full_command),
        ExecutionTarget::Remote { ip, port } => execute_remote(&full_command, &ip, port),
    }
}

fn execute_local(full_command: &str) -> Result<(), String> {
    let err = Command::new("sh")
        .arg("-c")
        .arg(full_command)
        .exec();

    Err(format!("Failed to execute local command via sh: {}", err))
}

fn execute_remote(full_command: &str, ip: &str, port: u16) -> Result<(), String> {
    let address = format!("{}:{}", ip, port);
    println!("[snt] Sending command to remote target: {}", address);

    let mut stream = TcpStream::connect(&address)
        .map_err(|e| format!("Failed to connect to remote daemon at {}: {}", address, e))?;

    // Send the command payload
    let payload = format!("{}\n", full_command);
    stream
        .write_all(payload.as_bytes())
        .map_err(|e| format!("Failed to send command to remote: {}", e))?;

    // Read the server's response back to the local terminal
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| format!("Failed to read response from remote: {}", e))?;

    println!("{}", response.trim());
    Ok(())
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
