const axios = require('axios');

const api = axios.create({ baseURL: 'http://localhost:3333' });

// {
// 	"status": "ok" | "device_not_found" | "device_error",
// 	"version": 3
// }
async function status() {
	return (await api.get(`/status`)).data;
}

// {
// 	"segments": [
// 		{
// 			"name": "name",
// 			"data_type": "plain" | "encrypted",
// 			"size": 123
// 		},
// 		...
// 	],
// 	"error": undefined | "device_not_found" | "device_error"
// }
async function list() {
	return (await api.get(`/list`)).data;
}

// {
// 	"name": "name",
// 	"data": "decrypted data",
// 	"error": undefined | "device_not_found" | "device_error" | "password_error" | "error_decode_password" | "password_not_provided" | "data_not_found" | "error_read_data"
// }
async function get(name, password = undefined, useRootPassword = true) {
	return (await api.get(`/get?name=${name}` + (password ? `&password=${password}` : '') + (password ? `&use_root_rassword=${useRootPassword}` : ''))).data;
}

module.exports = { status, list, get };
