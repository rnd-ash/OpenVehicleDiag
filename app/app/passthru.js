
// File that handles IPC with the passthru library
const ipc = require('electron').ipcMain;
const passthru_lib = require('../index.node');
let consts = require('./ptconsts');

let dev_lock = false;
let dev_id = 0;

function log(msg) {
    console.log(`IPC_PASSTHRU => ${msg}`)
}



ipc.on(consts.PT_GET_DEV_LIST, (event) => {
    log("Retrieving PT device list");
    event.returnValue = passthru_lib.get_device_list();
});

ipc.on(consts.PT_CONNECT, (event, json) => {
    log(`Opening device. JSON: ${json}`);
    event.returnValue = passthru_lib.connect_device(json);
});

ipc.on(consts.PT_CLOSE, (event, dev_id) => {
    log(`Closing device ${dev_id}`);
    if (dev_lock) { // No DON'T close the connection - Something is being written or read!
        event.returnValue = {"ERR": "Device in operation"};
    } else {
        event.returnValue = {"ERR": "TODO"};
    }
});

