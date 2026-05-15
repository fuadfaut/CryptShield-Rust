use std::process::{Command, Stdio};

pub const POLKIT_POLICY_PATH: &str = "/usr/share/polkit-1/actions/com.fuadfaut.cryptshield.policy";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyStatus {
    pub dnscrypt_proxy: String,
    pub nmcli: String,
    pub systemctl: String,
    pub pkexec: String,
    pub polkit_policy: String,
}

impl DependencyStatus {
    pub fn detect() -> Self {
        Self {
            dnscrypt_proxy: availability("dnscrypt-proxy", &["-version"]),
            nmcli: availability("nmcli", &["--version"]),
            systemctl: availability("systemctl", &["--version"]),
            pkexec: availability("pkexec", &["--version"]),
            polkit_policy: policy_status(),
        }
    }
}

fn availability(binary: &str, args: &[&str]) -> String {
    let spawned = Command::new(binary)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match spawned {
        Ok(_) => "found".to_owned(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "missing".to_owned(),
        Err(_) => "unavailable".to_owned(),
    }
}

fn policy_status() -> String {
    if std::path::Path::new(POLKIT_POLICY_PATH).exists() {
        "installed".to_owned()
    } else {
        "not installed".to_owned()
    }
}
