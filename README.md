# Non-Halo2 implementation of Spectre for use with LLVM-compatible ZKVMs (Risc0, SP1)

> [!NOTE]
> This repository is a hackathon submission for AlignedLayer that is a proof of concept for integrating their infrastructure into Chainsafe's lightclient operations

> [!WARNING]
> Todo: Utilize precompiles to accelerate hashing

# Context for Judges
Spectre, a ZK lightclient developed and operated by Chainsafe Systems,
mainly consists of two ZK circuits:

- CommitteeUpdateCircuit
- StepCircuit

see [here](https://github.com/ChainSafe/Spectre/tree/main/lightclient-circuits%2Fsrc) the original implementation in [Halo2](https://zcash.github.io/halo2/).

For the hackathon I developed a proof of concept on how to re-implement the Spectre lightclient in Risc0 with end to end tests for proof verification on AlignedLayer.

My implementation focuses on:

- integration of AlignedLayer infrastructure
- modular design that is easily extendable
- a Risc0 implementation of Spectre's CommitteeUpdateCircuit
- real world test data

For the scope of this hackathon I chose to demonstrate how an integration with Aligned is possible for Spectre 
by implementing one of the two circuits that are necessary to run the full light client. 

Due to my modular design choices it is possible to extend this implementation to support all of Spectre's functionality and 
the StepCircuit. This is however not what I am submitting for the hackathon and it is important to clarify that this proof of concept
only has some of Spectre's functionality. The functionality that it has however is fully integrated with AlignedLayer / proofs for the CommitteeUpdateCircuit
are verifiable on Aligend.

How the circuit works:

1. The 512 public keys and expected root of the merkle tree are loaded from the test data
2. The merkle tree with the 512 keys as leafs is hashed using `sha256`
3. It is asserted / constrained that the computed merkle root matches the expected merkle root

The merkle root will be committed to the journal upon successful verification.

# Spectre Use Cases
Spectre is a trustless light-client for `Ethereum`. With Spectre it is possible to cryptographically verify the integrity of Ethereum `state root`.
Lightclients are essentially cryptographic infrastructure that make all queries to a blockchain verifiable and therefore enhance security.
Without lightclients the development of secure decentralized applications would be challenging since they would have to query many nodes rather than 
relying a single source of truth. Additionally there would be no way to verify the integrity of the state transitions even when querying a large number
of nodes.

TLDR; Spectre makes blockchain queries secure by proving that the state is valid through cryptography(ZK). 

# Overview of components
This submission consists of:

- a Risc0 implementation of the CommitteeUpdateCircuit (Chainsafe Spectre)
- a Client that makes it easy to specify paths to real world committee data, keys & more
- a Database implementation that will premanently store information of all proofs that have been verified on Aligned.
- end to end integration tests that can be run in isolation

The circuit logic is located in `committee-iso/src/`

The risc0 guest is located in `circuits/committee-circuit/`

The AlignedLayer integration is located in `prover/src/aligned.rs`

The Client logic is loated in `prover/src/client.rs`

The Database logic is located in `prover/src/aligned/storage.rs`

## Naming convention for crates

`*-iso`: Raw Rust implementation of a Spectre component in Isolation. Example: `committee-iso` for the `CommitteeUpdateCircuit` logic.

`circuit/rz-*`: Risc0 circuit implementation of an `iso` component.

`circuit/sp1-*`: SP1 circuit implementation of an `iso` component.


`prover`: A special crate that generates proofs using either of the `circuits`. This crate will be extended to support verification on `AlignedLayer`.

## Generate a proof for the Committee Circuit in Risc0

Prerequisites:

- Rust installation
- Risc0 toolchain
- Risc0 client `1.0.1`

```bash
cargo test test_risc0 -- --nocapture
```

Make sure to specify the path to `rotation_512.json` as an environment variable when running any of the integration tests.
This is not required when using the client.

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

## Test Data

Test data for the circuit can be found in `data/rotation_512.json`. 
It contains a committee update for Beacon with `512` public keys, a merkle branch and the resulting root.

## Command line Client interactions
`cargo run` output:

```js
Commands:
  prove  
  help   Print this message or the help of the given subcommand(s)
```

`prove` command arguments:

```js
  --path <PATH>
  --rpc <RPC>
  --chain-id <CHAIN_ID>
  --network <NETWORK>
  --keystore <KEYSTORE>
  --password <PASSWORD>
  --gas <GAS>
```

Full Example command:

```
cargo run prove --path data/rotation_512.json --rpc https://ethereum-holesky-rpc.publicnode.com --chain-id 17000 --network Holesky --address ec3f9f8FF528862aa99Bf4648Fa4844C3d9a50a3 --keystore aligned/keystore0 --password 1234 --gas 3000000000000
```

## Proof Generation and Submission E2E Video

<img src="https://github.com/jonas089/spectre-rad/blob/master/resources/demo.gif" alt="demo" height="540" width="960"/>

The video was significantly sped up to illustrate the insertion of multiple `proofs` in the sqlite database that is created
on the client side. Each proof that was successfully verified using the client will be inserted into the database so that the 
inclusion proof on the verification chain can later be obtained.

## Integration test to submit a Risc0 proof to AlignedLayer for verification

Prerequisite:
- Holesky funded account & keystore (see [AlignedLayer docs](https://docs.alignedlayer.com/))
- Keystore should be placed in `./aligned/`

Generate and submit the proof:

```bash
cargo test test_committee_submit_aligned
```

Example output:

```rust
successes:

---- test_risc0::test_committee_submit_aligned stdout ----
Proof submitted and verified successfully on batch ebe6ea81087c1f4063f0a1d3b632e64be6925d8903fd1acacfede0241427e459


successes:
    test_risc0::test_committee_submit_aligned

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 687.50s
```
