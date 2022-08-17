use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
// use std::path::PathBuf;
// use serde_derive::{Serialize, Deserialize};
use crate::message::errors::MessageError;
use crate::message::input::BatchInput;
use crate::message::ipfs::IpfsFile;
use crate::message::message::{EncryptedMessage, Message};
use serde_derive::{Serialize, Deserialize};
use blake2::Digest;
use blake2::digest::Update;
// use crate::message::utils::{base58_to_public_key, base58_to_secret_key};

use std::io::Cursor;
use sodiumoxide::crypto::box_;
// use crate::message::nonce::{EncryptedNonce, Nonce};
// use crate::message::parties::{EncryptedParties, Parties};
use crate::message::session::{EncryptedSession, Session};


#[derive(Debug, Serialize, Deserialize)]
pub struct Batch {
    pub nonce: String,
    pub broker: String,
    pub hash: String,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub sessions: Vec<EncryptedSession>,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub messages: Vec<EncryptedMessage>,
}


impl Batch {
    pub fn new(bi: &BatchInput, secret_nonce: &box_::Nonce) -> Result<Batch, MessageError> {

        let public_nonce = box_::gen_nonce();
        // let secret_nonce = box_::gen_nonce();
        let broker = box_::gen_keypair();

        let mut sessions: Vec<EncryptedSession> = Vec::new();
        let mut messages: Vec<EncryptedMessage> = Vec::new();

        let session = Session::new(&bi, &secret_nonce);
        let message = Message::new(&bi, &secret_nonce);

        let encrypted_session = session.encrypt(&public_nonce, &bi.sender.public, &broker.1);
        sessions.push(encrypted_session);

        for pk in &bi.recipients {
            let encrypted_session = session.encrypt(&public_nonce, &pk, &broker.1);
            sessions.push(encrypted_session);

            let encrypted_message = message.encrypt(&pk, &bi.sender.secret);
            messages.push(encrypted_message);
        }


        Ok(Batch {
            nonce: base64::encode(&public_nonce),
            broker: base64::encode(&broker.0),
            hash: Self::get_batch_hash(&bi, &secret_nonce),
            sessions,
            messages
        })
    }

    pub fn get_batch_hash(bi: &BatchInput, nonce: &box_::Nonce) -> String {
        let mut hash = blake2::Blake2s256::new();
        Update::update(&mut hash, &nonce.as_ref());
        Update::update(&mut hash, &bi.sender.public.as_ref());

        for pk in &bi.recipients {
            Update::update(&mut hash, pk.as_ref());
        }

        for entry in &bi.entries {
            Update::update(&mut hash, entry.key.as_ref());
            Update::update(&mut hash, entry.value.as_ref());
        }

        for file in &bi.files {
            Update::update(&mut hash, file.name.as_ref());
            Update::update(&mut hash, file.binary.as_ref());
        }

        base64::encode(hash.finalize().to_vec())
    }


    pub async fn save(&self) -> Result<IpfsFile, MessageError> {
        let contents = match toml::to_string(&self) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(MessageError::CouldNotCreateTomlFileContents);
            },
        };

        let client = IpfsClient::default();
        let data = Cursor::new(contents);
        match client.add(data).await {
            Ok(res) => {
                match client.pin_add(&res.hash, true).await {
                    Ok(_res) => {
                        println!("File has been saved to the IPFS network with ID: {:?}", res.hash);
                    },
                    Err(e) => {
                        eprintln!("Error on pinning IPFS file: {:#?}", e);
                        return Err(MessageError::CouldNotAddFileToIPFS)
                    }
                }
                Ok(IpfsFile {
                    ipfs_id: res.hash
                })
            },
            Err(e) => {
                eprintln!("Error on adding file to IPFS: {:#?}", e);
                return Err(MessageError::CouldNotAddFileToIPFS)
            }
        }
    }
}
