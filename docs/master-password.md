# Master Password & Key Recovery

This document describes how the **master password** and **encryption keys** fit together, and how
recovery/rotation works.

## What the master password is (and isn’t)

- The master password is the **root secret** for the vault.
- The vault does **not** store the master password.
- The vault stores only a **check-hash** so the software can verify that a provided master password
  is correct.

If you lose the master password, it cannot be recovered from the device/storage.

## Deterministic encryption keys

An encryption key is derived deterministically from:

- the master password
- the key name (used as a stable “seed”)
- the vault’s dump protection level (as part of the KDF parameters)

Consequences:

- With the correct master password, SVPI can **recreate encryption keys** even if their stored
  encrypted blobs are lost.
- The key name is security-critical: changing the key name creates a _different_ derived key.

## Key passwords and why they exist

Stored encryption keys are saved **encrypted** with a regular password (“key password”).

This password protects the stored key blob, but it does **not** define the underlying key material
used for data encryption (that material comes from the master password + key name).

That design enables:

- **Key-password rotation** without re-encrypting data
- **Recovery** when a key password is lost, as long as the master password is available

## Common recovery scenarios

### You lost a key password, but still have the master password

You can recreate the key material from the master password (same key name), then protect it again
with a new key password. Existing encrypted data remains decryptable because it is encrypted with
the same underlying key material.

After recreating/re-protecting a key, you may need to update per-entry metadata that points to the
key (the 4-byte key selector fingerprint). SVPI includes mechanisms to re-link these references.

### You lost the stored encryption key blob, but still have the master password

Same as above: recreate the key from master password + key name, then re-link encrypted entries.

### You lost the master password

You cannot recreate master-derived encryption keys. Encrypted entries that depend on those keys
become unrecoverable.

The only encrypted data that can still be accessed in this situation is data that was encrypted
using the “default key” preset (where the user password itself was used as key material).

## Master password operational safety

Because the master password can unlock (directly or indirectly) the entire vault, treat it as a
root credential:

- never type it on untrusted machines
- assume screen capture/keyloggers are the primary threat during entry
- store it offline (password manager, printed/physical vault, etc.)

If you suspect the master password was exposed, consider the vault compromised and migrate to a
new master password and new keys.
