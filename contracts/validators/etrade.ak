use aiken/collection/list
use cardano/address.{Address, VerificationKey}
use cardano/assets
use cardano/transaction.{
  InlineDatum, Input, Output, OutputReference, Transaction,
}
use etrade/types.{EBuy, EDelist, EEdit, ETradeAction, ETradeDatum}

// Energy Market Buy Validator
validator energymarket {
  spend(
    datum: Option<ETradeDatum>,
    reedemer: ETradeAction,
    oref: OutputReference,
    tx: Transaction,
  ) {
    expect Some(d) = datum

    when reedemer is {
      // buy e-tokens
      EBuy -> {
        // e token seller gets paid the listed amount
        expect Some(payment) =
          // find output to seller address
          list.find(
            tx.outputs,
            fn(output) {
              output.address.payment_credential == VerificationKey(d.seller)
            },
          )
        // payment value is correct
        assets.lovelace_of(payment.value) == d.price
      }
      EEdit { price } -> {
        // just change the price
        expect Some(ownInput) =
          list.find(tx.inputs, fn(input) { input.output_reference == oref })

        let ownHash = ownInput.output.address.payment_credential

        // script output has same value as input
        expect Some(ownOutput) =
          list.find(
            tx.outputs,
            fn(output) {
              output.address.payment_credential == ownHash && output.value == ownInput.output.value
            },
          )

        // check output datum = new listing
        expect InlineDatum(datum) = ownOutput.datum
        expect newDatum: ETradeDatum = datum

        // check newDatum Values && signature
        and {
          newDatum.seller == d.seller,
          newDatum.price == price,
          list.has(tx.extra_signatories, d.seller),
        }
      }
      // just delist the tokens
      EDelist -> list.has(tx.extra_signatories, d.seller)?
    }
  }

  else(_) {
    fail
  }
}
