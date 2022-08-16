extern crate core;

pub mod cli;
pub mod node;
pub mod wallet;
pub mod account;
pub mod message;
pub mod send;
pub mod owner;
pub mod whitelist;

use std::error::Error;
use colored::Colorize;

use wallet::Wallet;
use crate::account::{Account, AccountInput};
use crate::cli::config::{Config, ConfigFile};
use crate::cli::input::{Command, FlagKey, Input};
use crate::message::input::MessageInput;
use crate::owner::Owner;
use crate::wallet::WalletInput;
use crate::node::errors::NodeError;
use crate::node::socket::Socket;
use sp_core::crypto::AccountId32;
use sp_keyring::AccountKeyring;
use crate::message::batch::Batch;
use crate::node::events::{BalanceTransferEvent, NodeEvent};
use crate::node::extrinsics::balance_transfer;
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

            let wallet_input = match WalletInput::new(input, password) {
                Ok(input) => input,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let wallet = match Wallet::new(wallet_input) {
                Ok(wallet) => wallet,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let config_file = ConfigFile::new();
            if let Err(e) = Wallet::add(config_file, wallet) {
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
            if let Err(e) = Account::add(&config_file, account) {
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

            match owner.add().await {
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

            match whitelist.update().await {
                Ok(_res) => {},
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            }
        },
        Command::ComposeMessgae => {
            let config_file: ConfigFile = ConfigFile::new();

            let mi = match MessageInput::new(&mut input, &config_file) {
                Ok(input) => input,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };


            // let em= match mi.encrypt(&pk) {
            //     Ok(em) => em,
            //     Err(e) => return Err(Box::<dyn Error>::from(e)),
            // };

            let _batch = match Batch::new(&mi) {
                Ok(batch) => batch,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            // for pk in &mi.recipients {
                // let em= match mi.encrypt(&pk) {
                //     Ok(em) => em,
                //     Err(e) => return Err(Box::<dyn Error>::from(e)),
                // };

                // let ipfs_file = match em.save().await {
                //     Ok(ipfs_file) => ipfs_file,
                //     Err(e) => return Err(Box::<dyn Error>::from(e)),
                // };
                //
                // match ipfs_file.send(&wallet, &mi.sender.public, &pk).await {
                //     Ok(_res) => {}
                //     Err(e) => return Err(Box::<dyn Error>::from(e)),
                // }
            // }
        },
        Command::SendMessage => {
            let config_file: ConfigFile = ConfigFile::new();

            // let mi = match MessageInput::new(&mut input, &config_file) {
            //     Ok(input) => input,
            //     Err(e) => return Err(Box::<dyn Error>::from(e)),
            // };

            let wallet_alias = match input.get_flag_value(FlagKey::Wallet) {
                Ok(name) => name,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let _ipfs_id = match input.get_flag_value(FlagKey::IpfsId) {
                Ok(id) => id,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let password = match Wallet::password_input_once() {
                Ok(password) => Some(password),
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let _wallet = match Wallet::get(&config_file, wallet_alias, password) {
                Ok(wallet) => wallet,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            // for pk in &mi.recipients {
            //     let em= match mi.encrypt(&pk) {
            //         Ok(em) => em,
            //         Err(e) => return Err(Box::<dyn Error>::from(e)),
            //     };
            //
            //     let ipfs_file = match em.save().await {
            //         Ok(ipfs_file) => ipfs_file,
            //         Err(e) => return Err(Box::<dyn Error>::from(e)),
            //     };
            //
            //     match ipfs_file.send(&wallet, &mi.sender.public, &pk).await {
            //         Ok(_res) => {}
            //         Err(e) => return Err(Box::<dyn Error>::from(e)),
            //     }
            // }
        },
        Command::GetMessages => {

            let bs58seed = "4ecF8kHC5xfAf6FLNKkc1KnQk6KAXwub1HbpZE7Xe6nhhneHzNb8rDxCSk3r8zC1VHjE5b8EcGDtN9WXxxEJyuWh4XN5r8oxpgjQiUu7hTT";
            let phrase_vec = bs58::decode(&bs58seed).into_vec().unwrap();
            let phrase = String::from_utf8(phrase_vec).unwrap();
            println!("PHRASE {phrase}");
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

            let extrinsic_hash = match balance_transfer(sender, &recipient).await {
                Ok(hash) => hash,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let event = BalanceTransferEvent;
            match event.submit(&extrinsic_hash).await {
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
