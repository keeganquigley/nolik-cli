use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey};
use crate::message::input::MessageInput;
use crate::message::message::EncryptedMessage;
use crate::message::utils::{base58_to_public_key, base64_to_public_key, base64_to_vec, Box};
use crate::message::errors::MessageError;
use serde_derive::{Serialize, Deserialize};
use crate::message::encryption::Encryption;


#[derive(Debug, Serialize, Deserialize)]
pub struct Sender {
    #[serde(rename(serialize = "0", deserialize = "public"))]
    pub(crate) public: String,

    #[serde(rename(serialize = "1", deserialize = "ciphertext"))]
    ciphertext: String,

    #[serde(rename(serialize = "2", deserialize = "hash"))]
    hash: String,
}


impl Encryption for Sender {}

impl Sender {
    pub fn encrypt(mi: &MessageInput, pk: &PublicKey) -> Result<Sender, MessageError> {
        let sender_pk = match base58_to_public_key(&mi.sender.public) {
            Ok(pk) => pk,
            Err(e) => return Err(e),
        };

        let ciphertext = match Sender::encrypt_data(&sender_pk.as_ref(), &mi.otu.nonce.secret, pk, &mi.otu.sender.secret) {
            Ok(ciphertext) => ciphertext,
            Err(e) => return Err(e),
        };

        Ok(Sender {
            public: base64::encode(&mi.otu.sender.public),
            ciphertext,
            hash: Sender::hash_data(&mi.otu.sender.secret.as_ref(), &mi.otu.nonce.secret),
        })
    }

    pub fn decrypt(em: &EncryptedMessage, nonce: &Nonce, recipient_sk: &SecretKey) -> Result<PublicKey, MessageError> {
        let data = match base64_to_vec(&em.sender.ciphertext) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };

        let pk = match base64_to_public_key(&em.sender.public) {
            Ok(pk) => pk,
            Err(e) => return Err(e),
        };

        let sender = match Sender::decrypt_data(&data.as_slice(), &nonce, &pk, &recipient_sk) {
            Ok(sender) => sender,
            Err(e) => return Err(e),
        };

        match PublicKey::from_slice(sender.as_slice()) {
            Some(res) => Ok(res),
            None => return Err(MessageError::DecryptionError),
        }
    }
}


