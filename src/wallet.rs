use sp_core::{Pair, sr25519};
use crate::config::errors::ConfigError;
use serde_derive::{Serialize, Deserialize};
use rpassword;
use crate::{FlagKey, Input, InputError};


pub struct WalletInput {
    name: String,
    phrase: Option<String>,
    password: Option<String>,
}

impl WalletInput {
    pub fn new(input: Input) -> Result<WalletInput, InputError> {

        let name = match input.get_flag_value(FlagKey::Name) {
            Ok(name) => name,
            Err(e) => return Err(e)
        };

        let phrase: Option<String> = match input.get_flag_value(FlagKey::Import) {
            Ok(bs58seed) => Some(bs58seed),
            Err(e) => {
                match e {
                    InputError::NoSuchKey => None,
                    _ => return Err(e),
                }
            }
        };

        let with_password: Option<bool> = match input.get_flag_value(FlagKey::WithPassword) {
            Ok(value) => {
                match value.as_str() {
                    "no" => None,
                    _ => Some(true),
                }
            },
            Err(e) => {
                match e {
                    InputError::NoSuchKey => Some(true),
                    _ => return Err(e),
                }
            }
        };

        let password = match with_password {
            Some(_value) =>
                match Wallet::password() {
                    Ok(password) => Some(password),
                    Err(e) => return Err(e),
                },
            None => None
        };

        Ok(WalletInput {
            name,
            phrase,
            password
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub public: String,
    pub seed: String,
    pub bs58seed: String,
    pub name: String,
}


impl Wallet {
    pub fn new(input: WalletInput) -> Result<Wallet, ConfigError> {

        let wallet = match input.phrase {
            Some(bs58seed) => {
                let decoded_vec = match bs58::decode(bs58seed).into_vec() {
                    Ok(vec) => vec,
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        return Err(ConfigError::CouldNotParseSeed)
                    }
                };

                let phrase = match String::from_utf8(decoded_vec) {
                    Ok(phrase) => phrase,
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        return Err(ConfigError::CouldNotParseSeed)
                    }
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
            public: wallet.0.public().to_string(),
            seed: wallet.1.clone(),
            bs58seed: bs58::encode(wallet.1).into_string(),
            name: input.name
        })
    }

    pub fn delete(_wallet: Wallet) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn password() -> Result<String, InputError> {
        let password = rpassword::prompt_password("Your wallet password: ").unwrap();
        let password_again = rpassword::prompt_password("Please type your wallet password again").unwrap();
        match password.eq(&password_again) {
            true => Ok(password),
            false => return Err(InputError::PasswordsDoNotMatch)
        }
    }
}