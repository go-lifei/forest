// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::rpc_api::data_types::RPCState;
use fvm_ipld_blockstore::Blockstore;
use jsonrpc_v2::{Data, Error as JsonRpcError};

pub(in crate::rpc) async fn db_gc<DB: Blockstore>(
    data: Data<RPCState<DB>>,
) -> Result<(), JsonRpcError> {
    let (tx, rx) = flume::bounded(1);
    data.gc_event_tx.send_async(tx).await?;
    rx.recv_async().await??;
    Ok(())
}
