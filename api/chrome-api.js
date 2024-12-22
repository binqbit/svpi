// {
// 	"status": "ok" | "device_not_found" | "device_error",
// 	"version": 3
// }
async function get_status() {
	return await new Promise((resolve, reject) => {
		chrome.runtime.sendNativeMessage('com.binqbit.svpi_chrome_app', { status: {} }, (response) => {
			if (chrome.runtime.lastError) {
				reject(chrome.runtime.lastError.message);
			} else {
				resolve(response);
			}
		});
	});
}

// {
//	"status": "ok" | "device_not_found" | "device_error",
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
	return await new Promise((resolve, reject) => {
		chrome.runtime.sendNativeMessage('com.binqbit.svpi_chrome_app', { list: {} }, (response) => {
			if (chrome.runtime.lastError) {
				reject(chrome.runtime.lastError.message);
			} else {
				resolve(response);
			}
		});
	});
}

// {
// 	"status": "ok" | "device_not_found" | "device_error" | "password_error" | "error_decode_password" | "password_not_provided" | "data_not_found" | "error_read_data",
// 	"name": "name",
// 	"data": "decrypted data"
// }
async function get_data(name, password = undefined, useRootPassword = true) {
	return await new Promise((resolve, reject) => {
		chrome.runtime.sendNativeMessage('com.binqbit.svpi_chrome_app', { get_data: { name, password, useRootPassword } }, (response) => {
			if (chrome.runtime.lastError) {
				reject(chrome.runtime.lastError.message);
			} else {
				resolve(response);
			}
		});
	});
}
