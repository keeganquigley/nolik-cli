use sp_keyring::AccountKeyring;
use sp_runtime::{MultiAddress, MultiSignature, generic::Era};
use parity_scale_codec::{ Compact, Encode };
use sodiumoxide::crypto::box_::PublicKey;
use sp_core::crypto::AccountId32;
use sp_core::{H256, Pair};
use sp_core::sr25519::Signature;

use crate::node::calls::{get_nonce, get_genesis_hash, get_runtime_version, StateGetRuntimeVersion};
use crate::node::errors::NodeError;
use crate::Socket;


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


async fn get_meta(owner: &AccountId32) -> Result<(u32, H256, StateGetRuntimeVersion), NodeError> {

    let mut socket = match Socket::new() {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let nonce = match get_nonce(&mut socket, &owner).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let genesis_hash = match get_genesis_hash(&mut socket).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let runtime_version = match get_runtime_version(&mut socket).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    if let Err(e) = socket.close() {
        eprintln!("Error: {:?}", e);
        return Err(NodeError::CouldNotGetMetadata);
    }

    Ok((nonce, genesis_hash, runtime_version))
}


pub async fn add_owner(
    owner: AccountId32,
    pair: &sp_core::sr25519::Pair,
    address: &PublicKey) -> Result<String, NodeError> {

    let (nonce, genesis_hash, runtime_version) = match get_meta(&owner).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let pallet_index: u8 = 8;
    let method_index: u8 = 0;
    let call_index: [u8; 2] = [pallet_index, method_index];
    let call_owner: [u8; 1] = [0];

    let address = bs58::encode(&address).into_string();

    let call: Vec<u8>  = [
        call_index.to_vec(),
        address.encode(),
        call_owner.to_vec()
    ].concat();

    let extra = (
        Era::Immortal,
        Compact(nonce),
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
        address,
        call_owner,
    );

    let payload = (
        &call_tup,
        &extra,
        &additional
    );

    let signature = payload.using_encoded(|payload| pair.sign(&payload));
    let extrinsic = encode_extrinsic(owner, signature, extra, call);
    let extrinsic_hash = format!("0x{}", hex::encode(extrinsic));
    Ok(extrinsic_hash)
}


pub async fn add_to_whitelist(
    owner: AccountId32,
    pair: &sp_core::sr25519::Pair,
    add_to: Vec<u8>,
    new_address: Vec<u8>) -> Result<String, NodeError> {

    // let owner = identity.to_account_id();
    let (nonce, genesis_hash, runtime_version) = match get_meta(&owner).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let pallet_index: u8 = 8;
    let method_index: u8 = 1;
    let call_index: [u8; 2] = [pallet_index, method_index];

    let call: Vec<u8>  = [call_index.to_vec(), add_to.encode().clone(), new_address.encode().clone()].concat();


    let extra = (
        Era::Immortal,
        Compact(nonce),
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

    let signature = payload.using_encoded(|payload| pair.sign(&payload));
    let extrinsic = encode_extrinsic(owner, signature, extra, call);
    let extrinsic_hash = format!("0x{}", hex::encode(extrinsic));
    Ok(extrinsic_hash)
}


pub async fn add_to_blacklist(identity: AccountKeyring, add_to: Vec<u8>, new_address: Vec<u8>) -> Result<String, NodeError> {
    let owner = identity.to_account_id();
    let (nonce, genesis_hash, runtime_version) = match get_meta(&owner).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let pallet_index: u8 = 8;
    let method_index: u8 = 2;
    let call_index: [u8; 2] = [pallet_index, method_index];

    let call: Vec<u8>  = [call_index.to_vec(), add_to.encode().clone(), new_address.encode().clone()].concat();


    let extra = (
        Era::Immortal,
        Compact(nonce),
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
    let extrinsic_hash = format!("0x{}", hex::encode(extrinsic));
    Ok(extrinsic_hash)
}


pub async fn send_message(
    owner: AccountId32,
    pair: &sp_core::sr25519::Pair,
    sender: &PublicKey,
    recipient: &PublicKey,
    ipfs_id: &String) -> Result<String, NodeError> {

    let (nonce, genesis_hash, runtime_version) = match get_meta(&owner).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let pallet_index: u8 = 8;
    let method_index: u8 = 3;
    let call_index: [u8; 2] = [pallet_index, method_index];

    let call: Vec<u8>  = [
        call_index.to_vec(),
        sender.as_ref().encode(),
        recipient.as_ref().encode(),
        ipfs_id.as_bytes().encode()
    ].concat();


    let extra = (
        Era::Immortal,
        Compact(nonce),
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
        sender.as_ref(),
        recipient.as_ref(),
        ipfs_id.as_bytes(),
    );

    let payload = (
        &call_tup,
        &extra,
        &additional
    );

    let signature = payload.using_encoded(|payload| pair.sign(&payload));
    let extrinsic = encode_extrinsic(owner, signature, extra, call);
    let extrinsic_hash = format!("0x{}", hex::encode(extrinsic));
    Ok(extrinsic_hash)
}
