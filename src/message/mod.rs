use std::fs;
use std::io::Write;
use std::path::PathBuf;
use crate::{Account, Config, ConfigFile, FlagKey, Input, InputError};
use crate::inputs::Flag;
use crate::message::errors::MessageError;

use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey, Seed};
use blake2::{Blake2b, Digest};
use serde_derive::{Serialize, Deserialize};

pub mod errors;


pub struct BatchFile {
    pub path: PathBuf,
    dir: PathBuf,
}

impl BatchFile {
    pub fn new(batch: &Batch) -> BatchFile {
        let contents = toml::to_string(&batch).unwrap();
        let file_base64 = base64::encode(contents.as_bytes());
        let file_hash = sp_core::twox_128(file_base64.as_bytes());
        let file_hash_string = format!("{}", hex::encode(file_hash));

        let home_dir = dirs::home_dir().unwrap();
        let home_path = home_dir.as_path();
        let batch_dir = home_path.join(".nolik").join("data");
        let file_name = format!("{}.toml", file_hash_string);
        let batch_file = batch_dir.join(file_name);

        BatchFile {
            path: batch_file,
            dir: batch_dir,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Public {
    nonce: String,
    sender: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Nonce {
    ciphertext: String,
    hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sender {
    ciphertext: String,
    hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recipient {
    ciphertext: String,
    hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    ciphertext: String,
    hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub public: Public,
    pub nonce: Nonce,
    pub sender: Sender,
    pub recipient: Recipient,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    data: Vec<Data>,
}

impl Message {
    pub fn decrypt_nonce(&self, recipient_sk: SecretKey) -> Result<box_::Nonce, MessageError> {
        match box_::open(
            base64::decode(&self.nonce.ciphertext).unwrap().as_slice(),
            &match box_::Nonce::from_slice(base64::decode(&self.public.nonce).unwrap().as_slice()) {
                Some(nonce) => nonce,
                None => return Err(MessageError::DecryptionError)
            },
           &match box_::PublicKey::from_slice(bs58::decode(&self.public.sender).into_vec().unwrap().as_slice()) {
               Some(pk) => pk,
               None => return Err(MessageError::DecryptionError),
            },
            &recipient_sk
        ) {
            Ok(result) => Ok(
                match box_::Nonce::from_slice(result.as_slice()) {
                    Some(nonce) => nonce,
                    None => return Err(MessageError::DecryptionError),
                }
            ),
            Err(_e) => return Err(MessageError::DecryptionError)
        }
    }
}

#[derive(Debug)]
pub struct MessageInput {
    pub sender: Account,
    recipients: Vec<PublicKey>,
    data: Vec<(Flag, Flag)>,
    blobs: Vec<Flag>,
    pub secret_nonce: box_::Nonce,
    public_nonce: box_::Nonce,
    public_sender: (box_::PublicKey, box_::SecretKey),
}

impl MessageInput {
    pub fn new(input: Input, config: Config) -> Result<MessageInput, InputError> {

        let sender_key = match input.get_flag_value(FlagKey::Sender) {
            Ok(key) => key,
            Err(e) => return Err(e)
        };

        let sender = match config.get_account(sender_key) {
            Some(account) => account,
            None => return Err(InputError::SenderDoesNotExist),
        };

        let recipient_keys = match input.get_flag_values(FlagKey::Recipient) {
            Ok(values) => values,
            Err(e) => return Err(e),
        };

        let mut recipient_addresses: Vec<Vec<u8>> = Vec::new();

        for key in recipient_keys {
            let address_vec = match bs58::decode(key).into_vec() {
                Ok(value) => value,
                Err(_e) => return Err(InputError::InvalidAddress)
            };

            recipient_addresses.push(address_vec);
        }

        let recipients: Vec<PublicKey> = recipient_addresses
            .iter()
            .map(|address| PublicKey::from_slice(address.as_slice()).unwrap())
            .collect();

        let mut data = Vec::new();
        let mut flags = input.flags;
        while let Some(key_flag) = flags.pop() {
            match key_flag.key {
                FlagKey::Key => {
                    let value_flag = flags.iter().next().unwrap();
                    data.push((
                        key_flag.clone(),
                        value_flag.clone()
                    ));
                },
                _ => continue
            }
        }

        Ok(MessageInput {
            sender,
            recipients,
            data,
            blobs: vec![],
            secret_nonce: box_::gen_nonce(),
            public_nonce: box_::gen_nonce(),
            public_sender: box_::gen_keypair(),
        })
    }

    pub fn sender_keys(&self ) -> (PublicKey, SecretKey) {
        let sender_pk_decoded = bs58::decode(&self.sender.public)
            .into_vec()
            .unwrap();
        let sender_pk = box_::PublicKey::from_slice(sender_pk_decoded.as_slice()).unwrap();

        let sender_sk_decoded = bs58::decode(&self.sender.secret)
            .into_vec()
            .unwrap();
        let sender_sk = box_::SecretKey::from_slice(sender_sk_decoded.as_slice()).unwrap();
        (sender_pk, sender_sk)
    }


    pub fn public_section(&self) -> Public {
        Public {
            nonce: base64::encode(self.public_nonce),
            sender: bs58::encode(self.public_sender.0).into_string(),
        }
    }

    pub fn nonce_section(&self, recipient: &PublicKey) -> Nonce {
        let mut hasher = blake2::Blake2s256::new();
        hasher.update(&self.secret_nonce);
        hasher.update(&self.secret_nonce);
        let hash = hasher.finalize().to_vec();

        Nonce {
            ciphertext: base64::encode(box_::seal(
                &self.secret_nonce.as_ref(),
                &self.public_nonce,
                &recipient,
                &self.public_sender.1)),
            hash: base64::encode(hash),
        }
    }


    pub fn sender_section(&self, recipient: &PublicKey) -> Sender {
        let mut hasher = blake2::Blake2s256::new();
        hasher.update(&self.sender_keys().0);
        hasher.update(&self.secret_nonce);
        let hash = hasher.finalize().to_vec();

        Sender {
            ciphertext: base64::encode(box_::seal(
                &self.sender_keys().0.as_ref(),
                &self.secret_nonce,
                &recipient,
                &self.public_sender.1)),
            hash: base64::encode(hash),
        }
    }

    pub fn recipient_section(&self, recipient: &PublicKey) -> Recipient {
        let mut hasher = blake2::Blake2s256::new();
        hasher.update(&recipient);
        hasher.update(&self.secret_nonce);
        let hash = hasher.finalize().to_vec();

        Recipient {
            ciphertext: base64::encode(box_::seal(
                &recipient.as_ref(),
                &self.secret_nonce,
                &self.sender_keys().0,
                &self.sender_keys().1)),
            hash: base64::encode(hash),
        }
    }


}


#[derive(Debug, Serialize, Deserialize)]
pub struct Batch {
    pub messages: Vec<Message>,
}

impl Batch {
    pub fn new(message_input: MessageInput) -> Result<Batch, MessageError> {
        let mut messages: Vec<Message> = Vec::new();
        for recipient in &message_input.recipients {
            let message = Message {
                public: message_input.public_section(),
                nonce: message_input.nonce_section(recipient),
                sender: message_input.sender_section(recipient),
                recipient: message_input.recipient_section(recipient),
                data: vec![],
            };
            messages.push(message);
        }

        Ok(Batch { messages })
    }

    pub fn save(&self, batch_file: &BatchFile) -> Result<(), MessageError> {

        if let false = batch_file.dir.exists() {
            if let Err(e) = fs::create_dir(&batch_file.dir) {
                eprintln!("Error: {}", e);
                return Err(MessageError::CouldNotCreateDataDir)
            }
        }

        match fs::File::create(&batch_file.path) {
            Ok(mut file) => {
                let contents = match toml::to_string(&self) {
                    Ok(contents) => contents,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return Err(MessageError::CouldNotSaveBatchFile);
                    },
                };

                match file.write_all(contents.as_ref()) {
                    Ok(_) => {
                        Ok(())
                    },
                    Err(e) => {
                        eprintln!("file name: {:?}", &batch_file.path);
                        eprintln!("Error: {}", e);
                        return Err(MessageError::CouldNotSaveBatchFile);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(MessageError::CouldNotSaveBatchFile);
            }
        }
    }


}