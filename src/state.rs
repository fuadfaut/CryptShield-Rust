use crate::commands::ReadOnlyStatus;
use crate::core::config_manager;
use crate::core::dependencies::DependencyStatus;
use crate::core::resolvers::{self, DEFAULT_RESOLVER_ID};
use crate::core::service;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub is_protected: bool,
    pub active_tab: i32,
    pub service_status: String,
    pub dependencies: DependencyStatus,
    pub current_resolver_id: String,
    pub config_status: String,
    pub cache_enabled: bool,
    pub dnssec_enabled: bool,
    pub autostart_enabled: bool,
    pub is_applying: bool,
    pub toast_message: String,
    pub logs_status: String,
    pub log_text: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_protected: false,
            active_tab: DASHBOARD_TAB,
            service_status: "unknown".to_owned(),
            dependencies: DependencyStatus {
                dnscrypt_proxy: "unknown".to_owned(),
                nmcli: "unknown".to_owned(),
                systemctl: "unknown".to_owned(),
                pkexec: "unknown".to_owned(),
                polkit_policy: "unknown".to_owned(),
            },
            current_resolver_id: DEFAULT_RESOLVER_ID.to_owned(),
            config_status: "unknown".to_owned(),
            cache_enabled: true,
            dnssec_enabled: true,
            autostart_enabled: false,
            is_applying: false,
            toast_message: String::new(),
            logs_status: "Logs are loaded only when requested.".to_owned(),
            log_text: String::new(),
        }
    }
}

const DASHBOARD_TAB: i32 = 0;
const SETUP_TAB: i32 = 3;

impl AppState {
    pub fn load_initial() -> Self {
        let config_result = config_manager::read_default_config();
        let service_status = service::dnscrypt_proxy_status();
        let mut state = Self {
            is_protected: service_status == "active",
            service_status,
            dependencies: DependencyStatus::detect(),
            config_status: config_result.status,
            ..Self::default()
        };

        if let Some(config) = config_result.config {
            if let Some(resolver_id) = config.resolver_id {
                state.select_resolver(&resolver_id);
            }

            if let Some(cache_enabled) = config.cache_enabled {
                state.cache_enabled = cache_enabled;
            }

            if let Some(dnssec_required) = config.dnssec_required {
                state.dnssec_enabled = dnssec_required;
            }
        }

        state
    }

    pub fn current_resolver_name(&self) -> &'static str {
        resolvers::display_name(&self.current_resolver_id)
    }

    pub fn select_resolver(&mut self, resolver_id: &str) {
        if resolvers::is_supported(resolver_id) {
            self.current_resolver_id = resolver_id.to_owned();
        }
    }

    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
    }

    pub fn set_dnssec_enabled(&mut self, enabled: bool) {
        self.dnssec_enabled = enabled;
    }

    pub fn set_autostart_enabled(&mut self, enabled: bool) {
        self.autostart_enabled = enabled;
    }

    pub fn apply_read_only_status(&mut self, status: ReadOnlyStatus) {
        self.service_status = status.service_status;
        self.dependencies = status.dependencies;

        if !self.is_applying && self.service_status != "preview" {
            self.is_protected = self.service_status == "active";
        }
    }

    pub fn begin_protection_preview(&mut self) -> Option<bool> {
        if self.is_applying {
            self.toast_message = "Protection change already in progress.".to_owned();
            return None;
        }

        self.is_applying = true;
        self.toast_message.clear();
        Some(!self.is_protected)
    }

    pub fn finish_protection_preview(&mut self, desired_protected: bool, message: String) {
        self.is_applying = false;
        self.is_protected = desired_protected;
        self.service_status = "preview".to_owned();
        self.toast_message = message;
    }

    pub fn select_tab(&mut self, tab: i32) {
        if (DASHBOARD_TAB..=SETUP_TAB).contains(&tab) {
            self.active_tab = tab;
        }
    }

    pub fn begin_log_refresh(&mut self) {
        self.logs_status = "Loading latest dnscrypt-proxy journal snapshot...".to_owned();
    }

    pub fn apply_log_snapshot(&mut self, status: String, text: String) {
        self.logs_status = status;
        self.log_text = trim_log_text(text);
    }

    pub fn clear_logs(&mut self) {
        self.logs_status = "Log buffer cleared.".to_owned();
        self.log_text.clear();
    }
}

fn trim_log_text(text: String) -> String {
    const MAX_CHARS: usize = 12_000;

    let char_count = text.chars().count();
    if char_count <= MAX_CHARS {
        return text;
    }

    text.chars().skip(char_count - MAX_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::AppState;

    #[test]
    fn previews_protection_state() {
        let mut state = AppState::default();

        let desired = state.begin_protection_preview();
        assert_eq!(desired, Some(true));
        assert!(state.is_applying);

        state.finish_protection_preview(true, "done".to_owned());
        assert!(state.is_protected);
        assert!(!state.is_applying);
        assert_eq!(state.toast_message, "done");
    }

    #[test]
    fn ignores_unknown_tabs() {
        let mut state = AppState::default();

        state.select_tab(2);
        state.select_tab(9);

        assert_eq!(state.active_tab, 2);
    }

    #[test]
    fn ignores_unknown_resolvers() {
        let mut state = AppState::default();

        state.select_resolver("quad9");
        state.select_resolver("not-supported");

        assert_eq!(state.current_resolver_id, "quad9");
        assert_eq!(state.current_resolver_name(), "Quad9");
    }
}
