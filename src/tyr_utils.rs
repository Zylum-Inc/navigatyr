use anyhow::Error;

pub fn process_command(command: &[&str], error_msg: &str) -> Result<(), Error> {

    println!("Running command: {:?}", command);

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

    if !output.status.success() {
        println!("{}", error_msg);
        std::process::exit(1);
    } else {
        println!("Command output: {}", String::from_utf8_lossy(&output.stdout));
    }

    Ok(())
}
