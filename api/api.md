# API Documentation: Server and Chrome App

This document describes the API for both the server and the Chrome application. The goal is to integrate descriptions and highlight similarities in their requests and responses.

## Server and Chrome App API

### 1. Device Status

#### Command

- **Server API**: `GET /status`
- **Chrome App API**: `get_status()`

#### Request

- **Server API**:

```http
GET /status HTTP/1.1
Host: localhost:3333
```

- **Chrome App API**:

```json
{
  "status": {}
}
```

#### Response

- Items:
  - `status`: can be one of the following values:
    - `ok`
    - `device_not_found`
    - `device_error`
  - `version`: Architecture version of the device

#### Description

Fetches the status of the device, returning the status and version. The status can be `ok`, `device_not_found`, or `device_error`.

### 2. Retrieve Segments List

#### Command

- **Server API**: `GET /list`
- **Chrome App API**: `get_list()`

#### Request

- **Server API**:

```http
GET /list HTTP/1.1
Host: localhost:3333
```

- **Chrome App API**:

```json
{
  "list": {}
}
```

#### Response

- Items:
  - `status`: can be one of the following values:
    - `ok`
    - `device_not_found`
    - `device_error`
  - `segments`: a collection of segment items, each containing:
    - `name`: The name of the segment
    - `data_type`: can be one of the following values:
      - `plain`
      - `encrypted`
    - `size`: The size of the segment

#### Description

Retrieves a list of segments from the device, including their names, data types (`plain` or `encrypted`), and sizes. The status indicates the success or failure of the request.

### 3. Fetch Segment Data

#### Command

- **Server API**: `GET /get?name={name}&password={password}&use_root_password={useRootPassword}`
- **Chrome App API**: `get_data(name, password, useRootPassword)`

#### Request

- **Server API**:

```http
GET /get?name={name}&password={password}&use_root_password={useRootPassword} HTTP/1.1
Host: localhost:3333
```

- **Chrome App API**:

```json
{
  "get_data": {
    "name": "name",
    "password": "password",
    "useRootPassword": true
  }
}
```

#### Response

- Items:
  - `status`: can be one of the following values:
    - `ok`
    - `device_not_found`
    - `device_error`
    - `password_error`
    - `error_decode_password`
    - `password_not_provided`
    - `data_not_found`
    - `error_read_data`
  - `name`: The name of the retrieved segment
  - `data`: The decrypted data

#### Description

Fetches decrypted data for the specified segment. The response includes the status of the request and the decrypted data if successful. Possible status values indicate different types of errors or success.

## Conclusion

Both systems use similar request and response structures, differing mainly in the method of request dispatch: HTTP requests for server interaction and `chrome.runtime.sendNativeMessage` for Chrome app communication.