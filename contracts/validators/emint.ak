use cardano/assets.{PolicyId, quantity_of}
use cardano/transaction.{Transaction}
use etrade/types.{EBurn, EMint, EMintAction}

// energy trade validator
validator mintcontract {
  // minting and burning of energy token
  mint(redeemer action: EMintAction, policy_id: PolicyId, tx: Transaction) {
    // get transaction inputs and mint or burn quantitys 
    let Transaction { mint, .. } = tx
    let token_minted = quantity_of(mint, policy_id, "E-Token")

    // action case
    when action is {
      // mint energy tokens
      EMint { amount } -> and {
          token_minted > 0,
          token_minted == amount,
        }
      // burn energy tokens
      EBurn { amount } -> and {
          // needs to be negative
          token_minted < 0,
          // needs to be the right amount
          token_minted * -1 == amount,
        }
    }
  }

  else(_) {
    fail
  }
}
