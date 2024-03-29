use sp_keyring::AccountKeyring;
use sp_runtime::{MultiAddress, MultiSignature, generic::Era};
use parity_scale_codec::{Compact, Encode};
use sodiumoxide::crypto::box_::PublicKey;
use sp_core::crypto::AccountId32;
use sp_core::{H256, Pair};
use sp_core::sr25519::{Signature};
use subxt::{Call, ClientBuilder, DefaultConfig, PolkadotExtrinsicParams};

use crate::node::calls::{get_nonce, get_genesis_hash, get_runtime_version, StateGetRuntimeVersion};
use crate::node::errors::NodeError;
use crate::{Config, ConfigError, ConfigFile, Socket};
use crate::message::utils::hash_address;


#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}



pub trait ExtrinsicCall {
    type Call: parity_scale_codec::Codec;
}

pub struct Extrinsic<T: ExtrinsicCall> {
    pair: sp_core::sr25519::Pair,
    owner: AccountId32,
    bytes: Vec<u8>,
    call: T::Call,
    node_url: String,
}


impl<T: ExtrinsicCall> Extrinsic<T> {
    pub async fn hash<C: Call>(&self) -> Result<String, NodeError> {
        let (nonce, genesis_hash, runtime_version) = match self.meta(&self.owner).await {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let indexes = match self.indexes::<C>().await {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let call: Vec<u8>  = [
            indexes.to_vec(),
            self.bytes.clone(),
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
            indexes,
            &self.call,
        );

        let payload = (
            call_tup,
            &extra,
            &additional
        );

        let signature = payload.using_encoded(|payload| self.pair.sign(&payload));
        let extrinsic = self.encode_extrinsic(&self.owner, signature, extra, call);
        let extrinsic_hash = format!("0x{}", hex::encode(extrinsic));
        Ok(extrinsic_hash)
    }

    async fn meta(&self, owner: &AccountId32) -> Result<(u32, H256, StateGetRuntimeVersion), NodeError> {
        let mut socket = match Socket::new(&self.node_url) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let nonce = match get_nonce(&mut socket, owner).await {
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

    async fn indexes<C: Call>(&self) -> Result<[u8; 2], NodeError> {
        let api = ClientBuilder::new()
            .set_url(self.node_url.as_str())
            .build()
            .await
            .unwrap()
            .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>>();

        let locked_metadata = api.client.metadata();
        let metadata = locked_metadata.read();

        let pallet = match metadata.pallet(C::PALLET) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(NodeError::CouldNotGetCallIndex)
            },
        };

        let call = match pallet.call_index::<C>() {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(NodeError::CouldNotGetCallIndex)
            },
        };

        Ok([pallet.index(), call])
    }

    fn encode_extrinsic(
        &self,
        owner: &AccountId32,
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
}



pub struct BalancesTransfer {}

impl BalancesTransfer {
    pub fn new(config_file: &ConfigFile, sender: &AccountKeyring, recipient: &AccountId32) -> Result<Extrinsic<Self>, ConfigError> {
        let config = match Config::new(&config_file) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let dest: [u8; 1] = [0];
        let value = Compact::from(1_000_000_000u128);

        let tup = (dest, recipient.clone(), value);
        let bytes = [
            tup.0.encode(),
            tup.1.encode(),
            tup.2.encode(),
        ].concat();

        Ok(Extrinsic {
            owner: AccountId32::from(sender.clone()),
            pair: sender.pair(),
            bytes,
            call: tup,
            node_url: config.data.url,
        })
    }
}

impl ExtrinsicCall for BalancesTransfer { type Call = ([u8; 1], AccountId32, Compact<u128>); }
impl Encode for BalancesTransfer {}
impl Call for BalancesTransfer {
    const PALLET: &'static str = "Balances";
    const FUNCTION: &'static str = "transfer";
}


pub struct NolikAddOwner;

impl NolikAddOwner {
    pub fn new(config_file: &ConfigFile, pair: &sp_core::sr25519::Pair, address: &PublicKey) -> Result<Extrinsic<Self>, ConfigError> {
        let config = match Config::new(&config_file) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let owner: AccountId32 = sp_core::crypto::AccountId32::from(pair.public());
        let call_owner: [u8; 1] = [0];
        let address = hash_address(&address);

        let tup = (address, call_owner);
        let bytes = [
            tup.0.encode(),
            tup.1.to_vec(),
        ].concat();

        Ok(Extrinsic {
            owner,
            pair: pair.clone(),
            bytes,
            call: tup,
            node_url: config.data.url,
        })
    }
}

impl ExtrinsicCall for NolikAddOwner { type Call = (String, [u8; 1]); }
impl Encode for NolikAddOwner {}
impl Call for NolikAddOwner {
    const PALLET: &'static str = "Nolik";
    const FUNCTION: &'static str = "add_owner";
}


pub struct NolikAddToWhitelist;

impl NolikAddToWhitelist {
    pub fn new(config_file: &ConfigFile, pair: &sp_core::sr25519::Pair, add_to: &PublicKey, new_address: &PublicKey) -> Result<Extrinsic<Self>, ConfigError> {
        let config = match Config::new(&config_file) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let owner: AccountId32 = sp_core::crypto::AccountId32::from(pair.public());
        let add_to = hash_address(&add_to);
        let new_address = hash_address(&new_address);

        let tup = (add_to, new_address);
        let bytes = [
            tup.0.encode(),
            tup.1.encode(),
        ].concat();

        Ok(Extrinsic {
            owner,
            pair: pair.clone(),
            bytes,
            call: tup,
            node_url: config.data.url,
        })
    }
}

impl ExtrinsicCall for NolikAddToWhitelist { type Call = (String, String); }
impl Encode for NolikAddToWhitelist {}
impl Call for NolikAddToWhitelist {
    const PALLET: &'static str = "Nolik";
    const FUNCTION: &'static str = "add_to_whitelist";
}


pub struct NolikAddToBlacklist;

impl NolikAddToBlacklist {
    pub fn new(config_file: &ConfigFile, pair: &sp_core::sr25519::Pair, add_to: &PublicKey, new_address: &PublicKey) -> Result<Extrinsic<Self>, ConfigError> {
        let config = match Config::new(&config_file) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let owner: AccountId32 = sp_core::crypto::AccountId32::from(pair.public());
        let add_to = hash_address(&add_to);
        let new_address = hash_address(&new_address);

        let tup = (add_to, new_address);
        let bytes = [
            tup.0.encode(),
            tup.1.encode(),
        ].concat();

        Ok(Extrinsic {
            owner,
            pair: pair.clone(),
            bytes,
            call: tup,
            node_url: config.data.url,
        })
    }
}

impl ExtrinsicCall for NolikAddToBlacklist { type Call = (String, String); }
impl Encode for NolikAddToBlacklist {}
impl Call for NolikAddToBlacklist {
    const PALLET: &'static str = "Nolik";
    const FUNCTION: &'static str = "add_to_blacklist";
}


pub struct NolikSendMessage;

impl NolikSendMessage {
    pub fn new(config_file: &ConfigFile, pair: &sp_core::sr25519::Pair, sender: &PublicKey, recipients: &Vec<PublicKey>, ipfs_id: &String) -> Result<Extrinsic<Self>, ConfigError> {
        let config = match Config::new(&config_file) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let owner: AccountId32 = sp_core::crypto::AccountId32::from(pair.public());
        let sender = hash_address(&sender);
        let recipients: Vec<String> = recipients.iter().map(|pk| hash_address(pk)).collect();

        let tup = (sender, recipients, ipfs_id.clone());
        let bytes = [
            tup.0.encode(),
            tup.1.encode(),
            tup.2.encode(),
        ].concat();

        Ok(Extrinsic {
            owner,
            pair: pair.clone(),
            bytes,
            call: tup,
            node_url: config.data.url,
        })
    }
}

impl ExtrinsicCall for NolikSendMessage { type Call = (String, Vec<String>, String); }
impl Encode for NolikSendMessage {}
impl Call for NolikSendMessage {
    const PALLET: &'static str = "Nolik";
    const FUNCTION: &'static str = "send_message";
}

