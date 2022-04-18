use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::{Account, Config, Input};
use crate::cli::errors::InputError;
use crate::cli::input::{Flag, FlagKey};
use crate::message::data::Data;
use crate::message::errors::MessageError;
use crate::message::message::{EncryptedMessage, Public};
use crate::message::nonce::Nonce;
use crate::message::sender::Sender;
use crate::message::recipient::Recipient;
use crate::message::utils::base58_to_public_key;

#[derive(Debug)]
pub struct OneTimeUseNonce {
    pub(crate) public: box_::Nonce,
    pub secret: box_::Nonce,
}


impl OneTimeUseSender {
    fn new() -> OneTimeUseSender {
        let sender = box_::gen_keypair();
        OneTimeUseSender {
            public: sender.0,
            secret: sender.1,
        }
    }
}


#[derive(Debug)]
pub struct OneTimeUseSender {
    pub(crate) public: PublicKey,
    pub(crate) secret: SecretKey,
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
pub struct OneTimeUse {
    pub nonce: OneTimeUseNonce,
    pub(crate) sender: OneTimeUseSender,
}


impl OneTimeUse {
    fn new() -> OneTimeUse {
        OneTimeUse {
            nonce: OneTimeUseNonce::new(),
            sender: OneTimeUseSender::new(),
        }
    }
}


#[derive(Debug)]
pub struct MessageInput {
    pub sender: Account,
    pub recipients: Vec<box_::PublicKey>,
    pub data: Vec<(String, String)>,
    blobs: Vec<Flag>,
    pub otu: OneTimeUse,
}


impl MessageInput {
    pub fn new(input: &mut Input, config: &Config) -> Result<MessageInput, InputError> {
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

        let mut recipients: Vec<PublicKey> = Vec::new();
        for key in recipient_keys {
            let address = match base58_to_public_key(&key) {
                Ok(pk) => pk,
                Err(_e) => return Err(InputError::InvalidAddress)
            };

            recipients.push(address);
        }

        let mut data: Vec<(String, String)> = Vec::new();
        let flags = &mut input.flags;
        while let Some(key_flag) = flags.pop() {
            let key = key_flag.value;
            match key_flag.key {
                FlagKey::Key => {
                    let value: String = match flags.iter().next() {
                        Some(flag) => flag.value.to_string(),
                        None => return Err(InputError::NoCorrespondingValue),
                    };
                    data.push((key, value));
                },
                _ => continue
            }
        }

        Ok(MessageInput {
            sender,
            recipients,
            data,
            blobs: vec![],
            otu: OneTimeUse::new(),
        })
    }

    pub fn encrypt(&self, sender_pk: &PublicKey, sender_sk: &SecretKey, recipient_pk: &PublicKey) -> Result<EncryptedMessage, MessageError> {
        let public = Public {
            nonce: base64::encode(self.otu.nonce.public),
            sender: base64::encode(self.otu.sender.public),
        };

        let nonce = match Nonce::encrypt(&self, &sender_pk, &recipient_pk) {
            Ok(sender) => sender,
            Err(e) => return Err(e),
        };

        let sender = match Sender::encrypt(&self, &recipient_pk) {
            Ok(sender) => sender,
            Err(e) => return Err(e),
        };

        let recipient = match Recipient::encrypt(&self, &recipient_pk) {
            Ok(recipient) => recipient,
            Err(e) => return Err(e),
        };

        let data = match Data::encrypt(&self, &recipient_pk, &sender_sk) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };

        Ok(EncryptedMessage {
            public,
            nonce,
            sender,
            recipient,
            data,
        })
    }
}
