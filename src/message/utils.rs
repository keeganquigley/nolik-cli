use sodiumoxide::crypto::box_;
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

fn base58_to_vec(data: &String) -> Result<Vec<u8>, MessageError> {
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

pub struct Box<'a> {
    data: &'a [u8],
    nonce: &'a box_::Nonce,
    pk: &'a box_::PublicKey,
    sk: &'a box_::SecretKey,
}


impl<'a> Box<'a> {
    pub(crate) fn new(
        data: &'a [u8],
        nonce: &'a box_::Nonce,
        pk: &'a box_::PublicKey,
        sk: &'a box_::SecretKey
    ) -> Box<'a> {
        Box {
            data,
            nonce,
            pk,
            sk,
        }
    }

    pub(crate) fn encrypt(&self) -> Vec<u8> {
        box_::seal(self.data, &self.nonce, &self.pk, &self.sk)
    }

    pub(crate) fn decrypt(&self) -> Result<Vec<u8>, MessageError> {
        match box_::open(&self.data, &self.nonce, &self.pk, &self.sk) {
            Ok(res) => Ok(res),
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(MessageError::DecryptionError);
            }
        }
    }
}
