let passthru_lib = require('./index.node');
var selected_elm = null;

window.onload = function() {
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
}