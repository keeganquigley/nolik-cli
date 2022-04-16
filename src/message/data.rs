use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey};
use crate::message::encryption::Encryption;
use crate::message::errors::MessageError;
use crate::message::input::MessageInput;
use crate::message::message::EncryptedMessage;
use crate::message::utils::{base64_to_vec, Box};


#[derive(Debug, Serialize, Deserialize)]
struct DataKey {
    ciphertext: String,
    hash: String,
}


impl Encryption for DataKey {}


#[derive(Debug, Serialize, Deserialize)]
struct DataValue {
    ciphertext: String,
    hash: String,
}


impl Encryption for DataValue {}


#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    key: DataKey,
    value: DataValue,
}

#[derive(Debug)]
pub struct DecryptedData {
    pub key: String,
    pub value: String,
}


impl Data {
    pub fn encrypt(message_input: &MessageInput, recipient_pk: &PublicKey, sender_sk: &SecretKey) -> Result<Vec<Data>, MessageError> {
        let mut data: Vec<Data> = Vec::new();
        for data_input in message_input.data.iter() {
            let key = match Self::encrypt_key(&data_input.0.as_bytes(), &message_input.otu.nonce.secret, &recipient_pk, &sender_sk) {
                Ok(encrypted_key) => encrypted_key,
                Err(e) => return Err(e),
            };

            let value = match Self::encrypt_value(&data_input.1.as_bytes(), &message_input.otu.nonce.secret, &recipient_pk, &sender_sk) {
                Ok(encrypted_value) => encrypted_value,
                Err(e) => return Err(e),
            };

            data.push(Data { key, value })
        }

        Ok(data)
    }


    fn encrypt_key(data: &[u8], nonce: &Nonce, recipient_pk: &PublicKey, sender_sk: &SecretKey) -> Result<DataKey, MessageError> {
        let ciphertext = match DataKey::encrypt_data(data, nonce, recipient_pk, sender_sk) {
            Ok(ciphertext) => ciphertext,
            Err(e) => return Err(e),
        };

        Ok(DataKey {
            ciphertext,
            hash: DataKey::hash_data(data, &nonce),
        })
    }


    fn encrypt_value(data: &[u8], nonce: &Nonce, recipient_pk: &PublicKey, sender_sk: &SecretKey) -> Result<DataValue, MessageError> {
        let ciphertext = match DataKey::encrypt_data(data, nonce, recipient_pk, sender_sk) {
            Ok(ciphertext) => ciphertext,
            Err(e) => return Err(e),
        };

        Ok(DataValue {
            ciphertext,
            hash: DataKey::hash_data(&data, &nonce),
        })
    }


    pub(crate) fn decrypt(em: &EncryptedMessage, nonce: &Nonce, sender_pk: &PublicKey, recipient_sk: &SecretKey) -> Result<Vec<DecryptedData>, MessageError> {
        let mut data: Vec<DecryptedData> = Vec::new();
        for data_input in em.data.iter() {
            let encrypted_key = match base64_to_vec(&data_input.key.ciphertext) {
                Ok(key) => key,
                Err(e) => return Err(e),
            };

            let key = match DataKey::decrypt_data(&encrypted_key.as_slice(), &nonce, sender_pk, recipient_sk) {
                Ok(res) => String::from_utf8(res).unwrap(),
                Err(e) => return Err(e),
            };

            let encrypted_value = match base64_to_vec(&data_input.value.ciphertext) {
                Ok(value) => value,
                Err(e) => return Err(e),
            };

            let value = match DataValue::decrypt_data(encrypted_value.as_slice(), &nonce, sender_pk, recipient_sk) {
                Ok(res) => String::from_utf8(res).unwrap(),
                Err(e) => return Err(e),
            };

            data.push(DecryptedData { key, value })
        }

        Ok(data)
    }
}
