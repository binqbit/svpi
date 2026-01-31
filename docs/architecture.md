# Storage Architecture

SVPI stores everything inside a single contiguous storage region (a file-backed vault, or a
serial-connected device exposed as a USB serial / COM port). That region is treated as a byte array
of size `memory_size`.

This document describes the **binary layout**, **segment format**, and **encoding rules**.

## High-level layout

The storage has three main zones:

1. **Initialization markers + metadata** (at the beginning)
2. **Segment payloads** (grow upward, low → high addresses)
3. **Segment metadata table + segments_count** (grow downward, high → low addresses)

Diagram (not to scale):

```text
0
├─ "\0<METADATA>\0"                (marker)
├─ Metadata (borsh, fixed-size)
├─ "\0</METADATA>\0"               (marker)
├─ Segment data blobs              (allocated from low addresses upward)
├─ Free space
├─ Segment metadata entries        (allocated from high addresses downward)
└─ segments_count (u32, last 4 bytes)
memory_size
```

## Metadata

The metadata is a fixed-size borsh-encoded struct:

- `version: u32` — storage architecture version
- `memory_size: u32` — total storage size in bytes
- `dump_protection: u8` — global cryptographic protection level (see [security.md](security.md))
- `master_password_hash: [u8; 32]` — master password check-hash (not the password itself)

The metadata is surrounded by initialization markers so SVPI can quickly detect whether the
storage was initialized correctly.

## Segments

SVPI stores user data as **segments**.

Each segment consists of:

- a **payload** (raw bytes) stored in the “segment data blobs” area
- a fixed-size **metadata entry** (`DataInfo`, borsh) stored in the metadata table

### `DataInfo` (segment metadata entry)

`DataInfo` is borsh-encoded and has fixed size.

Fields:

- `name: [u8; 32]` — UTF-8 bytes, zero-padded (deleted segments have all zeroes)
- `address: u32` — payload start address
- `size: u32` — payload size in bytes
- `data_type: u8` — how to interpret **decrypted** data (`plain`, `hex`, `base58`, `base64`, `binary`)
- `password_fingerprint: Option<[u8; 4]>` — present means **encrypted payload**
- `fingerprint: { fingerprint: [u8; 4], probe: u8 }` — short segment fingerprint + collision probe

Notes:

- `password_fingerprint` is an encryption **selector** (what key to use). Its meaning is explained
  in [security.md](security.md) because it depends on whether you use stored encryption keys or
  the “default key” fallback.
- `fingerprint` is computed from the segment payload bytes (first 4 bytes of SHA-256), plus a
  `probe` field used to avoid collisions among active segments.

### Segment allocation

- Segment payloads are allocated sequentially from the start of the payload region.
- Segment metadata entries are allocated from the end of the storage backwards.
- The last 4 bytes store `segments_count` (`u32`), which controls how many `DataInfo` entries are
  read during load.

## Data encoding rules

### Unencrypted payloads

When `password_fingerprint` is **absent**, the payload bytes are stored as the decoded bytes of the
selected `data_type`:

- `plain`: UTF-8 bytes
- `hex`: decoded bytes (the payload is not ASCII hex)
- `base58`: decoded bytes
- `base64`: decoded bytes
- `binary`: raw bytes

### Encrypted payloads

When `password_fingerprint` is **present**, the payload is stored as a binary encrypted blob.
The `data_type` still represents the **type of the decrypted data** (so consumers know how to
interpret the plaintext after decryption).

Encrypted blob format:

```text
salt(16) | nonce(24) | ciphertext(...)
```

The cipher is XChaCha20-Poly1305, with a key derived via Argon2id (parameters depend on the global
`dump_protection`).

## Wiping & optimization

SVPI tries to reduce data remanence:

- Removing a segment overwrites its payload and metadata entry with zeroes.
- Optimization compacts active segments to remove gaps, then overwrites the freed region with
  zeroes.
