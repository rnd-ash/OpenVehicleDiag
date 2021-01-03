const { ipcRenderer } = require('electron')
let consts = require('./../ptconsts');
require('bootstrap-darkmode');
require("./../passthru_lib");
const {FILTER_TYPE} = require("../passthru_lib");
const {TX_FLAG} = require("../passthru_lib");
const {PROTOCOL} = require("../passthru_lib");
const {PASSTHRU_MSG} = require("../passthru_lib");

let vbatt = 0.0;

function set_batt_voltage() {
    title.innerText = `VBatt: ${vbatt}V`;
    if (vbatt < 11) {
        title.classList.toggle('text-white', false);
        title.classList.toggle('text-danger', true);
    } else {
        title.classList.toggle('text-danger', false);
        title.classList.toggle('text-white', true);
    }
}

function show_popup(header, body, isError = true) {
    document.getElementById("error-body").classList.toggle('alert-success', !isError);
    document.getElementById("error-body").classList.toggle('alert-danger', isError);
    document.getElementById("error-body").innerText = body;
    document.getElementById("error-title").innerText = header;
    $("#errorModal").modal('show');
}

function setIntervalNow(func, interval) {
    func();
    return setInterval(func, interval);
}

function set_bool(id, prot, value) {
    if (value === true) {
        id.innerText = `${prot} \u2713`
    } else if (value === false) {
        id.innerText = `${prot} \u2717`
    }
}

