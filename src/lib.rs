// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

#![recursion_limit = "1024"]
#![cfg_attr(not(test), deny(clippy::todo, clippy::dbg_macro))]
#![cfg_attr(
    doc,
    deny(rustdoc::all),
    allow(
        // We build with `--document-private-items` on both docs.rs and our
        // vendored docs.
        rustdoc::private_intra_doc_links,
        // See module `doctest_private` below.
        rustdoc::private_doc_tests,
        // TODO(aatifsyed): https://github.com/ChainSafe/forest/issues/3602
        rustdoc::missing_crate_level_docs
    )
)]

cfg_if::cfg_if! {
    if #[cfg(feature = "rustalloc")] {
    } else if #[cfg(feature = "mimalloc")] {
        use crate::cli_shared::mimalloc::MiMalloc;
        #[global_allocator]
        static GLOBAL: MiMalloc = MiMalloc;
    } else if #[cfg(feature = "jemalloc")] {
        use crate::cli_shared::tikv_jemallocator::Jemalloc;
        #[global_allocator]
        static GLOBAL: Jemalloc = Jemalloc;
    }
}

pub mod auth;
pub mod beacon;
pub mod blocks;
pub mod chain;
pub mod chain_sync;
pub mod cid_collections;
pub mod cli;
pub mod cli_shared;
pub mod daemon;
pub mod db;
pub mod documentation;
pub mod fil_cns;
pub mod genesis;
pub mod interpreter;
pub mod ipld;
pub mod key_management;
pub mod libp2p;
pub mod libp2p_bitswap;
pub mod lotus_json;
pub mod message;
pub mod message_pool;
pub mod metrics;
pub mod networks;
pub mod rpc;
pub mod rpc_api;
pub mod rpc_client;
pub mod shim;
pub mod state_manager;
pub mod state_migration;
pub mod statediff;
#[cfg(test)]
mod test_utils;
pub mod tool;
pub mod utils;
pub mod wallet;

/// These items are semver-exempt, and exist for forest author use only
// We want to have doctests, but don't want our internals to be public because:
// - We don't want to be concerned with library compat
//   (We want our cargo semver to be _for the command line_).
// - We don't want to mistakenly export items which we never actually use.
//
// So we re-export the relevant items and test with `cargo test --doc --features doctest-private`
#[cfg(feature = "doctest-private")]
#[doc(hidden)]
pub mod doctest_private {
    pub use crate::{
        blocks::{BlockHeader, Ticket, TipsetKeys},
        cli::humantoken::{parse, TokenAmountPretty},
        shim::{
            address::Address, crypto::Signature, econ::TokenAmount, error::ExitCode,
            randomness::Randomness, sector::RegisteredSealProof, state_tree::ActorState,
            version::NetworkVersion,
        },
        utils::io::progress_log::WithProgress,
        utils::{encoding::blake2b_256, io::read_toml},
    };
}

/// These items are semver-exempt, and exist for forest author use only
// Allow benchmarks of forest internals
#[cfg(feature = "benchmark-private")]
#[doc(hidden)]
pub mod benchmark_private {
    pub use crate::utils::cid;
    pub use crate::utils::db::car_index;
}

// These should be made private in https://github.com/ChainSafe/forest/issues/3013
pub use auth::{verify_token, JWT_IDENTIFIER};
pub use cli::main::main as forest_main;
pub use cli_shared::cli::{Client, Config};
pub use daemon::main::main as forestd_main;
pub use key_management::{
    KeyStore, KeyStoreConfig, ENCRYPTED_KEYSTORE_NAME, FOREST_KEYSTORE_PHRASE_ENV, KEYSTORE_NAME,
};
pub use tool::main::main as forest_tool_main;
pub use wallet::main::main as forest_wallet_main;
