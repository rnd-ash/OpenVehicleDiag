
// File that handles IPC with the passthru library
const ipc = require('electron').ipcMain;
const passthru_lib = require('../index.node');
require("./passthru_lib");
let consts = require('./ptconsts');

let dev_lock = false;
let dev_id = 0;
let dev_desc = null;



// If any Passthru functions return JSON with 'err' in the key, then it is an API Error
function log(msg) {
    console.log(`IPC_PASSTHRU => ${msg}`)
}

function log_res(input) {
    if (input['err'] != null) {
        let msg = input['err'];
        if (msg === "Unspecified error") { // Try to get the error string from library
            let detail = passthru_lib.get_last_err();
            if (detail !== "") {
                msg = `Operation failed - ${detail}`;
                input['err'] = msg; // Set the new error message for the UI to use
            }
        }
        log(`Passthru library error: ${msg}`);
    }
    return input;
}

ipc.on(consts.PT_GET_DEV_LIST, (event) => {
    log("Retrieving PT device list");
    event.returnValue = log_res(passthru_lib.get_device_list());
});

ipc.on(consts.PT_OPEN, (event, json) => {
    log(`Opening device. JSON: ${json}`);
    let resp = passthru_lib.open(json);
    if (resp['dev_id'] != null) {
        dev_id = resp['dev_id']; // Set device ID here so we don't have to keep querying it later on
    }
    dev_desc = json;
    event.returnValue = log_res(resp);
});

ipc.handle(consts.PT_GET_VBATT, async (event) => {
   log("Getting VBATT");
   return log_res(passthru_lib.get_vbatt(dev_id));
});

ipc.on(consts.PT_CLOSE, (event) => {
    log(`Closing device ${dev_id}`);
    if (dev_lock) { // No DON'T close the connection - Something is being written or read!
        event.returnValue = {"err": "Device in operation"};
    } else {
        event.returnValue = log_res(passthru_lib.close(dev_id));
    }
});

ipc.on(consts.PT_GET_VERSION, (event) => {
    log(`Querying device info ${dev_id}`);
    if (dev_lock) { // No DON'T close the connection - Something is being written or read!
        event.returnValue = {"err": "Device in operation"};
    } else {
        event.returnValue = log_res(passthru_lib.get_version(dev_id));
    }
});

ipc.on(consts.PT_LOAD_FILE, (event, path) => {
    event.returnValue = log_res(passthru_lib.load_file(path));
})

ipc.on(consts.PT_GET_DEV_DESC, (event) => {
    event.returnValue = dev_desc;
})

ipc.handle(consts.PT_CONNECT, async (event, protocol, baud, flags) => {
    log(`Creating channel`);
    let res = passthru_lib.connect(dev_id, protocol, flags, baud)
    return log_res(res);
});

ipc.handle(consts.PT_SET_FILTER, async (event, channel_id, type, mask, ptn, flow_control) => {
    log(`Creating channel filter`);
    let fc = flow_control;
    if (fc == null) {
        fc = new Uint8Array(0);
    }
    let res = passthru_lib.set_filter(channel_id, type, mask, ptn, flow_control)
    return log_res(res);
});

ipc.handle(consts.PT_SEND_MSGS, async(event, channel_id, msgs, timeout) => {
    let tmp = [];
    for (let i = 0; i < msgs.length; i++) {
        tmp.push(msgs[i].to_raw())
    }
    let res = passthru_lib.send_msgs(channel_id, tmp, timeout);
    return log_res(res);
})

ipc.handle(consts.PT_DISCONNECT, async (event, id) => {
    log(`Removing channel ${id}`);
    let res = passthru_lib.disconnect(id);
    return log_res(res);
});

