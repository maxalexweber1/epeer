use std::collections::HashSet;

use balius_sdk::{
    Ack, Config, FnHandler, Params, Worker, WorkerResult,
    txbuilder::{
        AddressPattern, BuildError, FeeChangeReturn, OutputBuilder, TxBuilder, UtxoPattern,
        UtxoSource,
    },
};
use firefly_balius::{
    CoinSelectionInput, FinalizationCondition, NewMonitoredTx, SubmittedTx, WorkerExt as _,
    balius_sdk::{self, Json},
    kv,
};
use pallas_addresses::Address;
use serde::{Deserialize, Serialize};

// For each method, define a struct with all its parameters.
// Don't forget the "rename_all = camelCase" annotation.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SendAdaRequest {
    pub from_address: String,
    pub to_address: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CurrentState {
    submitted_txs: HashSet<String>,
}

/// This function builds a transaction to send ADA from one address to another.
fn send_ada(_: Config<()>, req: Params<SendAdaRequest>) -> WorkerResult<NewMonitoredTx> {
    let from_address =
        Address::from_bech32(&req.from_address).map_err(|_| BuildError::MalformedAddress)?;

    // Build an "address source" describing where the funds to transfer are coming from.
    let address_source = UtxoSource::Search(UtxoPattern {
        address: Some(AddressPattern {
            exact_address: from_address.to_vec(),
        }),
        ..UtxoPattern::default()
    });

    // In Cardano, addresses don't hold ADA or native tokens directly.
    // Instead, they control uTXOS (unspent transaction outputs),
    // and those uTXOs contain some amount of ADA and native tokens.
    // You can't spent part of a uTXO in a transaction; instead, transactions
    // include inputs with more funds than they need, and a "change" output
    // to give any excess funds back to the original sender.

    // Build a transaction with
    //  - One or more inputs containing at least `amount` ADA at the address `from_address`
    //  - One output containing exactly `amount` ADA at the address `to_address`
    //  - One output containing any change at the address `from_address`
    let tx = TxBuilder::new()
        .with_input(CoinSelectionInput(address_source.clone(), req.amount))
        .with_output(
            OutputBuilder::new()
                .address(req.to_address.clone())
                .with_value(req.amount),
        )
        .with_output(FeeChangeReturn(address_source));

    // Return that TX. The framework will sign, submit, and monitor it.
    // By returning a `NewMonitoredTx`, we tell the framework that we want it to monitor this transaction.
    // This enables the TransactionApproved, TransactionRolledBack, and TransactionFinalized events from before.
    // Note that we decide the transaction has been finalized after 4 blocks have reached the chain.
    Ok(NewMonitoredTx(
        Box::new(tx),
        FinalizationCondition::AfterBlocks(4),
    ))
}

/// This function is called when a TX produced by this contract is submitted to the blockchain, but before it has reached a block.
fn handle_submit(_: Config<()>, tx: SubmittedTx) -> WorkerResult<Ack> {
    // Keep track of which TXs have been submitted.
    let mut state: CurrentState = kv::get("current_state")?.unwrap_or_default();
    state.submitted_txs.insert(tx.hash);
    kv::set("current_state", &state)?;

    Ok(Ack)
}

fn query_current_state(_: Config<()>, _: Params<()>) -> WorkerResult<Json<CurrentState>> {
    Ok(Json(kv::get("current_state")?.unwrap_or_default()))
}

#[balius_sdk::main]
fn main() -> Worker {
    Worker::new()
        .with_request_handler("send_ada", FnHandler::from(send_ada))
        .with_request_handler("current_state", FnHandler::from(query_current_state))
        .with_tx_submitted_handler(handle_submit)
}
