use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey};
use crate::message::encryption::Encryption;
use crate::message::errors::MessageError;
use crate::message::utils::base64_to_vec;

#[derive(Debug)]
pub struct File {
    pub binary: Vec<u8>,
    pub name: String,
}


impl Encryption for File {}


impl File {
    pub fn encrypt(&self, nonce: &Nonce, pk: &PublicKey, sk: &SecretKey) -> EncryptedFile {
        let file = File::encrypt_data(&self.binary.as_slice(), &nonce, pk, sk);
        let name = File::encrypt_data(&self.name.as_bytes(), &nonce, pk, sk);

        EncryptedFile { file, name }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedFile {
    file: String,
    name: String,
}


impl Encryption for EncryptedFile {}


impl EncryptedFile {
    pub fn decrypt(&self, nonce: &Nonce, pk: &PublicKey, sk: &SecretKey) -> Result<File, MessageError> {
        let encrypted_binary = match base64_to_vec(&self.file) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let binary = match EncryptedFile::decrypt_data(&encrypted_binary, nonce, pk, sk) {
            Ok(binary ) => binary,
            Err(e) => return Err(e),
        };

        let encrypted_name = match base64_to_vec(&self.name) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let name = match EncryptedFile::decrypt_data(&encrypted_name, nonce, pk, sk) {
            Ok(name) => match String::from_utf8(name) {
                Ok(name) => name,
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    return Err(MessageError::DecryptionError)
                }
            }
            Err(e) => return Err(e),
        };

        Ok(File { binary, name })
    }
}
