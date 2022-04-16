use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey};
use crate::message::errors::MessageError;
use crate::message::utils::Box;
use blake2::Digest;

pub trait Encryption {
    fn encrypt_data(data: &[u8], nonce: &Nonce, recipient_pk: &PublicKey, sender_sk: &SecretKey) -> Result<String, MessageError> {
        let encrypted_data_key = Box::new(
            &data,
            &nonce,
            &recipient_pk,
            &sender_sk,
        ).encrypt();

        Ok(base64::encode(encrypted_data_key))
    }

    fn hash_data(data: &[u8], nonce: &Nonce) -> String {
        let mut hasher = blake2::Blake2s256::new();
        hasher.update(&data);
        hasher.update(&nonce);
        let hash = hasher.finalize().to_vec();
        base64::encode(hash)
    }

    fn decrypt_data(data: &[u8], nonce: &Nonce, sender_pk: &PublicKey, recipient_sk: &SecretKey) -> Result<Vec<u8>, MessageError> {
        let decrypted_data = match Box::new(&data, &nonce, &sender_pk, &recipient_sk).decrypt() {
            Ok(res) => res,
            Err(e) => return Err(e),
        };
        Ok(decrypted_data)
    }
}
