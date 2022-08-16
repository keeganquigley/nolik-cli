use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey};
use crate::message::entry::{Entry, EncryptedEntry};
use serde_derive::{Serialize, Deserialize};
use crate::message::file::{File, EncryptedFile};
use crate::message::errors::MessageError;
use crate::message::session::Session;


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub parties: String,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub entries: Vec<EncryptedEntry>,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub files: Vec<EncryptedFile>,
}


impl EncryptedMessage {
    pub fn decrypt(&self, session: &Session, pk: &PublicKey, sk: &SecretKey) -> Result<DecryptedMessage, MessageError> {

        let sender = session.group.get_sender();
        let recipients = session.group.get_recipients();

        let mut entries = vec![];
        let mut files = vec![];

        for entry in &self.entries {
            let decrypted_entry = match entry.decrypt(&session.nonce.0, pk, sk) {
                Ok(res) => res,
                Err(e) => return Err(e),
            };
            entries.push(decrypted_entry);
        }

        for file in &self.files {
            let decrypted_file = match file.decrypt(&session.nonce.0, pk, sk) {
                Ok(res) => res,
                Err(e) => return Err(e),
            };
            files.push(decrypted_file);
        }


        Ok(DecryptedMessage {
            nonce: session.nonce.0,
            sender,
            recipients,
            entries,
            files,
        })
    }
}



#[derive(Debug)]
pub struct DecryptedMessage {
    pub nonce: Nonce,
    pub sender: PublicKey,
    pub recipients: Vec<PublicKey>,
    pub entries: Vec<Entry>,
    pub files: Vec<File>,
}