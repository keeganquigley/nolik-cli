pub mod inputs;
mod rpc;
pub mod wallet;
pub mod config;

use std::error::Error;
use inputs::{
    Input,
    Command,
    FlagKey,
    errors::InputError,
};
use wallet::Wallet;
use crate::config::{Config, ConfigFile};
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
        Command::DeleteWallet => {}
        Command::AddAccount => {}
        Command::DeleteAccount => {}
    }
    Ok(())
}


