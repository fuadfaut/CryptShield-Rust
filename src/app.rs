slint::include_modules!();

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use crate::commands::{self, UiEvent};
use crate::state::AppState;

pub fn run() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let state = Rc::new(RefCell::new(AppState::load_initial()));
    let (event_sender, event_receiver) = mpsc::channel::<UiEvent>();

    apply_state(&ui, &state.borrow());
    commands::spawn_status_worker(event_sender.clone());
    let _event_timer = start_event_timer(&ui, Rc::clone(&state), event_receiver);

    let ui_weak = ui.as_weak();
    let toggle_state = Rc::clone(&state);
    let toggle_sender = event_sender.clone();
    ui.on_toggle_protection(move || {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };

        let mut state = toggle_state.borrow_mut();
        if let Some(desired_protected) = state.begin_protection_preview() {
            commands::request_protection_preview(toggle_sender.clone(), desired_protected);
        }
        apply_state(&ui, &state);
    });

    let ui_weak = ui.as_weak();
    let tab_state = Rc::clone(&state);
    let tab_sender = event_sender.clone();
    ui.on_select_tab(move |tab| {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };

        let mut state = tab_state.borrow_mut();
        state.select_tab(tab);
        if tab == 2 && state.log_text.is_empty() {
            state.begin_log_refresh();
            commands::request_log_snapshot(tab_sender.clone());
        }
        apply_state(&ui, &state);
    });

    let ui_weak = ui.as_weak();
    let resolver_state = Rc::clone(&state);
    ui.on_select_resolver(move |resolver_id| {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };

        let mut state = resolver_state.borrow_mut();
        state.select_resolver(resolver_id.as_str());
        apply_state(&ui, &state);
    });

    let ui_weak = ui.as_weak();
    let cache_state = Rc::clone(&state);
    ui.on_set_cache_enabled(move |enabled| {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };

        let mut state = cache_state.borrow_mut();
        state.set_cache_enabled(enabled);
        apply_state(&ui, &state);
    });

    let ui_weak = ui.as_weak();
    let dnssec_state = Rc::clone(&state);
    ui.on_set_dnssec_enabled(move |enabled| {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };

        let mut state = dnssec_state.borrow_mut();
        state.set_dnssec_enabled(enabled);
        apply_state(&ui, &state);
    });

    let ui_weak = ui.as_weak();
    let autostart_state = Rc::clone(&state);
    ui.on_set_autostart_enabled(move |enabled| {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };

        let mut state = autostart_state.borrow_mut();
        state.set_autostart_enabled(enabled);
        apply_state(&ui, &state);
    });

    let ui_weak = ui.as_weak();
    let refresh_state = Rc::clone(&state);
    let refresh_sender = event_sender.clone();
    ui.on_refresh_status(move || {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };

        let mut state = refresh_state.borrow_mut();
        state.toast_message = "Refreshing diagnostics...".to_owned();
        apply_state(&ui, &state);
        commands::request_read_only_status(refresh_sender.clone());
    });

    let ui_weak = ui.as_weak();
    let logs_state = Rc::clone(&state);
    let logs_sender = event_sender.clone();
    ui.on_refresh_logs(move || {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };

        let mut state = logs_state.borrow_mut();
        state.begin_log_refresh();
        apply_state(&ui, &state);
        commands::request_log_snapshot(logs_sender.clone());
    });

    let ui_weak = ui.as_weak();
    let clear_logs_state = Rc::clone(&state);
    ui.on_clear_logs(move || {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };

        let mut state = clear_logs_state.borrow_mut();
        state.clear_logs();
        apply_state(&ui, &state);
    });

    ui.run()
}

fn start_event_timer(
    ui: &AppWindow,
    state: Rc<RefCell<AppState>>,
    receiver: mpsc::Receiver<UiEvent>,
) -> slint::Timer {
    let timer = slint::Timer::default();
    let ui_weak = ui.as_weak();

    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(250),
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            let mut changed = false;
            {
                let mut state = state.borrow_mut();
                for event in receiver.try_iter() {
                    match event {
                        UiEvent::ReadOnlyStatus(status) => {
                            state.apply_read_only_status(status);
                            if state.toast_message == "Refreshing diagnostics..." {
                                state.toast_message = "Diagnostics refreshed.".to_owned();
                            }
                        }
                        UiEvent::ProtectionPreviewFinished {
                            desired_protected,
                            message,
                        } => state.finish_protection_preview(desired_protected, message),
                        UiEvent::LogSnapshot { status, text } => {
                            state.apply_log_snapshot(status, text);
                        }
                    }
                    changed = true;
                }
            }

            if changed {
                apply_state(&ui, &state.borrow());
            }
        },
    );

    timer
}

fn apply_state(ui: &AppWindow, state: &AppState) {
    ui.set_protected(state.is_protected);
    ui.set_active_tab(state.active_tab);
    ui.set_service_status(state.service_status.clone().into());
    ui.set_dnscrypt_proxy_status(state.dependencies.dnscrypt_proxy.clone().into());
    ui.set_nmcli_status(state.dependencies.nmcli.clone().into());
    ui.set_systemctl_status(state.dependencies.systemctl.clone().into());
    ui.set_pkexec_status(state.dependencies.pkexec.clone().into());
    ui.set_polkit_policy_status(state.dependencies.polkit_policy.clone().into());
    ui.set_current_resolver_id(state.current_resolver_id.clone().into());
    ui.set_current_resolver_name(state.current_resolver_name().into());
    ui.set_config_status(state.config_status.clone().into());
    ui.set_cache_enabled(state.cache_enabled);
    ui.set_dnssec_enabled(state.dnssec_enabled);
    ui.set_autostart_enabled(state.autostart_enabled);
    ui.set_is_applying(state.is_applying);
    ui.set_toast_message(state.toast_message.clone().into());
    ui.set_logs_status(state.logs_status.clone().into());
    ui.set_log_text(state.log_text.clone().into());
}
