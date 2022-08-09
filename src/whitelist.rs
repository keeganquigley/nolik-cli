use sodiumoxide::crypto::box_::PublicKey;
use sp_core::Pair;
use crate::{Account, ConfigFile, FlagKey, Input, NodeEvent, Wallet};
use crate::cli::errors::InputError;
use crate::node::events::AddToWhitelist;
use crate::node::extrinsics::add_to_whitelist;
use colored::Colorize;


pub struct Whitelist {
    wallet: Wallet,
    account: Account,
    new_address: PublicKey,
}


impl Whitelist {
    pub fn new(input: &Input, config_file: &ConfigFile, password: Option<String>) -> Result<Whitelist, InputError>{
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
                None => return Err(InputError::CouldNotUpdateWhitelist),
            },
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(InputError::CouldNotUpdateWhitelist)
            }
        };

        Ok(Whitelist {
            wallet,
            account,
            new_address: public_key,
        })
    }

    pub async fn update(&self) -> Result<(), InputError> {
        let (pair, _seed) = match sp_core::sr25519::Pair::from_phrase(&self.wallet.seed, self.wallet.password.as_deref()) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(InputError::CouldNotUpdateWhitelist);
            }
        };

        let extrinsic_hash = match add_to_whitelist(&pair, &self.account.public, &self.new_address).await {
            Ok(hash) => hash,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(InputError::CouldNotUpdateWhitelist);
            }
        };

        let event = AddToWhitelist;
        match event.submit(&extrinsic_hash).await {
            Ok(_res) => {
                let new_address = bs58::encode(self.new_address).into_string();
                let res = format!("Whitelist for \"{}\" has been successfully updated. Added new address: \"{}\"", self.account.alias, new_address);
                println!("{}", res.bright_green());
                Ok(())
            },
            Err(_e) => return Err(InputError::CouldNotUpdateWhitelist),
        }
    }
}

