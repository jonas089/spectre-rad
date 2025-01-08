// The Licensed Work is (c) 2023 ChainSafe
// Code: https://github.com/ChainSafe/Spectre
// SPDX-License-Identifier: LGPL-3.0-only
use beacon_api_client::{BlockId, Client, ClientTypes};
use committee_iso::types::CommitteeUpdateArgs;
use eth_types::Spec;
use ethereum_consensus_types::LightClientUpdateCapella;
use itertools::Itertools;
use log::debug;
use ssz_rs::Merkleized;

use crate::{get_block_header, get_light_client_update_at_period};

/// Fetches LightClientUpdate from the beacon client and converts it to a [`CommitteeUpdateArgs`] witness
pub async fn fetch_rotation_args<S: Spec, C: ClientTypes>(
    client: &Client<C>,
) -> eyre::Result<CommitteeUpdateArgs>
where
    [(); S::SYNC_COMMITTEE_SIZE]:,
    [(); S::FINALIZED_HEADER_DEPTH]:,
    [(); S::BYTES_PER_LOGS_BLOOM]:,
    [(); S::MAX_EXTRA_DATA_BYTES]:,
    [(); S::SYNC_COMMITTEE_ROOT_INDEX]:,
    [(); S::SYNC_COMMITTEE_DEPTH]:,
    [(); S::FINALIZED_HEADER_INDEX]:,
{
    let block = get_block_header(client, BlockId::Head).await?;
    let slot = block.slot;
    let period = slot / (32 * 256);
    debug!(
        "Fetching light client update at current Slot: {} at Period: {}",
        slot, period
    );

    let update = get_light_client_update_at_period(client, period).await?;
    rotation_args_from_update(&update).await
}

/// Converts a [`LightClientUpdateCapella`] to a [`CommitteeUpdateArgs`] witness.
pub async fn rotation_args_from_update<S: Spec>(
    update: &LightClientUpdateCapella<
        { S::SYNC_COMMITTEE_SIZE },
        { S::SYNC_COMMITTEE_ROOT_INDEX },
        { S::SYNC_COMMITTEE_DEPTH },
        { S::FINALIZED_HEADER_INDEX },
        { S::FINALIZED_HEADER_DEPTH },
        { S::BYTES_PER_LOGS_BLOOM },
        { S::MAX_EXTRA_DATA_BYTES },
    >,
) -> eyre::Result<CommitteeUpdateArgs>
where
    [(); S::SYNC_COMMITTEE_SIZE]:,
    [(); S::FINALIZED_HEADER_DEPTH]:,
    [(); S::BYTES_PER_LOGS_BLOOM]:,
    [(); S::MAX_EXTRA_DATA_BYTES]:,
    [(); S::SYNC_COMMITTEE_ROOT_INDEX]:,
    [(); S::SYNC_COMMITTEE_DEPTH]:,
    [(); S::FINALIZED_HEADER_INDEX]:,
{
    let mut update = update.clone();
    let pubkeys_compressed = update
        .next_sync_committee
        .pubkeys
        .iter()
        .map(|pk| pk.to_bytes().to_vec())
        .collect_vec();
    let mut sync_committee_branch = update.next_sync_committee_branch.as_ref().to_vec();

    sync_committee_branch.insert(
        0,
        update
            .next_sync_committee
            .aggregate_pubkey
            .hash_tree_root()
            .unwrap(),
    );

    assert!(
        ssz_rs::is_valid_merkle_branch(
            update.next_sync_committee.pubkeys.hash_tree_root().unwrap(),
            &sync_committee_branch
                .iter()
                .map(|n| n.as_ref())
                .collect_vec(),
            S::SYNC_COMMITTEE_PUBKEYS_DEPTH,
            S::SYNC_COMMITTEE_PUBKEYS_ROOT_INDEX,
            update.attested_header.beacon.state_root,
        )
        .is_ok(),
        "Execution payload merkle proof verification failed"
    );

    let args = CommitteeUpdateArgs {
        pubkeys_compressed,
        finalized_header: committee_iso::types::BeaconBlockHeader {
            slot: update.finalized_header.beacon.clone().slot.to_string(),
            proposer_index: update
                .finalized_header
                .beacon
                .clone()
                .proposer_index
                .to_string(),
            parent_root: update.finalized_header.beacon.clone().parent_root,
            state_root: update.finalized_header.beacon.clone().state_root,
            body_root: update.finalized_header.beacon.clone().body_root,
        },
        sync_committee_branch: sync_committee_branch
            .into_iter()
            .map(|n| n.to_vec())
            .collect_vec(),
    };
    Ok(args)
}
