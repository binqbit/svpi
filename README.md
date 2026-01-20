# Secure Vault Personal Information (SVPI)

## Project Description

Secure Vault Personal Information (SVPI) is software that securely stores personal data on a [Blaustahl Storage Device](https://github.com/binqbit/blaustahl). The project is developed based on a [Simple Read Write Protocol (SRWP)](https://github.com/binqbit/serialport_srwp), ensuring reliable interaction with the device through a serial port. SVPI provides users with functionality for managing, organizing, and optimizing data storage, making the use of the Blaustahl device more convenient and efficient.

The primary goal of SVPI is to offer a simple and intuitive interface for working with personal data, which avoids complex operations and eases data management for the user. The project is targeted at those who need a reliable and easily accessible storage solution for confidential information.

## Build

To build the SVPI project, execute the provided build scripts for your operating system: [Linux](./build.sh) and [Windows](./build.bat).

## How It Works (Concepts)

SVPI is built around a small set of core concepts. The exact list of CLI commands is intentionally not duplicated here — run `svpi --help` (or `svpi help`) to see the current command set.

### Storage Backends

SVPI can use multiple backends:

- **SerialPort**: the Blaustahl device (default backend).
- **File**: a file-backed storage (useful for testing, CI, or local vault files).
- **Memory**: in-memory backend (used mainly in tests).

### Modes (Execution + Output)

SVPI has a single execution pipeline, with different modes controlling interaction and output format:

- **CLI mode** (`--mode=cli`): interactive (can prompt for passwords / confirmations).
- **JSON mode** (`--mode=json`): machine-friendly JSON responses; no interactive prompts. For destructive actions, `--confirm` is required.
- **Server mode** (`--mode=server`): HTTP API server.
- **Chrome mode** (`--mode=chrome`): Chrome Native Messaging transport (stdin/stdout).

Note: some global flags require `=` (e.g. `--mode=json`, `--file=vault.bin`).

### Data Entries

Data is stored as named **segments**:

- Each entry has a **name** (up to 32 bytes) and a **data type** (`plain`, `hex`, `base58`, `base64`, `binary`).
- Each entry may be **unencrypted** (stored as decoded bytes) or **encrypted** (stored as an encrypted blob).
- Encrypted entries keep `data_type` as the **type of the decrypted data**.

Each segment also has small hashes in its metadata:

- `fingerprint`: a short segment fingerprint (hex, 4 bytes).
- `password_fingerprint` (optional): presence means the segment is encrypted. For normal data entries this field stores the **encryption-key fingerprint** used to decrypt the entry.

### Encryption Model

SVPI uses a two-layer model: **Master Password → Encryption Keys → Data**.

- **Master password** is the root secret. The device stores only a check-hash of it in metadata (it does not store the password itself).
- **Encryption keys** are derived from the master password + key name, and then encrypted with a regular **key password**. This makes it possible to recreate keys from the master password (and then re-link encrypted data).
- **Data encryption** uses XChaCha20-Poly1305 with a key derived via Argon2id. Encrypted blobs are stored as: `salt(16) | nonce(24) | ciphertext`.

### Dump Protection Level

When initializing a storage, you choose a dump protection level: `low` (1), `medium` (2), `strong` (4).

This level is stored in metadata and influences KDF parameters (Argon2id) used across cryptographic operations.

### Dump / Load

SVPI supports raw dumps for backups and migrations:

- **Dump** is a raw byte-for-byte snapshot of the storage memory region.
- **Load** overwrites the storage with a provided dump (destructive operation).

### Wiping & Optimization

To reduce data remanence:

- Removing an entry overwrites its data and metadata with zeroes.
- Optimizing compacts segments and overwrites the freed region with zeroes.

## Export / Import Format

### Text file list of data with the following format

- Plain data:

```plaintext
<name> = <data>
```

- Encrypted data:

```plaintext
<name> = data:application/vnd.binqbit.svpi;fp=<key_fingerprint>;<data_type>,<hex_ciphertext>
```

`data_type` is the type of the decrypted data; encrypted payload is stored as hex.

## Data Storage Architecture

SVPI uses a carefully designed segment architecture for managing and storing data on the Blaustahl device. This structure allows for efficient organization of information, ensuring quick access and ease of management.

### Data Storage Format

1. Metadata Initialization:

   - `"\0<METADATA>\0"`: Marker for the start of metadata segment initialization.
   - `Metadata` (borsh, fixed-size):
     - `version` (`u32`): architecture version stored on the device.
     - `memory_size` (`u32`): total storage size in bytes.
     - `dump_protection` (`u8`): protection level (`low`, `medium`, `strong`).
     - `master_password_hash` (`[u8; 32]`): master password check-hash.
   - `"\0</METADATA>\0"`: Marker for the end of metadata segment initialization.

2. Segment Data:

   - A sequence of segment data blobs stored from low addresses upwards.

3. Segment Metadata:
   - Segment metadata is stored from high addresses downwards as fixed-size `DataInfo` entries (borsh):
     - `name` (`[u8; 32]`)
     - `address` (`u32`)
     - `size` (`u32`)
     - `data_type` (`u8` enum)
     - `password_fingerprint_present` (`bool`) + `password_fingerprint` (`[u8; 4]`, always stored; zeroed when absent)
     - `fingerprint` (`[u8; 4]`) + `probe` (`u8`)
   - `segments_count` (`u32`) is stored in the last 4 bytes of the storage.

### Why This Structure is Needed?

This architecture provides a clear organization of data on the device, allowing for easy addition, extraction, and deletion of information. Initialization markers help verify data integrity, while segment metadata ensures ease of management and memory optimization. This storage method effectively utilizes available space and minimizes fragmentation, thereby increasing the device's performance.

### API

[API](./api/api.md) for developers who want to integrate SVPI into their software.
