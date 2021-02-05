use serde::{Serialize, Deserialize};

use super::diag::{dtc::ECUDTC, service::Service};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUVariantDefinition {
    pub name: String,
    pub description: String,
    pub patterns: Vec<ECUVariantPattern>,
    pub errors: Vec<ECUDTC>,
    pub services: Vec<Service>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUVariantPattern {
    pub vendor: String,
    pub vendor_id: u32,
}