import requests
from status import AppStatus

# TODO put this into env
FIRE_FLY_URL = "http://localhost:5000"
MY_ADDRESS = "addr_test1vq6drteps2atsyyars65363r9navemqzdj8p2jw9pf29lrqgym9dw"
SC_ADDRESS = "addr_test1wqjrcl728gm89zet8yfdmxdzqns3mzk7cmh33fhhr44nt4ganw6kc"
BLOCKFROST_API_KEY = "previewAe2uMQNis3wntrX0BQtZ5G5i9jyFPsIu"
BASE_URL = "https://cardano-preview.blockfrost.io/api/v0"
REQUIRED_APIS = {"sell-order", "buy-token", "easy-mint"}
FIREFLY_API_LIST_URL = "http://localhost:5000/api/v1/namespaces/default/apis"

def mint_tokens(quantity: int) -> str:
    url = f"{FIRE_FLY_URL}/api/v1/namespaces/default/apis/epeer-contract-0.1.0/invoke/mint?confirm=true"
    payload = {
        "input": {
            "address": MY_ADDRESS,
            "quantity": quantity
        },
        "key": MY_ADDRESS,
        "options": {}
    }
    headers = {
        "accept": "application/json",
        "Request-Timeout": "2m0s",
        "Content-Type": "application/json"
    }

    try:
        print("Sending Mint Request...")
        print("URL:", url)
        print("Payload:", payload)

        response = requests.post(url, headers=headers, json=payload)

        if not response.ok:
            print("FireFly Error:", response.status_code)
            print("Response Body:", response.text)
            response.raise_for_status()

        data = response.json()
        tx_hash = data.get("output", {}).get("transactionHash")

        if not tx_hash:
            raise ValueError(data)
    except Exception as e:
        return f" Error: {str(e)}"
    status = AppStatus()
    status.update(token=status.stored_token + quantity)
    return tx_hash

def burn_tokens(quantity: int) -> str:
    url = f"{FIRE_FLY_URL}/api/v1/namespaces/default/apis/epeer-contract-0.1.0/invoke/burn?confirm=true"
    payload = {
        "input": {
            "address": MY_ADDRESS,
            "quantity": quantity
        },
        "key": MY_ADDRESS,
        "options": {}
    }
    headers = {
        "accept": "application/json",
        "Request-Timeout": "2m0s",
        "Content-Type": "application/json"
    }

    try:
        print("Sending Mint Request...")
        print("URL:", url)
        print("Payload:", payload)

        response = requests.post(url, headers=headers, json=payload)

        if not response.ok:
            print("FireFly Error:", response.status_code)
            print("Response Body:", response.text)
            response.raise_for_status()

        data = response.json()
        tx_hash = data.get("output", {}).get("transactionHash")

        if not tx_hash:
            raise ValueError(data)
    except Exception as e:
        return f" Error: {str(e)}"
    
    status = AppStatus()
    status.update(token=status.stored_token + quantity)
    return tx_hash

def send_sell_order(price: int, quantity: int):
    url = f"{FIRE_FLY_URL}/api/v1/namespaces/default/apis/epeer-contract-0.1.0/invoke/sell?confirm=true"
    payload = {
        "input": {
            "address": MY_ADDRESS,    
            "price": price,
            "quantity": quantity
        },
        "key": MY_ADDRESS,
        "options": {}
    }

    headers = {
        "accept": "application/json",
        "Content-Type": "application/json",
        "Request-Timeout": "2m0s"
    }
    try:
        response = requests.post(url, json=payload, headers=headers)
        print("Sending Sell Request...")
        print("URL:", url)
        print("Payload:", payload)

        response = requests.post(url, headers=headers, json=payload)

        if not response.ok:
            print("FireFly Error:", response.status_code)
            print("Response Body:", response.text)
            response.raise_for_status()

        data = response.json()
        tx_hash = data.get("output", {}).get("transactionHash")

        if not tx_hash:
            raise ValueError(data)
    except Exception as e:
        return f" Error: {str(e)}"
    status = AppStatus()
    status.update(token=status.stored_token - quantity)
    return tx_hash

def send_buy_order(amount: int):
    payload = {
        "input": {
            "address": MY_ADDRESS,
            "utxo": "0000000000",
        },
        "key": MY_ADDRESS,
        "options": {}
    }

    headers = {
        "accept": "application/json",
        "Content-Type": "application/json",
        "Request-Timeout": "2m0s"
    }

    response = requests.post(FIRE_FLY_URL, json=payload, headers=headers)
    response.raise_for_status()






