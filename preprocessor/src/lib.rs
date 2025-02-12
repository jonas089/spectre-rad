// The Licensed Work is (c) 2023 ChainSafe
// Code: https://github.com/ChainSafe/Spectre
// SPDX-License-Identifier: LGPL-3.0-only

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod rotation;
mod step;

use beacon_api_client::{BeaconHeaderSummary, BlockId, Client, ClientTypes, Value, VersionedValue};
use committee_iso::utils::{commit_to_keys_with_sign, decode_pubkeys_x};
use eth_types::Spec;
use ethereum_consensus_types::bls::BlsSignature;
use ethereum_consensus_types::{
    BeaconBlockHeader, BlsPublicKey, ByteVector, LightClientBootstrap, LightClientFinalityUpdate,
    LightClientUpdateCapella, Root,
};

use committee_iso::types::{BeaconBlockHeader as CommitteeBeaconBlockHeader, CommitteeUpdateArgs};
use itertools::Itertools;
use step_iso::types::SyncStepArgs;

use beacon_api_client::mainnet::Client as MainnetClient;
use beacon_api_client::StateId;
use eth_types::Testnet;
use ethereum_consensus_types::signing::{compute_domain, DomainType};
use ethereum_consensus_types::ForkData;
use reqwest::Url;
pub use rotation::*;
use serde::{Deserialize, Serialize};
use ssz_rs::{Node, Vector};
pub use step::*;

pub async fn get_light_client_update_at_period<S: Spec, C: ClientTypes>(
    client: &Client<C>,
    period: u64,
) -> eyre::Result<
    LightClientUpdateCapella<
        { S::SYNC_COMMITTEE_SIZE },
        { S::SYNC_COMMITTEE_ROOT_INDEX },
        { S::SYNC_COMMITTEE_DEPTH },
        { S::FINALIZED_HEADER_INDEX },
        { S::FINALIZED_HEADER_DEPTH },
        { S::BYTES_PER_LOGS_BLOOM },
        { S::MAX_EXTRA_DATA_BYTES },
    >,
>
where
    [(); S::SYNC_COMMITTEE_SIZE]:,
    [(); S::FINALIZED_HEADER_DEPTH]:,
    [(); S::BYTES_PER_LOGS_BLOOM]:,
    [(); S::MAX_EXTRA_DATA_BYTES]:,
    [(); S::SYNC_COMMITTEE_ROOT_INDEX]:,
    [(); S::SYNC_COMMITTEE_DEPTH]:,
    [(); S::FINALIZED_HEADER_INDEX]:,
{
    let route = "eth/v1/beacon/light_client/updates";
    let mut updates: Vec<VersionedValue<_>> = client
        .http
        .get(client.endpoint.join(route)?)
        .query(&[("start_period", period), ("count", 1)])
        .send()
        .await?
        .json()
        .await?;
    assert!(updates.len() == 1, "should only get one update");
    Ok(updates.pop().unwrap().data)
}

pub async fn get_light_client_bootstrap<S: Spec, C: ClientTypes>(
    client: &Client<C>,
    block_root: Node,
) -> eyre::Result<
    LightClientBootstrap<
        { S::SYNC_COMMITTEE_SIZE },
        { S::SYNC_COMMITTEE_DEPTH },
        { S::BYTES_PER_LOGS_BLOOM },
        { S::MAX_EXTRA_DATA_BYTES },
    >,
>
where
    [(); S::SYNC_COMMITTEE_SIZE]:,
    [(); S::BYTES_PER_LOGS_BLOOM]:,
    [(); S::MAX_EXTRA_DATA_BYTES]:,
    [(); S::SYNC_COMMITTEE_DEPTH]:,
{
    let route = format!("eth/v1/beacon/light_client/bootstrap/{block_root:?}");
    let bootstrap = client.get::<VersionedValue<_>>(&route).await?.data;
    Ok(bootstrap)
}

pub async fn get_light_client_finality_update<S: Spec, C: ClientTypes>(
    client: &Client<C>,
) -> eyre::Result<
    LightClientFinalityUpdate<
        { S::SYNC_COMMITTEE_SIZE },
        { S::FINALIZED_HEADER_DEPTH },
        { S::BYTES_PER_LOGS_BLOOM },
        { S::MAX_EXTRA_DATA_BYTES },
    >,
