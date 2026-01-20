const axios = require('axios');

const api = axios.create({ baseURL: 'http://localhost:3333' });

// All endpoints return SvpiResponse:
// {
//   "schema": "svpi.response.v1",
//   "ok": true,
//   "command": "api.status",
//   "result": { ... },
//   "meta": { "app_version": "6.0.0", "architecture_version": 8 }
// }
//
// or (error):
// {
//   "schema": "svpi.response.v1",
//   "ok": false,
//   "command": "api.get",
//   "error": { "code": "...", "message": "...", "details": { ... } },
//   "meta": { "app_version": "6.0.0", "architecture_version": 8 }
// }
async function get_status() {
	return (await api.get('/status')).data;
}

async function get_list() {
	return (await api.get('/list')).data;
}

// GET /get?name=... [&password=...]
// Password is required only for encrypted segments.
async function get_data(name, password = undefined) {
	const params = { name };
	if (password) {
		params.password = password;
	}
	return (await api.get('/get', { params })).data;
}

module.exports = { get_status, get_list, get_data };
