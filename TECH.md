# Technical Design: CRYPTSHIELD-RUST

## Platform

- OS target: Fedora Linux
- Desktop target: GNOME first, Linux desktop generally possible
- GUI toolkit: Slint
- Language: Rust
- Package target: RPM

## Core Dependencies

```toml
slint = "1.16"
slint-build = "1.16"
toml = "0.9"
toml_edit = "0.25"
```

The current CRYPTSHIELD-RUST prototype uses `toml` for read-only config parsing and `toml_edit` for in-memory config update plans. Backend async/process dependencies such as `tokio` should be added only when real command execution, streaming logs, or long-running workers need them. `zbus` is optional and should only be added if the app exposes a D-Bus API for tray/GNOME extension integration.

## Proposed Project Layout

```text
CRYPTSHIELD-RUST/
├── Cargo.toml
├── build.rs
├── data/
│   ├── com.fuadfaut.cryptshield.desktop
│   ├── com.fuadfaut.cryptshield.policy
│   └── icons/
├── ui/
│   ├── app.slint
│   ├── dashboard.slint
│   ├── configuration.slint
│   ├── logs.slint
│   ├── components.slint
│   └── setup.slint
├── assets/
│   ├── logo.svg
│   └── icons/
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── state.rs
│   ├── commands.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── config_manager.rs
│   │   ├── command_plan.rs
│   │   ├── dependencies.rs
│   │   ├── resolvers.rs
│   │   ├── service.rs
│   │   ├── system_actions.rs
│   │   └── system_helper.rs
└── packaging/
    └── rpm/
```

## Runtime Architecture

```text
Slint UI
   |
   v
Rust AppState + callbacks
   |
   +--> unprivileged reads
   |    - systemctl is-active
   |    - read dnscrypt TOML
   |    - dependency checks
   |
   +--> background workers
   |    - read-only status polling
   |    - lazy journal snapshot loading
   |    - read active NetworkManager connection UUIDs
   |
   +--> privileged mutations
        - pkexec cryptshield --system-helper ...
        - helper validates action and arguments
        - helper builds structured system/config plans
        - future helper runs systemctl, nmcli, and config writes
```

## UI Binding Model

Slint should own the visual state through properties, while Rust owns real app state and side effects.

Initial generated UI properties:

- `protected`
- `active_tab`
- `current_resolver_name`
- `uptime_text`
- `latency_text`
- `stats_total`
- `stats_blocked`
- `is_applying`
- `toast_message`
- `service_status`
- `config_status`
- `logs_status`
- `log_text`

Callbacks from Slint into Rust:

- `toggle_protection()`
- `select_tab(index)`
- `select_resolver(id)`
- `set_cache_enabled(value)`
- `set_dnssec_enabled(value)`
- `clear_logs()`
- `refresh_logs()`
- `refresh_status()`
- `copy_logs()`

Rust should update Slint properties on the UI thread using Slint's weak handles and event-loop helpers. Long-running work must not block callbacks.

## State Model

Initial Rust state fields:

- `is_running`
- `is_starting`
- `current_resolver`
- `caching_enabled`
- `dnssec_enabled`
- `autostart_enabled`
- `uptime`
- `stats_total`
- `stats_blocked`
- `logs`
- `dependencies`
- `active_tab`
- `config_status`
- `is_applying`
- `toast_message`
- `logs_status`
- `log_text`

State should be held in Rust, not only in `.slint`, once backend wiring starts. The MVP may keep mock visual state in `.slint`.

## Privilege Model

The GUI must not run as root.

Privileged mutation path:

```bash
pkexec /usr/bin/cryptshield --system-helper start cloudflare true true <uuid...>
pkexec /usr/bin/cryptshield --system-helper stop <uuid...>
pkexec /usr/bin/cryptshield --system-helper restart quad9 true true <uuid...>
```

Helper requirements:

- Check effective UID is root.
- Validate action: `start`, `stop`, `restart`.
- Validate resolver against a fixed allowlist.
- Validate booleans exactly as `true` or `false`.
- Treat NetworkManager identifiers as args, not shell fragments.
- Never run `bash -c` for user-influenced actions.
- Build structured command/config plans before execution.
- Current implementation validates and builds dry-run plans, but does not execute privileged mutations yet.
- Current implementation refuses non-root `--system-helper` callers before validation/planning.

## Backend Migration Notes

