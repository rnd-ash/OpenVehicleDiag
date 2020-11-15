const { ipcRenderer } = require('electron')
let consts = require('./../ptconsts');


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


window.onload = function() {
    let title = document.getElementById("title");
    let load_ecu = document.getElementById("load_ecu");
    set_batt_voltage();

    load_ecu.onclick = function() {
        ipcRenderer.send("f_open")
    }


    // Display battery votlage every 2 seconds
    setIntervalNow(function() {
        ipcRenderer.send(consts.PT_GET_VBATT);
    }, 2000);

    window.onunload = function() {
        ipcRenderer.sendSync(consts.PT_CLOSE);
    }
}