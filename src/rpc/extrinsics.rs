use sp_keyring::AccountKeyring;
use sp_runtime::{MultiAddress, MultiSignature, generic::Era};
use parity_scale_codec::{ Compact, Encode };
use sp_core::crypto::AccountId32;
use sp_core::sr25519::Signature;

use crate::rpc::calls::{get_nonce, get_genesis_hash, get_runtime_version};


// #[allow(dead_code)]
// async fn create_storage_key() -> String {
//
//     let client = reqwest::Client::new();
//     let req = NodeRequest {
//         id: 1,
//         jsonrpc: "2.0".to_string(),
//         method: "system_syncState".to_string(),
//         params: vec![],
//     };
//
//     let res = client
//         .post("http://localhost:9933")
//         .json(&serde_json::json!(req))
//         .send()
//         .await;
//
//     match res {
//         Ok(res) => {
//             match res.status() {
//                 reqwest::StatusCode::OK => {
//                     match res.json::<SystemSyncStateSuccess>().await {
//                         Ok(_parsed) => {
//                             // println!("BLOCK {:?}", parsed.result.current_block);
//                             // self.height = Some(parsed.result.current_block);
//                             // self.connection_status = ConnectionStatus::Success;
//                         },
//                         Err(e) => {
//                             println!("ERR {:?}", e)
//                         },
//                     }
//                 },
//                 _ => {}
//             }
//         },
//         Err(_e) => {}
//     }
//
//     let module_name = "Nolik";
//     let module_name_twox = sp_core::twox_128(module_name.as_bytes());
//     let module_name_hex = hex::encode(module_name_twox.clone());
//     println!("MODULE {:?}", module_name_hex);
//
//     let method_name = "AddressOwners";
//     let method_name_twox = sp_core::twox_128(method_name.as_bytes());
//     let method_name_hex = hex::encode(method_name_twox.clone());
//     println!("METHOD {:?}", method_name_hex);
//
//     let storage_key = "4d4c14c40d1b7ecb942455794693fa68";
//     let storage_key_twox = sp_core::twox_64(storage_key.as_bytes().encode().as_ref());
//     let storage_key_hex = hex::encode(storage_key_twox.clone());
//     println!("TWOX64 {:?}", storage_key_hex);
//
//     let twox_64_concat: Vec<u8> = storage_key_twox
//         .iter()
//         .chain::<&[u8]>(storage_key.as_bytes().encode().as_ref())
//         .cloned()
//         .collect();
//
//     let twox_64_concat_hex = hex::encode(twox_64_concat.clone());
//     println!("TWOX64CONCAT {:?}", twox_64_concat_hex);
//
//     let key = format!("0x{}{}{}", module_name_hex, method_name_hex, twox_64_concat_hex);
//     key
// }


fn encode_extrinsic(
    owner: AccountId32,
    signature: Signature,
    extra: (Era, Compact<u32>, Compact<u128>),
    call: Vec<u8>) -> Vec<u8> {
    let extrinsic = {
        let mut encoded_inner = Vec::new();

        (0b1000_0000 + 4u8).encode_to(&mut encoded_inner);

        MultiAddress::Id::<_, u32>(owner.clone()).encode_to(&mut encoded_inner);
        MultiSignature::from(signature).encode_to(&mut encoded_inner);
        extra.encode_to(&mut encoded_inner);
        encoded_inner.extend(&call);

        let len = Compact(encoded_inner.len() as u32);
        let mut encoded = Vec::new();
        len.encode_to(&mut encoded);
        encoded.extend(&encoded_inner);
        encoded
    };
    extrinsic
}

pub async fn add_owner(identity: AccountKeyring, address: Vec<u8>) -> String {
    let owner = identity.to_account_id();
    let owner_nonce = get_nonce(&owner).await.unwrap();
    let runtime_version = get_runtime_version().await.unwrap();
    let genesis_hash = get_genesis_hash().await.unwrap();

    let pallet_index: u8 = 8;
    let method_index: u8 = 0;
    let call_index: [u8; 2] = [pallet_index, method_index];
    let call_owner: [u8; 1] = [0];

    let call: Vec<u8>  = [call_index.to_vec(), address.encode().clone(), call_owner.to_vec()].concat();


    let extra = (
        Era::Immortal,
        Compact(owner_nonce),
        Compact(0u128),
    );

    let additional = (
        runtime_version.spec_version,
        runtime_version.transaction_version,
        genesis_hash,
        genesis_hash,
    );

    let call_tup = (
        pallet_index,
        method_index,
        address.clone(),
        call_owner
    );

    let payload = (
        &call_tup,
        &extra,
        &additional
    );

    let signature = payload.using_encoded(|payload| identity.sign(&payload));
    let extrinsic = encode_extrinsic(owner, signature, extra, call);
    format!("0x{}", hex::encode(extrinsic))
}

pub async fn add_to_whitelist(identity: AccountKeyring, add_to: Vec<u8>, new_address: Vec<u8>) -> String {
    let owner = identity.to_account_id();
    let owner_nonce = get_nonce(&owner).await.unwrap();
    let runtime_version = get_runtime_version().await.unwrap();
    let genesis_hash = get_genesis_hash().await.unwrap();

    let pallet_index: u8 = 8;
    let method_index: u8 = 1;
    let call_index: [u8; 2] = [pallet_index, method_index];

    let call: Vec<u8>  = [call_index.to_vec(), add_to.encode().clone(), new_address.encode().clone()].concat();


    let extra = (
        Era::Immortal,
        Compact(owner_nonce),
        Compact(0u128),
    );

    let additional = (
        runtime_version.spec_version,
        runtime_version.transaction_version,
        genesis_hash,
        genesis_hash,
    );

    let call_tup = (
        pallet_index,
        method_index,
        add_to,
        new_address,
    );

    let payload = (
        &call_tup,
        &extra,
        &additional
    );

    let signature = payload.using_encoded(|payload| identity.sign(&payload));
    let extrinsic = encode_extrinsic(owner, signature, extra, call);
    format!("0x{}", hex::encode(extrinsic))
}

pub async fn add_to_blacklist(identity: AccountKeyring, add_to: Vec<u8>, new_address: Vec<u8>) -> String {
    let owner = identity.to_account_id();
    let owner_nonce = get_nonce(&owner).await.unwrap();
    let runtime_version = get_runtime_version().await.unwrap();
    let genesis_hash = get_genesis_hash().await.unwrap();

    let pallet_index: u8 = 8;
    let method_index: u8 = 2;
    let call_index: [u8; 2] = [pallet_index, method_index];

    let call: Vec<u8>  = [call_index.to_vec(), add_to.encode().clone(), new_address.encode().clone()].concat();


    let extra = (
        Era::Immortal,
        Compact(owner_nonce),
        Compact(0u128),
    );

    let additional = (
        runtime_version.spec_version,
        runtime_version.transaction_version,
        genesis_hash,
        genesis_hash,
    );

    let call_tup = (
        pallet_index,
        method_index,
        add_to,
        new_address,
    );

    let payload = (
        &call_tup,
        &extra,
        &additional
    );

    let signature = payload.using_encoded(|payload| identity.sign(&payload));
    let extrinsic = encode_extrinsic(owner, signature, extra, call);
    format!("0x{}", hex::encode(extrinsic))
}
