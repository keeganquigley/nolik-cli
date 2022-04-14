use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::message::errors::MessageError;
use crate::message::input::MessageInput;
use crate::message::message::EncryptedMessage;
use crate::message::utils::{base64_to_nonce, base64_to_public_key, base64_to_vec, Box};
use blake2::Digest;

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


impl Nonce {
    pub(crate) fn encrypt(message_input: &MessageInput, sender_pk: &PublicKey, recipient_pk: &PublicKey) -> Nonce {
        Nonce {
            public: Self::public_nonce_encoded(&message_input),
            ciphertext_for_sender: Self::encrypt_nonce_for_sender(&message_input, &sender_pk),
            ciphertext_for_recipient: Self::encrypt_nonce_for_recipient(&message_input, &recipient_pk),
            hash: Self::nonce_hash(message_input),
        }
    }

    fn public_nonce_encoded(mi: &MessageInput) -> String {
        base64::encode(mi.otu.nonce.public)
    }

    pub fn encrypt_nonce_for_sender(mi: &MessageInput, sender_pk: &PublicKey) -> String {
        let encrypted_nonce = Box::new(
            &mi.otu.nonce.secret.as_ref(),
            &mi.otu.nonce.public,
            &sender_pk,
            &mi.otu.sender.secret,
        ).encrypt();

        base64::encode(encrypted_nonce)
    }

    fn encrypt_nonce_for_recipient(mi: &MessageInput, recipient_pk: &PublicKey) -> String {
        let encrypted_nonce = Box::new(
            &mi.otu.nonce.secret.as_ref(),
            &mi.otu.nonce.public,
            &recipient_pk,
            &mi.otu.sender.secret,
        ).encrypt();

        base64::encode(encrypted_nonce)
    }

    fn nonce_hash(mi: &MessageInput) -> String {
        let mut hasher = blake2::Blake2s256::new();
        hasher.update(&mi.otu.nonce.secret);
        hasher.update(&mi.otu.nonce.secret);
        let hash = hasher.finalize().to_vec();

        base64::encode(hash)
    }

    fn decrypt_nonce(data: &[u8], pn: &box_::Nonce, pk: &PublicKey, sk: &SecretKey) -> Result<box_::Nonce, MessageError> {
        let nonce_box = Box::new(&data, &pn, &pk, &sk);

        let nonce = match nonce_box.decrypt() {
            Ok(nonce) => nonce,
            Err(e) => return Err(e)
        };

        match box_::Nonce::from_slice(nonce.as_slice()) {
            Some(nonce) => Ok(nonce),
            None => return Err(MessageError::DecryptionError),
        }
    }

    pub fn decrypt_nonce_for_sender(em: &EncryptedMessage, sk: &SecretKey) -> Result<box_::Nonce, MessageError> {
        let data = match base64_to_vec(&em.nonce.ciphertext_for_sender) {
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

        Self::decrypt_nonce(&data, &pn, &pk, &sk)
    }

    pub fn decrypt_nonce_for_recipient(em: &EncryptedMessage, sk: &SecretKey) -> Result<box_::Nonce, MessageError> {
        let data = match base64_to_vec(&em.nonce.ciphertext_for_recipient) {
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
        Self::decrypt_nonce(&data, &pn, &pk, &sk)
    }
}
