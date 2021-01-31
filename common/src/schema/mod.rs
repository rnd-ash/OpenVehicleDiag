pub mod variant;
use serde::{Serialize, Deserialize};
use variant::ECUVariantDefinition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvdECU {
    pub name: String,
    pub description: String,
    pub variants: Vec<ECUVariantDefinition>
}

