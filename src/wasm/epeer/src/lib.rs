use std::collections::{ HashSet,BTreeMap};
use std::str::FromStr;
use balius_sdk::wit::balius::app as worker;
use serde::{Deserialize, Serialize};
// pallas
use pallas_crypto::hash::Hash;
use pallas_addresses::{ Address, ShelleyAddress,ShelleyPaymentPart,ShelleyDelegationPart,Network};
use pallas_codec::minicbor;
use pallas_codec::utils::CborWrap;
use pallas_primitives::babbage::GenTransactionOutput;
use pallas_primitives::conway::{ Value,TransactionOutput, DatumOption};
use pallas_primitives::alonzo::Value as AlonzoValue;
use pallas_primitives::BoundedBytes;
use pallas_primitives::babbage::GenPostAlonzoTransactionOutput;

// primitive data types from pallas
use pallas_primitives::{ 
    BigInt,
    Constr, 
    Int,
    MaybeIndefArray,
    PlutusData,
    PositiveCoin,
    Bytes,
};
// tx builder 
use pallas_txbuilder::{
    BuiltTransaction,
    StagingTransaction,
    Input,
    Output,
    ExUnits,
    ScriptKind, 
    BuildConway,
};
// firefly-balius
use firefly_balius::{ 
    FinalizationCondition, 
    NewMonitoredTx, 
    SubmittedTx, 
    WorkerExt as _,balius_sdk::{self, Json},
    kv,
};
// balius sdk
use balius_sdk::{
    Ack, 
    Config, 
    FnHandler, 
    Error, 
    Params, 
    Worker, 
    WorkerResult };

