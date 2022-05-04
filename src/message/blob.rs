use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey};
use crate::message::encryption::Encryption;
use crate::message::errors::MessageError;
use crate::message::utils::base64_to_vec;

#[derive(Debug)]
pub struct Blob {
    pub binary: Vec<u8>,
    pub name: String,
}


impl Encryption for Blob {}


impl Blob {
    pub fn encrypt(&self, nonce: &Nonce, pk: &PublicKey, sk: &SecretKey) -> EncryptedBlob {
        let file = Blob::encrypt_data(&self.binary.as_slice(), &nonce, pk, sk);
        let name = Blob::encrypt_data(&self.name.as_bytes(), &nonce, pk, sk);

        EncryptedBlob { file, name }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedBlob {
    // #[serde(rename = "f")]
    file: String,

    // #[serde(rename = "n")]
    name: String,
}


impl Encryption for EncryptedBlob {}


impl EncryptedBlob {
    pub fn decrypt(&self, nonce: &Nonce, pk: &PublicKey, sk: &SecretKey) -> Result<Blob, MessageError> {
        let encrypted_binary = match base64_to_vec(&self.file) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let binary = match EncryptedBlob::decrypt_data(&encrypted_binary, nonce, pk, sk) {
            Ok(binary ) => binary,
            Err(e) => return Err(e),
        };

        let encrypted_name = match base64_to_vec(&self.name) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let name = match EncryptedBlob::decrypt_data(&encrypted_name, nonce, pk, sk) {
            Ok(name) => match String::from_utf8(name) {
                Ok(name) => name,
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    return Err(MessageError::DecryptionError)
                }
            }
            Err(e) => return Err(e),
        };

        Ok(Blob { binary, name })
    }
}
