use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey};
use crate::message::encryption::Encryption;
use crate::message::errors::MessageError;
use crate::message::utils::base64_to_vec;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    pub key: String,
    pub value: String,
}


impl Encryption for Entry {}


impl Entry {
    pub fn encrypt(&self, nonce: &box_::Nonce, pk: &PublicKey, sk: &SecretKey) -> EncryptedEntry {
        let key = Entry::encrypt_data(&self.key.as_bytes(), &nonce, pk, sk);
        let value = Entry::encrypt_data(&self.value.as_bytes(), &nonce, pk, sk);

        EncryptedEntry { key, value }
    }

}


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedEntry {
    pub key: String,
    pub value: String,
}


impl Encryption for EncryptedEntry {}


impl EncryptedEntry {
    pub fn new(key: String, value: String) -> EncryptedEntry {
        EncryptedEntry {
            key,
            value,
        }
    }

    pub fn decrypt(&self, nonce: &Nonce, sender_pk: &PublicKey, recipient_sk: &SecretKey) -> Result<Entry, MessageError> {
        let encrypted_key = match base64_to_vec(&self.key) {
            Ok(key) => key,
            Err(e) => return Err(e),
        };

        let key = match Entry::decrypt_data(&encrypted_key.as_slice(), &nonce, sender_pk, recipient_sk) {
            Ok(res) => String::from_utf8(res).unwrap(),
            Err(e) => return Err(e),
        };

        let encrypted_value = match base64_to_vec(&self.value) {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        let value = match Entry::decrypt_data(encrypted_value.as_slice(), &nonce, sender_pk, recipient_sk) {
            Ok(res) => String::from_utf8(res).unwrap(),
            Err(e) => return Err(e),
        };

        Ok(Entry {key, value})
    }
}

