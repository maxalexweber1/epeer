import requests

# TODO put this into env
FIRE_FLY_URL = "http://localhost:5000"
MY_ADDRESS = "addr_test1vq6drteps2atsyyars65363r9navemqzdj8p2jw9pf29lrqgym9dw"
SC_ADDRESS = "addr_test1wqjrcl728gm89zet8yfdmxdzqns3mzk7cmh33fhhr44nt4ganw6kc"
BLOCKFROST_API_KEY = "previewAe2uMQNis3wntrX0BQtZ5G5i9jyFPsIu"
BASE_URL = "https://cardano-preview.blockfrost.io/api/v0"
FIREFLY_API_LIST_URL = "http://localhost:5000/api/v1/namespaces/default/apis"

def get_market_utxos():
    url = f"{BASE_URL}/addresses/{SC_ADDRESS}/utxos"
    headers = {
        "project_id": BLOCKFROST_API_KEY
    }

    response = requests.get(url, headers=headers)
    response.raise_for_status()
    utxos = response.json()

    orders = []
    for utxo in utxos:
        tx_hash = utxo["tx_hash"]
        index = utxo["output_index"]
        value = utxo["amount"]

        ada_amount = next((int(v["quantity"]) for v in value if v["unit"] == "lovelace"), 0)
        token_info = [v for v in value if v["unit"] != "lovelace"]

        for token in token_info:
            orders.append({
                "tx": tx_hash,
                "index": index,
                "lovelace": ada_amount,
                "token": token["unit"],
                "quantity": int(token["quantity"])
            })

    # sort utxos to best price
    orders.sort(key=lambda o: o["lovelace"] / o["quantity"])
    return orders

def get_e_token() -> int:
    url = f"{BASE_URL}/addresses/{MY_ADDRESS}/utxos"
    headers = {
        "project_id": BLOCKFROST_API_KEY
    }

    response = requests.get(url, headers=headers)
    response.raise_for_status()
    utxos = response.json()

    total = 0
    for utxo in utxos:
        for entry in utxo["amount"]:
            if entry["unit"] == "6720754d24e56e1a5e470f3f228260817c2b6d5c596ef5a60aa56d85452d546f6b656e":
                total += int(entry["quantity"])
    return total

def check_firefly_and_apis(timeout_sec=2):
    try:
        url = "http://localhost:5000/api/v1/namespaces/default/apis"
        response = requests.get(url, timeout=timeout_sec)
        response.raise_for_status()

        apis = response.json()
        expected = {"sell-order", "buy-token", "easy-mint"}
        found = {api["name"].split("-")[0] for api in apis}
        missing = expected - found

        if missing:
            return False, f"Missing APIs: {', '.join(missing)}"
        return True, "OK"
    except Exception as e:
        return False, str(e)


def check_cardano_connector():
    return True
