use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use blake2::Digest;
use blake2::digest::Update;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::{Account, ConfigFile, Input};
use crate::cli::errors::InputError;
use crate::cli::input::FlagKey;
use crate::message::file::{File, EncryptedFile};
use crate::message::entry::{Entry, EncryptedEntry};
// use crate::message::errors::MessageError;
use crate::message::message::EncryptedMessage;
use crate::message::utils::base58_to_public_key;
// use crate::message::nonce::Nonce;
// use crate::message::parties::Parties;

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
    pub files: Vec<File>,
    pub otu: OneTimeUse,
}


impl MessageInput {
    pub fn new(input: &mut Input, config_file: &ConfigFile) -> Result<MessageInput, InputError> {
        let sender_key = match input.get_flag_value(FlagKey::Sender) {
            Ok(key) => key,
            Err(e) => return Err(e)
        };

        let sender = match Account::get(&config_file, sender_key) {
            Ok(account) => account,
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

        let file_values = match input.get_flag_values(FlagKey::File) {
            Ok(values) => values,
            Err(e) => {
                match e {
                    InputError::NoSuchKey => vec![],
                    _ => return Err(e),
                }
            }
        };

        let mut files: Vec<File> = Vec::new();
        for path in file_values {
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

            let file = File {
                binary,
                name,
            };

            files.push(file);
        }

        Ok(MessageInput {
            sender,
            recipients,
            entries,
            files,
            otu: OneTimeUse::new(),
        })
    }

    pub fn encrypt(&self, pk: &PublicKey) -> EncryptedMessage {
        // let nonce: Nonce = Nonce::new(self.otu.nonce.secret);
        // let encrypted_nonce = nonce.encrypt(&self.otu.nonce.public, &pk, &self.sender.secret);
        //
        // let mut parties: Parties = Parties::new();
        // parties.add(&self.sender.public);
        // for recipient in &self.recipients {
        //     parties.add(recipient);
        // }
        //
        // let encrypted_parties = parties.encrypt(&self.otu.nonce.secret, pk, &self.sender.secret);
        let mut parties = blake2::Blake2s256::new();
        Update::update(&mut parties, &self.sender.public.as_ref());
        Update::update(&mut parties, &pk.as_ref());
        let parties_hash = base64::encode(parties.finalize().to_vec());


        let mut entries: Vec<EncryptedEntry> = Vec::new();
        let mut files: Vec<EncryptedFile> = Vec::new();

        for entry in &self.entries {
            let encrypted_entry = entry.encrypt(&self.otu.nonce.secret, &pk, &self.sender.secret);
            entries.push(encrypted_entry);
        }

        for file in &self.files {
            let encrypted_file = file.encrypt(&self.otu.nonce.secret, &pk, &self.sender.secret);
            files.push(encrypted_file);
        }

        EncryptedMessage {
            parties: parties_hash,
            entries,
            files,
        }
    }

    // fn other_pks(&self, pk: &PublicKey) -> Vec<PublicKey> {
    //     let mut pks: Vec<PublicKey> = Vec::new();
    //     pks.push(self.sender.public);
    //     for pk in &self.recipients {
    //         pks.push(pk.to_owned());
    //     }
    //     pks.iter().filter(|&el| el.ne(pk)).map(|pk| *pk).collect()
    // }
}
