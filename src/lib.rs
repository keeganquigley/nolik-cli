extern crate core;

pub mod cli;
pub mod node;
pub mod wallet;
pub mod account;
pub mod message;
pub mod owner;
pub mod whitelist;
pub mod blacklist;

use std::error::Error;
use blake2::Digest;
use blake2::digest::Update;
use colored::Colorize;
use sodiumoxide::crypto::box_;

use wallet::Wallet;
use crate::account::{Account, AccountInput, AccountOutput};
use crate::cli::config::{Config, ConfigFile};
use crate::cli::input::{Command, FlagKey, Input};
use crate::message::input::BatchInput;
use crate::owner::Owner;
use crate::wallet::WalletInput;
use crate::node::errors::NodeError;
use crate::node::socket::Socket;
use sp_core::crypto::AccountId32;
use sp_keyring::AccountKeyring;
use crate::blacklist::Blacklist;
use crate::cli::errors::ConfigError;

use crate::message::batch::Batch;
use crate::message::errors::MessageError;
use crate::message::index::{Index, IndexFile, IndexMessage};
use crate::message::ipfs::{IpfsFile, IpfsInput};
use crate::message::message::EncryptedMessage;
use crate::message::session::Session;
use crate::message::utils::{base64_to_nonce, base64_to_public_key};
use crate::node::events::{BalanceTransferEvent, NodeEvent};
use crate::node::extrinsics::BalancesTransfer;
use crate::whitelist::Whitelist;



#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}


