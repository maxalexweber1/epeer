use cardano/assets.{PolicyId, quantity_of}
use cardano/transaction.{Transaction}
use etrade/types.{EBurn, EMint, EMintAction}

// energy trade validator
validator mintcontract {
  // minting and burning of energy token
  mint(redeemer action: EMintAction, policy_id: PolicyId, tx: Transaction) {
    // get transaction inputs and mint or burn quantitys 
    let Transaction { mint, .. } = tx
    let token_minted = quantity_of(mint, policy_id, "EPEER 1KWH")

    // action case
    when action is {
      // mint tradeable energy tokens
      EMint { kwh_amount, ipfs_hash } -> and {
          token_minted > 0,
          token_minted == kwh_amount * 10,
          // signer = producer address ?!
          // ifps hash not null (more verification ?!)
          // TODO check if datum in output ta is correct
          ipfs_hash != "",
        }
      // burn tokens //
      EBurn { kwh_amount, ipfs_hash } -> and {
          // needs to be negative
          token_minted < 0,
          // needs to be the right amount
          token_minted * -1 == kwh_amount * 10,
          ipfs_hash != "",
        }
    }
  }

  else(_) {
    fail
  }
}
