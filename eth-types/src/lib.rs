// The Licensed Work is (c) 2023 ChainSafe
// Code: https://github.com/ChainSafe/Spectre
// SPDX-License-Identifier: LGPL-3.0-only

#![feature(trait_alias)]

mod spec;
pub use spec::{Mainnet, Minimal, Spec, Testnet};

pub const NUM_LIMBS: usize = 5;
pub const LIMB_BITS: usize = 104;
