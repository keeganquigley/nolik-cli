use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey};
// use blake2::Digest;
use sodiumoxide::crypto::box_;
use crate::message::errors::MessageError;


pub trait Encryption {
    fn encrypt_data(data: &[u8], nonce: &Nonce, recipient_pk: &PublicKey, sender_sk: &SecretKey) -> String {
        let encoded_data = box_::seal(&data, &nonce, &recipient_pk, &sender_sk);
        base64::encode(encoded_data)
    }

    // fn hash_data(data: &[u8], nonce: &Nonce) -> String {
    //     let mut hasher = blake2::Blake2s256::new();
    //     hasher.update(&data);
    //     hasher.update(&nonce);
    //     let hash = hasher.finalize().to_vec();
    //     base64::encode(hash)
    // }

    fn decrypt_data(data: &[u8], nonce: &Nonce, pk: &PublicKey, sk: &SecretKey) -> Result<Vec<u8>, MessageError> {
        let decrypted_data = match box_::open(&data, &nonce, &pk, &sk) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(MessageError::DecryptionError);
            }
        };
        Ok(decrypted_data)
    }
}
