const { ipcRenderer } = require('electron')
let consts = require('./../ptconsts');
require('bootstrap-darkmode');


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

function setIntervalNow(func, interval) {
    func();
    return setInterval(func, interval);
}

ipcRenderer.on(consts.PT_GET_VBATT, (event, resp) => {
    if (resp['mv'] != null) {
        vbatt = (resp['mv'] / 1000).toFixed(1);
    }
    set_batt_voltage()
});

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
                    document.getElementById("error-body").innerText = `Error message: ${res['err']}`;
                    document.getElementById("error-title").innerText = `Error loading ${filename.filePaths[0]}`;
                    $("#errorModal").modal('show');
                } else {
                    console.log(res);
                }
            }
        });
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
        ipcRenderer.send(consts.PT_GET_VBATT);
    }, 2000);

    window.onbeforeunload = function() {
        ipcRenderer.sendSync(consts.PT_CLOSE);
    }
}