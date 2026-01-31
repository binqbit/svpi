# Export / Import (Migration Format)

SVPI supports a text-based export/import format designed for portability and simple migrations.
This format focuses on **segments** (user data entries), not on the full raw storage layout.

## What export/import includes (and excludes)

Included:

- all non-key segments (regular data entries)
- both unencrypted and encrypted entries

Excluded:

- stored encryption keys (segments of type `encryptionkey`)
- vault metadata (including dump protection level and master password check-hash)

Because encryption keys are excluded, encrypted entries that depend on stored keys require that the
corresponding keys exist (or be recreated) in the destination vault.

## Line format

The file is line-oriented. Empty lines are ignored.

### Unencrypted entry

```text
<name> = <value>
```

`<value>` is written as text. For binary data the value is represented as hex.

### Encrypted entry

Encrypted entries are exported as opaque ciphertext plus metadata:

```text
<name> = data:application/vnd.binqbit.svpi;fp=<key_fp>;<data_type>,<hex_ciphertext>
```

Where:

- `<key_fp>` is a 4-byte fingerprint (hex) used as an encryption selector
- `<data_type>` is the type of the _decrypted_ data (`plain`, `hex`, `base58`, `base64`, `binary`)
- `<hex_ciphertext>` is the encrypted blob encoded as hex (`salt|nonce|ciphertext`)

## Important migration notes

- Export does **not** decrypt encrypted entries. It preserves encryption by exporting ciphertext.
- Import restores the segments exactly as described (including the encryption selector fingerprint).
- Because vault metadata is not part of this format:
  - the destination vault must use a compatible dump protection level for existing ciphertext to
    decrypt
  - stored encryption keys must be present/recreated for master-derived encryption to work

If you need a full-fidelity migration (including metadata and stored keys), use a raw dump instead
(see [dump.md](dump.md)).
