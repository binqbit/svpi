# API Documentation (Server + Chrome)

SVPI exposes a small JSON API over two transports:

- **Server API (HTTP)**: `svpi --mode=server` (Rocket on `0.0.0.0:3333`)
- **Chrome Native Messaging**: `svpi --mode=chrome` (stdin/stdout length-prefixed messages)

Both transports share the same response envelope.

## Response envelope

All responses are `SvpiResponse` with `schema: "svpi.response.v1"`.

### Success

```json
{
  "schema": "svpi.response.v1",
  "ok": true,
  "command": "api.status",
  "result": { "status": "ok", "architecture_version": 8 },
  "meta": {
    "app_version": "6.0.0",
    "architecture_version": 8
  }
}
```

### Error

```json
{
  "schema": "svpi.response.v1",
  "ok": false,
  "command": "api.get",
  "error": {
    "code": "password_required",
    "message": "Password required for decryption",
    "details": { "name": "my-secret" }
  },
  "meta": {
    "app_version": "6.0.0",
    "architecture_version": 8
  }
}
```

Notes:

- `meta.app_version` / `meta.architecture_version` come from the running SVPI binary.
- `result.architecture_version` (in `/status`) is the architecture version found on the device.
- `command` uses `api.*` for HTTP and `chrome.*` for Native Messaging.

## 1) Device status

### Command

- **Server API**: `GET /status`
- **Chrome App API**:

```json
{ "status": {} }
```

### Success result (`result`)

- `status`: `"ok"`
- `architecture_version`: device architecture version (`u32`)

### Error codes

- `device_not_found`
- `device_not_initialized`
- `architecture_mismatch`
- `device_error`

## 2) Retrieve segments list

### Command

- **Server API**: `GET /list`
- **Chrome App API**:

```json
{ "list": {} }
```

### Success result (`result`)

- `segments`: array of items
  - `name`: segment name
  - `data_type`: `"plain" | "hex" | "base58" | "base64" | "binary"`
  - `size`: bytes
  - `fingerprint`: segment fingerprint (hex)
  - `password_fingerprint`: encryption selector fingerprint (hex) or `null`

Notes:

- Encryption key segments are not returned by `/list`.

### Error codes

- `device_not_found`
- `device_not_initialized`
- `architecture_mismatch`
- `device_error`

## 3) Fetch segment data

### Command

- **Server API**: `GET /get?name={name}[&password={password}]`
- **Chrome App API**:

```json
{ "get_data": { "name": "name", "password": "password" } }
```

### Success result (`result`)

- `name`: segment name
- `data`: decoded data (string)
- `data_type`: `"plain" | "hex" | "base58" | "base64" | "binary"`
- `encrypted`: `true | false`

Notes:

- `password` is optional for unencrypted segments. For encrypted segments it is required.
- You can detect encryption via `/list`: `password_fingerprint != null`.
- Encryption key segments are not readable via API.

### Error codes

- `device_not_found`
- `device_not_initialized`
- `architecture_mismatch`
- `device_error`
- `data_not_found`
- `password_required`
- `password_error`
- `forbidden`
