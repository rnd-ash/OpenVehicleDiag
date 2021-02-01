use serde::{Serialize, Deserialize};

use super::diag::dtc::ECUDTC;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUVariantDefinition {
    pub name: String,
    pub description: String,
    pub patterns: Vec<ECUVariantPattern>,
    pub errors: Vec<ECUDTC>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUVariantPattern {
    pub vendor: String,
    pub vendor_id: u32,
    pub hw_id: u32,
}