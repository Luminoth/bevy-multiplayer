use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FindServerResponseV1 {
    pub address: String,
    pub port: u16,
}
