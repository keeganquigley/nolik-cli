use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey};
use crate::message::errors::MessageError;
use crate::message::input::MessageInput;
use crate::message::message::EncryptedMessage;
use crate::message::utils::{base58_to_public_key, base64_to_public_key, base64_to_vec, Box};
use crate::message::encryption::Encryption;

#[derive(Debug, Serialize, Deserialize)]
pub struct Recipient {
    #[serde(rename(serialize = "0", deserialize = "ciphertext"))]
    ciphertext: String,

    #[serde(rename(serialize = "1", deserialize = "hash"))]
    hash: String,
}


impl Encryption for Recipient {}


impl Recipient {
    pub fn encrypt(mi: &MessageInput, pk: &PublicKey) -> Result<Recipient, MessageError> {
        let sender_pk = match base58_to_public_key(&mi.sender.public) {
            Ok(pk) => pk,
            Err(e) => return Err(e),
        };

        let ciphertext = match Recipient::encrypt_data(&pk.as_ref(), &mi.otu.nonce.secret, &sender_pk, &mi.otu.sender.secret) {
            Ok(ciphertext) => ciphertext,
            Err(e) => return Err(e),
        };

        Ok(Recipient {
            ciphertext,
            hash: Recipient::hash_data(&mi.otu.sender.secret.as_ref(), &mi.otu.nonce.secret),
        })
    }

    pub fn decrypt(em: &EncryptedMessage, nonce: &Nonce, recipient_sk: &SecretKey) -> Result<PublicKey, MessageError> {
        let data = match base64_to_vec(&em.recipient.ciphertext) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };

        let pk = match base64_to_public_key(&em.sender.public) {
            Ok(pk) => pk,
            Err(e) => return Err(e),
        };

        let recipient = match Recipient::decrypt_data(&data.as_slice(), &nonce, &pk, &recipient_sk) {
            Ok(recipient) => recipient,
            Err(e) => return Err(e),
        };

        match PublicKey::from_slice(recipient.as_slice()) {
            Some(res) => Ok(res),
            None => return Err(MessageError::DecryptionError),
        }
    }
}
