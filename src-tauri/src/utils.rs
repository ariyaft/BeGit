use std::path::Path;
use std::process::Command;

/// Creates a Git process configured for use from the desktop application.
///
/// Git for Windows can otherwise create a visible console for each child
/// process. The application invokes Git frequently for background refreshes,
/// so keep those processes detached from the user's desktop on Windows.
pub fn git_command() -> Command {
    let mut command = Command::new("git");

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        // CREATE_NO_WINDOW
        command.creation_flags(0x0800_0000);
    }

    // Git output is consumed by the application, so neither a pager nor an
    // interactive terminal prompt is useful. Credential helpers can still
    // supply existing credentials without spawning a terminal.
    command
        .env("GIT_PAGER", "cat")
        .env("GIT_TERMINAL_PROMPT", "0");
    command
}

pub fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        encoded.push(TABLE[(first >> 2) as usize] as char);
        encoded.push(TABLE[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);
        encoded.push(if chunk.len() > 1 { TABLE[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char } else { '=' });
        encoded.push(if chunk.len() > 2 { TABLE[(third & 0b0011_1111) as usize] as char } else { '=' });
    }

    encoded
}

pub fn image_mime_type(file_path: &str) -> Option<&'static str> {
    let extension = Path::new(file_path)
        .extension()?
        .to_str()?
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "avif" => Some("image/avif"),
        "bmp" => Some("image/bmp"),
        "ico" => Some("image/x-icon"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

pub fn git_image_data_url(path: &str, revision: &str, file_path: &str, mime_type: &str) -> Result<Option<String>, String> {
    let output = git_command()
        .arg("show")
        .arg(format!("{}:{}", revision, file_path))
        .current_dir(path)
        .output()
        .map_err(|e| e.to_string())?;

    // A missing blob is expected for newly added or deleted images.
    if !output.status.success() {
        return Ok(None);
    }

    Ok(Some(format!("data:{};base64,{}", mime_type, base64_encode(&output.stdout))))
}

pub fn git_config_value(path: Option<&str>, scope: &str, key: &str) -> Result<Option<String>, String> {
    let mut command = git_command();
    command.arg("config").arg(scope).arg("--get").arg(key);
    if let Some(path) = path {
        command.current_dir(path);
    }
    let output = command.output().map_err(|e| e.to_string())?;
    if output.status.success() {
        return Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_string()));
    }
    if output.status.code() == Some(1) {
        return Ok(None);
    }
    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
}
