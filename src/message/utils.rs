use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::PublicKey;
use crate::message::errors::MessageError;

pub fn base64_to_vec(data: &String) -> Result<Vec<u8>, MessageError> {
    match base64::decode(data) {
        Ok(nonce) => Ok(nonce),
        Err(e) => {
            eprintln!("Error {:?}", e);
            return Err(MessageError::DecryptionError);
        }
    }
}


pub fn base64_to_nonce(data: &String) -> Result<box_::Nonce, MessageError> {
    match base64_to_vec(data) {
        Ok(vec) => match box_::Nonce::from_slice(vec.as_slice()) {
            Some(nonce) => Ok(nonce),
            None => return Err(MessageError::DecryptionError),
        }
        Err(e) => return Err(e),
    }
}


pub fn base64_to_public_key(data: &String) -> Result<box_::PublicKey, MessageError> {
    match base64_to_vec(data) {
        Ok(vec) => match box_::PublicKey::from_slice(vec.as_slice()) {
            Some(pk) => Ok(pk),
            None => return Err(MessageError::DecryptionError),
        }
        Err(e) => return Err(e),
    }
}

pub fn base58_to_vec(data: &String) -> Result<Vec<u8>, MessageError> {
    match bs58::decode(data).into_vec() {
        Ok(res) => Ok(res),
        Err(e) => {
            eprintln!("Error {:?}", e);
            return Err(MessageError::DecryptionError);
        }
    }
}

pub fn base58_to_public_key(data: &String) -> Result<box_::PublicKey, MessageError> {
    match base58_to_vec(data) {
        Ok(vec) => {
            match box_::PublicKey::from_slice(vec.as_slice()) {
                Some(pk) => Ok(pk),
                None => return Err(MessageError::DecryptionError),
            }
        },
        Err(e) => return Err(e),
    }
}

pub fn base58_to_secret_key(data: &String) -> Result<box_::SecretKey, MessageError> {
    match base58_to_vec(data) {
        Ok(vec) => {
            match box_::SecretKey::from_slice(vec.as_slice()) {
                Some(sk) => Ok(sk),
                None => return Err(MessageError::DecryptionError),
            }
        },
        Err(e) => return Err(e),
    }
}

pub fn base58_to_seed(data: &String) -> Result<box_::Seed, MessageError> {
    match base58_to_vec(data) {
        Ok(vec) => {
            match box_::Seed::from_slice(vec.as_slice()) {
                Some(sk) => Ok(sk),
                None => return Err(MessageError::DecryptionError),
            }
        },
        Err(e) => return Err(e),
    }
}


pub fn hash_address(data: &PublicKey) -> String {
    let hash_512 = sp_core::hashing::blake2_512(&data.as_ref());
    let hash_128 = sp_core::hashing::blake2_128(&hash_512);

    hex::encode(hash_128)
}
