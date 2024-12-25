use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FindServerResponseV1 {
    // TODO: set all of the addresses
    // and let the client pick the one to try
    // (or try them all)
    pub address: String,
    pub port: u16,
}
