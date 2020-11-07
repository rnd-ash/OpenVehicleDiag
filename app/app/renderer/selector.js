console.log("Test");
let passthru_lib = require('../../index.node');
var selected_elm = null;

function onWindowCreated(win) {
    console.log('Current directory: ' + process.cwd());
    var dropdown = document.getElementById('select-dev');
    var launch_btn = document.getElementById('launch-btn');

    var dev_list = passthru_lib.get_device_list();

    if (dev_list.length == 0) {
        dropdown.disabled = true;
        opt.text = "No devices found";
        dropdown.options.add(opt);

        launch_btn.disabled = true;
    } else {
        for (var i = 0; i < dev_list.length; i++) {
            var opt = document.createElement("option");
            opt.text = dev_list[i]["name"] + " by " + dev_list[i]["vendor"];
            dropdown.options.add(opt);
        }
    }

    launch_btn.onclick = function() {
        console.log("Launching!");
        let idx = dropdown.selectedIndex;
        let res = passthru_lib.connect_device(dev_list[idx]);
        if (res.hasOwnProperty("dev_id")) { // OK! We have a device ID!
            let dev_id = res["dev_id"];
            const { dialog } = require('electron').remote;
            
            return;
        } else { // Error in library - Display popup saying Boo boo
            const { dialog } = require('electron').remote;
            let err = res["err"];
            const options = {
                type: 'error',
                buttons: ["OK"],
                title: 'Adapter init failed',
                message: 'Error: '+err,
            };
            dialog.showMessageBox(win, options);
            //console.log(d);
        }

        console.log(res);
    }
}

module.exports.onWindowCreated = onWindowCreated;