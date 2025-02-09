use std::{io::Write, process::Command};

pub fn copy_to_clipboard(input: &str) -> Result<(), &str> {
    let Ok(mut cmd) = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(std::process::Stdio::piped())
        .spawn()
    else {
        return Err("Failed to start xclip");
    };

    cmd.stdin
        .as_mut()
        .expect("Failed to open stdin")
        .write_all(input.as_bytes())
        .expect("Failed to write to xclip");

    Ok(())
}
