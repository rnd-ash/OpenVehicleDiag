const {
  ipcRenderer
} = require('electron');

let consts = require('./../ptconsts');

window.onload = function () {
  console.log("Ready");
  let dev_list = ipcRenderer.sendSync(consts.PT_GET_DEV_LIST);
  let title = document.getElementById("title");
  let body = document.getElementById("body");

  if (dev_list.hasOwnProperty("err")) {
    // Error getting device list!?
    let text = `Native function returned error. Message: ${dev_list['err']}`;
    title.textContent = "No Passthru Devices detected";
    body.innerHTML += "<p class='card-text' id='msg'>$text</p>";
    document.getElementById('msg').innerText = text;
  } else {
    title.textContent = "Select Passthru device";
    body.innerHTML += "<select class='browser-default custom-select' id='select-dev' style='margin: 5px'>";
    body.innerHTML += "<button type='button' class='btn btn-outline-info' id='btn' style='margin: 5px'>Launch OVD</button>";
    body.innerHTML += `<div class='alert alert-danger' role='alert' style='margin: 5px' id='error_txt'></div>`;
    let err = document.getElementById('error_txt');
    let btn = document.getElementById("btn");
    let dropdown = document.getElementById("select-dev");
    err.style.display = "none"; // Hide error msg

    for (let i = 0; i < dev_list.length; i++) {
      let opt = document.createElement("option");
      opt.text = dev_list[i]["name"] + " by " + dev_list[i]["vendor"];
      dropdown.options.add(opt);
    }

    btn.onclick = function () {
      let dev = dev_list[dropdown.selectedIndex];
      let res = ipcRenderer.sendSync(consts.PT_OPEN, dev);

      if (res["dev_id"] != null) {
        ipcRenderer.send('newWindow', "./renderer/main.html", 1280, 720);
      } else {
        err.style.display = "block"; // Show error msg

        err.innerText = `Library err: ${res['err']}`; // Set error to what was returned by API
      }
    };
  }
};