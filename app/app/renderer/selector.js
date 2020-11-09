let passthru_lib = require('../../index.node');
let selected_elm = null;


window.onload = function() {
    console.log("Ready");
    let dev_list = passthru_lib.get_device_list();
    let title = document.getElementById("title");
    let body = document.getElementById("body");
    if (dev_list.hasOwnProperty("err")) { // Error getting device list!?
        let text = `Native function returned error. Message: ${dev_list['err']}`;
        title.textContent = "No Passthru Devices detected";
        body.innerHTML += "<p class='card-text' id='msg'>$text</p>";
        document.getElementById('msg').innerText = text;
    } else {
        title.textContent = "Select Passthru device";
        body.innerHTML += "<select class='browser-default custom-select' id='select-dev' style='margin: 5px'>";
        body.innerHTML += "<button type='button' class='btn btn-outline-info' id='btn' style='margin: 5px'>Launch OVD</button>";

        let btn = document.getElementById("btn");
        let dropdown = document.getElementById("select-dev");

        for (let i = 0; i < dev_list.length; i++) {
            let opt = document.createElement("option");
            opt.text = dev_list[i]["name"] + " by " + dev_list[i]["vendor"];
            dropdown.options.add(opt);
        }
        btn.onclick = function() {
            let dev = dev_list[dropdown.selectedIndex];
            console.log(passthru_lib.connect_device(dev));

        }
    }
}

console.log(dev_list);