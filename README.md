# Non-Halo2 implementation of Spectre for use with LLVM-compatible ZKVMs (Risc0, SP1)

## Summary of current state - bottlenecks

The step circuit written in `Risc0` is fast and efficient. Submitting proofs to `Aligned Layer` is possible.
Both the `Risc0` and `SP1` step circuits are highly inefficient and struggle with the `ECC` point arithmetic.
The precompile for `SP1` doesn't seem to resolve this issue with respect to parsing uncompressed points as `G1Affine`.
Blst is not supported by `SP1` and there for is only used in `Risc0` context, though it might be entirely removed at some point in the future.
Currently verifying the aggregate signature in `SP1` is not possible due to the `blst` restriction. 

We will most likely have to implement or find a custom verifier
using curve arithmetic that depends on the precompile. This should be sufficiently fast. The main concern for now is to parse the keys as `G1Affine` and optimize the
aggregation logic or perhaps outsource it (if possible at all).

Risc0 does not have a precompile for `bls12_381` but to my surprise it generally seems to be `faster than SP1` with respect to both hashing and `ECC` arithmetic.
Once properly optimized it could very well be that there is a way to effectively utilize the `SP1` precompile to exceed the proving speed of `Risc0`.
A `Risc0` precompile for the entirety of `bls12_381` would be extremely useful here (definitely worth reaching out to the team at Risc0).


> [!NOTE]
> The original Halo2 implementation of Spectre is located at `Chainsafe`.
> This implementation of Spectre leverages ZKVMs like Risc0 for increased performance and groth16 support.

# Spectre Use Cases
Spectre is a trustless light-client for `Ethereum`. With Spectre it is possible to cryptographically verify the integrity of Ethereum `state root`.
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

## Generate a proof for the Committee Circuit in Risc0

Prerequisites:

- Rust installation
- Risc0 toolchain
- Risc0 client `1.1.2`

[Install Risc0](https://dev.risczero.com/api/zkvm/install)

`rzup install cargo-risczero <version>`

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

## Metal Acceleration

Use the `-F metal` flag to enable `metal` acceleration on MacOS, for example:

```
cargo test test_step_circuit_risc0 --release -F metal
```

to run the accelerated `step circuit`.

## Test Data

Test data for the circuit can be found in `data/rotation_512.json`. 
It contains a committee update for Beacon with `512` public keys, a merkle branch and the resulting root.

## Command line Client interactions

> [!WARNING]
> The Client currently only supports the `CommitteeUpdate` circuit in `Risc0`.


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

[click here](https://youtu.be/fHt3cDbzV0U)

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
