pub mod cli;
mod rpc;
pub mod wallet;
pub mod account;
pub mod message;

use std::error::Error;
use wallet::Wallet;
use crate::account::{Account, AccountInput};
use crate::cli::config::{Config, ConfigFile};
use crate::cli::input::{Command, Input};
use crate::wallet::WalletInput;



pub fn run(input: Input) -> Result<(), Box<dyn Error>> {
    match input.command {
        Command::AddWallet => {
            let wallet_input = match WalletInput::new(input) {
                Ok(input) => input,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let wallet = match Wallet::new(wallet_input) {
                Ok(wallet) => wallet,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            let config_file = ConfigFile::new();
            let mut config = match Config::new(config_file) {
                Ok(config) => config,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            if let Err(e) = config.add_wallet(wallet) {
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
            let mut config = match Config::new(config_file) {
                Ok(config) => config,
                Err(e) => return Err(Box::<dyn Error>::from(e)),
            };

            if let Err(e) = config.add_account(account) {
                return Err(Box::<dyn Error>::from(e));
            }
        },
        Command::ComposeMessage => {

        }
    }
    Ok(())
}


