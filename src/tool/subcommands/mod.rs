// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod api_cmd;
pub mod archive_cmd;
pub mod benchmark_cmd;
pub mod car_cmd;
pub mod db_cmd;
pub mod fetch_params_cmd;
pub mod snapshot_cmd;
pub mod state_migration_cmd;

use crate::cli_shared::cli::HELP_MESSAGE;
use crate::cli_shared::cli::*;
use crate::networks::NetworkChain;
use crate::utils::version::FOREST_VERSION_STRING;
use crate::utils::{io::read_file_to_string, io::read_toml};
use clap::Parser;

/// Command-line options for the `forest-tool` binary
#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"), author = env!("CARGO_PKG_AUTHORS"), version = FOREST_VERSION_STRING.as_str(), about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(help_template(HELP_MESSAGE))]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Subcommand,
}

/// forest-tool sub-commands
#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Benchmark various Forest subsystems
    #[command(subcommand)]
    Benchmark(benchmark_cmd::BenchmarkCommands),

    /// State migration tools
    #[command(subcommand)]
    StateMigration(state_migration_cmd::StateMigrationCommands),

    /// Manage snapshots
    #[command(subcommand)]
    Snapshot(snapshot_cmd::SnapshotCommands),

    /// Download parameters for generating and verifying proofs for given size
    #[command(name = "fetch-params")]
    Fetch(fetch_params_cmd::FetchCommands),

    /// Manage archives
    #[command(subcommand)]
    Archive(archive_cmd::ArchiveCommands),

    /// Database management
    #[command(subcommand)]
    DB(db_cmd::DBCommands),

    /// Utilities for manipulating CAR files
    #[command(subcommand)]
    Car(car_cmd::CarCommands),

    /// API tooling
    #[command(subcommand)]
    Api(api_cmd::ApiCommands),
}

fn read_config(
    config_path_opt: &Option<String>,
    chain_opt: &Option<NetworkChain>,
) -> anyhow::Result<Config> {
    match find_config_path(config_path_opt) {
        Some(path) => {
            // Read from config file
            let toml = read_file_to_string(path.to_path_buf())?;
            // Parse and return the configuration file
            let mut config: Config = read_toml(&toml)?;
            if let Some(chain) = chain_opt {
                config.chain = chain.clone();
            }

            Ok(config)
        }
        None => Ok(Config::default()),
    }
}