>
where
    [(); S::SYNC_COMMITTEE_SIZE]:,
    [(); S::BYTES_PER_LOGS_BLOOM]:,
    [(); S::MAX_EXTRA_DATA_BYTES]:,
    [(); S::FINALIZED_HEADER_DEPTH]:,
{
    Ok(client
        .get::<VersionedValue<_>>("eth/v1/beacon/light_client/finality_update")
        .await?
        .data)
}

pub async fn get_block_header<C: ClientTypes>(
    client: &Client<C>,
    id: BlockId,
) -> eyre::Result<BeaconBlockHeader> {
    // TODO: Once the ethereum beacon_api_client is updated, we can avoid this struct definition
    #[derive(Serialize, Deserialize)]
    struct BeaconHeaderSummary {
        pub root: Root,
        pub canonical: bool,
        pub header: SignedBeaconBlockHeader,
    }
    #[derive(Serialize, Deserialize)]
    struct SignedBeaconBlockHeader {
        pub message: BeaconBlockHeader,
        pub signature: BlsSignature,
    }

    let route = format!("eth/v1/beacon/headers/{id}");
    let block: BeaconHeaderSummary = client.get::<Value<_>>(&route).await?.data;
    Ok(block.header.message)
}

pub async fn get_block_summary<C: ClientTypes>(
    client: &Client<C>,
    id: BlockId,
) -> eyre::Result<BeaconHeaderSummary> {
    // TODO: Once the ethereum beacon_api_client is updated, we can avoid this struct definition

    let route = format!("eth/v1/beacon/headers/{id}");
    let block: BeaconHeaderSummary = client.get::<Value<_>>(&route).await?.data;
    Ok(block)
}

pub async fn light_client_update_to_args<S: Spec>(
    update: &LightClientUpdateCapella<
        { S::SYNC_COMMITTEE_SIZE },
        { S::SYNC_COMMITTEE_ROOT_INDEX },
        { S::SYNC_COMMITTEE_DEPTH },
        { S::FINALIZED_HEADER_INDEX },
        { S::FINALIZED_HEADER_DEPTH },
        { S::BYTES_PER_LOGS_BLOOM },
        { S::MAX_EXTRA_DATA_BYTES },
    >,
    pubkeys_compressed: Vector<BlsPublicKey, { S::SYNC_COMMITTEE_SIZE }>,
    domain: [u8; 32],
) -> eyre::Result<(SyncStepArgs, CommitteeUpdateArgs)>
where
    [(); S::SYNC_COMMITTEE_SIZE]:,
    [(); S::FINALIZED_HEADER_DEPTH]:,
    [(); S::BYTES_PER_LOGS_BLOOM]:,
    [(); S::MAX_EXTRA_DATA_BYTES]:,
    [(); S::SYNC_COMMITTEE_ROOT_INDEX]:,
    [(); S::SYNC_COMMITTEE_DEPTH]:,
    [(); S::FINALIZED_HEADER_INDEX]:,
{
    let finality_update = LightClientFinalityUpdate {
        attested_header: update.attested_header.clone(),
        finalized_header: update.finalized_header.clone(),
        finality_branch: Vector::try_from(
            update
                .finality_branch
                .iter()
                .map(|v| ByteVector(Vector::try_from(v.to_vec()).unwrap()))
                .collect_vec(),
        )
        .unwrap(),
        sync_aggregate: update.sync_aggregate.clone(),
        signature_slot: update.signature_slot,
    };

    let rotation_args = rotation::rotation_args_from_update(update).await?;
    let sync_args =
        step::step_args_from_finality_update(finality_update, pubkeys_compressed, domain).await?;

    Ok((sync_args, rotation_args))
}

