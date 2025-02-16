use std::{io::Write, process::Command};

pub fn copy_to_clipboard(input: &str) -> Result<(), &'static str> {
    if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd")
            .args(["/C", "clip"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|_| {
                "Failed to start clip command. Ensure 'clip' is available in your system path."
            })?;

        cmd.stdin
            .as_mut()
            .ok_or("Failed to open stdin")?
            .write_all(input.as_bytes())
            .map_err(|_| "Failed to write to clip")?;
    } else if cfg!(target_os = "linux") {
        let mut cmd = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|_| "Failed to start xclip. Ensure it is installed using 'sudo apt install xclip' or 'sudo yum install xclip'.")?;

        cmd.stdin
            .as_mut()
            .ok_or("Failed to open stdin")?
            .write_all(input.as_bytes())
            .map_err(|_| "Failed to write to xclip")?;
    } else if cfg!(target_os = "macos") {
        let mut cmd = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|_| "Failed to start pbcopy. Ensure it is available on your system.")?;

        cmd.stdin
            .as_mut()
            .ok_or("Failed to open stdin")?
            .write_all(input.as_bytes())
            .map_err(|_| "Failed to write to pbcopy")?;
    } else {
        return Err(
            "Unsupported OS. Clipboard functionality is not implemented for this platform.",
        );
    }

    Ok(())
}
