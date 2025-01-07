// Primarily used for Response Types (Beacon API)
// these response types are manipulated to fit into the types of iso-step/types.rs, iso-committee/types.rs
// this involves public key decompression for the step circuit

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct CommitteeUpdateResponse {}

#[derive(Serialize, Deserialize)]
pub struct SyncStepResponse {}
