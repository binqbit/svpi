# Security Model

This document describes how security works in SVPI: the key hierarchy, cryptographic primitives,
protection levels, and operational recommendations.

## Key hierarchy (two-layer model)

SVPI uses a layered model:

1. **Master password** → derives encryption keys (deterministically)
2. **Encryption keys** → encrypt/decrypt user data

In addition, the storage may be exported as a **dump file**, which can optionally be encrypted as a
separate “transport layer” (see [dump.md](dump.md)).

## Secrets and where they live

### Master password

- The **master password** is the root secret.
- SVPI does **not** store the master password itself.
- The storage keeps only a **check-hash** of the master password in metadata, to validate that a
  provided master password is correct.

### Encryption keys (named)

Encryption keys are named objects (their name is part of the derivation “seed”).

- **Key material** is derived from:
  - master password, and
  - key name (hashed to a salt)
- The derived key material is then **stored encrypted** using a regular user password (“key
  password”).

This means:

- data can remain encrypted with stable key material
- key passwords can be changed without re-encrypting data (see [master-password.md](master-password.md))

### Per-entry encryption selector (`password_fingerprint`)

If a segment is encrypted, its metadata contains a 4-byte `password_fingerprint`.

Conceptually, it is a **selector** that helps SVPI decide which key is expected for decryption:

- it can refer to a stored encryption key (via that key-segment fingerprint), or
- it can represent the “default key” fingerprint (when no encryption key is stored and the user
  password itself is used as key material).

## Cryptography used

- **KDF**: Argon2id
- **Cipher**: XChaCha20-Poly1305 (AEAD)
- **Encrypted blob format** (for data and encrypted key material):
  - `salt(16) | nonce(24) | ciphertext(...)`

## Protection levels

SVPI uses a shared enum of levels:

- `low`
- `medium` (default)
- `strong`
- `hardened`

These levels primarily affect Argon2id parameters (time/parallelism/memory).

Default parameters:

| Level    | Argon2id m_cost (KiB) | t_cost | p_cost | Intended use                    |
| -------- | --------------------: | -----: | -----: | ------------------------------- |
| low      |                32,768 |      1 |      1 | low-power devices / fast unlock |
| medium   |               131,072 |      1 |      2 | default balance                 |
| strong   |               262,144 |      2 |      4 | slower, higher resistance       |
| hardened |               262,144 |      4 |      4 | slowest, most conservative      |

## “Dump protection” vs “encryption level”

SVPI intentionally separates two knobs that are easy to confuse:

### 1) Storage dump protection (global)

- Stored in metadata as `dump_protection`.
- Used as the baseline cost for cryptographic operations across the storage:
  - data encryption/decryption
  - deriving encryption keys from the master password
  - validating the master password (check-hash)
- Chosen mainly based on **device capabilities** and desired baseline security.

Important property: the dump protection level becomes part of the cryptographic domain. If you
move encrypted blobs into a storage with a different dump protection level, you generally must
decrypt and re-encrypt them; otherwise decryption will fail.

### 2) Encryption key level (per key)

Each stored encryption key also has its own `level`. This controls how expensive it is to process
the **key password** when encrypting/decrypting the stored encryption-key blob.

Effective behavior:

- The effective level is chosen as the strongest by category between:
  - the per-key `level`, and
  - the vault `dump_protection`.
- Category is defined by a simple multiplier: `low=1`, `medium=2`, `strong=4`, `hardened=4`.
  (So `strong` and `hardened` are considered the same category for this comparison; if both sides
  are in the same category, the per-key level wins the tie.)

## Default behavior when no encryption key exists

SVPI supports a “no stored key” mode:

- If no stored encryption key can be found/unlocked, SVPI treats the provided password as the
  **encryption key material itself** (the “default key” preset).
- This allows encrypted storage without ever setting a master password or creating stored
  encryption keys.

Trade-off:

- You lose the master-password recovery path for those entries, because there is no independent
  master-derived key material to recreate (see [master-password.md](master-password.md)).

## Operational recommendations

- Treat the master password as a **root password**:
  - use high entropy
  - store offline
  - enter it only on trusted machines (assume keyloggers/recording are the primary risk)
- Choose dump protection based on device limits (RAM/CPU), but prefer higher levels when feasible.
- Prefer stored encryption keys for long-term vaults (they enable recovery and key-password
  rotation).