Reuse from current Tauri backend:

- Resolver allowlist and exact resolver IDs.
- `get_active_connections` using NetworkManager UUIDs.
- `systemctl enable --now`, `disable --now`, `restart`.
- `nmcli --wait 15 connection up`.
- Polkit policy with `auth_admin_keep`.
- Lazy log streaming approach.

Rewrite or improve:

- Replace shell/TOML mutation with TOML-aware writing where feasible.
- Add helper unit tests for argument validation.
- Add integration test scripts for stop restoring DNS.
- Move UI-specific Tauri commands into Rust app callbacks.

## Log Streaming

Logs should remain lazy:

- `journalctl -u dnscrypt-proxy -f --no-pager -n 50` only while logs view is visible.
- `tail -n 0 -F /var/log/dnscrypt-query.log` only while protection is active.
- Both child processes must be killed when no longer needed.
- Slint UI should receive bounded log arrays or text snapshots, not unbounded strings.

Current implementation loads a bounded read-only `journalctl -u dnscrypt-proxy --no-pager -n 80` snapshot on demand. Follow-mode streaming and query-log tailing are still pending.

## UI Design

Use Slint for custom parity with Tauri:

- Keep CryptShield's dark Catppuccin-style palette.
- Use SVG assets for icons, not font-dependent Unicode symbols.
- Build a custom animated protection switch.
- Keep dashboard layout close to Tauri: full-height sidebar, main header, status card, live traffic card, compact metrics.
- Prefer fixed proportions with responsive content width over arbitrary stretching.

Avoid:

- Running business logic in `.slint`.
- Embedding large state machines in UI markup.
- Depending on glyphs that differ by system font.

## Packaging

RPM should install:

- `/usr/bin/cryptshield`
- desktop file
- app icon
- supporting SVG assets if needed at runtime
- Polkit policy

Runtime dependencies:

- `dnscrypt-proxy`
- `NetworkManager`
- `polkit`
- Slint runtime dependencies pulled by the binary/backend stack

Development dependencies:

```bash
sudo dnf install -y rust cargo gcc pkgconf-pkg-config
```

Depending on Slint backend feature selection, additional X11/Wayland/OpenGL development libraries may be needed on a clean Fedora builder.

## Slint Feature Strategy

Default Slint is easiest for MVP and pulls renderer/windowing features automatically. After UI approval, evaluate a slimmer configuration:

- Keep only required desktop backend.
- Avoid unused renderers if binary size or compile time matters.
- Keep SVG/image support because the UI uses copied CryptShield assets.

Do this after the visual direction stabilizes, not during the first UI spike.

## Testing Strategy

- Unit tests for resolver validation and helper argument parsing.
- Unit tests for config read/write helpers.
- Dry-run command builder tests where possible.
- Slint compile check in CI via `cargo check`.
- Manual Fedora tests for Polkit and NetworkManager.
- Memory comparison against current Tauri release and GTK MVP.

Current TDD coverage includes resolver validation, helper argument parsing, structured `pkexec` plans, dry-run `systemctl`/`nmcli` plans, read-only TOML parsing, and in-memory TOML update plans.
It also covers parsing active NetworkManager connection UUIDs from `nmcli` output.
It also covers root-only helper gating via effective UID parsing.

## Open Questions

- Should Slint own tray UI, or should tray be a separate Rust platform module?
- Should a GNOME extension talk to CryptShield through CLI or D-Bus?
- Should resolver metadata live in Rust code, TOML, or JSON?
- Should config mutation preserve comments exactly or rewrite canonical TOML?
- Should GTK remain the fallback path if Slint desktop integration is not sufficient?


## Original README alignment

- Resolver support mirrors `fuadfaut/CryptShield`: default load-balanced mode plus Cloudflare, Google, Quad9, AdGuard, NextDNS, Cisco, Mullvad, CleanBrowsing, and Tiarap DNS.
- UI resolver IDs are mapped to exact `dnscrypt-proxy` `server_names` values before config updates.
- Default mode removes the explicit `server_names` entry so `dnscrypt-proxy` can use its own resolver selection.
- NetworkManager plans follow the original helper shape: point active connections to `127.0.0.1`, disable IPv4/IPv6 auto DNS while protected, and restore both on stop.
- Active connection discovery reads `nmcli -t -f UUID,DEVICE connection show --active` and ignores `lo`.
