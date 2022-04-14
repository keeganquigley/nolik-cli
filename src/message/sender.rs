use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::message::input::MessageInput;
use crate::message::message::EncryptedMessage;
use crate::message::utils::{base58_to_public_key, base64_to_public_key, base64_to_vec, Box};
use blake2::Digest;
use crate::message::errors::MessageError;
use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Sender {
    #[serde(rename(serialize = "0", deserialize = "public"))]
    pub(crate) public: String,

    #[serde(rename(serialize = "1", deserialize = "ciphertext"))]
    ciphertext: String,

    #[serde(rename(serialize = "2", deserialize = "hash"))]
    hash: String,
}


impl Sender {
    pub(crate) fn encrypt(message_input: &MessageInput, pk: &PublicKey) -> Result<Sender, MessageError> {
        let ciphertext = match Self::encrypt_sender(&message_input, &pk) {
            Ok(ciphertext) => ciphertext,
            Err(e) => return Err(e),
        };

        Ok(Sender {
            public: Self::public_sender(&message_input),
            ciphertext,
            hash: Self::sender_hash(message_input),
        })
    }

    fn public_sender(mi: &MessageInput) -> String {
        base64::encode(mi.otu.sender.public)
    }

    fn encrypt_sender(mi: &MessageInput, recipient_pk: &PublicKey) -> Result<String, MessageError> {
        let sender_pk = match base58_to_public_key(&mi.sender.public) {
            Ok(pk) => pk,
            Err(e) => return Err(e),
        };

        let encrypted_sender = Box::new(
            &sender_pk.as_ref(),
            &mi.otu.nonce.secret,
            &recipient_pk,
            &mi.otu.sender.secret,
        ).encrypt();

        Ok(base64::encode(encrypted_sender))
    }

    fn sender_hash(mi: &MessageInput) -> String {
        let mut hasher = blake2::Blake2s256::new();
        hasher.update(&mi.otu.sender.secret);
        hasher.update(&mi.otu.nonce.secret);
        let hash = hasher.finalize().to_vec();
        base64::encode(hash)
    }

    pub fn decrypt(em: &EncryptedMessage, nonce: &box_::Nonce, recipient_sk: &SecretKey) -> Result<PublicKey, MessageError> {
        let data = match base64_to_vec(&em.sender.ciphertext) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };

        let pk = match base64_to_public_key(&em.sender.public) {
            Ok(pk) => pk,
            Err(e) => return Err(e),
        };

        let sender = match Box::new(&data.as_slice(), &nonce, &pk, &recipient_sk).decrypt() {
            Ok(sender) => sender,
            Err(e) => return Err(e),
        };

        match PublicKey::from_slice(sender.as_slice()) {
            Some(res) => Ok(res),
            None => return Err(MessageError::DecryptionError),
        }
    }
}
