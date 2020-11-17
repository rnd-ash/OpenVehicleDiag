use crate::ecu::*;
use crate::caesar::*;
use common::*;


// Converts CContainer to OVD JSON ECU


pub fn convert(container: &CContainer) {
    container.ecus.iter().enumerate().for_each(|(idx, ecu)| {
        ecu.ecu_ifaces_subtype.iter().for_each(|iface| {
            let baud = iface.com_params.iter()
                .find(|i| {i.com_param.0 == ComParam::CP_BAUDRATE})
                .map(|x| x.com_param.1)
                .expect("Error, ECU has no baud rate!");
            println!("Baud rate: {} bps", baud);


            println!("{:?}", iface.name_ctf);
        })
    })
}