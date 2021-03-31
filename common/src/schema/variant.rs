use serde::{Serialize, Deserialize};

use super::diag::{dtc::ECUDTC, service::Service};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUVariantDefinition {
    /// Name of the ECU
    pub name: String,
    /// Description of the ECU
    pub description: String,
    /// List of ECU Variant patterns
    pub patterns: Vec<ECUVariantPattern>,
    /// List of Diagnostic trouble codes, and their descriptions
    pub errors: Vec<ECUDTC>,
    /// Adjustments are functions that write an adjustment to the ECUs memory
    /// which is permanently set between ECU Resets, such as setting engine idle RPM
    #[serde(default = "Vec::new")]
    pub adjustments: Vec<Service>,
    /// Actuations are functions that request the ECU to do something now (Such as open/close a valve)
    /// but are reset to normal when the ECU is either power cycled or returns to its normal default state
    #[serde(default = "Vec::new")]
    pub actuations: Vec<Service>,
    /// Miscellaneous functions
    #[serde(default = "Vec::new")]
    pub functions: Vec<Service>,
    /// These are functions that simply retrieve data from an ECU, and do not
    /// write anything to it. For example, asking the ECU for current fuel rail pressure
    #[serde(default = "Vec::new")]
    pub downloads: Vec<Service>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUVariantPattern {
    /// Hardware vendor ID of the ECU. This would be the mfg of the ECU itself,
    /// rather than the OEM who uses it. Example: Siemens makes ECUs for Mercedes
    pub vendor: String,
    /// Vendor ID (HWID) of the ECU to match against
    pub vendor_id: u32,
}