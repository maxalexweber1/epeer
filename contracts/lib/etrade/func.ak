use aiken/collection/dict
use aiken/collection/list
use cardano/assets.{PolicyId, Value, tokens}
use cardano/transaction.{InlineDatum, Output}
use pool/types.{PoolDatum}

// function for extracting the mint quantity
pub fn mint_qty(mint: Value, policy_id: PolicyId) -> Int {
  expect [Pair(_, qty)] =
    mint
      |> tokens(policy_id)
      |> dict.to_pairs()
  qty
}
