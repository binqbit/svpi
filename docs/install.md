# Installation / Quickstart (file-backed vault)

This guide walks through creating a **file-backed** SVPI vault, setting a master password, adding
an encryption key, and storing your first secret (recommended flow).

## Basic setup flow

1. Set the default vault file for the current directory (writes `.svpi`):

```bash
svpi set-file ./data/vault.bin
```

Note: ensure the folder exists first (for example: `mkdir -p ./data`).

2. Start interactive mode:

```bash
svpi
```

3. Initialize storage (example: 1,000,000 bytes, `medium`):

```text
svpi> init 1000000 medium
```

Notes:

- `low/medium/strong/hardened` mainly affect KDF parameters (security vs speed).
- for higher resistance (slower operations), prefer `strong` or `hardened`.

4. Set the master password:

```text
svpi> set-master
```

Security notes:

- Treat the master password as a **root credential**: with it, an attacker can eventually unlock
  everything.
- Prefer prompts/interactive entry; avoid passing secrets via command-line flags (they can leak via
  shell history/process listings).

5. Add a stored encryption key (recommended):

```text
svpi> add-key default-key medium
```

You will be prompted for:

- the **master password**
- a **key password** (entered twice) used to protect the stored key blob

6. Store a secret entry:

```text
svpi> set name password
```

When prompted for `password`, enter the **key password** you set in step 5 (so SVPI can use the
stored key).

7. List stored entries:

```text
svpi> list
```

Optional verification (recommended): `svpi> list` and ensure your entry’s `Pass Hash` matches the
`Hash` of `default-key` (`Data Type` = `EncryptionKey`).

8. Clear screen + wipe in-memory REPL history:

```text
svpi> clear
```

9. Show help:

```text
svpi> help
```
