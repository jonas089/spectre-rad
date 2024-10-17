pub mod types;
pub mod utils;

use ethereum_consensus_types::BeaconBlockHeader;
use serde::{Deserialize, Serialize};
use types::{CommitteeBranch, PublicKeys};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeUpdateArgs {
    pub pubkeys_compressed: PublicKeys,
    pub finalized_header: BeaconBlockHeader,
    pub sync_committee_branch: CommitteeBranch,
}

#[cfg(test)]
mod test {
    use crate::{
        types::{PublicKeyHashes, Root},
        utils::{hash_keys, load_test_args, merkleize_keys, verify_merkle_proof},
        CommitteeUpdateArgs,
    };

    #[test]
    fn test_verify_committee_root() {
        let args: CommitteeUpdateArgs = load_test_args();
        let key_hashs: PublicKeyHashes = hash_keys(args.pubkeys_compressed.clone());
        let committee_root_ssz: Root = merkleize_keys(key_hashs);
        let finalized_state_root: Root = args.finalized_header.state_root.to_vec();

        verify_merkle_proof(
            args.sync_committee_branch,
            committee_root_ssz,
            &finalized_state_root,
            110,
        );
    }
}
