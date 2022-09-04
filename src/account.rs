use rand::Rng;
use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey, Seed};
use crate::cli::errors::{ConfigError, InputError};
use crate::cli::input::FlagKey;
use crate::{Config, ConfigFile, Input, NodeError, Socket};
use crate::message::errors::MessageError;
use crate::message::utils::{base58_to_public_key, base58_to_secret_key, base58_to_seed, hash_address};
use colored::Colorize;
use parity_scale_codec::Encode;
use sp_core::{twox_128, twox_64};
use crate::node::calls::get_storage_value;


pub struct AccountInput {
    alias: String,
    secret: Option<String>,
}

impl AccountInput {
    pub fn new(input: Input) -> Result<AccountInput, InputError> {

        let alias = match input.get_flag_value(FlagKey::Alias) {
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
            alias,
            secret,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub alias: String,
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
            alias: input.alias,
        })
    }

    pub fn add(config_file: &ConfigFile, account: &Account) -> Result<(), ConfigError> {
        let mut config  = match Config::new(&config_file) {
            Ok(config) => config,
            Err(e) => return Err(e),
        };

        let account_output = AccountOutput::serialize(account);

        let same_account_names = config.data.accounts
            .iter()
            .filter(|el| el.alias == account_output.alias)
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
        match config.save() {
            Ok(_) => {
                let res = format!("Account \"{}\" has been created", account.alias);
                println!("{}", res.bright_green());
                Ok(())
            },
            Err(e) => return Err(e),
        }
    }


    pub fn get(config_file: &ConfigFile, key: String) -> Result<Account, ConfigError>{
        let config  = match Config::new(&config_file) {
            Ok(config) => config,
            Err(e) => return Err(e),
        };

        let account_outputs: Vec<AccountOutput> = config.data.accounts
            .iter()
            .filter(|account| vec![&account.alias, &account.public].contains(&&key))
            .map(|account| account.to_owned())
            .collect();

        let account = match account_outputs.len() {
            1 => {
                let last_account_output = account_outputs.last().unwrap().to_owned();
                match AccountOutput::deserialize(&last_account_output) {
                    Ok(account) => account,
                    Err(_e) => return Err(ConfigError::CouldNotGetAccount),
                }
            }
            _ => return Err(ConfigError::CouldNotGetAccount),
        };


        Ok(account)
    }


    pub async fn index(&self, config_file: &ConfigFile) -> Result<Option<u32>, NodeError> {

        let config = match Config::new(&config_file) {
            Ok(res) => res,
            Err(_e) => return Err(NodeError::CouldNotGetAccountIndex),
        };

        let node_url = String::from(config.data.url);

        let address = hash_address(&self.public);

        let module = twox_128("Nolik".as_bytes());
        let module_hex = hex::encode(module);

        let method = twox_128("MessagesCount".as_bytes());
        let method_hex = hex::encode(method);

        let bytes = address.as_bytes();
        let twox_64 = twox_64(bytes.encode().as_ref());
        let twox_64_concat: Vec<u8> = twox_64
            .iter()
            .chain::<&[u8]>(bytes.encode().as_ref())
            .cloned()
            .collect();
        let twox_64_concat_hex = hex::encode(twox_64_concat);
        let storage_key = format!("0x{}{}{}", module_hex, method_hex, twox_64_concat_hex);

        let mut socket = match Socket::new(&node_url) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        match get_storage_value(&mut socket, storage_key).await {
            Ok(res) => {
                match res {
                    Some(res) => {
                        let mut buf = [0; 4];
                        let index = match hex::decode_to_slice(res.replace("0x", ""), &mut buf) {
                            Ok(_) => u32::from_le_bytes(buf),
                            Err(_) => return Err(NodeError::CouldNotGetAccountIndex),
                        };

                        Ok(Some(index))
                    },
                    None => Ok(None),
                }
            },
            Err(e) => {
                eprintln!("Error {}", e);
                return Err(NodeError::CouldNotSubmitEvent);
            },
        }
    }


    pub async fn message(&self, config_file: &ConfigFile, index: u32) -> Result<Option<String>, NodeError> {
        let config = match Config::new(&config_file) {
            Ok(res) => res,
            Err(_e) => return Err(NodeError::CouldNotGetAccountMessage),
        };

        let node_url = String::from(config.data.url);

        let address = hash_address(&self.public);

        let module = twox_128("Nolik".as_bytes());
        let module_hex = hex::encode(module);

        let method = twox_128("Messages".as_bytes());
        let method_hex = hex::encode(method);

        let address_bytes = address.as_bytes();
        let address_twox_64 = twox_64(address_bytes.encode().as_ref());
        let address_twox_64_concat: Vec<u8> = address_twox_64
            .iter()
            .chain::<&[u8]>(address_bytes.encode().as_ref())
            .cloned()
            .collect();
        let address_twox_64_concat_hex = hex::encode(address_twox_64_concat);

        let index_twox_64 = twox_64(index.encode().as_ref());
        let index_twox_64_concat: Vec<u8> = index_twox_64
            .iter()
            .chain::<&[u8]>(index.encode().as_ref())
            .cloned()
            .collect();
        let index_twox_64_concat_hex = hex::encode(index_twox_64_concat);

        let storage_key = format!("0x{}{}{}{}", module_hex, method_hex, address_twox_64_concat_hex, index_twox_64_concat_hex);


        let mut socket = match Socket::new(&node_url) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        match get_storage_value(&mut socket, storage_key).await {
            Ok(res) => {
                match res {
                    Some(res) => {

                        let hash = match hex::decode(res.replace("0x", "")) {
                            Ok(mut hash_bytes) => {
                                hash_bytes.remove(0);
                                match String::from_utf8(hash_bytes) {
                                    Ok(hash) => hash,
                                    Err(_) => return Err(NodeError::CouldNotGetAccountMessage),
                                }
                            },
                            Err(_) => return Err(NodeError::CouldNotGetAccountMessage),
                        };

                        Ok(Some(hash))
                    },
                    None => Ok(None),
                }
            },
            Err(e) => {
                eprintln!("Error {}", e);
                return Err(NodeError::CouldNotSubmitEvent);
            },
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountOutput {
    pub alias: String,
    pub public: String,
    pub secret: String,
    pub seed: String,
}

impl AccountOutput {
    pub fn serialize(account: &Account) -> AccountOutput {
        AccountOutput {
            alias: account.alias.clone(),
            public: bs58::encode(&account.public).into_string(),
            secret: bs58::encode(&account.secret).into_string(),
            seed: bs58::encode(&account.seed).into_string(),
        }
    }

    pub fn deserialize(account_output: &AccountOutput) -> Result<Account, MessageError> {
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
            alias: account_output.alias.clone(),
            public,
            secret,
            seed,
        })
    }
}