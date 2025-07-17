use cardano/assets.{Lovelace}

pub type EMintAction {
  EMint { kwh_amount: Int, ipfs_hash: ByteArray }
  EBurn { kwh_amount: Int, ipfs_hash: ByteArray }
}

// E Trade Datum
pub type ETradeDatum {
  // amount etoken
  amount: Int,
  // total price
  price: Lovelace,
  // address
  seller: ByteArray,
  // ifps hash
  ifps_hash: ByteArray,
}

// trade Action
pub type ETradeAction {
  EBuy
  EEdit { price: Int }
  EDelist
}
