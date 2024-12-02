## 1. Aligned Layer (awaiting support for Risc0 1.1.x, last checked 1.0.1)
> [!WARNING]
> 🚧 Under construction
> 🚧 The Aligned Layer integration and Client will be finished once the circuits reach the desired proving speeds.

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