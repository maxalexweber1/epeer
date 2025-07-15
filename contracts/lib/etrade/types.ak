use cardano/assets.{Lovelace}

// pool actions
pub type PoolAction {
  InitPool(Lovelace, Int)
  //-> Deposit
  Mint(Lovelace, Int)
  //-> Withdraw
  Burn(Lovelace, Int)
}

// Pool Datum
pub type PoolDatum {
  // Pool balance
  balance: Lovelace,
  // amount lend out
  lend_out: Lovelace,
  // number existing lp token
  lp_tokens: Int,
  interest_accrued: Lovelace,
}
