pub mod errors;

use std::fs;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use crate::config::errors::ConfigError;
use crate::wallet::Wallet;
use serde_derive::{Serialize, Deserialize};
use rand::{distributions::Alphanumeric, Rng};
use crate::account::Account;

#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub path: PathBuf,
    dir: PathBuf,
}

impl ConfigFile {
    pub fn new() -> ConfigFile {
        let home_dir = dirs::home_dir().unwrap();
        let home_path = home_dir.as_path();
        let nolik_dir = home_path.join(".nolik");
        let config_file = nolik_dir.join("config.toml");

        ConfigFile {
            path: config_file,
            dir: nolik_dir,
        }
    }

    pub fn temp() -> ConfigFile {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let file_name = format!("temp_{}_config.toml", s);
        let home_dir = dirs::home_dir().unwrap();
        let home_path = home_dir.as_path();
        let nolik_dir = home_path.join(".nolik");
        let config_file = nolik_dir.join(file_name);

        ConfigFile {
            path: config_file,
            dir: nolik_dir,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigData {

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub wallets: Vec<Wallet>,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub accounts: Vec<Account>,
}


#[derive(Debug)]
pub struct Config {
    pub file: ConfigFile,
    pub data: ConfigData,
}

impl Config {
    pub fn new(config_file: ConfigFile) -> Result<Config, ConfigError> {
        match fs::read_to_string(&config_file.path) {
            Ok(contents) => {
                let config_data = Self::parse_config_data(contents);
                if let Err(e) = config_data { return Err(e); }

                Ok(Config {
                    file: config_file,
                    data: config_data.unwrap(),
                })
            },
            Err(e) => {
                if let ErrorKind::NotFound = e.kind() {
                    Ok(Config {
                        file: config_file,
                        data: ConfigData {
                            wallets: vec![],
                            accounts: vec![],
                        }
                    })
                } else {
                    eprintln!("Error: {}", e);
                    return Err(ConfigError::CouldNotReadConfigFile)
                }
            }
        }
    }

    pub fn parse_config_data(contents: String) -> Result<ConfigData, ConfigError> {
        match toml::from_str(contents.as_str()) {
            Ok(config_data) => Ok(config_data),
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(ConfigError::CouldNotParseConfigFile);
            },
        }
    }

    pub fn add_wallet(&mut self, wallet: Wallet) -> Result<(), ConfigError> {

        let same_wallet_names = self.data.wallets
            .iter()
            .filter(|el| el.name == wallet.name)
            .count();

        if let true = same_wallet_names > 0 {
            return Err(ConfigError::WalletNameIsNotUnique);
        }

        let same_wallet_seed_phrases = self.data.wallets
            .iter()
            .filter(|el| el.bs58seed == wallet.bs58seed)
            .count();

        if let true = same_wallet_seed_phrases > 0 {
            return Err(ConfigError::WalletAlreadyExists);
        }

        self.data.wallets.push(wallet);

        match self.save_config() {
            Ok(_) => {
                println!("The wallet has been successfully created");
                Ok(())
            },
            Err(e) => return Err(e),
        }
    }

    pub fn add_account(&mut self, account: Account) -> Result<(), ConfigError> {
        let same_account_names = self.data.accounts
            .iter()
            .filter(|el| el.name == account.name)
            .count();

        if let true = same_account_names > 0 {
            return Err(ConfigError::AccountNameIsNotUnique);
        }

        let same_account_seeds = self.data.accounts
            .iter()
            .filter(|el| el.seed == account.seed)
            .count();

        if let true = same_account_seeds > 0 {
            return Err(ConfigError::AccountAlreadyExists);
        }

        self.data.accounts.push(account);

        match self.save_config() {
            Ok(_) => {
                println!("The account has been successfully created");
                Ok(())
            },
            Err(e) => return Err(e),
        }
    }

    pub fn save_config(&self) -> Result<(), errors::ConfigError> {
        if let false = self.file.dir.exists() {
            if let Err(e) = fs::create_dir(&self.file.dir) {
                eprintln!("Error: {}", e);
                return Err(ConfigError::CouldNotCreateConfigDir)
            }
        }

        match fs::File::create(&self.file.path) {
            Ok(mut file) => {
                let contents = match toml::to_string(&self.data) {
                    Ok(contents) => contents,
                    Err(e) => {
                        eprintln!("DATA: {:?}", &self.data);
                        eprintln!("Error: {}", e);
                        return Err(ConfigError::CouldNotCreateConfigFile);
                    },
                };
                match file.write_all(contents.as_ref()) {
                    Ok(_) => {
                        Ok(())
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return Err(ConfigError::CouldNotCreateConfigFile);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(ConfigError::CouldNotCreateConfigFile);
            }
        }
    }

    pub fn get_account(&self, key: String) -> Option<Account> {
        let accounts: Vec<Account> = self.data.accounts
            .iter()
            .filter(|account| vec![&account.name, &account.public].contains(&&key))
            .map(|account| account.to_owned())
            .collect();

        match accounts.len() {
            1 => Some(accounts.last().unwrap().to_owned()),
            _ => None,
        }
    }
}

