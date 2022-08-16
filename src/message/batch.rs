use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
// use std::path::PathBuf;
// use serde_derive::{Serialize, Deserialize};
use crate::message::errors::MessageError;
use crate::message::input::MessageInput;
use crate::message::ipfs::IpfsFile;
use crate::message::message::EncryptedMessage;
use serde_derive::{Serialize, Deserialize};
use blake2::Digest;
use blake2::digest::Update;
// use crate::message::utils::{base58_to_public_key, base58_to_secret_key};

use std::io::Cursor;
// use crate::message::nonce::{EncryptedNonce, Nonce};
// use crate::message::parties::{EncryptedParties, Parties};
use crate::message::session::{EncryptedSession, SessionInput};

// #[derive(Debug)]
// pub struct BatchFile {
//     path: PathBuf,
//     dir: PathBuf,
// }
//
//
// impl BatchFile {
//     pub fn new(batch: &Batch) -> BatchFile {
//         let contents = toml::to_string(&batch).unwrap();
//         let file_base64 = base64::encode(contents.as_bytes());
//         let file_hash = sp_core::twox_128(file_base64.as_bytes());
//         let file_hash_string = format!("{}", hex::encode(file_hash));
//
//         let home_dir = dirs::home_dir().unwrap();
//         let home_path = home_dir.as_path();
//         let batch_dir = home_path.join(".nolik").join("export");
//         let file_name = format!("{}.toml", file_hash_string);
//         let batch_file = batch_dir.join(file_name);
//
//         BatchFile {
//             path: batch_file,
//             dir: batch_dir,
//         }
//     }
// }


#[derive(Debug, Serialize, Deserialize)]
pub struct Batch {
    pub nonce: String,
    pub broker: String,
    // pub sender: String,
    pub hash: String,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub sessions: Vec<EncryptedSession>,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub messages: Vec<EncryptedMessage>,
}


impl Batch {
    pub fn new(mi: &MessageInput) -> Result<Batch, MessageError> {
        let mut sessions: Vec<EncryptedSession> = Vec::new();
        let mut messages: Vec<EncryptedMessage> = Vec::new();

        let session = SessionInput::new(&mi);
        let encrypted_session = session.encrypt(&mi, &mi.sender.public);
        sessions.push(encrypted_session);

        for pk in &mi.recipients {
            let session = SessionInput::new(&mi);
            let encrypted_session = session.encrypt(&mi, pk);
            sessions.push(encrypted_session);

            let encrypted_message = mi.encrypt(&pk);
            messages.push(encrypted_message);
        }


        Ok(Batch {
            nonce: base64::encode(mi.otu.nonce.public),
            broker: base64::encode(mi.otu.broker.public),
            hash: Self::get_batch_hash(mi),
            sessions,
            messages
        })
    }

    pub fn get_batch_hash(mi: &MessageInput) -> String {
        let mut hash = blake2::Blake2s256::new();
        Update::update(&mut hash, &mi.otu.nonce.public.as_ref());
        Update::update(&mut hash, &mi.otu.nonce.secret.as_ref());
        Update::update(&mut hash, &mi.otu.broker.public.as_ref());
        Update::update(&mut hash, &mi.sender.public.as_ref());

        for pk in &mi.recipients {
            Update::update(&mut hash, pk.as_ref());
        }

        for entry in &mi.entries {
            Update::update(&mut hash, entry.key.as_ref());
            Update::update(&mut hash, entry.value.as_ref());
        }

        for file in &mi.files {
            Update::update(&mut hash, file.name.as_ref());
            Update::update(&mut hash, file.binary.as_ref());
        }

        base64::encode(hash.finalize().to_vec())
    }


    pub fn get_sender_hash(mi: &MessageInput) -> String {
        let mut sender_hasher = blake2::Blake2s256::new();
        Update::update(&mut sender_hasher, &mi.sender.public.as_ref());
        Update::update(&mut sender_hasher, &mi.otu.nonce.secret.as_ref());
        base64::encode(sender_hasher.finalize().to_vec())
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
