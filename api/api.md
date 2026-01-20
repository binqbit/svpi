# API Documentation: Server and Chrome App

SVPI exposes a small JSON API over two transports:

- **Server API (HTTP)**: `svpi --mode=server` (Rocket on `0.0.0.0:3333`)
- **Chrome Native Messaging**: `svpi --mode=chrome` (stdin/stdout length-prefixed messages)

Both transports share the same response envelope.

## Response Envelope

All responses are `SvpiResponse` with `schema: "svpi.response.v1"`.

### Success

```json
{
  "schema": "svpi.response.v1",
  "ok": true,
  "command": "api.status",
  "result": {},
  "meta": {
    "app_version": "5.0.0",
    "architecture_version": 7
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
    "app_version": "5.0.0",
    "architecture_version": 7
  }
}
```

## 1. Device Status

### Command

- **Server API**: `GET /status`
- **Chrome App API**:

```json
{ "status": {} }
```

### Success Result (`result`)

- `status`: `"ok"`
- `architecture_version`: device architecture version (`u32`)

### Error Codes

- `device_not_found`
- `device_not_initialized`
- `architecture_mismatch`
- `device_error`

## 2. Retrieve Segments List

### Command

- **Server API**: `GET /list`
- **Chrome App API**:

```json
{ "list": {} }
```

### Success Result (`result`)

- `segments`: array of items
  - `name`: segment name
  - `data_type`: `"plain" | "hex" | "base58" | "base64" | "binary"`
  - `size`: bytes
  - `fingerprint`: segment fingerprint (hex)
  - `password_fingerprint`: password fingerprint (hex) or `null` (when not encrypted)

### Error Codes

- `device_not_found`
- `device_not_initialized`
- `architecture_mismatch`
- `device_error`

## 3. Fetch Segment Data

### Command

- **Server API**: `GET /get?name={name}&password={password}`
- **Chrome App API**:

```json
{ "get_data": { "name": "name", "password": "password" } }
```

### Success Result (`result`)

- `name`: segment name
- `data`: decoded data (string)
- `data_type`: `"plain" | "hex" | "base58" | "base64" | "binary"`
- `encrypted`: `true | false`

### Error Codes

- `device_not_found`
- `device_not_initialized`
- `architecture_mismatch`
- `device_error`
- `data_not_found`
- `password_required`
- `password_error`
- `forbidden` (encryption key segments are not readable via API)
