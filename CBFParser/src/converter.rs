use crate::ecu::*;
use crate::caesar::*;
use common::*;


// Converts CContainer to OVD JSON ECU


pub fn convert(container: &CContainer) {
    println!("{}", serde_json::to_string_pretty(container).unwrap());
}