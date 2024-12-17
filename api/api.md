# API Endpoints Documentation for SVPI

This document describes the API endpoints available on the server running locally at `http://localhost:3333`. The API includes three endpoints: `/status`, `/list`, and `/get`.

## API Endpoints

### 1. GET /status

Fetches the status of the device.

#### Request

- **Method**: GET
- **URL**: `http://localhost:3333/status`

#### Response

- **Items**:
  - **status**: can be one of the following values:
    - `ok`
    - `device_not_found`
    - `device_error`
  - **version**: Architecture version of the device

### 2. GET /list

Retrieves a list of segments from the device.

#### Request

- **Method**: GET
- **URL**: `http://localhost:3333/list`

#### Response

- **Items**:
  - **segments**: a collection of segment items, each containing:
    - **name**: The name of the segment
    - **data_type**: can be one of the following values:
      - `plain`
      - `encrypted`
    - **size**: The size of the segment
  - **error**: can be one of the following values:
    - `undefined`
    - `device_not_found`
    - `device_error`

### 3. GET /get

Fetches decrypted data for a given segment name.

#### Request

- **Method**: GET
- **URL**: `http://localhost:3333/get?name={name}` (Additional query parameters may include `password` and `use_root_password`.)

#### Query Parameters

- **name**: Required. The name of the segment to retrieve.
- **password**: Optional. The password needed to decrypt the data.
- **use_root_password**: Optional. Defaults to `false`. Indicates whether to use the root password.

#### Response

- **Items**:
  - **name**: The name of the retrieved segment
  - **data**: The decrypted data
  - **error**: can be one of the following values:
    - `undefined`
    - `device_not_found`
    - `device_error`
    - `password_error`
    - `error_decode_password`
    - `password_not_provided`
    - `data_not_found`
    - `error_read_data`
