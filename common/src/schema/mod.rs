pub mod variant;
use serde::{Serialize, Deserialize};
use variant::ECUVariantDefinition;
pub mod diag;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvdECU {
    pub name: String,
    pub description: String,
    pub variants: Vec<ECUVariantDefinition>
}

