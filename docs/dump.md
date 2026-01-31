# Dumps (Backups)

A **dump** is a raw byte-for-byte snapshot of the SVPI storage memory region.

## What a dump contains

A dump includes everything inside the storage region:

- initialization markers
- metadata (including dump protection level and the master password check-hash)
- all segment payloads
- all segment metadata entries (including stored encryption keys)

Because it is a full snapshot, a dump is the most complete way to back up or clone a vault.

## Why dump encryption exists

SVPI can optionally encrypt dump files with a separate password (a “transport layer”).

This is defense-in-depth for cases where you want to store dumps in places that might be exposed
(cloud storage, public artifacts, shared drives). Even if the underlying vault data is encrypted,
an attacker can still attempt offline attacks against whatever encrypted material is present in the
dump.

## Encrypted dump format (high-level)

Encrypted dumps are wrapped in an envelope:

- a magic prefix (`SDP`)
- a borsh-encoded envelope containing:
  - `protection` (level code)
  - `payload` (the encrypted raw dump bytes)

The payload encryption uses:

- Argon2id (parameters from the chosen dump-encryption level)
- XChaCha20-Poly1305
- blob format: `salt(16) | nonce(24) | ciphertext(...)`

## Dump encryption levels vs vault dump protection

Dump files have their own optional encryption level. This is separate from the vault’s **dump
protection** stored in metadata:

- Vault dump protection controls how the vault encrypts its own data and derives keys.
- Dump-file encryption controls how hard it is to decrypt the dump file itself.