pub async fn run(mut input: Input) -> Result<(), Box<dyn Error>> {
    match input.command {
        Command::AddWallet => {
            let password = match Wallet::password_input_repeat() {
                Ok(password) => Some(password),
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let wallet_input = match WalletInput::new(&input, password) {
                Ok(input) => input,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let wallet = match Wallet::new(wallet_input) {
                Ok(wallet) => wallet,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let config_file = ConfigFile::new();
            if let Err(e) = Wallet::add(&config_file, &wallet) {
                return Err(Box::<dyn Error>::from(e));
            }
        }
        Command::AddAccount => {
            let account_input = match AccountInput::new(input) {
                Ok(input) => input,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let account = match Account::new(account_input) {
                Ok(account) => account,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let config_file = ConfigFile::new();
            if let Err(e) = Account::add(&config_file, &account) {
                return Err(Box::<dyn Error>::from(e));
            }
        },

        Command::AddOwner => {
            let config_file: ConfigFile = ConfigFile::new();

            let password = match Wallet::password_input_once() {
                Ok(password) => Some(password),
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let owner = match Owner::new(&input, &config_file, password) {
                Ok(account) => account,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            match owner.add(&config_file).await {
                Ok(_res) => {},
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            }
        },
        Command::UpdateWhitelist => {
            let config_file: ConfigFile = ConfigFile::new();

            let password = match Wallet::password_input_once() {
                Ok(password) => Some(password),
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let whitelist = match Whitelist::new(&input, &config_file, password) {
                Ok(whitelist) => whitelist,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            match whitelist.update(&config_file).await {
                Ok(_res) => {},
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            }
        },
        Command::UpdateBlacklist => {
            let config_file: ConfigFile = ConfigFile::new();

            let password = match Wallet::password_input_once() {
                Ok(password) => Some(password),
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let blacklist = match Blacklist::new(&input, &config_file, password) {
                Ok(blacklist) => blacklist,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            match blacklist.update(&config_file).await {
                Ok(_res) => {},
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            }
        },
        Command::ComposeMessage => {
            let config_file: ConfigFile = ConfigFile::new();

            let bi = match BatchInput::new(&mut input, &config_file) {
                Ok(input) => input,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let secret_nonce = box_::gen_nonce();

            let batch = match Batch::new(&bi, &secret_nonce) {
                Ok(batch) => batch,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            if let Err(e) = batch.save().await {
                return Err(Box::<dyn Error>::from(e));
            }
        },
        Command::SendMessage => {
            let config_file: ConfigFile = ConfigFile::new();

            let password = match Wallet::password_input_once() {
                Ok(password) => Some(password),
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let ipfs_input = match IpfsInput::new(&config_file, &input, password) {
                Ok(res) => res,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let batch = match ipfs_input.ipfs_file.get().await {
                Ok(res) => res,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let (sender, recipients) = match batch.parties(&config_file) {
                Ok(res) => res,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            if let Err(e) = ipfs_input.ipfs_file.send(&config_file, &sender, &recipients, &ipfs_input.wallet).await {
                return Err(Box::<dyn Error>::from(e));
            }
        },
        Command::GetMessages => {
            let config_file: ConfigFile = ConfigFile::new();

            let account_key = match input.get_flag_value(FlagKey::Account) {
                Ok(key) => key,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let account = match Account::get(&config_file, account_key) {
                Ok(res) => res,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };


            println!("Checking for new messages...");
            let chain_index = match account.index(&config_file).await {
                Ok(res) => match res {
                    Some(index) => index,
                    None => 0
                },
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };


            let index_file = IndexFile::new();
            let mut index_messages = match Index::new(&index_file) {
                Ok(res) => res,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let received_messages : Vec<&IndexMessage>= index_messages.data.messages
                .iter()
                .filter(|m| m.public == bs58::encode(account.public).into_string())
                .map(|m| m)
                .collect();

            let last_received_message_index = match received_messages.last() {
                Some(res) => res.index,
                None => 0,
            };

            if chain_index.eq(&last_received_message_index) {
                println!("No new messages");
            }

            let diff = chain_index - last_received_message_index;
            let output = format!("You have {} new message(s)", diff);
            println!("{}", output.bright_green());


            let mut hashes: Vec<(String, u32)> = Vec::new();
            for i in last_received_message_index..chain_index {
                let index = i + 1;
                let ipfs_hash = match account.message(&config_file, index).await {
                    Ok(res) => res,
                    Err(e) => return Err(Box::<dyn Error>::from(e)),
                };

                if let Some(hash) = ipfs_hash {
                    hashes.push((hash, index));
                }
            }

            for (hash, index) in hashes {

                let ipfs_file = IpfsFile::new(hash);
                let ipfs_data = match ipfs_file.get().await {
                    Ok(batch) => batch,
                    Err(e) => return Err(Box::<dyn Error>::from(e)),
                };


                let public_nonce = match base64_to_nonce(&ipfs_data.nonce) {
                    Ok(res) => res,
                    Err(e) => return Err(Box::<dyn Error>::from(e)),
                };

                let broker = match base64_to_public_key(&ipfs_data.broker) {
                    Ok(res) => res,
                    Err(e) => return Err(Box::<dyn Error>::from(e)),
                };


                let decrypted_sessions: Vec<Session> = ipfs_data.sessions
                    .iter()
                    .filter_map(|es| es.decrypt(&public_nonce, &broker, &account.secret).ok())
                    .collect();

                if decrypted_sessions.len() == 0 {
                    return Err(Box::<dyn Error>::from(MessageError::DecryptionError));
                }


                let session = decrypted_sessions.first().unwrap();
                let sender = session.group.get_sender();
                let recipients = session.group.get_recipients();
                let first_recipient = recipients.first().unwrap();


                let position = session.group.0.iter().position(|el| el.0 == account.public);
                if let None = position {
                    return Err(Box::<dyn Error>::from(MessageError::DecryptionError));
                }

                let (hash_pk, message_pk) = match position.unwrap() {
                    0 => (first_recipient, first_recipient),
                    _ => (&account.public, &sender),
                };

                let mut parties = blake2::Blake2s256::new();
                Update::update(&mut parties, &sender.as_ref());
                Update::update(&mut parties, &hash_pk.as_ref());
                let parties_hash = base64::encode(parties.finalize().to_vec());

                let encrypted_messages = ipfs_data.messages
                    .iter()
                    .filter(|em| em.parties == parties_hash)
                    .collect::<Vec<&EncryptedMessage>>();

                if let Some(em) = encrypted_messages.first() {
                    let decrypted_message = em.decrypt(session, &message_pk, &account.secret).unwrap();
                    let index_message = IndexMessage::new(&decrypted_message, &account.public, index as u32, ipfs_file.0);

                    index_messages.data.messages.push(index_message);
                    if let Err(e) = index_messages.save() {
                        return Err(Box::<dyn Error>::from(e));
                    }

                    println!("   ✅ Saved new message with Index: {}", index);
                }
            }
        },
        Command::GetCoins => {
            let config_file: ConfigFile = ConfigFile::new();

            let wallet_key = match input.get_flag_value(FlagKey::Wallet) {
                Ok(key) => key,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let password = match Wallet::password_input_once() {
                Ok(password) => Some(password),
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let wallet = match Wallet::get(&config_file, wallet_key, password) {
                Ok(wallet) => wallet,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let sender = AccountKeyring::Alice;
            let recipient = AccountId32::from(wallet.public);

            let extrinsic = match BalancesTransfer::new(&config_file, &sender, &recipient) {
                Ok(res) => res,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let extrinsic_hash = match extrinsic.hash::<BalancesTransfer>().await {
                Ok(res) => res,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            // println!("EH {}", extrinsic_hash);

            let event = BalanceTransferEvent;
            match event.submit(&config_file, &extrinsic_hash).await {
                Ok(_res) => {
                    let res = format!("Coins have been transferred to {:?}", recipient);
                    println!("{}", res.bright_green());
                },
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            }
        }
    }
    Ok(())
}
