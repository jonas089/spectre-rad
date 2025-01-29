# Fast, Modular & Secure ZK Ethereum Light Client 🧪
![cover](https://github.com/jonas089/spectre-rad/blob/master/resources/banner.png)

# Spectre Use Cases
Spectre is a trust minimized light-client for `Ethereum`. With Spectre it is possible to cryptographically verify the integrity of Ethereum `state root`.
Lightclients are essentially cryptographic infrastructure that make all queries to a blockchain verifiable and therefore enhance security.
Without lightclients the development of secure decentralized applications would be challenging since they would have to query many nodes rather than 
relying a single source of truth. Additionally there would be no way to verify the integrity of the state transitions even when querying a large number
of nodes.


> [!NOTE]
> Spectre makes blockchain queries secure 
> by proving that the state is valid through cryptography(ZK). 


## Naming convention for crates

`iso-*`: Raw Rust implementation of a Spectre component in Isolation. Example: `committee-iso` for the `CommitteeUpdateCircuit` logic.

`circuit/rz-*`: Risc0 circuit implementation of an `iso` component.

`circuit/sp1-*`: SP1 circuit implementation of an `iso` component.


`prover`: A special crate that generates proofs using either of the `circuits`. This crate will be extended to support verification on `AlignedLayer`.

# Benchmarks
Benchmarking the Step and Committee Circuits on different machines in SP1 and Risc0

## Proving Speed Benchmarks
Benchmarks were performed on SP1 v3.4.0, but we migrated to 4.x. 
Therefore more recently performed benchmarks may differ from the results listed here.

### Step Circuit
| Hardware | 1x H100 lambdalabs 80 GB sxm5 |
| --- | --- |
| wrapped (groth16) | 91.6s |
| wrapped (plonk) | 170.8s |
| generic (SP1) | 43.2s |

### Commitee Circuit
| Hardware | 1x H100 lambdalabs 80 GB sxm5 |
| --- | --- |
| wrapped (groth16) | 56.9s |
| wrapped (plonk) | 137.6s |
| generic (SP1) | 22.1 |

### Sync Committee Rotation
| Hardware | 1x GH200 lambdalabs 96 GB pcie |
| --- | --- |
| wrapped (groth16) | 239.1s |
| wrapped (plonk) | 289.5s |

> [!WARNING]
> This benchmark was performed on less powerful hardware, using SP1 v4.0.1.

# Circuit Inputs and Outputs
In ZKVMs we refer to public outputs as information committed to the `journal`. Inputs can either be committed or kept a secret.

## The Beacon Header
|  Input  | Type |
| ------------- | ------------- |
| slot  | int  |
| proposer_index  | int  |
| parent_root | Vec<u8> |
| state_root | Vec<u8> |
| body_root | Vec<u8> |

## 1. Committee-Circuit

### 1.1. Inputs
|  Input  | Type |
| ------------- | ------------- |
| Public Keys Compressed  | [u8;49] |
| Finalized Header  | BeaconBlockHeader  |
| Sync Committee Branch | Vec<Vec<u8>> |

### 1.2. Outputs
| Output | Type |
| ------------- | ------------- |
| Key Commitment  | [u8;32] |
| Finalized Block (Header) Root  | [u8;32] |

## 2. Step-Circuit
### 2.1. Inputs
| Input | Type |
| ------------- | ------------- |
| Aggregate Signature  | [u8;32] |
| Public Keys Uncompressed  | Vec<[u8;96]> |
| Participation bits  | Vec<bool> |
| Attested Header  | BeaconBlockHeader |
| Finalized Header | BeaconBlockHeader |
| Finality Branch | Vec<Vec<u8>> |
| Execution Payload Root | Vec<u8> |
| Execution Payload Branch | Vec<Vec<u8>> |
| domain | [u8;32] |
| Committee Commitment | [u8;32] |

### 2.2. Outputs
| Output | Type |
| ------------- | ------------- |
| Slot Number | u32 |
| Key Commitment  | [u8;32] |
| Finalized Block (Header) Root | [u8;32] |


## 3. Rotation Circuit
### 3.1. Inputs
```rust
pub struct RotationCircuitInputs {
    pub committee: CommitteeUpdateArgs,
    pub step: SyncStepCircuitInput,
}
```
### 3.2. Outputs
| Output | Type  |
| ------------- | ------------- |
| Slot Number | u32 |
| Commitment | bytes32 |
| Finalized Header Root | bytes32 |
| Next Commitment | bytes32 |

> [!NOTE]
> These are wrapped (Ethereum/Sol) types.

## Generate (&Verify) a proof for the Committee Circuit in Risc0
Prerequisites:

- Rust installation
- Risc0 toolchain
- Risc0 client `1.1.2`

[Install Risc0](https://dev.risczero.com/api/zkvm/install)

`rzup install cargo-risczero <version>`

Run this command:

```bash
cd prover
cargo test --bin prover test_committee_circuit_default_sp1 -- --nocapture
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
cd prover
cargo test test_step_circuit_default_sp1 -- --nocapture
```

Make sure to specify the path to `sync_step_512.json` as an environment variable when running any of the integration tests that are related to the step circuit.

Example:

`see above`

## Test Data

Test data for the circuit can be found in `data/*.json`. 
The `rotation_512.json` file is used with the committee circuit,
the `sync_step_512.json` file is used with the step circuit.

## Deployment - Theory

In order to deploy this prover in production, one would have to query one or more trusted Ethereum consensus nodes for `sync steps` and `committee updates`. 
Whenever a new `committee update` occurs, it must be applied before new `sync steps` can be proven. A solidity contract should handle the `committee updates` seperately to 
the `sync steps` and aim to expose the most recent trusted Ethereum root.

> [!NOTE]
> `Sync steps` are always verified against the most recent committee.

# Smart Contract
The verification Smart Contract is located in `spectre-verifier/src/Verifier.sol`. 
It is initialized with a `trusted committee commitment` and `finalized header`.
Note that a `committee commitment` refers to a commitment over the public keys in the active committee.

Experimental deployment (developer notes):

```rust
✅  [Success] Hash: 0xf3d1d7505ea334542cc1e0954974e42e320e0ec73f0b5de3968340205ea0acdb
Contract Address: 0xE45E2ae4C38B951e6ad0aDEd86C108Bb4b9ad16f
Block: 7549914
Paid: 0.000526296619253219 ETH (453433 gas * 1.160693243 gwei)
```

# Integrations - third party proof verification infrastructure
Work in progress ⚙️⚙️




