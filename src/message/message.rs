use std::fs;
use std::fs::OpenOptions;
use std::hash::Hash;
use std::io::{Cursor, Write};
// use async_std::io::WriteExt;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use crate::message::data::{Data, DecryptedData};
use serde_derive::{Serialize, Deserialize};
use sp_runtime::print;
use toml::to_string;
use crate::message::errors::MessageError;
use crate::message::nonce::Nonce;
use crate::message::recipient::Recipient;
use crate::message::sender::Sender;


pub enum SenderOrRecipient {
    Sender(SecretKey),
    Recipient(SecretKey),
}

pub enum MessageDirection {
    In,
    Out,
}


pub struct DecryptedMessage {
    pub nonce: box_::Nonce,
    pub direction: MessageDirection,
    pub address: PublicKey,
    pub data: Vec<DecryptedData>
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Public {
    #[serde(rename = "0")]
    pub nonce: String,

    #[serde(rename = "1")]
    pub sender: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub public: Public,
    pub nonce: Nonce,
    pub sender: Sender,
    pub recipient: Recipient,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub data: Vec<Data>
}


impl EncryptedMessage {
    pub fn decrypt(&self, sor: &SenderOrRecipient) -> Result<DecryptedMessage, MessageError> {
        let nonce = match sor {
            SenderOrRecipient::Sender(sk) => match Nonce::decrypt_nonce_for_sender(&self, &sk) {
                Ok(nonce) => nonce,
                Err(e) => return Err(e),
            },
            SenderOrRecipient::Recipient(sk) => match Nonce::decrypt_nonce_for_recipient(&self, &sk) {
                Ok(nonce) => nonce,
                Err(e) => return Err(e),
            }
        };


        let direction = match sor {
            SenderOrRecipient::Sender(_) => MessageDirection::In,
            SenderOrRecipient::Recipient(_) => MessageDirection::Out,
        };


        let address = match sor {
            SenderOrRecipient::Sender(sk) => match Recipient::decrypt(&self, &nonce, sk) {
                Ok(nonce) => nonce,
                Err(e) => return Err(e),
            },
            SenderOrRecipient::Recipient(sk) => match Sender::decrypt(&self, &nonce, &sk) {
                Ok(nonce) => nonce,
                Err(e) => return Err(e),
            }
        };


        let data = match sor {
            SenderOrRecipient::Sender(sk) | SenderOrRecipient::Recipient(sk) => match Data::decrypt(&self, &nonce, &address, &sk) {
                Ok(data) => data,
                Err(e) => return Err(e),
            }
        };


        Ok(DecryptedMessage {
            nonce,
            direction,
            address,
            data,
        })
    }

    pub async fn save(&self) -> Result<String, MessageError> {

        let contents = match toml::to_string(&self) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("DATA: {:?}", &self.data);
                eprintln!("Error: {}", e);
                return Err(MessageError::CouldNotCreateTomlFileContents);
            },
        };

        // let asd: EncryptedMessage = toml::from_str(contents.as_str()).unwrap();
        // println!("ASD {:?}", asd);
        // let asd = contents.clone();
        // let x = asd.as_bytes();

        // let file_base64 = base64::encode(contents.as_bytes());
        // let file_hash = sp_core::twox_128(file_base64.as_bytes());
        // let file_hash_string = format!("{}", hex::encode(file_hash));
        //
        // let home_dir = dirs::home_dir().unwrap();
        // let home_path = home_dir.as_path();
        // let batch_dir = home_path.join(".nolik").join("export");
        // let file_name = format!("{}.toml", file_hash_string);
        // let batch_file = batch_dir.join(file_name);
        //
        // let file_options = OpenOptions::new()
        //     .write(true)
        //     .read(true)
        //     .create(true)
        //     .open(&batch_file);

        let client = IpfsClient::default();
        // match client.bootstrap_add_default().await {
        //     Ok(res) => {
        //         println!("Boostrap nodes: {:?}", res);
        //     },
        //     Err(e) => {
        //         eprintln!("Error on pinning IPFS file: {:#?}", e);
        //         return Err(MessageError::CouldNotAddBootstrapPeers)
        //     },
        // }
        let data = Cursor::new(contents);
        match client.add(data).await {
            Ok(res) => {
                // let ipfs_record = res.last().unwrap();

                println!("Saved composed file to IPFS: {:?}", res.hash);

                match client.pin_add(&res.hash, true).await {
                    Ok(_res) => {
                        // println!("Pin result: {:?}", res);
                    },
                    Err(e) => {
                        eprintln!("Error on pinning IPFS file: {:#?}", e);
                        return Err(MessageError::CouldNotAddFileToIPFS)
                    }
                }

                // let asd = client.get(&res.hash);
                //     // .cat(res.hash.as_str())
                //     // .map_ok(|chunk| chunk.to_vec())
                //     // .try_concat();

                Ok(res.hash)
            },
            Err(e) => {
                eprintln!("Error on adding file to IPFS: {:#?}", e);
                return Err(MessageError::CouldNotAddFileToIPFS)
            }
        }

        // match file_options {
        //     Ok(mut file) => {
        //         println!("Composed a local file: {:?}", &batch_file);
        //
        //         match file.write_all(contents.as_ref()) {
        //             Ok(_) => {
        //
        //
        //             },
        //             Err(e) => {
        //                 eprintln!("Error: {}", e);
        //                 return Err(MessageError::CouldNotSaveContentsToLocalFile);
        //             }
        //         }
        //     },
        //     Err(e) => {
        //         eprintln!("Error: {}", e);
        //         return Err(MessageError::CouldNotCreateLocalFile);
        //     }
        // }
    }
}
























