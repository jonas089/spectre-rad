use ethereum_consensus_types::BeaconBlockHeader;
use serde::{Deserialize, Serialize};

pub type PublicKeys = Vec<Vec<u8>>;
pub type PublicKeyHashes = Vec<Vec<u8>>;
pub type CommitteeBranch = Vec<Vec<u8>>;
pub type Leaf = Vec<u8>;
pub type Root = Vec<u8>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStepArgs {
    pub signature_compressed: Vec<u8>,
    pub pubkeys_uncompressed: Vec<Vec<u8>>,
    pub pariticipation_bits: Vec<bool>,
    pub attested_header: BeaconBlockHeader,
    pub finalized_header: BeaconBlockHeader,
    pub finality_branch: Vec<Vec<u8>>,
    pub execution_payload_root: Vec<u8>,
    pub execution_payload_branch: Vec<Vec<u8>>,
    pub domain: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStepCircuitInput {}