// structs for contracts.json
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MintEnergyToken {
    quantity: u64,
    address: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BurnEnergyToken {
    quantity: u64,
    address: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuyEnergyToken {
     address: String,    
     utxoref: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SellEnergyToken {
    quantity: u64,
    price: u64,
    address: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CurrentState {
    minting_txs: HashSet<String>,
}


// structs for utxo selection
#[derive(Serialize, Deserialize)]
struct SelectedUtxos {
    inputs: Vec<Input>,
    collateral: Input,
    total_lovelace: u64,
    total_tokens: u64,
}

#[derive(Serialize, Deserialize)]
struct SelectedSellUtxos {
    inputs: Vec<Input>,
    total_lovelace: u64,
    total_tokens: u64,
}

#[derive(Serialize, Deserialize)]
pub struct SelectedContractUtxos {
    pub inputs: Vec<Input>,
    pub total_lovelace: u64,
    pub amount: u64,
    pub price: u64,
    pub seller: Vec<u8>,
}

// cost model plutus v3 
pub const COST_MODEL_PLUTUS_V3: &[i64] = &[
    100788, 420, 1, 1, 1000, 173, 0, 1, 1000, 59957, 4, 1, 11183, 32, 201305, 8356, 4,
    16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 16000, 100, 100, 100,
    16000, 100, 94375, 32, 132994, 32, 61462, 4, 72010, 178, 0, 1, 22151, 32, 91189,
    769, 4, 2, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 1, 1000, 42921,
    4, 2, 24548, 29498, 38, 1, 898148, 27279, 1, 51775, 558, 1, 39184, 1000, 60594, 1,
    141895, 32, 83150, 32, 15299, 32, 76049, 1, 13169, 4, 22100, 10, 28999, 74, 1,
    28999, 74, 1, 43285, 552, 1, 44749, 541, 1, 33852, 32, 68246, 32, 72362, 32, 7243,
    32, 7391, 32, 11546, 32, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1,
    90434, 519, 0, 1, 74433, 32, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0,
    1, 1, 85848, 123203, 7305, -900, 1716, 549, 57, 85848, 0, 1, 955506, 213312, 0, 2,
    270652, 22588, 4, 1457325, 64566, 4, 20467, 1, 4, 0, 141992, 32, 100788, 420, 1,
    1, 81663, 32, 59498, 32, 20142, 32, 24588, 32, 20744, 32, 25933, 32, 24623, 32,
    43053543, 10, 53384111, 14333, 10, 43574283, 26308, 10, 16000, 100, 16000, 100,
    962335, 18, 2780678, 6, 442008, 1, 52538055, 3756, 18, 267929, 18, 76433006, 8868,
    18, 52948122, 18, 1995836, 36, 3227919, 12, 901022, 1, 166917843, 4307, 36, 284546,
    36, 158221314, 26549, 36, 74698472, 36, 333849714, 1, 254006273, 72, 2174038, 72,
    2261318, 64571, 4, 207616, 8310, 4, 1293828, 28716, 63, 0, 1, 1006041, 43623, 251,
    0, 1, 100181, 726, 719, 0, 1, 100181, 726, 719, 0, 1, 100181, 726, 719, 0, 1,
    107878, 680, 0, 1, 95336, 1, 281145, 18848, 0, 1, 180194, 159, 1, 1, 158519, 8942,
    0, 1, 159378, 8813, 0, 1, 107490, 3298, 1, 106057, 655, 1, 1964219, 24520, 3
];

// excution units
pub const EX_UNITS: ExUnits = ExUnits { mem: 10_000_000, steps: 4_000_000_000 };

pub const POLICY_ID_HEX: &str = "6720754d24e56e1a5e470f3f228260817c2b6d5c596ef5a60aa56d85";

pub const ASSET_NAME: &[u8] = b"E-Token"; 
pub const MINTING_SCRIPT: &str = "5901ed01010029800aba2aba1aba0aab9faab9eaab9dab9a488888896600264653001300800198041804800cdc3a400130080024888966002600460106ea800e266446644b30013006001899192cc004c04c00a0091640406eb4c044004c034dd5004456600260060031323259800980980140122c8080dd6980880098069baa0088b2016402c264b30013005300b375400f15980099b8848000006266e1c004dd6980798061baa0078a50402915980099b8800148002266e1ccdc1000a40026eb4c03cc030dd5003c5282014402864b30013002300b375400314800226eb4c03cc030dd5000a0143259800980118059baa0018a6103d87a8000899198008009bab3010300d375400444b30010018a6103d87a8000899192cc004cdc8a4507452d546f6b656e000018acc004cdc7a44107452d546f6b656e00001899ba548000cc048c0400092f5c114c0103d87a80004039133004004301400340386eb8c038004c04400500f201432330010013756601e602060206020602060186ea8010896600200314c103d87a8000899192cc004cdc8803000c56600266e3c018006266e95200033011300f0024bd7045300103d87a80004035133004004301300340346eb8c034004c04000500e18051baa006375c601860126ea800cdc3a400516401c300800130033754011149a26cac8009";  // Hex-Bytes des Plutus V3 Scripts
pub const SPENDING_SCRIPT: &str = "5903f401010029800aba2aba1aba0aab9faab9eaab9dab9a488888896600264653001300800198041804800cdc3a400530080024888966002600460106ea800e2653001300d00198069807000cdc3a400091119912cc004c00c0062b3001300f37540150028b20208acc004c02000626464b300130150028024590121bad3013001300f375401515980099b87480100062b3001300f37540150028b20208b201a40348068566002600260186ea800a330013010300d375400523011301230120019180898091809180918091809180918091809000c88c8cc00400400c896600200314a115980099b8f375c602800200714a31330020023015001403c809260186ea8022460226024003374a90002444444464b300130090048992cc004c028c054dd5000c4cdc3992cc004c040c058dd5000c5200089bad301a3017375400280a8c9660026020602c6ea80062980103d87a8000899198008009bab301b3018375400444b30010018a6103d87a8000899192cc004cdc8a45000018acc004cdc7a44100001898041980e980d80125eb82298103d87a80004065133004004301f00340646eb8c064004c07000501a202a32330010013756600a602e6ea8c068c05cdd5001112cc004006298103d87a8000899192cc004cdc8a45000018acc004cdc7a44100001898039980e180d00125eb82298103d87a80004061133004004301e00340606eb8c060004c06c0050191bad300430163754013164050660026eb0c01cc054dd5006919baf3019301637546032602c6ea8004c00ccc060c020c058dd5004a5eb822b3001300e0048992cc004c028c054dd5000c4c8c8c966002601a60306ea8006264b30013370e9002180c9baa0018992cc004c03cc068dd5000c4c8c8ca60026eb8c0840066eb4c08400e6eb4c084009222598009812802456600266e3cdd7180998109baa007375c602660426ea80522b30013370e6eb4c03cc084dd50039bad3024302137540391330113758602460426ea8064dd7180998109baa0148a50407d14a080fa2c81106042002604000260366ea80062c80c8c074c068dd5000c590181805980c9baa301c3019375400316405c660086eb0c028c060dd500812cc004cdd7980e180c9baa301c3019375400200513375e600e60326ea8004c01cc064dd51803980c9baa0038a50405c6034602e6ea8c068c05cdd51802980b9baa001301930163754003164050660026eb0c060c054dd5006919baf3019301637540020191330053758600c602a6ea8034dd71803980a9baa008404c809888c8cc00400400c896600200314c0103d87a80008992cc004c0100062600c6603600297ae0899801801980e801202e301b00140648b2016300c37540103009375400716401c300800130033754011149a26cac8009";
pub const SPENDING_SCRIPT_HASH: &str = "243c7fca3a36728b2b3912dd99a204e11d8adec6ef18a6f71d6b35d5";
pub const FEE: u64 = 2_000_000;
pub const MIN_UTXO: u64 = 2_000_000;

type Multiasset = BTreeMap<Hash<28>, BTreeMap<Bytes, PositiveCoin>>;
type MultiassetAlonzo = BTreeMap<Hash<28>, BTreeMap<Bytes, u64>>;
pub fn collect_utxos_with_policy(
    page: &worker::ledger::UtxoPage,
    policy_id: Hash<28>,
    asset_name: Bytes,
    min_collateral_lovelace: u64,
) -> Result<SelectedUtxos, Error> {
    let mut inputs = Vec::new();
    let mut total_lovelace: u64 = 0;
    let mut total_tokens: u64 = 0;
    let mut collateral: Option<Input> = None;

    for utxo in page.utxos.iter() {
        let vec: Vec<u8> = utxo.ref_.tx_hash.clone();
        let array: [u8; 32] = vec
            .try_into()
            .map_err(|e| Error::Internal(format!("Array into Error: {:?}", e)))?;

        let tx_hash = Hash::<32>::new(array);
        let tx_index = utxo.ref_.tx_index;
        let input = Input::new(tx_hash.clone(), tx_index.into());

        let tx_output: TransactionOutput = minicbor::decode(&utxo.body)
            .map_err(|e| Error::Internal(format!("Cbor decode failed: {}", e)))?;

        let (coin, maybe_multiasset, _has_datum, _has_script): (
            u64,
            Option<MultiassetAlonzo>,
            bool,
            bool,
        ) = match tx_output {
            GenTransactionOutput::PostAlonzo(inner) => {
                let (coin, ma) = match &inner.value {
                    Value::Coin(c) => (*c, None),
                    Value::Multiasset(c, m) => {
                        let converted_map: MultiassetAlonzo = m
                            .clone()
                            .into_iter()
                            .map(|(policy_id, assets)| {
                                let converted_assets: BTreeMap<Bytes, u64> = assets
                                    .into_iter()
                                    .map(|(name, amt)| (name, amt.clone().into()))
                                    .collect();
                                (policy_id, converted_assets)
                            })
                            .collect();
                        (*c, Some(converted_map))
                    }
                };
                (
                    coin,
                    ma,
                    inner.datum_option.is_some(),
                    inner.script_ref.is_some(),
                )
            }
            GenTransactionOutput::Legacy(inner) => {
                let (coin, ma) = match &inner.amount {
                    AlonzoValue::Coin(c) => (*c, None),
                    AlonzoValue::Multiasset(c, m) => (*c, Some(m.clone())),
                };
                (coin, ma, inner.datum_hash.is_some(), false)
            }
        };

        if collateral.is_none() && maybe_multiasset.is_none() && coin >= min_collateral_lovelace {
            collateral = Some(input.clone());
            continue;
        }

        let quantity = maybe_multiasset
            .as_ref()
            .and_then(|m| m.get(&policy_id))
            .and_then(|assets| assets.get(&asset_name))
            .copied()
            .unwrap_or(0);

        if quantity > 0 || maybe_multiasset.is_none() {
            inputs.push(input.clone());
            total_lovelace += coin;
            total_tokens += quantity;
        }
    }

    let found_collateral = collateral.ok_or(Error::Internal("No collateral set".to_string()))?;

    Ok(SelectedUtxos {
        inputs,
        collateral: found_collateral,
        total_lovelace,
        total_tokens,
    })
}

pub fn pick_buy_asset_utxos( 
    address: String,
    utxoref: String
) -> Result<SelectedContractUtxos, Error> {

    let exact_address = hex::decode(&address)
        .map_err(|e| Error::Internal(format!("Hex decode failed: {}", e)))?;

    let utxoref_bytes = hex::decode(&utxoref)
    .map_err(|e| Error::Internal(format!("Invalid utxoref hex: {}", e)))?;

        if utxoref_bytes.len() != 36 {
            return Err(Error::Internal(format!("Invalid utxoref length: expected 36 bytes, got {}",utxoref_bytes.len())));
        }

    let tx_hash_array: [u8; 32] = utxoref_bytes[0..32]
        .try_into()
        .map_err(|_| Error::Internal("Failed to extract tx_hash (expected 32 bytes)".into()))?;

    let index_bytes: [u8; 4] = utxoref_bytes[32..36]
        .try_into()
        .map_err(|_| Error::Internal("Failed to extract index (expected 4 bytes)".into()))?;

    let utxoref_hash = Hash::<32>::new(tx_hash_array);
    let utxoidx = u32::from_be_bytes(index_bytes);

    let page = worker::ledger::search_utxos(
        &worker::ledger::UtxoPattern {
            address: Some(worker::ledger::AddressPattern { exact_address }),
            asset: None,
        },
        None,
        100,
    )?;

    let mut selected_inputs = vec![];
    let mut lovelace: u64 = 0;
    let mut amount = 0;
    let mut price = 0;
    let mut seller_pkh = vec![];

    for utxo in page.utxos.iter() {
        // UTxO prüfen
        let tx_hash = Hash::<32>::new(utxo.ref_.tx_hash.clone().try_into()
            .map_err(|_| Error::Internal("Invalid tx hash length".into()))?);
        let tx_index = utxo.ref_.tx_index;

        if tx_hash != Hash::<32>::new(tx_hash_array) || tx_index != utxoidx {
        }

        let tx_output: TransactionOutput = minicbor::decode(&utxo.body)
            .map_err(|_| Error::Internal("CBOR decode failed".into()))?;

        let GenTransactionOutput::PostAlonzo(inner) = tx_output else {
            return Err(Error::Internal("UTxO is not PostAlonzo".into()));
        };

        // Lovelace extrahieren
        let coin = match &inner.value {
            Value::Coin(c) => *c,
            Value::Multiasset(c, _) => *c,
        };

        lovelace = coin;

        let Some(raw_datum) = &inner.datum_option else {
            return Err(Error::Internal("No datum found".into()));
        };

        let DatumOption::Data(CborWrap(data)) = &**raw_datum else {
            return Err(Error::Internal("Datum is not inline data".into()));
        };

        let datum: PlutusData = (**data).clone();

        // get datum values
        if let Some((a, p, s)) = extract_trade_datum_fields(datum.clone()) {
            amount = a;
            price = p;
            seller_pkh = s;
        } else {
            return Err(Error::Internal("Datum field parse error".into()));
        }

        // save input
        let input = Input::new(tx_hash, tx_index.into());
        selected_inputs.push(input);
    }

    if selected_inputs.is_empty() {
        return Err(Error::Internal("Listing not found".into()));
    }

    Ok(SelectedContractUtxos {
        inputs: selected_inputs,
        total_lovelace: lovelace,
        amount: amount,
        price:  price,
        seller: seller_pkh,
    })
}

fn pick_own_buy_utxos(
    address: String,
    price: u64,
    policy_id: Hash::<28>,
    asset_name: Bytes,
) -> Result<SelectedUtxos, Error> {

    let target_lovelace = 5_000_000 + price;
    
    let exact_address = hex::decode(&address)
        .map_err(|e| Error::Internal(format!("Hex decode failed, Error: {}", e)))?;

    let page = worker::ledger::search_utxos(
        &worker::ledger::UtxoPattern {
            address: Some(worker::ledger::AddressPattern {  exact_address: exact_address, }),
            asset: None,
        },
        None,
        100,
    )?;

    let result = collect_utxos_with_policy(
        &page,
        policy_id,
        asset_name,
        target_lovelace )?;
    
    if result.total_lovelace < target_lovelace {
        return  Err(Error::Internal(format!("To less Lovelace to Buy Tokens")));
        }
    
    let collateral = result.collateral;
    
    Ok(SelectedUtxos {
        inputs: result.inputs,
        collateral: collateral,
        total_lovelace: result.total_lovelace,
        total_tokens: result.total_tokens
    })
}

// pick utxos which cover fees, minutxo and tokens to sell 
fn pick_sell_asset_utxos(
    address: String,
    target_tokens: u64,
    policy_id: Hash::<28>,
    asset_name: Bytes,
) -> Result<SelectedSellUtxos, Error> {
    
    let exact_address = hex::decode(&address)
        .map_err(|_| Error::Internal("Hex decode failed".to_string()))?;

    let page = worker::ledger::search_utxos(
        &worker::ledger::UtxoPattern {
            address: Some(worker::ledger::AddressPattern {  exact_address: exact_address, }),
            asset: None,
        },
        None,
        100,
    )?;

    let target_lovelace: u64 = 6_000_000;

   let result = collect_utxos_with_policy(
        &page,
        policy_id,
        asset_name,
        target_lovelace )?;
       
    if result.total_tokens < target_tokens {
        return Err(Error::Internal(String::from("Not enough Token")));
    }     
    
    if result.total_lovelace < target_lovelace {
        return Err(Error::Internal("Not enough Lovelace".to_string()));
    }
    
    Ok(SelectedSellUtxos {
        inputs: result.inputs,
        total_lovelace: result.total_lovelace,
        total_tokens: result.total_tokens,
    })
}

// pick a input utxo that covers fees and min utxo for assets 
fn pick_mint_input_utxos(
    address: String,
    target_value: u64,
    policy_id: Hash::<28>,
    asset_name: Bytes,
) -> Result<SelectedUtxos, Error> {

        let exact_address = hex::decode(&address)
        .map_err(|_| Error::Internal("Hex decode failed".to_string()))?;

    let page = worker::ledger::search_utxos(
        &worker::ledger::UtxoPattern {
            address: Some(worker::ledger::AddressPattern {  exact_address: exact_address, }),
            asset: None,
        },
        None,
        100,
    )?;

    let result = collect_utxos_with_policy(
        &page,
        policy_id,
        asset_name,
        5_000_000 )?;
        

    if result.total_lovelace < target_value {
        return  Err(Error::Internal(format!("Not enough free ada utxos found{:#?}", result.total_lovelace)));
        }
    
    let collateral = result.collateral;
    
    Ok(SelectedUtxos {
        inputs: result.inputs,
        collateral: collateral,
        total_lovelace: result.total_lovelace,
        total_tokens: result.total_tokens 
    })
}

// pick utxos for burning
fn pick_burn_asset_utxos(
    address: String,
    target_token: u64,
    policy_id: Hash::<28>,
    asset_name: Bytes,
) -> Result<SelectedUtxos, Error> {
    
    let target_lovelace: u64 = 4_000_000; // 4 Ada to cover burn and tx cost

    let exact_address = hex::decode(&address)
        .map_err(|_| Error::Internal("Hex decode failed".to_string()))?;

    let page = worker::ledger::search_utxos(
        &worker::ledger::UtxoPattern {
            address: Some(worker::ledger::AddressPattern {  exact_address: exact_address, }),
            asset: None,
        },
        None,
        100,
    )?;

     let result = collect_utxos_with_policy(
        &page,
        policy_id,
        asset_name,
        5_000_000 )?;
        
    
    if result.total_tokens < target_token {
        return Err(Error::Internal(format!("Not enough token: accumulated:{} target:{}",result.total_tokens, target_token)));
    }     
    
    if  result.total_lovelace < target_lovelace {
        return Err(Error::Internal("Not enough lovelace accumulated".to_string()));
    }

    let collateral = result.collateral;
    
    Ok(SelectedUtxos {
        inputs: result.inputs,
        collateral: collateral,
        total_lovelace: result.total_lovelace,
        total_tokens: result.total_tokens,
    })
}

// extract inline datum
pub fn extract_inline_datum(opt: Option<&DatumOption>) -> Option<PlutusData> {
    match opt? {
        DatumOption::Data(CborWrap(data)) => Some((**data).clone()),
        _ => None,
    }
}

// extract datum fields
pub fn extract_trade_datum_fields(datum: PlutusData) -> Option<(u64, u64, Vec<u8>)> {
    let PlutusData::Constr(Constr { fields, .. }) = datum else {
        return None;
    };

    let amount = extract_u64_from_plutusdata(fields.get(0)?)?;
    let price = extract_u64_from_plutusdata(fields.get(1)?)?;
    let seller = match fields.get(2)? {
        PlutusData::BoundedBytes(bytes) => bytes.to_vec(),
        _ => return None,
    };
    Some((amount, price, seller))
}

// extract value from plutusdata
pub fn extract_u64_from_plutusdata(pd: &PlutusData) -> Option<u64> {
    match pd {
        PlutusData::BigInt(BigInt::Int(Int(i))) => {
            match i.to_string().parse::<i128>().ok()? {
                n if n >= 0 => Some(n as u64),
                _ => None,
            }
        }
        _ => None,
    }
}

// extract utxo lovelace
fn extract_lovelace(value: &Value) -> u64 {
    match value {
        Value::Coin(c) => *c,
        Value::Multiasset(c, _) => *c,
    }
}

// extract address
pub fn extract_payment_hash(addr: Address) -> Option<Vec<u8>> {
      match addr {
        Address::Shelley(shelley_addr) => match shelley_addr.payment() {
            ShelleyPaymentPart::Key(hash) | ShelleyPaymentPart::Script(hash) => Some(hash.to_vec()),
        },
        _ => None,
    }
}

// get script address of given script hex
fn get_script_address(script_hash_hex: &str) -> Result<String, Box<dyn std::error::Error>> {

    let script_hash = Hash::<28>::from_str(&script_hash_hex)?;

    let address = ShelleyAddress::new(
    Network::Testnet,
    ShelleyPaymentPart::script_hash(script_hash),
    ShelleyDelegationPart::Null,
    );
    Ok(address.to_bech32()?)
}
        
// mint e token
fn mintetoken(_config: Config<()>, params: Params<MintEnergyToken>) -> WorkerResult<NewMonitoredTx> {
    // address 
    let address = Address::from_str(&params.address).expect("Invalid address provided");
    let addr_bytes = address.to_vec();
    let addr_hex = hex::encode(addr_bytes);

    // assetname & policy id
    let asset_name = ASSET_NAME.to_vec();
    let policy_id_bytes = hex::decode(POLICY_ID_HEX)
    .map_err(|e| Error::Internal(format!("Hex decode failed: {}", e)))?;

    let policy_id = Hash::<28>::from(policy_id_bytes.as_slice());

    // build mint redeemer (121 = 0) 
    let redeemer = PlutusData::Constr(Constr {
        tag: 121,
        any_constructor: None,
        fields: MaybeIndefArray::Def(vec![
            PlutusData::BigInt(BigInt::Int(Int::from(params.quantity as i64))),]), });
    
    // encode redemer
    let mut encoded_red: Vec<u8> = Vec::new();
    minicbor::encode(&redeemer, &mut encoded_red).expect("CBOR encoding failed");

    // calc script hash minting script
    let script_bytes: Vec<u8> = hex::decode(MINTING_SCRIPT)
        .map_err(|e| Error::Internal(format!("Script decode failed: {}", e)))?;
    
    
    // calculate inputs and output value
    let selected = pick_mint_input_utxos(
        addr_hex.clone(),
        6_000_000,
        policy_id.clone(),
        asset_name.clone().into(),
    )?;

    let output_return_lovelace = selected.total_lovelace - FEE - MIN_UTXO;

    let output_return_tokens = params.quantity + selected.total_tokens;
    // build output with minted assets + return tokens
    let output = Output::new(address.clone(), MIN_UTXO).add_asset(policy_id, asset_name.clone(), output_return_tokens as u64)
                            .map_err(|e| Error::Internal(format!("Error building output: {}", e)))?;
    
    let prepared_inputs: Vec<Input> = selected.inputs;
         
    // construct the tx 
    let mut tx = prepared_inputs.into_iter().fold(
        StagingTransaction::new(),
        |tx, input| tx.input(input),
        )
        .collateral_input(selected.collateral)
        .mint_asset(policy_id, asset_name.clone(), params.quantity as i64).map_err(|e|Error::Internal(format!("Mint Error: {}", e)))?
        .add_mint_redeemer(policy_id, encoded_red.clone(),Some(EX_UNITS))
        .script(ScriptKind::PlutusV3, script_bytes)
        .fee(FEE)
        .change_address(address.clone())
        .output(output)
        .language_view(ScriptKind::PlutusV3, COST_MODEL_PLUTUS_V3.to_vec());
        
        if output_return_lovelace > 0 {
            tx = tx.output(Output::new(address.clone(),output_return_lovelace));
        };

    // build tx
    let build: BuiltTransaction = tx
        .build_conway_raw()
        .map_err(|e| Error::Internal(format!("Build failed: {}", e)))?;
    // give it to balius sdk to submit and monitor 
    Ok(NewMonitoredTx{cbor_hex: hex::encode(&build.tx_bytes), condition: FinalizationCondition::AfterBlocks(4)})
    }

// burn e tokens
fn burnetoken(_config: Config<()>, params: Params<MintEnergyToken>) -> WorkerResult<NewMonitoredTx> {
    // address 
    let address = Address::from_str(&params.address).expect("Invalid address");
    let addr_bytes = address.to_vec();
    let addr_hex = hex::encode(addr_bytes);
    
    // assetname & policy id
    let asset_name = ASSET_NAME.to_vec();
    let policy_id_bytes = hex::decode(POLICY_ID_HEX)
        .map_err(|e| Error::Internal(format!("Hex decode failed: {}", e)))?;
    let policy_id = Hash::<28>::from(policy_id_bytes.as_slice());

   // build mint redeemer (122 = 2 -> Burn)
    let redeemer = PlutusData::Constr(Constr {
        tag: 122,
        any_constructor: None,
        fields: MaybeIndefArray::Def(vec![
            PlutusData::BigInt(BigInt::Int(Int::from(params.quantity as i64))),]),});
    
    // encode constr to cbor
    let mut encoded: Vec<u8> = Vec::new();
    minicbor::encode(&redeemer, &mut encoded).expect("CBOR encoding failed");
    
    // calc script hash & script address 
    let script_bytes: Vec<u8> = hex::decode(MINTING_SCRIPT).map_err(|e| Error::Internal(format!("Script decode failed: {}", e)))?;

    let script_add = get_script_address(POLICY_ID_HEX).map_err(|e| Error::Internal(format!("Get burn address fail: {}",e)))?;
    let script_address = Address::from_str(&script_add).map_err(|e| Error::Internal(format!("Script-address to address fail: {}", e)))?;
    
    // calc inputs and output value
    let selected = pick_burn_asset_utxos(
    addr_hex.clone(),
    params.quantity, 
    policy_id,
    asset_name.clone().into(),
    )?;
       
    let prepared_inputs: Vec<Input> = selected.inputs;
    let burn_amount: i64 = -(params.quantity as i64);

    let output_lovelace = selected.total_lovelace - FEE;
    let output_token =  selected.total_tokens - params.quantity;
    
        // construct the tx 
        let mut  tx = prepared_inputs.into_iter().fold(
        StagingTransaction::new(),
        |tx, input| tx.input(input),
        )
        .collateral_input(selected.collateral)
        .mint_asset(policy_id, asset_name.clone(), burn_amount).map_err(|e| Error::Internal(format!("Add Assets fail: {}",e)))?
        .add_mint_redeemer(policy_id, encoded.clone(),Some(EX_UNITS))
        .script(ScriptKind::PlutusV3, script_bytes)
        .fee(FEE)
        .change_address(address.clone())
        .language_view(ScriptKind::PlutusV3, COST_MODEL_PLUTUS_V3.to_vec());

            if output_token == 0 {
           tx = tx.output(Output::new(address.clone(), output_lovelace));
            } else {
           tx =  tx.output(
                    Output::new(script_address.clone(), MIN_UTXO)
                        .add_asset(policy_id, asset_name.clone().into(), output_token)
                        .map_err(|e| Error::Internal(format!("Add Assets fail: {}",e)))?,
                    );
                let change_lovelace = output_lovelace - MIN_UTXO;
           tx = tx.output(Output::new(script_address.clone(), change_lovelace)
                    );
            };
    // build tx
    let build: BuiltTransaction = tx
        .build_conway_raw()
        .map_err(|e| Error::Internal(format!("TX Build failed: {}", e)))?;
    // give it to balius sdk to submit and monitor 
    Ok(NewMonitoredTx{cbor_hex: hex::encode(&build.tx_bytes), condition: FinalizationCondition::AfterBlocks(4)})
    }

// build sell order for e token
fn selletoken(_config: Config<()>, params: Params<SellEnergyToken>) -> WorkerResult<NewMonitoredTx> {

    // address
    let address = Address::from_str(&params.address)
        .map_err(|e| Error::Internal(format!("Address conversion fail: {}",e)))?;
    let addr_bytes = address.to_vec();
    let addr_hex = hex::encode(addr_bytes);
    
    // assetname e-token
    let asset_name = ASSET_NAME.to_vec();
    //  policy_id
    let policy_id_bytes = hex::decode(POLICY_ID_HEX)
        .map_err(|e| Error::Internal(format!("Hex decode failed: {}", e)))?;
    let policy_id = Hash::<28>::from(policy_id_bytes.as_slice());

    // calc script adress
    let script_add = get_script_address(SPENDING_SCRIPT_HASH)
        .map_err(|e| balius_sdk::Error::Internal(format!("Get Script-Address fail: {}", e)))?;
    let script_address = Address::from_str(&script_add)
        .map_err(|e| balius_sdk::Error::Internal(format!("Script-Address to Address fail: {}", e)))?;
        
    // build datum
    let datum = PlutusData::Constr(Constr {
        tag: 121,
        any_constructor: None,
        fields: MaybeIndefArray::Def(vec![
            PlutusData::BigInt(BigInt::Int(Int::from(params.quantity as i64))), // token quantity
            PlutusData::BigInt(BigInt::Int(Int::from(params.price as i64))),  // token price
            PlutusData::BoundedBytes(BoundedBytes::from(
            extract_payment_hash(address.clone()).expect("Invalid address")))
        ]),
    });

    let mut encoded: Vec<u8> = Vec::new();
    minicbor::encode(&datum, &mut encoded).expect("CBOR encode failed");
    
    let output = Output::new(script_address.clone(), MIN_UTXO)
    .add_asset(policy_id, asset_name.clone().into(), params.quantity).map_err(|e|Error::Internal(format!("Add Asset fail:{}", e)))?
    .set_inline_datum(encoded);

    // calc inputs and output value
    let selected = pick_sell_asset_utxos(
    addr_hex,
    params.quantity, 
    policy_id,
    asset_name.clone().into(),
    )?;

    let prepared_inputs: Vec<Input> = selected.inputs;
    
        // construct the tx 
        let mut tx = prepared_inputs.clone().into_iter().fold(
        StagingTransaction::new(),
        |tx, input| tx.input(input),
        )
        .fee(FEE)
        .output(output.clone());

        let output_value_lovelace = selected.total_lovelace - FEE - MIN_UTXO;

        let output_coins = selected.total_tokens - params.quantity;

        if output_value_lovelace > 0 || output_coins > 0 {
                let mut output2 = Output::new(address.clone(), output_value_lovelace);
                if output_coins > 0 {
                  output2 = output2.clone().add_asset(policy_id, asset_name.clone().into(), output_coins).map_err(|_| Error::Internal(format!("add asset fail")))?;
                } 
            tx = tx.output(output2)
        };

    // build tx
    let build: BuiltTransaction = tx
        .build_conway_raw()
        .map_err(|e| balius_sdk::Error::Internal(format!("build_conway_raw failed: error={} encoded={:?}",e,datum.clone())))?;

    // give it to balius sdk to submit and monitor 
    Ok(NewMonitoredTx{cbor_hex: hex::encode(&build.tx_bytes), condition: FinalizationCondition::AfterBlocks(4)})
    }


// buy e token
fn buyetoken(_config: Config<()>, params: Params<BuyEnergyToken>) -> WorkerResult<NewMonitoredTx> {

    // address
    let address = Address::from_str(&params.address).expect("Invalid address");
    let addr_bytes = address.to_vec();
    let addr_hex = hex::encode(addr_bytes);

    // assetname & Polcy Id
    let asset_name = ASSET_NAME.to_vec();
    let policy_id_bytes = hex::decode(POLICY_ID_HEX)
        .map_err(|e| Error::Internal(format!("Hex decode failed: {}", e)))?;
     let policy_id = Hash::<28>::from(policy_id_bytes.as_slice());

   // build redeemer ( 121 -> 0 -> Buy)
    let redeemer = PlutusData::Constr(Constr {
        tag: 121,
        any_constructor: None,
        fields: MaybeIndefArray::Def(vec![]), });
    
    // encode to cbor
    let mut encoded: Vec<u8> = Vec::new();
    minicbor::encode(&redeemer, &mut encoded).expect("CBOR encoding failed");
    
    // calc script hash for validator script
    let script_bytes: Vec<u8> = hex::decode(SPENDING_SCRIPT).map_err(|_| Error::Internal(format!("decode failed")))?;
    let script_add = get_script_address(SPENDING_SCRIPT_HASH).map_err(|_| Error::Internal(format!("sc adress fail")))?;
    let sc_address = Address::from_str(&script_add).expect("Invalid address provided");
    let sc_addr_bytes = sc_address.to_vec();
    let sc_addr_hex = hex::encode(sc_addr_bytes);

   let buy_utxo = pick_buy_asset_utxos(
    sc_addr_hex.clone(),
    params.utxoref.clone())
    .map_err(|e| Error::Internal(format!(" error={} encoded={:?}",e,sc_addr_hex.clone())))?;
    
    let arr: [u8; 28] = buy_utxo.seller.clone().try_into().expect("Invalid pubkey hash length");
    let hash = Hash::<28>::new(arr);

    // calc seller address
    let seller_addr = ShelleyAddress::new(
        Network::Testnet,                     
        ShelleyPaymentPart::Key(hash),
        ShelleyDelegationPart::Null,                                 
    );

    let seller_address: Address = Address::Shelley(seller_addr.clone());
    
    let min_lovelace = buy_utxo.price + 4_000_000;
    let own_inputs = pick_own_buy_utxos(addr_hex,min_lovelace, policy_id,
    asset_name.clone().into(),)?;
    
    // prepere in an outputs
    let prepared_sc_inputs: Vec<Input> = buy_utxo.inputs;
    let prepared_own_inputs: Vec<Input> = own_inputs.inputs;
    
    // total value to outputs
    let output_value_total = buy_utxo.total_lovelace + own_inputs.total_lovelace - FEE - MIN_UTXO -  buy_utxo.price;
    
    let return_tokens = own_inputs.total_tokens;
    // build output with bought token and min ada
    let output_own = Output::new(address.clone(), MIN_UTXO)
        .add_asset(policy_id,asset_name.clone(), return_tokens as u64)
        .map_err(|_| Error::Internal(format!("output error")))?;

    // build change for second output
    let output_change = Output::new(address.clone(), output_value_total);

    // out put that gos to the seller                      
    let output_seller = Output::new(address.clone(), buy_utxo.price);

    // construct the tx 
        let tx = prepared_own_inputs.into_iter().fold(
        StagingTransaction::new(),
        |tx, input| tx.input(input),
        )
        .input(prepared_sc_inputs[0].clone())
        .collateral_input(own_inputs.collateral)
        .add_spend_redeemer(prepared_sc_inputs[0].clone(),encoded.clone(),Some(EX_UNITS))
        .script(ScriptKind::PlutusV3, script_bytes)
        .fee(FEE)
        .output(output_own)
        .output(output_seller)
        .output(output_change)
        .language_view(ScriptKind::PlutusV3, COST_MODEL_PLUTUS_V3.to_vec());
    // build tx
    let build: BuiltTransaction = tx
        .build_conway_raw()
        .map_err(|e| Error::Internal(format!("Build failed: {}", e)))?;
    // give it to balius sdk to submit and monitor 
    Ok(NewMonitoredTx{cbor_hex: hex::encode(&build.tx_bytes), condition: FinalizationCondition::AfterBlocks(4)})
}

// handle kv storage for submitted txs
fn handle_submit(_: Config<()>, tx: SubmittedTx) -> WorkerResult<Ack> {
    // Keep track of which TXs have been submitted.
    let mut state: CurrentState = kv::get("currentstate")?.unwrap_or_default();
    state.minting_txs.insert(tx.hash);
    kv::set("currentstate", &state)?;
    Ok(Ack)
}

// query the current state of kv storage
fn query_current_state(_: Config<()>, _: Params<()>) -> WorkerResult<Json<CurrentState>> {
    Ok(Json(kv::get("currentstate")?.unwrap_or_default()))
}

// balius worker
#[balius_sdk::main]
fn main() -> Worker {
Worker::new()
.with_request_handler("mint", FnHandler::from(mintetoken))
.with_request_handler("burn", FnHandler::from(burnetoken))
.with_request_handler("sell", FnHandler::from(selletoken))
.with_request_handler("buy", FnHandler::from(buyetoken))
.with_request_handler("currentstate", FnHandler::from(query_current_state))
.with_tx_submitted_handler(handle_submit)
}