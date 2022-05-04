use rand::Rng;
use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey, Seed};
use crate::cli::errors::{ConfigError, InputError};
use crate::cli::input::FlagKey;
use crate::{Config, ConfigFile, Input};
use crate::message::errors::MessageError;
use crate::message::utils::{base58_to_public_key, base58_to_secret_key, base58_to_seed};


pub struct AccountInput {
    name: String,
    secret: Option<String>,
}

impl AccountInput {
    pub fn new(input: Input) -> Result<AccountInput, InputError> {

        let name = match input.get_flag_value(FlagKey::Name) {
            Ok(name) => name,
            Err(e) => return Err(e)
        };

        let secret: Option<String> = match input.get_flag_value(FlagKey::Import) {
            Ok(secret) => Some(secret),
            Err(e) => {
                match e {
                    InputError::NoSuchKey => None,
                    _ => return Err(e),
                }
            }
        };

        Ok(AccountInput {
            name,
            secret,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub name: String,
    pub public: PublicKey,
    pub secret: SecretKey,
    pub seed: Seed,
}


impl Account {
    pub fn new(input: AccountInput) -> Result<Account, ConfigError> {
        let account: (PublicKey, SecretKey, Seed) = match input.secret {
            Some(seed) => {
                let mut output: [u8; 32] = [0; 32];
                let seed = match bs58::decode(seed).into(&mut output) {
                    Ok(_) => Seed(output),
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        return Err(ConfigError::CouldNotParseAccountSecretKey)
                    }
                };

                let (pk, sk) = box_::keypair_from_seed(&seed);
                (pk, sk, seed)
            },
            None => {
                let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
                let seed = Seed(random_bytes);
                let (pk, sk) = box_::keypair_from_seed(&seed);
                (pk, sk, seed)
            }
        };

        Ok(Account {
            public: account.0,
            secret: account.1,
            seed: account.2,
            name: input.name
        })
    }

    pub fn add(config_file: ConfigFile, account: Account) -> Result<(), ConfigError> {
        let mut config  = match Config::new(config_file) {
            Ok(config) => config,
            Err(e) => return Err(e),
        };

        let account_output = AccountOutput::serialize(account);

        let same_account_names = config.data.accounts
            .iter()
            .filter(|el| el.name == account_output.name)
            .count();

        if let true = same_account_names > 0 {
            return Err(ConfigError::AccountNameIsNotUnique);
        }

        let same_account_seeds = config.data.accounts
            .iter()
            .filter(|el| el.seed == account_output.seed)
            .count();

        if let true = same_account_seeds > 0 {
            return Err(ConfigError::AccountAlreadyExists);
        }


        config.data.accounts.push(account_output);
        config.save()
    }


    pub fn get(config_file: ConfigFile, key: String) -> Result<Option<Account>, ConfigError>{
        let config  = match Config::new(config_file) {
            Ok(config) => config,
            Err(e) => return Err(e),
        };
        let account_outputs: Vec<AccountOutput> = config.data.accounts
            .iter()
            .filter(|account| vec![&account.name, &account.public].contains(&&key))
            .map(|account| account.to_owned())
            .collect();

        let account = match account_outputs.len() {
            1 => {
                let last_account_output = account_outputs.last().unwrap().to_owned();
                match AccountOutput::deserialize(last_account_output) {
                    Ok(account) => Some(account),
                    Err(_e) => return Err(ConfigError::CouldNotGetAccount),
                }
            }
            _ => None,
        };

        Ok(account)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountOutput {
    pub public: String,
    pub secret: String,
    pub seed: String,
    pub name: String,
}

impl AccountOutput {
    pub fn serialize(account: Account) -> AccountOutput {
        AccountOutput {
            name: account.name,
            public: bs58::encode(&account.public).into_string(),
            secret: bs58::encode(&account.secret).into_string(),
            seed: bs58::encode(&account.seed).into_string(),
        }
    }

    pub fn deserialize(account_output: AccountOutput) -> Result<Account, MessageError> {
        let public = match base58_to_public_key(&account_output.public) {
            Ok(public) => public,
            Err(e) => return Err(e),
        };

        let secret = match base58_to_secret_key(&account_output.secret) {
            Ok(public) => public,
            Err(e) => return Err(e),
        };

        let seed = match base58_to_seed(&account_output.secret) {
            Ok(public) => public,
            Err(e) => return Err(e),
        };

        Ok(Account {
            name: account_output.name,
            public,
            secret,
            seed,
        })
    }
}