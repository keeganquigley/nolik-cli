use std::ptr::read_unaligned;
use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::message::errors::MessageError;
use crate::message::input::MessageInput;
use crate::message::message::EncryptedMessage;
use crate::message::utils::{base58_to_public_key, base64_to_public_key, base64_to_vec, Box};
use blake2::Digest;

#[derive(Debug, Serialize, Deserialize)]
pub struct Recipient {
    #[serde(rename(serialize = "0", deserialize = "ciphertext"))]
    ciphertext: String,

    #[serde(rename(serialize = "1", deserialize = "hash"))]
    hash: String,
}


impl Recipient {
    pub(crate) fn encrypt(message_input: &MessageInput, recipient_pk: &PublicKey) -> Result<Recipient, MessageError> {
        let ciphertext = match Self::encrypt_recipient(&message_input, &recipient_pk) {
            Ok(ciphertext) => ciphertext,
            Err(e) => return Err(e),
        };

        Ok(Recipient {
            ciphertext,
            hash: Self::recipient_hash(&message_input, &recipient_pk),
        })
    }

    fn encrypt_recipient(mi: &MessageInput, recipient_pk: &PublicKey) -> Result<String, MessageError> {
        let sender_pk = match base58_to_public_key(&mi.sender.public) {
            Ok(pk) => pk,
            Err(e) => return Err(e),
        };

        let encrypted_recipient = Box::new(
            &recipient_pk.as_ref(),
            &mi.otu.nonce.secret,
            &sender_pk,
            &mi.otu.sender.secret,
        ).encrypt();

        Ok(base64::encode(encrypted_recipient))
    }

    fn recipient_hash(mi: &MessageInput, pk: &PublicKey) -> String {
        let mut hasher = blake2::Blake2s256::new();
        hasher.update(&pk);
        hasher.update(&mi.otu.nonce.secret);
        let hash = hasher.finalize().to_vec();
        base64::encode(hash)
    }

    pub fn decrypt(em: &EncryptedMessage, nonce: &box_::Nonce, sk: &SecretKey) -> Result<PublicKey, MessageError> {
        let data = match base64_to_vec(&em.recipient.ciphertext) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };

        let pk = match base64_to_public_key(&em.sender.public) {
            Ok(pk) => pk,
            Err(e) => return Err(e),
        };

        let recipient = match Box::new(data.as_slice(), nonce, &pk, &sk).decrypt() {
            Ok(sender) => sender,
            Err(e) => return Err(e),
        };

        match box_::PublicKey::from_slice(recipient.as_slice()) {
            Some(res) => Ok(res),
            None => return Err(MessageError::DecryptionError),
        }
    }
}
