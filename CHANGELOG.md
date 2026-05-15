# Changelog

All notable changes to CRYPTSHIELD-RUST will be documented in this file.

This changelog is for the CRYPTSHIELD-RUST repository.

## [Unreleased]

### Added
- Added a CRYPTSHIELD-RUST project scaffold.
- Added a custom CryptShield dashboard mock with sidebar, header, status card, metrics, and live traffic panel.
- Added a custom animated switch-only protection control.
- Split the Slint UI into app shell, shared components, dashboard, and setup diagnostics views.
- Added Rust-owned `AppState` for active tab and mock protection state.
- Wired Slint tab and protection callbacks through Rust instead of mutating key state only in UI markup.
- Added read-only startup diagnostics for `dnscrypt-proxy`, `nmcli`, `systemctl`, `pkexec`, and `dnscrypt-proxy` service status.
- Added a supported resolver allowlist with validation tests.
- Added read-only dnscrypt-proxy TOML config loading for resolver, cache, and DNSSEC settings.
- Added a functional local Configuration view with resolver selection, DNS cache, DNSSEC, and autostart toggles.
- Added a read-only diagnostics refresh action in the Setup view.
- Added a `commands` layer for non-blocking read-only UI events and protection-action preview events.
- Added background service/dependency polling without blocking the Slint UI thread.
- Added dashboard Applying state and toast feedback for preview-only protection toggles.
- Added a lazy Logs view that loads a bounded read-only `journalctl` snapshot when requested.
- Added initial `--system-helper` argument validation for start, stop, and restart requests.
- Added structured `pkexec` command-plan construction tests without executing privileged commands.
- Added test-driven dry-run system action plans for `systemctl` and `nmcli` start, stop, and restart flows.
- Added test-driven TOML-aware dnscrypt config update helpers for resolver, cache, and DNSSEC settings.
- Added dry-run config update plans for start/restart helper requests without writing system files.
- Added test-driven parsing for active NetworkManager connection UUIDs from `nmcli` output.
- Added active-connection count feedback to preview-only protection toggles.
- Added test-driven root-only guard for `--system-helper` using effective UID from `/proc/self/status`.
- Added a Polkit policy template under `data/`.
- Added read-only Setup diagnostics for the installed CryptShield Polkit policy.
- Added focused unit tests for tab selection and protection state transitions.
- Added copied CryptShield SVG logo assets from the native GTK MVP.
- Reworked the Slint layout to mirror the current Tauri dashboard structure and responsive content sizing.
- Added clickable sidebar navigation with active tab state and placeholder views.
- Replaced text-symbol UI icons with SVG assets for navigation, header, live traffic, and metric cards.
- Replaced the switch knob power/check symbols with SVG icons and tightened icon/text alignment.
- Added PRD, technical design, roadmap, and migration documents for the Rust + Slint path.

### Notes
- MVP UI still does not change DNS, call Polkit, or edit system files.
- Configuration changes are local state only until the privileged helper migration is implemented.
- Protection toggles are preview-only until the restricted helper and Polkit path are migrated.
- `--system-helper` currently validates requests and exits without mutating the system.
- Helper system action plans are built as structured commands but are not executed yet.
- Config updates are built with `toml_edit` and validated in memory; they are not written to disk yet.
- Active NetworkManager connections are read for preview messaging only.
- `--system-helper` now refuses non-root callers before validation/planning.
