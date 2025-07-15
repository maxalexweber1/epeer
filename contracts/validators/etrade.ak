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
  // spend of utxos with a fitting price
  spend(
    _datum: Option<PoolDatum>,
    redeemer action: PoolAction,
    o_ref: OutputReference,
    tx: Transaction,
  ) {
    let Transaction { mint, inputs, outputs, .. } = tx
    // check whether own UTxO is consumed and extract the policy ID
    expect Some(Input {
      output: Output {
        address: this_address,
        value: _pool_input_value,
        datum: InlineDatum(input_datum),
        ..
      },
      output_reference: _,
    }) = list.find(inputs, fn(inp) { inp.output_reference == o_ref })

    // use full Datum record
    expect EnergyDatum {
      price,
      energy_tokens: input_lp_tokens,
      selleraddr: input_lend_out,
    } = input_datum

    when action is {
      
    }
  }

  else(_) {
    fail
  }
}
