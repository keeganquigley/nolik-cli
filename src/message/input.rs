use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::{Account, ConfigFile, Input};
use crate::cli::errors::InputError;
use crate::cli::input::FlagKey;
use crate::message::blob::{Blob, EncryptedBlob};
use crate::message::entry::{Entry, EncryptedEntry};
use crate::message::errors::MessageError;
use crate::message::message::EncryptedMessage;
use crate::message::party::{EncryptedParty, Party};
use crate::message::utils::base58_to_public_key;
use blake2::Digest;

#[derive(Debug)]
pub struct OneTimeUseNonce {
    pub public: box_::Nonce,
    pub secret: box_::Nonce,
}


impl OneTimeUseNonce {
    fn new() -> OneTimeUseNonce {
        OneTimeUseNonce {
            public: box_::gen_nonce(),
            secret: box_::gen_nonce(),
        }
    }
}


#[derive(Debug)]
pub struct OneTimeUseBroker {
    pub public: PublicKey,
    pub secret: SecretKey,
}


impl OneTimeUseBroker {
    fn new() -> OneTimeUseBroker {
        let broker = box_::gen_keypair();
        OneTimeUseBroker {
            public: broker.0,
            secret: broker.1,
        }
    }
}


#[derive(Debug)]
pub struct OneTimeUse {
    pub nonce: OneTimeUseNonce,
    pub broker: OneTimeUseBroker,
}


impl OneTimeUse {
    fn new() -> OneTimeUse {
        OneTimeUse {
            nonce: OneTimeUseNonce::new(),
            broker: OneTimeUseBroker::new(),
        }
    }
}


#[derive(Debug)]
pub struct MessageInput {
    pub sender: Account,
    pub recipients: Vec<PublicKey>,
    pub entries: Vec<Entry>,
    pub blobs: Vec<Blob>,
    pub otu: OneTimeUse,
}


impl MessageInput {
    pub fn new(input: &mut Input, config_file: ConfigFile) -> Result<MessageInput, InputError> {
        let sender_key = match input.get_flag_value(FlagKey::Sender) {
            Ok(key) => key,
            Err(e) => return Err(e)
        };

        let sender = match Account::get(config_file, sender_key) {
            Ok(account) => match account {
                Some(account) => account,
                None => return Err(InputError::SenderDoesNotExist),
            },
            Err(_e) => return Err(InputError::SenderDoesNotExist),
        };

        let recipient_values = match input.get_flag_values(FlagKey::Recipient) {
            Ok(values) => values,
            Err(e) => return Err(e),
        };

        let mut recipients: Vec<PublicKey> = Vec::new();
        for value in recipient_values {
            let address = match base58_to_public_key(&value) {
                Ok(pk) => pk,
                Err(_e) => return Err(InputError::InvalidAddress)
            };

            recipients.push(address);
        }

        let mut entries: Vec<Entry> = Vec::new();
        let mut flags = input.flags.iter();
        while let Some(flag) = flags.next() {
            if let FlagKey::Key = flag.key {
                if let Some(value_flag) = flags.next() {
                    entries.push(Entry {
                        key: flag.value.to_string(),
                        value: value_flag.value.to_string(),
                    });
                }
            }
        }

        let blob_values = match input.get_flag_values(FlagKey::Blob) {
            Ok(values) => values,
            Err(e) => {
                match e {
                    InputError::NoSuchKey => vec![],
                    _ => return Err(e),
                }
            }
        };

        let mut blobs: Vec<Blob> = Vec::new();
        for path in blob_values {
            let file = Path::new(&path);
            println!("PATH {:?}", file);
            let binary= match fs::read(file) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    return Err(InputError::CouldNotReadFileBinary);
                }
            };

            let name = match file.file_name().and_then(OsStr::to_str) {
                Some(name) => name.to_string(),
                _ => "".to_string(),
            };

            let blob = Blob {
                binary,
                name,
            };

            blobs.push(blob);
        }


        Ok(MessageInput {
            sender,
            recipients,
            entries,
            blobs,
            otu: OneTimeUse::new(),
        })
    }

    pub fn encrypt(&self, pk: &PublicKey) -> Result<EncryptedMessage, MessageError> {
        let mut parties: Vec<EncryptedParty> = Vec::new();
        let mut entries: Vec<EncryptedEntry> = Vec::new();
        let mut blobs: Vec<EncryptedBlob> = Vec::new();
        let mut hasher = blake2::Blake2s256::new();

        hasher.update(&self.otu.nonce.public);
        hasher.update(&self.otu.nonce.secret);
        hasher.update(&self.otu.broker.public);
        hasher.update(&self.sender.public);
        hasher.update(&pk);

        let party = Party {
            nonce: self.otu.nonce.secret,
            others: self.other_pks(&self.sender.public),
        };

        let encrypted_party = party.encrypt(&self, &self.sender.public);
        parties.push(encrypted_party);


        // for pk in &self.recipients {
        //
        // }


        let party = Party {
            nonce: self.otu.nonce.secret,
            others: self.other_pks(&pk),
        };
        let encrypted_party = party.encrypt(&self, &pk);
        parties.push(encrypted_party);

        for entry in &self.entries {
            hasher.update(&entry.key);
            hasher.update(&entry.value);

            let encrypted_entry = entry.encrypt(&self.otu.nonce.secret, &pk, &self.sender.secret);
            entries.push(encrypted_entry);
        }

        for blob in &self.blobs {
            hasher.update(&blob.binary);
            hasher.update(&blob.name);

            let encrypted_blob = blob.encrypt(&self.otu.nonce.secret, &pk, &self.sender.secret);
            blobs.push(encrypted_blob);
        }

        let mut sender_hasher = blake2::Blake2s256::new();
        sender_hasher.update(&self.sender.public);
        sender_hasher.update(&self.otu.nonce.secret);


        Ok(EncryptedMessage {
            nonce: base64::encode(self.otu.nonce.public),
            broker: base64::encode(self.otu.broker.public),
            sender: base64::encode(sender_hasher.finalize().to_vec()),
            parties,
            entries,
            blobs,
            hash: base64::encode(hasher.finalize().to_vec()),
        })
    }

    fn other_pks(&self, pk: &PublicKey) -> Vec<PublicKey> {
        let mut pks: Vec<PublicKey> = Vec::new();
        pks.push(self.sender.public);
        for pk in &self.recipients {
            pks.push(pk.to_owned());
        }
        pks.iter().filter(|&el| el.ne(pk)).map(|pk| *pk).collect()
    }
}
