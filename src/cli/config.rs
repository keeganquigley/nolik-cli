use std::fs;
use std::io::Write;
use std::path::PathBuf;
use serde_derive::{Serialize, Deserialize};
use rand::{distributions::Alphanumeric, Rng};
use crate::account::AccountOutput;
use crate::wallet::Wallet;
use crate::cli::errors::ConfigError;

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
    pub accounts: Vec<AccountOutput>,
}


#[derive(Debug)]
pub struct Config {
    pub file: ConfigFile,
    pub data: ConfigData,
}

impl Config {
    pub fn new(config_file: ConfigFile) -> Result<Config, ConfigError> {
        if let false = &config_file.path.exists() {
            return Ok(Config {
                file: config_file,
                data: ConfigData {
                    wallets: vec![],
                    accounts: vec![],
                }
            });
        }

        if let Err(e) = fs::read_to_string(&config_file.path) {
            eprintln!("Error: {}", e);
            return Err(ConfigError::CouldNotReadConfigFile)
        }

        let contents: String = fs::read_to_string(&config_file.path).unwrap();
        match toml::from_str(contents.as_str()) {
            Ok(config_data) => Ok(Config {
                file: config_file,
                data: config_data,
            }),
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(ConfigError::CouldNotParseConfigFile);
            },
        }
    }


    pub fn save(&self) -> Result<(), ConfigError> {
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
}


