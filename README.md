# Secure Vault Personal Information (SVPI)

## Project Description

Secure Vault Personal Information (SVPI) is a small utility for securely storing personal data
(passwords, tokens, notes, etc.) in an encrypted vault.

SVPI is designed to be usable both by humans and by other software:

- interactive CLI for day-to-day usage
- JSON API for integrations (HTTP server or Chrome Native Messaging)

Storage backend can be a file-based vault or a serial-connected device (USB serial / COM port, via
SRWP).

## Documentation

Technical documentation lives in `docs/`:

- [docs/install.md](docs/install.md) — quickstart setup (file-backed vault)
- [docs/architecture.md](docs/architecture.md) — storage layout, segment metadata, encoding rules
- [docs/security.md](docs/security.md) — security model and protection levels
- [docs/master-password.md](docs/master-password.md) — master password, encryption keys, recovery/rotation
- [docs/dump.md](docs/dump.md) — raw dumps and optional dump-file encryption
- [docs/migration.md](docs/migration.md) — export/import migration format
- [docs/api.md](docs/api.md) — JSON API (HTTP server + Chrome Native Messaging)

## Build

SVPI is a Rust project. For a production build, use the provided scripts:

- Linux/macOS: [build.sh](./build.sh) (builds `--release`, copies `svpi` to `./bin/`)
- Windows: [build.bat](./build.bat) (builds `--release`, copies `svpi.exe` to `./bin/`)

## Related repositories

- [serialport_srwp](https://github.com/binqbit/serialport_srwp) — SRWP implementation used by the serial backend
