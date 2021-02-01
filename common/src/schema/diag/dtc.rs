use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUDTC {
    pub error_name: String,
    pub summary: String,
    pub description: String,
}