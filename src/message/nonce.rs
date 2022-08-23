use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::message::encryption::Encryption;
use crate::message::errors::MessageError;
use crate::message::utils::base64_to_vec;


#[derive(Debug, Serialize, Deserialize)]
pub struct Nonce(pub box_::Nonce);

impl Encryption for Nonce {}

impl Nonce {
    pub fn new(nonce: &box_::Nonce) -> Nonce {
        Nonce(*nonce)
    }

    pub fn encrypt(&self, nonce: &box_::Nonce, pk: &PublicKey, sk: &SecretKey) -> EncryptedNonce {
        EncryptedNonce(Nonce::encrypt_data(&self.0.as_ref(), &nonce, pk, sk))
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedNonce(String);

impl Encryption for EncryptedNonce {}

impl EncryptedNonce {
    pub fn new(nonce: &String) -> EncryptedNonce {
        EncryptedNonce(nonce.to_string())
    }

    pub fn decrypt(&self, nonce: &box_::Nonce, pk: &PublicKey, sk: &SecretKey) -> Result<Nonce, MessageError> {
        let encrypted_nonce = match base64_to_vec(&self.0) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let nonce = match Nonce::decrypt_data(&encrypted_nonce.as_ref(), nonce, pk, sk) {
            Ok(res) => match box_::Nonce::from_slice(&res) {
                Some(nonce) => Nonce::new(&nonce),
                None => return Err(MessageError::DecryptionError),
            },
            Err(e) => return Err(e),
        };

        Ok(nonce)
    }
}

