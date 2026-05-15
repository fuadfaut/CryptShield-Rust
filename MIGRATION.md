# Migration Plan: Tauri/React to Rust + Slint

## Goal

Build a Fedora-focused CryptShield that uses Rust + Slint instead of Tauri/WebKit. The main motivation is lower idle memory, faster startup, and a UI that can stay visually close to the current Tauri app without shipping a browser engine.

## Recommended Stack

- GUI: Rust + Slint
- UI markup: `.slint` files compiled with `slint-build`
- Async/process work: `tokio`
- System control: reuse the current Rust helper model
- Privilege escalation: keep restricted `cryptshield --system-helper` + Polkit policy
- Packaging: RPM first, AppImage optional later
- Optional integration: CLI or D-Bus for tray/GNOME extension surfaces

## Why This Can Be Lighter

The current app pays the WebKitGTK cost:

- `WebKitWebProcess`
- `WebKitNetworkProcess`
- JavaScript runtime
- React runtime
- Vite/Tauri frontend bundle

A Slint app removes the browser engine and JavaScript runtime. The final app will still use a GUI renderer, SVG assets, DNSCrypt logs, and child processes when needed, but idle memory should be materially lower than the Tauri/WebKit version.

Slint is not a zero-cost CLI; it still brings windowing, rendering, font/text, image/SVG, and accessibility pieces. The tradeoff is much closer to a native Rust GUI than a WebView application.

## Why Slint Instead Of GTK

Slint is a good fit if the priority is:

- Rust-first app architecture.
- Custom visual parity with the current Tauri UI.
- Lower memory than WebView.
- Declarative UI without React.
- Portable Linux desktop UI that is not tied to GNOME conventions.

GTK/libadwaita remains better if the priority is:

- Maximum Fedora/GNOME-native behavior.
- Libadwaita preferences/navigation patterns.
- Mature desktop integration out of the box.

The current direction favors Slint because CryptShield already has a recognizable custom interface.

## What Can Be Reused

From current `src-tauri/src`:

- `system_ctl.rs`: service control, resolver validation, helper mode, NetworkManager control
- `config_manager.rs`: TOML read logic
- `log_streamer.rs`: log and traffic streaming concept
- Polkit policy under `src-tauri/polkit`
- RPM dependency list and packaging knowledge

From current frontend:

- Resolver database
- App state shape
- Dashboard/settings/logs/tutorial information architecture
- Visual language: protected/unprotected, live traffic, resolver, latency, uptime
- Logo and icon direction

## What Must Be Rewritten

- React components under `src/components`
- Zustand store under `src/store`
- Tailwind styling
- Tauri invoke/listen bridge
- Tauri tray setup

These become `.slint` components, Rust state structs, Rust callbacks, background tasks, and platform modules.

## Proposed Slint Architecture

```text
CRYPTSHIELD-RUST/
├── Cargo.toml
├── build.rs
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
├── data/
│   ├── com.fuadfaut.cryptshield.desktop
│   ├── com.fuadfaut.cryptshield.policy
│   └── icons/
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── state.rs
│   ├── commands.rs
│   ├── core/
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

## MVP Milestones

1. Slint UI shell
   - Sidebar navigation
   - Dashboard
   - Configuration view
   - Logs view
   - Setup diagnostics view
   - SVG icons and copied logo assets

2. State wiring
   - Rust `AppState`
   - Service status polling
   - UI updates without system mutation
   - Real tab and toggle callbacks

3. Backend port
   - Move helper mode from Tauri binary into Slint binary
   - Preserve resolver allowlist
   - Preserve `auth_admin_keep` policy
   - Build helper parser, structured command plans, and TOML update plans test-first before enabling execution

4. Real actions
   - Start/stop/restart DNSCrypt
   - Restore DNS on stop
   - Start log stream only when logs view is visible
   - Start traffic stream only while protection is active

5. Fedora packaging
   - RPM installs binary, desktop file, icon, Polkit policy
   - Dependencies: `dnscrypt-proxy`, `NetworkManager`, `polkit`, Slint runtime needs

## Privilege Model

Do not run the GUI as root.

The Slint app should keep the current helper design:

```bash
cryptshield --system-helper start cloudflare true true <connection-uuid...>
cryptshield --system-helper stop <connection-uuid...>
cryptshield --system-helper restart quad9 true true <connection-uuid...>
```

The helper must:

- require root
- validate action names
- validate resolver IDs
- validate booleans
- use structured `Command` args
- never execute arbitrary shell fragments

## GNOME Integration

Slint is not a GNOME Shell extension framework. For GNOME Quick Settings, keep the extension separate.

Recommended path:

- Slint desktop app first.
- Rust CLI or D-Bus interface second.
- Optional GNOME extension later.
- Extension should request actions through the safe backend path, not manipulate system DNS directly.

## Risks

- Slint desktop integration may need extra platform code for tray, notifications, and portals.
- Default Slint features can pull multiple renderer/windowing dependencies until optimized.
- Pixel-perfect Tauri parity needs custom components and careful spacing.
- SVG asset colors may need separate selected/unselected variants.
- Some GNOME-native behaviors are easier in GTK/libadwaita.

## Current MVP Folder

CRYPTSHIELD-RUST now contains a non-mutating Rust/Slint migration prototype. It includes the dashboard shell, local Configuration and Logs views, read-only diagnostics, validation-only `--system-helper` handling, a Polkit policy template, and test-driven dry-run plans for config/system changes. It is meant to answer: "Can CryptShield keep its current visual identity while dropping WebKit/Tauri, while moving toward a safer Rust-first backend?"
