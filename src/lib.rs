extern crate core;

pub mod cli;
pub mod node;
pub mod wallet;
pub mod account;
pub mod message;
pub mod send;
pub mod owner;

use std::error::Error;

use wallet::Wallet;
use crate::account::{Account, AccountInput};
use crate::cli::config::{Config, ConfigFile};
use crate::cli::input::{Command, FlagKey, Input};
use crate::message::input::MessageInput;
use crate::owner::Owner;
use crate::wallet::WalletInput;
use crate::node::errors::NodeError;
use crate::node::socket::Socket;
use futures::StreamExt;
use sp_core::crypto::AccountId32;
use sp_keyring::AccountKeyring;
use subxt::{ClientBuilder, DefaultConfig, PairSigner, PolkadotExtrinsicParams};
use num_format::{CustomFormat, Grouping};
use subxt::events::FilteredEventDetails;


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
        Command::SendMessage => {
            let config_file: ConfigFile = ConfigFile::new();

            let mi = match MessageInput::new(&mut input, &config_file) {
                Ok(input) => input,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let wallet_alias = match input.get_flag_value(FlagKey::Wallet) {
                Ok(name) => name,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let password = match Wallet::password_input_once() {
                Ok(password) => Some(password),
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let wallet = match Wallet::get(&config_file, wallet_alias, password) {
                Ok(wallet) => wallet,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            for pk in &mi.recipients {
                let em= match mi.encrypt(&pk) {
                    Ok(em) => em,
                    Err(e) => return Err(Box::<dyn Error>::from(e)),
                };

                let ipfs_file = match em.save().await {
                    Ok(ipfs_file) => ipfs_file,
                    Err(e) => return Err(Box::<dyn Error>::from(e)),
                };

                match ipfs_file.send(&wallet, &mi.sender.public, &pk).await {
                    Ok(_res) => {}
                    Err(e) => return Err(Box::<dyn Error>::from(e)),
                }
            }
        },
        Command::GetMessages => {
            // let data = "0x0c00000000000000585f8f090000000002000000010000000508ba6ef6480a2c4173bbbf0c27355c0e3c6d7a874a21cc2464e97d3fd5d92e245bd859730700000000000000000000000000000100000000010308011027000000000000000000";
            // let data = "0x1000000000000000585f8f090000000002000000010000000508ba6ef6480a2c4173bbbf0c27355c0e3c6d7a874a21cc2464e97d3fd5d92e245bd85973070000000000000000000000000000010000000800b047713578643563363277346672794a7838706f5965786f424a4179394a55706a697239765234714d4446367aba6ef6480a2c4173bbbf0c27355c0e3c6d7a874a21cc2464e97d3fd5d92e245b00000100000000001027000000000000000000";
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

            let sender = PairSigner::new(AccountKeyring::Alice.pair());
            let recipient = AccountId32::from(wallet.public).into();

            println!("Sending coins...");
            let api = ClientBuilder::new()
                .build()
                .await?
                .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>>();



            let mut transfer_events = api
                .events()
                .subscribe()
                .await?
                .filter_events::<(polkadot::balances::events::Transfer,)>();

            api.tx()
                .balances()
                .transfer(recipient, 1_000_000_000)
                .expect("Compatible transfer call on runtime node")
                .sign_and_submit_default(&sender)
                .await
                .unwrap();

            while let Some(transfer_event) = transfer_events.next().await {
                clearscreen::clear().expect("failed to clear screen");
                println!("Coins have been sent");

                let details = FilteredEventDetails::from(transfer_event.unwrap());
                println!("==> Block hash: {:?}", details.block_hash);
                println!("==> From: {:?}", details.event.from);
                println!("==> To: {:?}", details.event.to);

                let format = CustomFormat::builder()
                    .grouping(Grouping::Standard)
                    .minus_sign("-")
                    .separator("_")
                    .build()?;
                let mut amount = num_format::Buffer::new();
                amount.write_formatted(&details.event.amount, &format);
                println!("==> Amount: {:?}", amount);
                break;
            }
        }
    }
    Ok(())
}
