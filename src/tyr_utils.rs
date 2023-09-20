use anyhow::Error;
use serde_json::{json, Value};

pub fn process_command(command: &[&str], error_msg: &str) -> Result<(Value), Error> {
    debug!("Running command: {:?}", command);

    let mut retval: Value = json!({
        "status": "success",
        "result": {},
    });

    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .arg("/C")
            .args(command)
            .output()
            .expect("failed to execute process")
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(command.join(" "))
            .output()
            .expect("failed to execute process")
    };

    retval["result"] = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;

    if !output.status.success() {
        error!("Command Failed: {}", error_msg);
        error!(
            "Command output: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        error!("Command error: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    } else {
        debug!(
            "Command output: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        debug!("Command error: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok((retval))
}
