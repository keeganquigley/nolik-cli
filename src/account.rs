use rand::Rng;
use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey, Seed};
use crate::cli::errors::{ConfigError, InputError};
use crate::cli::input::FlagKey;
use crate::Input;


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
    pub public: String,
    pub secret: String,
    pub seed: String,
    pub name: String,
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
            public: bs58::encode(&account.0).into_string(),
            secret: bs58::encode(&account.1).into_string(),
            seed: bs58::encode(&account.2).into_string(),
            name: input.name
        })
    }

    pub fn delete(_wallet: Account) -> Result<(), &'static str> {
        Ok(())
    }
}
