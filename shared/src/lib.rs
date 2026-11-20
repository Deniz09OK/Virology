use serde::{Deserialize, Serialize};

/// Beacon check-in sent by the implant
#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconPing {
    pub id: u64,
    pub ts: u64,
}

/// C2 response — cmd is None for idle, Some("...") when tasking
#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconPong {
    pub id: u64,
    pub cmd: Option<String>,
}
