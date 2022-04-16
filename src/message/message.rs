use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::message::data::{Data, DecryptedData};
use serde_derive::{Serialize, Deserialize};
use crate::message::errors::MessageError;
use crate::message::nonce::Nonce;
use crate::message::recipient::Recipient;
use crate::message::sender::Sender;

pub enum SenderOrRecipient {
    Sender,
    Recipient,
}


pub struct DecryptedMessage {
    pub nonce: box_::Nonce,
    pub other: PublicKey,
    pub data: Vec<DecryptedData>
}


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub nonce: Nonce,
    pub(crate) sender: Sender,
    pub(crate) recipient: Recipient,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub(crate) data: Vec<Data>
}


impl EncryptedMessage {
    pub fn decrypt(&self, sor: &SenderOrRecipient, sk: &SecretKey) -> Result<DecryptedMessage, MessageError> {
        let nonce = match sor {
            SenderOrRecipient::Sender => match Nonce::decrypt_nonce_for_sender(&self, &sk) {
                Ok(nonce) => nonce,
                Err(e) => return Err(e),
            },
            SenderOrRecipient::Recipient => match Nonce::decrypt_nonce_for_recipient(&self, &sk) {
                Ok(nonce) => nonce,
                Err(e) => return Err(e),
            }
        };


        let other = match sor {
            SenderOrRecipient::Sender => match Recipient::decrypt(&self, &nonce, sk) {
                Ok(nonce) => nonce,
                Err(e) => return Err(e),
            },
            SenderOrRecipient::Recipient => match Sender::decrypt(&self, &nonce, &sk) {
                Ok(nonce) => nonce,
                Err(e) => return Err(e),
            }
        };


        let data = match Data::decrypt(&self, &nonce, &other, &sk) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };


        Ok(DecryptedMessage {
            nonce,
            other,
            data,
        })
    }
}
