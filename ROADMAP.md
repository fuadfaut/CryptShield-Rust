# Roadmap: CRYPTSHIELD-RUST

## Phase 0: Slint UI Spike

Status: complete for the current MVP shell.

- Create separate Rust + Slint folder.
- Rebuild dashboard shell to match current Tauri layout.
- Copy CryptShield logo assets and replace Unicode icons with SVG assets.
- Add clickable sidebar navigation.
- Document Slint migration plan.
- Validate local Slint build on Fedora.

Exit criteria:

- MVP UI launches with `cargo run`.
- Dashboard visual direction is close to Tauri.
- Basic navigation works.
- Animated protection switch works.

## Phase 1: Project Foundation

Status: mostly complete for the non-mutating foundation.

- Split `app.slint` into smaller Slint files. Done.
- Split Rust `main.rs` into `app`, `state`, `commands`, `core`, and `platform`. Partially done: `app`, `state`, `commands`, and `core` exist.
- Add resolver database module. Initial allowlist done.
- Add app state struct. Done.
- Add dependency checker. Initial read-only checker done.
- Add service status read. Initial `systemctl is-active dnscrypt-proxy` read done.
- Add config read. Initial read-only TOML load done for resolver, cache, and DNSSEC settings.
- Add config update helpers. Initial TDD TOML-aware in-memory update helper done.
- Add active NetworkManager connection read. Initial TDD parser and read-only preview integration done.

Exit criteria:

- Slint app shows real service status and current resolver.
- Missing dependencies are shown clearly.
- No privileged mutation yet.

## Phase 2: Rust State And UI Binding

Status: in progress.

- Move mock UI state into Rust `AppState`. Started for tab and protection state.
- Wire Slint callbacks to Rust commands. Started for tab selection, dashboard switch, resolver selection, settings toggles, and diagnostics refresh.
- Add non-blocking background tasks for status polling. Initial read-only worker done.
- Add apply/loading state for start/stop flows. Initial preview-only dashboard Applying state done.
- Add error/toast state. Initial toast feedback done.

Exit criteria:

- Dashboard reflects real Rust state.
- UI remains responsive while commands run.
- Failed reads show a useful message.

## Phase 3: Privileged Helper Migration

- Port `--system-helper` from current Tauri binary. Started with validation-only CLI handling.
- Add Polkit policy under `data/`. Initial policy template done.
- Add resolver/action/boolean validation tests. Initial helper parser tests done.
- Add root-only helper guard. Initial TDD effective-UID guard done.
- Add structured helper action plans for `systemctl` and `nmcli`. Initial TDD dry-run plans done.
- Add structured config update plan for helper start/restart. Initial TDD dry-run plan done.
- Discover active NetworkManager connections for helper requests. Initial read-only parser done.
- Wire start/stop/restart from Slint through Rust and `pkexec`.
- Keep GUI responsive during Polkit and NetworkManager work.

Exit criteria:

- Start protection works.
- Stop protection restores DNS.
- Restart applies resolver/cache/DNSSEC changes.
- No arbitrary shell execution.

Next tasks:
- Add an executor abstraction for structured command plans with unit tests for success/failure handling.
- Add validation-only integration tests for `--system-helper` exit codes.
- Wire dashboard toggle to build a `pkexec` plan from active NetworkManager connections without executing it.
- Add config-write executor behind a dry-run flag before enabling real `/etc` writes.

## Phase 4: Settings, Logs, And Traffic

- Build real Configuration view. Initial local-state view done.
- Add resolver selector. Initial allowlist selector done.
- Add cache/DNSSEC toggles. Initial local-state toggles done.
- Add lazy `journalctl` stream for logs page. Initial read-only snapshot loading done; streaming/follow is still pending.
- Add lazy query log tail while protection is active.
- Add bounded log buffer. Initial bounded text snapshot done.
- Parse total and blocked query counters.

Exit criteria:

- Settings changes can restart protection safely.
- Idle app has no `journalctl` or `tail` child process.
- Dashboard counters update while active.
- Logs page starts/stops stream correctly.

## Phase 5: Fedora Packaging

- Add desktop file.
- Add app icons.
- Add RPM packaging metadata.
- Install Polkit policy. Policy template exists; packaging install path is still pending.
- Verify install/reinstall/remove paths.
- Document Fedora install commands.

Exit criteria:

- RPM installs cleanly on Fedora.
- App launches from GNOME app grid.
- Polkit policy is installed and used.

## Phase 6: Polish, Memory, And Parity

- Tune spacing and icon alignment against Tauri screenshots.
- Improve empty states and error messages.
- Add copy logs action.
- Add autostart support.
- Add real uptime if available from systemd.
- Replace fake latency with a real diagnostic or remove it.
- Measure idle memory and startup time.
- Evaluate slimmer Slint feature/backend configuration.

Exit criteria:

- Slint app reaches practical feature parity with the Tauri app.
- Memory is materially lower at idle.
- UI feels like CryptShield, not a generic demo.

## Phase 7: Optional GNOME Extension

- Design a narrow CLI or D-Bus interface.
- Build a GNOME Quick Settings toggle.
- Keep privileged operations in CryptShield helper, not in the extension.
- Test against target GNOME Shell versions.

Exit criteria:

- Extension can show protected/unprotected status.
- Extension can request start/stop through the safe backend path.
- Shell remains stable if CryptShield is not installed or service is missing.

## Deferred Ideas

- Resolver health checks.
- Profiles for work/home networks.
- Per-network DNS behavior.
- Native notifications.
- Flatpak packaging, only if a safe privilege story is solved.
- Shared Rust core crate used by GUI, CLI, and GNOME extension bridge.
