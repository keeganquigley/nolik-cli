use crate::{Account, ConfigFile, FlagKey, Input, NodeError, Wallet};
use crate::cli::errors::InputError;
use crate::node::events::{AddOwnerEvent, NodeEvent};
use colored::Colorize;
use crate::node::extrinsics::NolikAddOwner;

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


    pub async fn add(&self, config_file: &ConfigFile) -> Result<(), NodeError> {
        let pair = match self.wallet.get_pair() {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(NodeError::CouldNotSubmitEvent);
            }
        };

        let extrinsic = match NolikAddOwner::new(&config_file, &pair, &self.account.public) {
            Ok(res) => res,
            Err(_e) => return Err(NodeError::CouldNotSubmitEvent),
        };

        let extrinsic_hash = match extrinsic.hash::<NolikAddOwner>().await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(e);
            }
        };

        let event = AddOwnerEvent;
        match event.submit(config_file, &extrinsic_hash).await {
            Ok(_res) => {
                let res = format!("Owner has been successfully added to \"{}\"", self.account.alias);
                println!("{}", res.bright_green());
                Ok(())
            },
            Err(e) => return Err(e),
        }
    }
}
