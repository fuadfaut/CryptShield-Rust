# CRYPTSHIELD-RUST

CRYPTSHIELD-RUST is a Rust/Slint native CryptShield desktop prototype for Fedora-focused DNSCrypt management.

It is intentionally non-mutating at this stage:

- It does not change DNS settings.
- It does not call Polkit.
- It does not start or stop `dnscrypt-proxy`.
- It only performs read-only startup checks for dependencies and service status.
- Its `--system-helper` mode validates requests and builds dry-run plans, but exits before executing them.

## Run

```bash
cargo run
```

## Check

```bash
cargo check
```

## Documents

- [PRD.md](PRD.md): product requirements for the Rust + Slint migration.
- [TECH.md](TECH.md): technical design, architecture, privilege model, packaging, and tests.
- [ROADMAP.md](ROADMAP.md): phased path from UI spike to backend parity and optional GNOME extension.
- [MIGRATION.md](MIGRATION.md): migration plan from Tauri/React to Rust + Slint.
- [CHANGELOG.md](CHANGELOG.md): changes in CRYPTSHIELD-RUST.

## Notes

The UI uses copied CryptShield SVG assets from the existing Tauri/GTK prototype and now mirrors the current Tauri dashboard structure: full-height sidebar, main header, centered `max-w-5xl` dashboard content, toggle card, live traffic card, and compact metric cards.

The Rust layer now owns the initial tab/protection state and wires Slint callbacks for navigation and the dashboard switch. Setup diagnostics are read-only and report whether key system tools are available.

The Configuration tab now loads read-only defaults from `/etc/dnscrypt-proxy/dnscrypt-proxy.toml` when available and lets the user adjust resolver/cache/DNSSEC/autostart state locally. These settings are not written back yet.

The app also starts a lightweight background poller for read-only service/dependency status. The Logs tab lazily loads a bounded `journalctl` snapshot on demand; it does not keep a streaming child process open yet.

The repository now includes an initial Polkit policy template and validation-only `--system-helper` argument parser. The helper refuses non-root callers, then intentionally exits before making system changes until the restricted backend actions are implemented.

The helper migration is being developed test-first. Current tests cover structured `pkexec`, `systemctl`, and `nmcli` command plans for start, stop, and restart without executing them.

Config mutation is also test-first: dnscrypt TOML updates are built in memory with `toml_edit` for resolver/cache/DNSSEC changes, then attached to dry-run helper plans. Nothing is written to `/etc` yet.

NetworkManager integration is still read-only. The app parses active connection UUIDs from `nmcli` output and uses the count only in preview messaging.
