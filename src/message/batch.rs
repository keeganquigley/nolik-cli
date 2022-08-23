use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use crate::message::errors::MessageError;
use crate::message::input::BatchInput;
use crate::message::ipfs::IpfsFile;
use crate::message::message::{EncryptedMessage, Message};
use serde_derive::{Serialize, Deserialize};
use blake2::Digest;
use blake2::digest::Update;
use std::io::Cursor;
use clipboard::{ClipboardContext, ClipboardProvider};
use sodiumoxide::crypto::box_;
use crate::message::session::{EncryptedSession, Session};
use colored::Colorize;
use crate::{AccountOutput, base64_to_nonce, base64_to_public_key, Config, ConfigError, ConfigFile};


#[derive(Debug, Serialize, Deserialize)]
pub struct Batch {
    pub nonce: String,
    pub broker: String,
    pub hash: String,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub sessions: Vec<EncryptedSession>,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub messages: Vec<EncryptedMessage>,
}


impl Batch {
    pub fn new(bi: &BatchInput, secret_nonce: &box_::Nonce) -> Result<Batch, MessageError> {

        let public_nonce = box_::gen_nonce();
        let broker = box_::gen_keypair();

        let mut sessions: Vec<EncryptedSession> = Vec::new();
        let mut messages: Vec<EncryptedMessage> = Vec::new();

        let session = Session::new(&bi, &secret_nonce);
        let message = Message::new(&bi, &secret_nonce);

        let encrypted_session = session.encrypt(&public_nonce, &bi.sender.public, &broker.1);
        sessions.push(encrypted_session);

        for pk in &bi.recipients {
            let encrypted_session = session.encrypt(&public_nonce, &pk, &broker.1);
            sessions.push(encrypted_session);

            let encrypted_message = message.encrypt(&pk, &bi.sender.secret);
            messages.push(encrypted_message);
        }


        Ok(Batch {
            nonce: base64::encode(&public_nonce),
            broker: base64::encode(&broker.0),
            hash: Self::hash(&bi, &secret_nonce),
            sessions,
            messages
        })
    }


    pub fn hash(bi: &BatchInput, nonce: &box_::Nonce) -> String {
        let mut hash = blake2::Blake2s256::new();
        Update::update(&mut hash, &nonce.as_ref());
        Update::update(&mut hash, &bi.sender.public.as_ref());

        for pk in &bi.recipients {
            Update::update(&mut hash, pk.as_ref());
        }

        for entry in &bi.entries {
            Update::update(&mut hash, entry.key.as_ref());
            Update::update(&mut hash, entry.value.as_ref());
        }

        for file in &bi.files {
            Update::update(&mut hash, file.name.as_ref());
            Update::update(&mut hash, file.binary.as_ref());
        }

        base64::encode(hash.finalize().to_vec())
    }


    pub async fn save(&self) -> Result<IpfsFile, MessageError> {
        let contents = match toml::to_string(&self) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(MessageError::CouldNotCreateTomlFileContents);
            },
        };

        let client = IpfsClient::default();
        let data = Cursor::new(contents);
        match client.add(data).await {
            Ok(res) => {
                match client.pin_add(&res.hash, true).await {
                    Ok(_pin) => {
                        let output = format!("Message has been composed!");
                        let hash = format!("IPFS hash: {}", &res.hash);

                        println!("{} {}", output.bright_green(), hash);

                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        ctx.set_contents(res.hash.clone()).unwrap();
                    },
                    Err(e) => {
                        eprintln!("Error on pinning IPFS file: {:#?}", e);
                        return Err(MessageError::CouldNotAddFileToIPFS)
                    }
                }
                let ipfs_file = IpfsFile::new(res.hash);
                Ok(ipfs_file)
            },
            Err(e) => {
                eprintln!("Error on adding file to IPFS: {:#?}", e);
                return Err(MessageError::CouldNotAddFileToIPFS)
            }
        }
    }


    pub fn parties(&self, config_file: &ConfigFile) -> Result<(box_::PublicKey, Vec<box_::PublicKey>), ConfigError> {

        let public_nonce = match base64_to_nonce(&self.nonce) {
            Ok(res) => res,
            Err(_e) => return Err(ConfigError::CouldNotGetBatchNonce)
        };

        let broker = match base64_to_public_key(&self.broker) {
            Ok(res) => res,
            Err(_e) => return Err(ConfigError::CouldNotGetBatchBroker)
        };

        let config = match Config::new(&config_file) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };


        let sessions: Vec<Session> = config.data.accounts
            .iter()
            .map(|ao| AccountOutput::deserialize(ao).unwrap())
            .map(|a| (a, self.sessions.first().unwrap()))
            .filter_map(|el| el.1.decrypt(&public_nonce, &broker, &el.0.secret).ok())
            .collect();


        if sessions.len() == 0 {
            return Err(ConfigError::CouldNotInitSender);
        }

        let session = sessions.first().unwrap();
        let sender = session.group.get_sender();
        let recipients = session.group.get_recipients();

        Ok((sender, recipients))
    }
}
