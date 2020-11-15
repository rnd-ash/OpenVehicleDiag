
// File that handles IPC with the passthru library
const ipc = require('electron').ipcMain;
const passthru_lib = require('../index.node');
let consts = require('./ptconsts');

let dev_lock = false;
let dev_id = 0;

// If any Passthru functions return JSON with 'err' in the key, then it is an API Error

function log(msg) {
    console.log(`IPC_PASSTHRU => ${msg}`)
}

function log_res(input) {
    if (input['err'] != null) {
        log(`Passthru library error: ${input['err']}`);
    }
    return input;
}

ipc.on(consts.PT_GET_DEV_LIST, (event) => {
    log("Retrieving PT device list");
    event.returnValue = log_res(passthru_lib.get_device_list());
});

ipc.on(consts.PT_CONNECT, (event, json) => {
    log(`Opening device. JSON: ${json}`);
    let resp = passthru_lib.connect_device(json);
    if (resp['dev_id'] != null) {
        dev_id = resp['dev_id']; // Set device ID here so we don't have to keep querying it later on
    }
    event.returnValue = log_res(resp);
});

ipc.on(consts.PT_GET_VBATT, (event) => {
   log("Getting VBATT");
   event.reply(consts.PT_GET_VBATT, log_res(passthru_lib.get_vbatt(dev_id)));
});

ipc.on(consts.PT_CLOSE, (event) => {
    log(`Closing device ${dev_id}`);
    if (dev_lock) { // No DON'T close the connection - Something is being written or read!
        event.returnValue = {"err": "Device in operation"};
    } else {
        event.returnValue = log_res(passthru_lib.close(dev_id));
    }
});

