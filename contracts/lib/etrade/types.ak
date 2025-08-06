use cardano/assets.{Lovelace}

pub type EMintAction {
  EMint { amount: Int }
  EBurn { amount: Int }
}

// E Trade Datum
pub type ETradeDatum {
  // amount etoken
  amount: Int,
  // total price
  price: Lovelace,
  // address phk hash
  seller: ByteArray,
}

// trade Action
pub type ETradeAction {
  EBuy
  EEdit { price: Int }
  EDelist
}
