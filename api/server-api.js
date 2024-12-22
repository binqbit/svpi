const axios = require('axios');

const api = axios.create({ baseURL: 'http://localhost:3333' });

// {
// 	"status": "ok" | "device_not_found" | "device_error",
// 	"version": 3
// }
async function get_status() {
	return (await api.get(`/status`)).data;
}

// {
// 	"status": "ok" | "device_not_found" | "device_error",
// 	"segments": [
// 		{
// 			"name": "name",
// 			"data_type": "plain" | "encrypted",
// 			"size": 123
// 		},
// 		...
// 	]
// }
async function get_list() {
	return (await api.get(`/list`)).data;
}

// {
// 	"status": "ok" | "device_not_found" | "device_error" | "password_error" | "error_decode_password" | "password_not_provided" | "data_not_found" | "error_read_data",
// 	"name": "name",
// 	"data": "decrypted data"
// }
async function get_data(name, password = undefined, useRootPassword = true) {
	return (await api.get(`/get?name=${name}` + (password ? `&password=${password}` : '') + (password ? `&use_root_rassword=${useRootPassword}` : ''))).data;
}

module.exports = { get_status, get_list, get_data };
