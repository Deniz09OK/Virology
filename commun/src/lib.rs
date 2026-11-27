use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconPing {
    pub id: u64,
    pub ts: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconPong {
    pub id: u64,
    pub cmd: Option<String>,
}
