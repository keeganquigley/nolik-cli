use sodiumoxide::crypto::box_::PublicKey;
use crate::{Account, ConfigFile, FlagKey, Input, NodeError, NodeEvent, Wallet};
use crate::cli::errors::InputError;
use crate::node::events::AddToBlacklist;
// use crate::node::extrinsics::add_to_blacklist;
use colored::Colorize;
use crate::node::extrinsics::NolikAddToBlacklist;


pub struct Blacklist {
    wallet: Wallet,
    account: Account,
    new_address: PublicKey,
}


impl Blacklist {
    pub fn new(input: &Input, config_file: &ConfigFile, password: Option<String>) -> Result<Blacklist, InputError>{
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

        let account_key = match input.get_flag_value(FlagKey::For) {
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

        let new_address = match input.get_flag_value(FlagKey::Add) {
            Ok(key) => key,
            Err(e) => return Err(e),
        };

        let mut output: [u8; 32] = [0; 32];
        let public_key = match bs58::decode(new_address).into(&mut output) {
            Ok(_) => match PublicKey::from_slice(output.as_slice()) {
                Some(pk) => pk,
                None => return Err(InputError::CouldNotUpdateBlacklist),
            },
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(InputError::CouldNotUpdateBlacklist)
            }
        };

        Ok(Blacklist {
            wallet,
            account,
            new_address: public_key,
        })
    }

    pub async fn update(&self, config_file: &ConfigFile) -> Result<(), NodeError> {
        let pair = match self.wallet.get_pair() {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(NodeError::CouldNotSubmitEvent);
            }
        };

        let extrinsic = match NolikAddToBlacklist::new(&config_file, &pair, &self.account.public, &self.new_address) {
            Ok(res) => res,
            Err(_e) => return Err(NodeError::CouldNotSubmitEvent),
        };

        let extrinsic_hash = match extrinsic.hash::<NolikAddToBlacklist>().await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(e);
            }
        };

        let event = AddToBlacklist;
        match event.submit(config_file, &extrinsic_hash).await {
            Ok(_res) => {
                let new_address = bs58::encode(self.new_address).into_string();
                let res = format!("Blacklist for \"{}\" has been successfully updated. Added new address: \"{}\"", self.account.alias, new_address);
                println!("{}", res.bright_green());
                Ok(())
            },
            Err(e) => return Err(e),
        }
    }
}

