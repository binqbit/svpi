// All requests are JSON messages sent to the native host.
// All responses are SvpiResponse (same envelope as the Server API).
function sendNative(request) {
	return new Promise((resolve, reject) => {
		chrome.runtime.sendNativeMessage('com.binqbit.svpi_chrome_app', request, (response) => {
			if (chrome.runtime.lastError) {
				reject(chrome.runtime.lastError.message);
			} else {
				resolve(response);
			}
		});
	});
}

async function get_status() {
	return await sendNative({ status: {} });
}

async function get_list() {
	return await sendNative({ list: {} });
}

// { "get_data": { "name": "name", "password": "password?" } }
// Password is required only for encrypted segments.
async function get_data(name, password = undefined) {
	const request = password ? { get_data: { name, password } } : { get_data: { name } };
	return await sendNative(request);
}
