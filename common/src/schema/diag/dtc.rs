use serde::{Serialize, Deserialize};
use super::service::Parameter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUDTC {
    /// Error code name (EG: P2001)
    pub error_name: String,
    /// Summary of error 
    pub summary: String,
    /// Detailed description of the error
    pub description: String,
    /// Optional list of parameters
    /// that can be queried with either KWP2000 or UDS
    /// to get diagnostic data from the ECU about what
    /// was happening at the time of the error
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub envs: Vec<Parameter>
}