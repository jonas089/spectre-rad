pub mod types;
pub mod utils;

/*
1. aggregate pubkeys into one
2. message hash is hash of signing root
3. bls.assert_valid_signature(signature, msghash, agg_pubkey);
4. compute finalized_header_root & verify merkle proof
5. compute finalized_block_body_root & verify merkle proof
*/
