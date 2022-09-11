use std::io::{stdout, Write};
use std::time::Duration;
use futures_util::TryStreamExt;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use sodiumoxide::crypto::box_::PublicKey;
use crate::{Account, base64_to_nonce, base64_to_public_key, Batch, ConfigFile, EncryptedMessage, FlagKey, Index, IndexMessage, Input, NodeError, NodeEvent, Session, Wallet};
use crate::message::errors::MessageError;
use crate::node::events::SendMessage;
use colored::Colorize;
use crate::cli::errors::InputError;
use crate::node::extrinsics::NolikSendMessage;
use blake2::Digest;
use blake2::digest::Update;
use crossterm::{cursor, ExecutableCommand, QueueableCommand};


pub struct IpfsInput {
    pub ipfs_file: IpfsFile,
    pub wallet: Wallet,
}

impl IpfsInput {
    pub fn new(config_file: &ConfigFile, input: &Input, password: Option<String>) -> Result<IpfsInput, InputError> {
        let wallet_alias = match input.get_flag_value(FlagKey::Wallet) {
            Ok(name) => name,
            Err(e) => return Err(e),
        };

        let ipfs_id = match input.get_flag_value(FlagKey::IpfsId) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };

        let wallet = match Wallet::get(&config_file, wallet_alias, password){
            Ok(res) => res,
            Err(_e) => return Err(InputError::CouldNotGetWallet)
        };

        let ipfs_file = IpfsFile::new(ipfs_id);

        Ok(IpfsInput {
            ipfs_file,
            wallet,
        })
    }
}

#[derive(Clone)]
pub struct IpfsFile(pub String);

impl IpfsFile {
    pub fn new(id: String) -> IpfsFile {
        IpfsFile(id)
    }


    pub async fn get(&self) -> Result<Batch, MessageError> {

        let client = IpfsClient::default();
        if let Err(e) = client.bootstrap_add_default().await {
            eprintln!("Error on adding default peers: {:?}", e);
        }

        loop {
            let data = match client.cat(&self.0).map_ok(|chunk| chunk.to_vec()).try_concat().await {
                Ok(res) => {
                    match String::from_utf8(res) {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("Error on getting IPFS data: {:?}", e);
                            return Err(MessageError::CouldNotReadIpfsData)
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error on getting IPFS data: {:?}", e);
                    eprintln!("Trying again...");
                    async_std::task::sleep(Duration::from_secs(2)).await;
                    continue
                }
            };

            let batch: Batch = match toml::from_str(data.as_str()) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Error on parsing IPFS data: {:?}", e);
                    return Err(MessageError::CouldNotReadIpfsData);
                }
            };

            return Ok(batch)
        }
    }


    pub async fn send(&self, config_file: &ConfigFile, sender: &PublicKey, recipients: &Vec<PublicKey>, wallet: &Wallet) -> Result<(), NodeError> {

        let pair = match wallet.get_pair() {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(NodeError::CouldNotSubmitEvent);
            }
        };

        let extrinsic = match NolikSendMessage::new(&config_file, &pair, &sender, &recipients, &self.0) {
            Ok(res) => res,
            Err(_e) => return Err(NodeError::CouldNotSubmitEvent),
        };

        let extrinsic_hash = match extrinsic.hash::<NolikSendMessage>().await {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let event = SendMessage;
        match event.submit(&config_file, &extrinsic_hash).await {
            Ok(_res) => {
                // let to = bs58::encode(recipient).into_string();
                let res = format!("Message has been sent");
                println!("{}", res.bright_green());
                Ok(())
            },
            Err(e) => return Err(e),
        }
    }


    pub async fn save(&self, message_index: u32, account: &Account, index: &mut Index) -> Result<(), MessageError> {

        let output_a = format!("=> Saving new message with ID: {}...", &self.0);
        let mut stdout = stdout();
        stdout.execute(cursor::Hide).unwrap();
        stdout.queue(cursor::SavePosition).unwrap();
        stdout.write_all(output_a.as_bytes()).unwrap();


        let batch = self.get().await?;
        let public_nonce = base64_to_nonce(&batch.nonce)?;
        let broker = base64_to_public_key(&batch.broker)?;


        let decrypted_sessions: Vec<Session> = batch.sessions
            .iter()
            .filter_map(|es| es.decrypt(&public_nonce, &broker, &account.secret).ok())
            .collect();

        if decrypted_sessions.len() == 0 {
            return Err(MessageError::DecryptionError);
        }


        let session = decrypted_sessions.first().unwrap();
        let sender = session.group.get_sender();
        let recipients = session.group.get_recipients();
        let first_recipient = recipients.first().unwrap();


        let position = session.group.0.iter().position(|el| el.0 == account.public);
        if let None = position {
            return Err(MessageError::DecryptionError);
        }

        let (hash_pk, message_pk) = match position.unwrap() {
            0 => (first_recipient, first_recipient),
            _ => (&account.public, &sender),
        };

        let mut parties = blake2::Blake2s256::new();
        Update::update(&mut parties, &sender.as_ref());
        Update::update(&mut parties, &hash_pk.as_ref());
        let parties_hash = base64::encode(parties.finalize().to_vec());

        let encrypted_messages = batch.messages
            .iter()
            .filter(|em| em.parties == parties_hash)
            .collect::<Vec<&EncryptedMessage>>();

        if let Some(em) = encrypted_messages.first() {
            let decrypted_message = em.decrypt(session, &message_pk, &account.secret).unwrap();
            let index_message = IndexMessage::new(&decrypted_message, &account.public, message_index as u32, self.0.clone());

            index.data.messages.push(index_message);
            if let Err(e) = index.save() {
                eprintln!("Error: {}", e);
                return Err(MessageError::CouldNotSaveIndexMessage);
            }
        }

        let ok = String::from("OK");
        println!(" {} Index: {}", ok.bright_green(), message_index);

        Ok(())
    }
}