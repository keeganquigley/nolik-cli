use blake2::Digest;
use blake2::digest::Update;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::message::entry::{Entry, EncryptedEntry};
use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use crate::message::file::{File, EncryptedFile};
use crate::message::errors::MessageError;
use crate::message::input::BatchInput;
use crate::message::session::Session;


#[derive(Debug)]
pub struct Message {
    pub nonce: box_::Nonce,
    pub sender: PublicKey,
    pub recipients: Vec<PublicKey>,
    pub entries: Vec<Entry>,
    pub files: Vec<File>,
}


impl Message {
    pub fn new(bi: &BatchInput, secret_nonce: &box_::Nonce) -> Message {
        let mut recipients = Vec::new();
        for pk in &bi.recipients {
            recipients.push(*pk);
        }

        let mut entries = Vec::new();
        for el in &bi.entries {
            let entry = Entry {
                key: el.key.clone(),
                value: el.value.clone(),
            };
            entries.push(entry);
        }

        let mut files = Vec::new();
        for el in &bi.files {
            let file = File {
                binary: el.binary.clone(),
                name: el.name.to_string(),
            };
            files.push(file);
        }


        Message {
            nonce: *secret_nonce,
            sender: PublicKey::from_slice(&bi.sender.public.as_ref()).unwrap(),
            recipients,
            entries,
            files,
        }
    }

    pub fn encrypt(&self, pk: &PublicKey, sk: &SecretKey) -> EncryptedMessage {

        let mut parties = blake2::Blake2s256::new();
        Update::update(&mut parties, &self.sender.as_ref());
        Update::update(&mut parties, &pk.as_ref());
        let parties_hash = base64::encode(parties.finalize().to_vec());


        let mut entries: Vec<EncryptedEntry> = Vec::new();
        let mut files: Vec<EncryptedFile> = Vec::new();

        for entry in &self.entries {
            let encrypted_entry = entry.encrypt(&self.nonce, &pk, &sk);
            entries.push(encrypted_entry);
        }

        for file in &self.files {
            let encrypted_file = file.encrypt(&self.nonce, &pk, &sk);
            files.push(encrypted_file);
        }

        EncryptedMessage {
            parties: parties_hash,
            entries,
            files,
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub parties: String,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub entries: Vec<EncryptedEntry>,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub files: Vec<EncryptedFile>,
}


impl EncryptedMessage {
    pub fn decrypt(&self, session: &Session, pk: &PublicKey, sk: &SecretKey) -> Result<Message, MessageError> {

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


        Ok(Message {
            nonce: session.nonce.0,
            sender,
            recipients,
            entries,
            files,
        })
    }
}