window.onload = function() {
    const themeConfig = new ThemeConfig();
    themeConfig.initTheme();
    let title = document.getElementById("title");
    let load_ecu = document.getElementById("load_ecu");
    let theme_toggle = document.getElementById("theme-toggle");

    let drv_data = ipcRenderer.sendSync(consts.PT_GET_VERSION);
    let dev_capabilities = ipcRenderer.sendSync(consts.PT_GET_DEV_DESC);
    if (drv_data['err'] == null) {
        document.getElementById('fw-version').innerText = `Firmware version: ${drv_data['fw_version']}`
        document.getElementById('api-version').innerText = `Passthru API version: ${drv_data['api_version']}`
        document.getElementById('lib-version').innerText = `Library version: ${drv_data['dll_version']}`
    }
    document.getElementById('ovd-version').innerText = `OVD version: ${require('electron').remote.app.getVersion()}`
    document.getElementById('adapt-name').innerText = `Name: ${dev_capabilities['name']}`
    document.getElementById('adapt-vend').innerText = `Vendor: ${dev_capabilities['vendor']}`
    document.getElementById('adapt-path').innerText = `Library path: ${dev_capabilities['drv_path']}`

    console.log(dev_capabilities);
    // CAN Protocols
    set_bool(document.getElementById('prot-can'), "CAN", dev_capabilities['can']);
    set_bool(document.getElementById('prot-iso15765'), "ISO15765", dev_capabilities['iso15765']);

    // K-Line Protocols
    set_bool(document.getElementById('prot-iso9141'), "ISO9141", dev_capabilities['iso9141']);
    set_bool(document.getElementById('prot-iso14230'), "ISO14230", dev_capabilities['iso14230']);

    // J1850 Protocols
    set_bool(document.getElementById('prot-j1850vpw'), "J1850 VPW", dev_capabilities['j1850vpw']);
    set_bool(document.getElementById('prot-j1850pwm'), "J1850 PWM", dev_capabilities['j1850pwm']);

    // SCI Protocols
    set_bool(document.getElementById('prot-sciaengine'), "SCI A (ENGINE)", dev_capabilities['sci_a_engine']);
    set_bool(document.getElementById('prot-sciatrans'), "SCI A (TRANS)", dev_capabilities['sci_a_trans']);
    set_bool(document.getElementById('prot-scibengine'), "SCI B (ENGINE)", dev_capabilities['sci_b_engine']);
    set_bool(document.getElementById('prot-scibtrans'), "SCI B (TRANS)", dev_capabilities['sci_b_trans']);


    load_ecu.onclick = function() {
        let dialog = require('electron').remote.dialog;
        dialog.showOpenDialog(null, {
            properties: ['openFile'],
            filters: [
                { name: 'OVD Json', extensions: ['ovdjson'] }
            ]
        }).then((filename) => {
            if (!filename.canceled) {
                let res = ipcRenderer.sendSync(consts.PT_LOAD_FILE, filename.filePaths[0]);
                if (res['err'] != null) {
                    show_popup(`Error loading ${filename.filePaths[0]}`, `Error message: ${res['err']}`);
                }
            }
        });
    }

    let channels = [];
    document.getElementById("test_connect").onclick = function() {
        ipcRenderer.invoke(consts.PT_CONNECT, 0x06, 500000, 0x00).then((resp) => {
            if (resp['err'] != null) {
                show_popup(`Error creating PT COM channel`, `PT Driver message: ${resp['err']}`);
            } else {
                channels.push(resp["channel_id"]);
                show_popup(`Opened channel OK!`, `Channel ${resp['channel_id']} was opened successfully`, false);
            }
        });
    }
    document.getElementById("test_filter").onclick = function() {
        if (channels.length === 0) {
            show_popup(`Error creating channel filter`, `No channels created`);
        } else {
            let mask = new Uint8Array(new Buffer.from([0xff, 0xff, 0xff, 0xff]));
            let ptn = new Uint8Array(new Buffer.from([0x00, 0x00, 0x07, 0xE8]));
            let fc = new Uint8Array(new Buffer.from([0x00, 0x00, 0x07, 0xE0]));
            ipcRenderer.invoke(consts.PT_SET_FILTER, channels[0], FILTER_TYPE.FLOW_CONTROL_FILTER, mask, ptn, fc).then((resp) => {
                if (resp['err'] != null) {
                    show_popup(`Error creating PT COM channel`, `PT Driver message: ${resp['err']}`);
                } else {
                    show_popup(`Set channel filter OK!`, `Filter ${resp['id']} was opened successfully`, false);
                }
            });
        }
    }
    document.getElementById("test_disconnect").onclick = function() {
        if (channels.length > 0) {
            let target = channels.pop();
            ipcRenderer.invoke(consts.PT_DISCONNECT, target).then((resp) => {
                if (resp['err'] != null) {
                    show_popup(`Error removing PT COM channel`, `PT Driver message: ${resp['err']}`);
                } else {
                    show_popup(`Closing channel OK!`, `Channel ${target} was closed successfully`, false);
                }
            })
        } else {
            show_popup(`Error closing channel`, `There are no more channels to close`);
        }
    }

    document.getElementById("test_send").onclick = function() {
        let ptmsg = new PASSTHRU_MSG(PROTOCOL.ISO15765, new Uint8Array(new Buffer.from([0x00, 0x00, 0x07, 0xE0, 0x3E, 0x01])));
        ptmsg.set_tx_flags([TX_FLAG.ISO15765_FRAME_PAD]);
        console.log(ptmsg)
        ipcRenderer.invoke(consts.PT_SEND_MSGS, channels[0], [ptmsg], 0).then((resp) => {
            console.log(resp);
        })
    }

    if (themeConfig.getTheme() === 'dark') {
        theme_toggle.innerText = "🌞";
    }

    let isDark = false; //  TODO local storage!
    theme_toggle.onclick = function() {
        if (!isDark) { // Switch to dark mode
            theme_toggle.innerText = "🌞";
            themeConfig.setTheme('dark');
        } else { // Switch to light mode
            theme_toggle.innerText = "🌙";
            themeConfig.setTheme('light');
        }
        isDark = !isDark;
    }

    set_batt_voltage();
    // Display battery voltage every 2 seconds
    setIntervalNow(function() {
        ipcRenderer.invoke(consts.PT_GET_VBATT).then((resp) => {
            if (resp['mv'] != null) {
                vbatt = (resp['mv'] / 1000).toFixed(2);
            }
            set_batt_voltage()
        });
    }, 2000);

    window.onbeforeunload = function() {
        ipcRenderer.sendSync(consts.PT_CLOSE);
    }
}