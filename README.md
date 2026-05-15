# CRYPTSHIELD-RUST

CRYPTSHIELD-RUST is a Rust/Slint native implementation path for CryptShield, a Fedora-focused desktop GUI for managing encrypted DNS through `dnscrypt-proxy`.

This repository now follows the original `fuadfaut/CryptShield` README contract: one-click DNSCrypt protection, NetworkManager DNS routing to `127.0.0.1`, resolver switching, diagnostics, logs, and Fedora/Polkit-oriented system integration. The current Rust app is still intentionally non-mutating while the privileged helper is completed.

## Current Safety State

- The GUI reads dependency, service, config, NetworkManager, and journal status.
- Protection toggles are preview-only and do not call `pkexec` yet.
- `--system-helper` validates root-only requests and builds structured command/config plans, then exits before mutation.
- Config edits are generated in memory with `toml_edit`; nothing is written to `/etc` yet.

## Target Feature Parity

- One-click protection using `dnscrypt-proxy`, Polkit, and a restricted helper.
- System-wide DNS routing through NetworkManager to `127.0.0.1`.
- Resolver switching for the original 10 README choices.
- Cache and DNSSEC options in `dnscrypt-proxy.toml`.
- Tutorial & diagnostics for `dnscrypt-proxy`, `nmcli`, `pkexec`, `systemctl`, and Polkit policy presence.
- Bounded `journalctl` log reading, with query-log traffic counters planned.
- Fedora packaging path with policy and desktop integration.

## Supported Resolvers

| UI choice | dnscrypt-proxy `server_names` value |
| --- | --- |
| Default | no explicit `server_names` entry |
| Cloudflare | `cloudflare` |
| Google | `google` |
| Quad9 | `quad9` |
| AdGuard DNS | `adguard` |
| NextDNS | `nextdns` |
| Cisco | `cisco` |
| Mullvad | `mullvad-doh` |
| CleanBrowsing | `cleanbrowsing-adult` |
| Tiarap DNS | `doh.tiar.app` |

## Run

```bash
cargo run
```

## Test

```bash
cargo fmt --check
cargo check
cargo test
```

## Documents

- [TECH.md](TECH.md): technical design, privilege model, packaging, and tests.
- [ROADMAP.md](ROADMAP.md): phased path from UI spike to backend parity and optional GNOME extension.
- [MIGRATION.md](MIGRATION.md): migration plan from Tauri/React to Rust + Slint.
- [CHANGELOG.md](CHANGELOG.md): changes in CRYPTSHIELD-RUST.

`PRD.md` may exist locally, but it is ignored by this repository's `.gitignore`.
