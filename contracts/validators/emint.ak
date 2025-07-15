use aiken/collection/list
use cardano/address.{Address, Script}
use cardano/assets.{PolicyId, quantity_of}
use cardano/transaction.{
  InlineDatum, Input, Output, OutputReference, Transaction,
}
use pool/func.{calculate_lp_mint,
  calculate_withdraw, check_init_datum, mint_qty}
use pool/types.{Burn, InitPool, Mint, PoolAction, PoolDatum}

// enegery trade validator
validator tradecontract {
  // minting and burning of energy token
  mint(redeemer action: EnergyAction, policy_id: PolicyId, tx: Transaction) {
    // get transaction inputs and mint or burn quantitys 
    let Transaction { mint, inputs, outputs, .. } = tx
    let token_minted = quantity_of(mint, policy_id, "EPEER 1KWH")

    // action case
    when action is {
      // mint tradeable energy tokens
      Mint(_, lp_tokens_minted) -> and {
          lp_minted == lp_tokens_minted,
          lp_tokens_minted > 0,
        }
      // burn tokens // consume energy
      Burn(_, lp_tokens_burned) -> and {
          lp_minted == -lp_tokens_burned,
          lp_tokens_burned > 0,
        }
    }
  }

  else(_) {
    fail
  }
}
