const { ipcRenderer } = require('electron')
let consts = require('./../ptconsts');

let vbatt = 0.0;
ipcRenderer.on(consts.PT_GET_VBATT, (event, resp) => {
    if (resp['mv'] != null) {
        vbatt = resp['mv'] / 1000;
    }
    title.innerText = `Battery voltage: ${vbatt}V`;
    if (vbatt < 11) {
        title.classList.toggle('text-secondary', false);
        title.classList.toggle('text-warning', true);
    } else {
        title.classList.toggle('text-warning', false);
        title.classList.toggle('text-secondary', true);
    }
});


window.onload = function() {
    let title = document.getElementById("title");
    // Display battery votlage every 2 seconds
    setInterval(function() {
        ipcRenderer.send(consts.PT_GET_VBATT);
    }, 2000);
}