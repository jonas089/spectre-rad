# Fast, Modular & Secure ZK Ethereum Light Client 🧪

## Summary of current state - bottlenecks

> [!NOTE]
> I was able to solve a serious performance issue by using `from_uncompressed_unchecked` instead of `from_uncompressed`. 📈
> This is still secure since the pubkeys are public inputs. ⛓
> If you are a developer note that in cases where the pubkeys are private inputs this would be problematic! ⛓️‍💥

`Risc0` does not have a precompile for `bls12_381` but to my surprise it generally seems to be `faster than SP1` with respect to both hashing and `ECC` arithmetic. I was reassured that `Risc0` will receive a precompile for `bls12_381` SOON ⏱.

Benchmarks for both `Risc0` and `SP1` will be added to this README SOON ⏳!


> [!NOTE]
> The original Halo2 implementation of Spectre is located at `Chainsafe`.
> This implementation of Spectre leverages ZKVMs like Risc0 for increased performance and groth16 support.

# Spectre Use Cases
Spectre is a trust minimized light-client for `Ethereum`. With Spectre it is possible to cryptographically verify the integrity of Ethereum `state root`.
Lightclients are essentially cryptographic infrastructure that make all queries to a blockchain verifiable and therefore enhance security.
Without lightclients the development of secure decentralized applications would be challenging since they would have to query many nodes rather than 
relying a single source of truth. Additionally there would be no way to verify the integrity of the state transitions even when querying a large number
of nodes.

TLDR; Spectre makes blockchain queries secure by proving that the state is valid through cryptography(ZK). 

## Naming convention for crates

`iso-*`: Raw Rust implementation of a Spectre component in Isolation. Example: `committee-iso` for the `CommitteeUpdateCircuit` logic.

`circuit/rz-*`: Risc0 circuit implementation of an `iso` component.

`circuit/sp1-*`: SP1 circuit implementation of an `iso` component.


`prover`: A special crate that generates proofs using either of the `circuits`. This crate will be extended to support verification on `AlignedLayer`.

## Benchmarks

## Circuit Inputs and Outputs
In ZKVMs we refer to public outputs as information committed to the `journal`. Inputs can either be committed or kept a secret.

### The Beacon Header
|  Input  | Type |
| ------------- | ------------- |
| slot  | int  |
| proposer_index  | int  |
| parent_root | Vec<u8> |
| state_root | Vec<u8> |
| body_root | Vec<u8> |

### 1. CommitteUpdateCircuit
|  Input  | Type |
| ------------- | ------------- |
| Public Keys Compressed  | [u8;49] |
| Finalized Header  | BeaconBlockHeader  |
| Sync Committee Branch | Vec<Vec<u8>> |

| Output | Type |
| ------------- | ------------- |
| Finalized Block (Header) Root  | [u8;32] |
| Key Commitment  | [u8;32] |

### 2. StepCircuit
| Input | Type |
| ------------- | ------------- |
| Aggregate Signature  | [u8;32] |
| Public Keys Uncompressed  | Vec<[u8;96]> |
| Participation bits  | Vec<bool> |
| Attested Header  | BeaconBlockHeader |
| Finalized Header | BeaconBlockHeader |
| Execution Payload Root | Vec<u8> |
| Execution Payload Branch | Vec<Vec<u8>> |
| domain | [u8;32] |

| Output | Type |
| ------------- | ------------- |
| Finalized Block (Header) Root | [u8;32] |
| Key Commitment  | [u8;32] |

> [!NOTE]
> If the merkle proofs are `valid`,
> and the data was `signed` by the committee,
> and the root is `unique`,
> then the block is `trusted`.


## Generate (&Verify) a proof for the Committee Circuit in Risc0

Prerequisites:

- Rust installation
- Risc0 toolchain
- Risc0 client `1.1.2`

[Install Risc0](https://dev.risczero.com/api/zkvm/install)

`rzup install cargo-risczero <version>`

Run this command:

```bash
cargo test --bin prover test_committee_circuit_risc0 -- --nocapture
```

- `-F metal` for metal acceleration (M2, M3 Macbooks)
- `-F cuda` for cuda acceleration (NVIDIA GPUs)

> [!NOTE]
> Running the step circuit in `Risc0` only really makes sense on powerful hardware,
> currently it takes 40 minutes on my M3 Max Macbook Pro.
> `SP1` does not have metal acceleration so therefore `cuda` is recommended. 
> For `Risc0` a strong Nvidia GPU or Rig should be better than the M3.

Make sure to specify the path to `rotation_512.json` as an environment variable when running any of the integration tests that are related to the committee circuit.

Example:

```bash
export COMMITTEE_UPDATE_TEST_PATH="/Users/USERNAME/Desktop/spectre-rad/data/rotation_512.json"
```

Example output:

```rust
   Compiling host v0.1.0 (/Users/chef/Desktop/spectre-rad/circuits/rz-committee/host)
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.95s
     Running `target/debug/host`
Verified Committee Root: [25, 122, 75, 125, 192, 12, 117, 238, 92, 109, 3, 192, 224, 63, 84, 28, 196, 131, 90, 32, 180, 39, 160, 7, 188, 177, 162, 100, 181, 205, 38, 142]
```

## Generate (&Verify) a proof for the Step Circuit in Risc0

Run this command:

```bash
cargo test --bin prover test_step_circuit_risc0 -- --nocapture
```

Make sure to specify the path to `sync_step_512.json` as an environment variable when running any of the integration tests that are related to the step circuit.

Example:

`see above`



## Metal Acceleration

Use the `-F metal` flag to enable `metal` acceleration on MacOS, for example:

```bash
cargo test --bin prover test_step_circuit_risc0 --release -F metal
```

to run the accelerated `step circuit`.

## Test Data

Test data for the circuit can be found in `data/rotation_512.json`. 
It contains a committee update for Beacon with `512` public keys, a merkle branch and the resulting root.

# Integrations - third party proof verification infrastructure

Integrations can be found in `prover/integrations/*`. The first integration `POC` for the Aligned Builders Hackathon can be found in `prover/integrations/*`.
Note that this will only work for the `Risc0` version(s) supported by Aligned. I will keep an eye on them and update the integration once support for `1.1.x` has arrived.
In the meantime performance optimizations in `1.1.x` are a priority.

## Aligned Layer

Read more [here](https://github.com/jonas089/spectre-rad/tree/breaking/prover/src/integrations/aligned).
