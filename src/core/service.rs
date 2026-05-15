use std::process::Command;

pub fn dnscrypt_proxy_status() -> String {
    let output = Command::new("systemctl")
        .args(["is-active", "dnscrypt-proxy"])
        .output();

    match output {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            if text.is_empty() {
                "unknown".to_owned()
            } else {
                text
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            "systemctl missing".to_owned()
        }
        Err(_) => "unknown".to_owned(),
    }
}
