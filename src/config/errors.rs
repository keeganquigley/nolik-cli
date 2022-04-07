use std::fmt::{Display, Formatter};
use serde::de::StdError;

#[derive(PartialEq, Debug)]
pub enum ConfigError {
    CouldNotCreateConfigDir,
    CouldNotCreateConfigFile,
    CouldNotReadConfigFile,
    CouldNotParseConfigFile,
    WalletNameIsNotUnique,
    CouldNotParseSeed,
    WalletAlreadyExists,
    AccountAlreadyExists,
    CouldNotParseAccountSecretKey,
    AccountNameIsNotUnique,
}


impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::CouldNotCreateConfigDir => f.write_str("Could not create config dir"),
            ConfigError::CouldNotCreateConfigFile => f.write_str("Could not create config file"),
            ConfigError::CouldNotReadConfigFile => f.write_str("Could not read config file"),
            ConfigError::CouldNotParseConfigFile => f.write_str("Could not parse config file"),
            ConfigError::WalletNameIsNotUnique => f.write_str("Wallet name is not unique"),
            ConfigError::CouldNotParseSeed => f.write_str("Could not parse provided seed phrase"),
            ConfigError::WalletAlreadyExists => f.write_str("Wallet already exists"),
            ConfigError::AccountAlreadyExists => f.write_str("Account already exists"),
            ConfigError::CouldNotParseAccountSecretKey => f.write_str("Could not parse account secret key"),
            ConfigError::AccountNameIsNotUnique => f.write_str("Account name is not unique"),
        }
    }
}

impl StdError for ConfigError {}