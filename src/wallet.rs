use sp_core::{Pair, sr25519};
use serde_derive::{Serialize, Deserialize};
use rpassword;
use sp_core::sr25519::Public;
use crate::cli::errors::{ConfigError, InputError};
use crate::cli::input::FlagKey;
use crate::{Config, ConfigFile, Input};
use clearscreen;
use colored::Colorize;


#[derive(Debug)]
pub struct WalletInput {
    alias: String,
    bs58seed: Option<String>,
    password: Option<String>,
}

impl WalletInput {
    pub fn new(input: &Input, password: Option<String>) -> Result<WalletInput, InputError> {

        let alias = match input.get_flag_value(FlagKey::Alias) {
            Ok(name) => name,
            Err(e) => return Err(e),
        };

        let bs58seed: Option<String> = match input.get_flag_value(FlagKey::Import) {
            Ok(bs58seed) => Some(bs58seed),
            Err(e) => {
                match e {
                    InputError::NoSuchKey => None,
                    _ => return Err(e),
                }
            }
        };

        Ok(WalletInput {
            alias,
            bs58seed,
            password
        })
    }
}


#[derive(Clone, Debug)]
pub struct Wallet {
    pub alias: String,
    pub public: Public,
    pub seed: String,
    pub bs58seed: String,
    pub password: Option<String>,
}


impl Wallet {
    pub fn new(input: WalletInput) -> Result<Wallet, ConfigError> {
        let wallet = match input.bs58seed {
            Some(bs58seed) => {
                let phrase = match Self::bs58seed_to_phrase(&bs58seed) {
                    Ok(res) => res,
                    Err(_e) => return Err(ConfigError::CouldNotParseSeed),
                };

                match sr25519::Pair::from_phrase(&phrase, input.password.as_deref()) {
                    Ok(res) => (res.0, phrase, res.1),
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        return Err(ConfigError::CouldNotParseSeed)
                    },
                }
            },
            None => sr25519::Pair::generate_with_phrase(input.password.as_deref())
        };

        Ok(Wallet {
            public: wallet.0.public(),
            bs58seed: bs58::encode(&wallet.1).into_string(),
            seed: wallet.1,
            alias: input.alias,
            password: input.password,
        })
    }


    fn bs58seed_to_phrase(bs58seed: &String) -> Result<String, ConfigError> {
        let phrase_vec = match bs58::decode(&bs58seed).into_vec() {
            Ok(res) => res,
            Err(_e) => return Err(ConfigError::CouldNotParseSeed),
        };

        let phrase = match String::from_utf8(phrase_vec) {
            Ok(res) => res,
            Err(_e) => return Err(ConfigError::CouldNotParseSeed),
        };

        Ok(phrase)
    }


    pub fn get_pair(&self) -> Result<sr25519::Pair, ConfigError> {
        let phrase = match Self::bs58seed_to_phrase(&self.bs58seed) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let pair = match sr25519::Pair::from_phrase(&phrase, self.password.as_deref()) {
            Ok(res) => res.0,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(ConfigError::CouldNotParseSeed)
            },
        };
        Ok(pair)
    }


    pub fn add(config_file: &ConfigFile, wallet: &Wallet) -> Result<(), ConfigError> {
        let mut config= match Config::new(&config_file) {
            Ok(config) => config,
            Err(e) => return Err(e),
        };

        let same_wallet_names = config.data.wallets
            .iter()
            .filter(|el| el.alias == wallet.alias)
            .count();

        if let true = same_wallet_names > 0 {
            return Err(ConfigError::WalletNameIsNotUnique);
        }

        let same_wallet_seed_phrases = config.data.wallets
            .iter()
            .filter(|el| el.bs58seed == wallet.bs58seed)
            .count();

        if let true = same_wallet_seed_phrases > 0 {
            return Err(ConfigError::WalletAlreadyExists);
        }

        config.data.wallets.push(WalletOutput::new(wallet));
        match config.save() {
            Ok(_) => {
                let res = format!("Wallet \"{}\" has been created", wallet.alias);
                println!("{}", res.bright_green());
                Ok(())
            },
            Err(e) => return Err(e),
        }
    }


    pub fn get(config_file: &ConfigFile, key: String, password: Option<String>) -> Result<Wallet, ConfigError> {
        let config= match Config::new(&config_file) {
            Ok(config) => config,
            Err(e) => return Err(e),
        };

        let same_wallet_names: Vec<WalletOutput> = config.data.wallets
            .iter()
            .filter(|el| vec![el.alias.clone(), el.public.clone()].contains(&key))
            .map(|el| el.clone())
            .collect();

        if let true = same_wallet_names.len() == 0 {
            return Err(ConfigError::CouldNotGetWallet);
        }

        let wallet_output = same_wallet_names.last().unwrap().to_owned();

        let wallet_input = WalletInput {
            alias: wallet_output.alias,
            bs58seed: Some(wallet_output.bs58seed),
            password
        };

        let wallet = match Wallet::new(wallet_input) {
            Ok(wallet) => wallet,
            Err(e) => return Err(e),
        };

        Ok(wallet)
    }


    pub fn password_input_repeat() -> Result<String, InputError> {
        clearscreen::clear().expect("failed to clear screen");
        let password = match rpassword::prompt_password("Your wallet password") {
            Ok(input) => input,
            Err(_e) => return Err(InputError::PasswordInputError),
        };

        let password_again = match rpassword::prompt_password("Your wallet password again") {
            Ok(input) => input,
            Err(_e) => return Err(InputError::PasswordInputError),
        };

        match password.eq(&password_again) {
            true => {
                clearscreen::clear().expect("failed to clear screen");
                Ok(password)
            },
            false => return Err(InputError::PasswordsDoNotMatch)
        }
    }


    pub fn password_input_once() -> Result<String, InputError> {
        clearscreen::clear().expect("failed to clear screen");
        let password = match rpassword::prompt_password("Your wallet password") {
            Ok(input) => input,
            Err(_e) => return Err(InputError::PasswordInputError),
        };

        clearscreen::clear().expect("failed to clear screen");
        Ok(password)
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletOutput {
    pub alias: String,
    pub public: String,
    pub seed: String,
    pub bs58seed: String,
}


impl WalletOutput {
    pub fn new(wallet: &Wallet) -> WalletOutput {
        WalletOutput {
            alias: wallet.alias.clone(),
            public: wallet.public.to_string(),
            seed: wallet.seed.clone(),
            bs58seed: wallet.bs58seed.clone(),
        }
    }
}