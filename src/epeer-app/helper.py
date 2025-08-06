# policy-id for E Token
TARGET_ASSET_ID = "def68337867cb4f1f95b6b811fedbfcdd7780d10a95cc072077088ea452d544f4b454e"

def parse_utxos_summary(utxos):
    total_lovelace = 0
    token_totals = {}

    for utxo in utxos:
        for amount in utxo.get("amount", []):
            unit = amount["unit"]
            quantity = int(amount["quantity"])

            if unit == "lovelace":
                total_lovelace += quantity
            elif unit == TARGET_ASSET_ID:
                token_amount += quantity

    return {
        "total_ada": total_lovelace,
        "tokens": token_totals
    }