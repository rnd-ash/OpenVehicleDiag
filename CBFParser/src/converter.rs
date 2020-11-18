use crate::ecu::*;
use crate::caesar::*;
use common::*;


// Converts CContainer to OVD JSON ECU


pub fn convert(container: &CContainer) {
    container.ecus.iter().enumerate().for_each(|(idx, ecu)| {
        println!("{}", serde_json::to_string_pretty(&ecu.ecu_varients).unwrap());
    })
}