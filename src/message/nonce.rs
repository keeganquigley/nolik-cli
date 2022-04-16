use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::message::errors::MessageError;
use crate::message::input::MessageInput;
use crate::message::message::EncryptedMessage;
use crate::message::utils::{base64_to_nonce, base64_to_public_key, base64_to_vec, Box};
use crate::message::encryption::Encryption;


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Nonce {
    #[serde(rename(serialize = "0", deserialize = "public"))]
    public: String,

    #[serde(rename(serialize = "1", deserialize = "ciphertext_for_sender"))]
    pub ciphertext_for_sender: String,

    #[serde(rename(serialize = "2", deserialize = "ciphertext_for_recipient"))]
    pub ciphertext_for_recipient: String,

    #[serde(rename(serialize = "3", deserialize = "hash"))]
    hash: String,
}


impl Encryption for Nonce {}


impl Nonce {
    pub(crate) fn encrypt(mi: &MessageInput, sender_pk: &PublicKey, recipient_pk: &PublicKey) -> Result<Nonce, MessageError> {
        let ciphertext_for_sender = match Self::encrypt_nonce_for(&mi, &sender_pk) {
            Ok(ciphertext) => ciphertext,
            Err(e) => return Err(e),
        };

        let ciphertext_for_recipient = match Self::encrypt_nonce_for(&mi, &recipient_pk) {
            Ok(ciphertext) => ciphertext,
            Err(e) => return Err(e),
        };

        Ok(Nonce {
            public: base64::encode(mi.otu.nonce.public),
            ciphertext_for_sender,
            ciphertext_for_recipient,
            hash: Self::hash_data(&mi.otu.nonce.secret.as_ref(), &mi.otu.nonce.secret),
        })
    }


    pub fn encrypt_nonce_for(mi: &MessageInput, pk: &PublicKey) -> Result<String, MessageError> {
        match Self::encrypt_data(
            &mi.otu.nonce.secret.as_ref(),
            &mi.otu.nonce.public,
            &pk,
            &mi.otu.sender.secret) {
            Ok(ciphertext) => Ok(ciphertext),
            Err(e) => return Err(e),
        }
    }


    pub fn decrypt_nonce_for_sender(em: &EncryptedMessage, sk: &SecretKey) -> Result<box_::Nonce, MessageError> {
        Self::decrypt_nonce_for(&em, &em.nonce.ciphertext_for_sender, &sk)
    }


    pub fn decrypt_nonce_for_recipient(em: &EncryptedMessage, sk: &SecretKey) -> Result<box_::Nonce, MessageError> {
        Self::decrypt_nonce_for(&em, &em.nonce.ciphertext_for_recipient, &sk)
    }


    pub fn decrypt_nonce_for(em: &EncryptedMessage, data: &String, sk: &SecretKey) -> Result<box_::Nonce, MessageError> {
        let data = match base64_to_vec(&data) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let pn = match base64_to_nonce(&em.nonce.public) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let pk = match base64_to_public_key(&em.sender.public) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let nonce = match Self::decrypt_data(&data, &pn, &pk, &sk) {
            Ok(nonce) => nonce,
            Err(e) => return Err(e)
        };
        match box_::Nonce::from_slice(nonce.as_slice()) {
            Some(nonce) => Ok(nonce),
            None => return Err(MessageError::DecryptionError),
        }
    }
}
