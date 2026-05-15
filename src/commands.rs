use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use crate::core::command_plan::pkexec_helper_plan;
use crate::core::config_manager::DEFAULT_DNSCRYPT_CONFIG_PATH;
use crate::core::dependencies::DependencyStatus;
use crate::core::network_manager;
use crate::core::service;
use crate::core::system_actions::{build_config_update_plan, build_system_action_plan};
use crate::core::system_helper::{
    current_effective_uid, helper_execution_allowed, parse_helper_request,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadOnlyStatus {
    pub service_status: String,
    pub dependencies: DependencyStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent {
    ReadOnlyStatus(ReadOnlyStatus),
    ProtectionPreviewFinished {
        desired_protected: bool,
        message: String,
    },
    LogSnapshot {
        status: String,
        text: String,
    },
}

pub fn collect_read_only_status() -> ReadOnlyStatus {
    ReadOnlyStatus {
        service_status: service::dnscrypt_proxy_status(),
        dependencies: DependencyStatus::detect(),
    }
}

pub fn validate_system_helper_invocation(args: &[String]) -> i32 {
    let effective_uid = current_effective_uid().unwrap_or(u32::MAX);
    if let Err(error) = helper_execution_allowed(effective_uid) {
        eprintln!("refusing system-helper request: {error:?}");
        return 77;
    }

    match parse_helper_request(args) {
        Ok(request) => {
            let pkexec_plan = pkexec_helper_plan(&request);
            let system_plan = build_system_action_plan(&request);
            let existing_config =
                std::fs::read_to_string(DEFAULT_DNSCRYPT_CONFIG_PATH).unwrap_or_default();
            let config_update_count = build_config_update_plan(&existing_config, &request)
                .ok()
                .flatten()
                .map_or(0, |_| 1);
            eprintln!(
                "validated system-helper request for {}; prepared {} structured system command(s) and {} config update(s); privileged execution is not implemented yet",
                pkexec_plan.args.join(" "),
                system_plan.commands.len(),
                config_update_count
            );
            2
        }
        Err(error) => {
            eprintln!("invalid system-helper request: {error:?}");
            64
        }
    }
}

pub fn request_read_only_status(sender: Sender<UiEvent>) {
    thread::spawn(move || {
        let _ = sender.send(UiEvent::ReadOnlyStatus(collect_read_only_status()));
    });
}

pub fn spawn_status_worker(sender: Sender<UiEvent>) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(10));

        if sender
            .send(UiEvent::ReadOnlyStatus(collect_read_only_status()))
            .is_err()
        {
            break;
        }
    });
}

pub fn request_protection_preview(sender: Sender<UiEvent>, desired_protected: bool) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(650));

        let action = if desired_protected { "start" } else { "stop" };
        let connections = network_manager::active_connection_uuids().unwrap_or_default();
        let message = if connections.is_empty() {
            format!("Preview only: {action} needs an active NetworkManager connection.")
        } else {
            format!(
                "Preview only: {action} will use Polkit for {} active connection(s).",
                connections.len()
            )
        };
        let _ = sender.send(UiEvent::ProtectionPreviewFinished {
            desired_protected,
            message,
        });
    });
}

pub fn request_log_snapshot(sender: Sender<UiEvent>) {
    thread::spawn(move || {
        let (status, text) = collect_log_snapshot();
        let _ = sender.send(UiEvent::LogSnapshot { status, text });
    });
}

fn collect_log_snapshot() -> (String, String) {
    let output = Command::new("journalctl")
        .args(["-u", "dnscrypt-proxy", "--no-pager", "-n", "80"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            if text.is_empty() {
                (
                    "No dnscrypt-proxy journal entries found.".to_owned(),
                    String::new(),
                )
            } else {
                (
                    "Loaded latest dnscrypt-proxy journal snapshot.".to_owned(),
                    text,
                )
            }
        }
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let detail = if text.is_empty() {
                "journalctl returned no details".to_owned()
            } else {
                text
            };
            (format!("Unable to read logs: {detail}"), String::new())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            ("journalctl is not available.".to_owned(), String::new())
        }
        Err(_) => ("Unable to read journal logs.".to_owned(), String::new()),
    }
}