/// Get the most recent sync step
pub async fn get_current_sync_step() -> (SyncStepArgs, [u8; 32]) {
    let client = MainnetClient::new(Url::parse("https://lodestar-sepolia.chainsafe.io").unwrap());
    let finality_update = get_light_client_finality_update::<Testnet, _>(&client)
        .await
        .unwrap();
    let block_root = client
        .get_beacon_block_root(BlockId::Slot(finality_update.finalized_header.beacon.slot))
        .await
        .unwrap();
    let bootstrap = get_light_client_bootstrap::<Testnet, _>(&client, block_root)
        .await
        .unwrap();
    let active_committee: Vec<Vec<u8>> = bootstrap
        .current_sync_committee
        .pubkeys
        .iter()
        .map(|k| k.to_bytes().to_vec())
        .collect();
    let (keys, signs) = decode_pubkeys_x(active_committee.clone());
    let commitment = commit_to_keys_with_sign(&keys, &signs);
    let bits = finality_update
        .sync_aggregate
        .sync_committee_bits
        .to_bitvec();
    let mut participation: Vec<bool> = vec![];
    for bit in bits {
        participation.push(bit);
    }
    assert_eq!(participation.len(), 512);

    let fork_version = client
        .get_fork(StateId::Head)
        .await
        .unwrap()
        .current_version;
    let genesis_validators_root = client
        .get_genesis_details()
        .await
        .unwrap()
        .genesis_validators_root;
    let fork_data = ForkData {
        genesis_validators_root,
        fork_version,
    };
    let domain = compute_domain(DomainType::SyncCommittee, &fork_data).unwrap();

    let sync_args = step::step_args_from_finality_update(
        finality_update,
        bootstrap.current_sync_committee.pubkeys,
        domain,
    )
    .await
    .unwrap();
    (sync_args, commitment)
}

/// Gets the latest light client update
pub async fn get_light_client_update_at_slot(
    slot: u64,
) -> (
    Option<(SyncStepArgs, CommitteeUpdateArgs)>,
    Option<Vec<Vec<u8>>>,
) {
    // hardcoded for now - todo: take client or url as arg
    let client = MainnetClient::new(Url::parse("https://lodestar-sepolia.chainsafe.io").unwrap());
    let period = slot / (32 * 256);
    println!(
        "Fetching light client update at current Slot: {} at Period: {}",
        slot, period
    );
    let (sc, oc): (
        Option<(SyncStepArgs, CommitteeUpdateArgs)>,
        Option<Vec<Vec<u8>>>,
    ) = {
        let update = match get_light_client_update_at_period(&client, period).await {
            Ok(update) => Some(update),
            Err(_) => return (None, None),
        };
        let block_root = client
            .get_beacon_block_root(BlockId::Slot(slot))
            .await
            .unwrap();
        let bootstrap = get_light_client_bootstrap(&client, block_root)
            .await
            .unwrap();
        let pubkeys_compressed = bootstrap.current_sync_committee.pubkeys;
        let oc = pubkeys_compressed
            .iter()
            .map(|pk| pk.to_bytes().to_vec())
            .collect_vec();
        let fork_version = client
            .get_fork(StateId::Head)
            .await
            .unwrap()
            .current_version;
        let genesis_validators_root = client
            .get_genesis_details()
            .await
            .unwrap()
            .genesis_validators_root;
        let fork_data = ForkData {
            genesis_validators_root,
            fork_version,
        };
        match update {
            Some(_) => {}
            None => return (None, None),
        }
        let domain = compute_domain(DomainType::SyncCommittee, &fork_data).unwrap();
        (
            Some(
                light_client_update_to_args::<Testnet>(
                    &update.unwrap(),
                    pubkeys_compressed,
                    domain,
                )
                .await
                .unwrap(),
            ),
            Some(oc),
        )
    };

    let s_unwrapped = sc.clone().unwrap().0;
    let mut c_unwrapped = sc.unwrap().1;

    let mut finalized_sync_committee_branch = {
        let block_root = client
            .get_beacon_block_root(BlockId::Slot(
                u64::from_str_radix(&s_unwrapped.finalized_header.slot, 10).unwrap(),
            ))
            .await
            .unwrap();

        get_light_client_bootstrap::<Testnet, _>(&client, block_root)
            .await
            .unwrap()
            .current_sync_committee_branch
            .iter()
            .map(|n| n.to_vec())
            .collect_vec()
    };
    finalized_sync_committee_branch.insert(0, c_unwrapped.sync_committee_branch[0].clone());
    finalized_sync_committee_branch[1] = c_unwrapped.sync_committee_branch[1].clone();
    c_unwrapped.sync_committee_branch = finalized_sync_committee_branch;
    c_unwrapped.finalized_header = CommitteeBeaconBlockHeader {
        slot: s_unwrapped.finalized_header.clone().slot,
        proposer_index: s_unwrapped.finalized_header.clone().proposer_index,
        parent_root: s_unwrapped.finalized_header.clone().parent_root,
        state_root: s_unwrapped.finalized_header.clone().state_root,
        body_root: s_unwrapped.finalized_header.clone().body_root,
    };
    (Some((s_unwrapped, c_unwrapped)), oc)
}

#[tokio::test]
async fn test_query_sync_step() {}
