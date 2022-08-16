use crate::{Account, ConfigFile, FlagKey, Input, Wallet};
use crate::cli::errors::InputError;
use crate::node::extrinsics::add_owner;
use crate::node::events::{AddOwnerEvent, NodeEvent};
use colored::Colorize;

pub struct Owner {
    pub wallet: Wallet,
    pub account: Account,
}


impl Owner {
    pub fn new(input: &Input, config_file: &ConfigFile, password: Option<String>) -> Result<Owner, InputError> {
        let wallet_key = match input.get_flag_value(FlagKey::Wallet) {
            Ok(key) => key,
            Err(e) => return Err(e),
        };

        let wallet = match Wallet::get(&config_file, wallet_key, password) {
            Ok(wallet) => wallet,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(InputError::CouldNotAddOwner);
            }
        };

        let account_key = match input.get_flag_value(FlagKey::Account) {
            Ok(key) => key,
            Err(e) => return Err(e),
        };

        let account = match Account::get(&config_file, account_key) {
            Ok(account) => account,
            Err (e) => {
                eprintln!("Error: {}", e);
                return Err(InputError::CouldNotAddOwner);
            }
        };

        Ok(Owner {
            wallet,
            account,
        })
    }


    pub async fn add(&self) -> Result<(), InputError> {
        let pair = match self.wallet.get_pair() {
            Ok(pair) => pair,
            Err(_e) => return Err(InputError::CouldNotAddOwner),
        };

        let extrinsic_hash = match add_owner(&pair, &self.account.public).await {
            Ok(hash) => hash,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(InputError::CouldNotAddOwner);
            }
        };

        let event = AddOwnerEvent;
        match event.submit(&extrinsic_hash).await {
            Ok(_res) => {
                let res = format!("Owner has been successfully added to \"{}\"", self.account.alias);
                println!("{}", res.bright_green());
                Ok(())
            },
            Err(_e) => return Err(InputError::CouldNotAddOwner),
        }
    }
}
