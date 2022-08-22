use std::fmt::{Display, Formatter};
use serde::ser::StdError;

#[derive(PartialEq, Debug)]
pub enum InputError {
    UnrecognisedCommand,
    NotEnoughArguments,
    NoArguments,
    UnrecognisedFlag,
    NoValueForFlag,
    NoCorrespondingValue,
    RequiredKeysMissing,
    InvalidFlag,
    NonUniqueKeys,
    NoSuchKey,
    PasswordsDoNotMatch,
    PasswordInputError,
    SenderDoesNotExist,
    InvalidAddress,
    CouldNotReadFileBinary,
    CouldNotAddOwner,
    CouldNotUpdateWhitelist,
    CouldNotUpdateBlacklist,
    CouldNotGetWallet,
}


impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::UnrecognisedCommand => f.write_str("Unrecognised command"),
            InputError::NotEnoughArguments => f.write_str("Not enough arguments"),
            InputError::NoArguments => f.write_str("No arguments"),
            InputError::UnrecognisedFlag => f.write_str("Unrecognised flag"),
            InputError::InvalidFlag => f.write_str("Invalid flag"),
            InputError::NoValueForFlag => f.write_str("No value for provided flag"),
            InputError::NoCorrespondingValue => f.write_str("No corresponding value to provided key"),
            InputError::RequiredKeysMissing => f.write_str("Required keys are missing"),
            InputError::NonUniqueKeys => f.write_str("Non unique flags"),
            InputError::NoSuchKey => f.write_str("Required key is missing"),
            InputError::PasswordsDoNotMatch => f.write_str("Provided passwords do not match"),
            InputError::PasswordInputError => f.write_str("Password input error"),
            InputError::SenderDoesNotExist => f.write_str("Provided sender name does not exist"),
            InputError::InvalidAddress => f.write_str("Provided address is not valid"),
            InputError::CouldNotReadFileBinary => f.write_str("Could not read file binary"),
            InputError::CouldNotAddOwner => f.write_str("Could not add owner"),
            InputError::CouldNotUpdateWhitelist => f.write_str("Could not update Whitelist"),
            InputError::CouldNotUpdateBlacklist => f.write_str("Could not update Blacklist"),
            InputError::CouldNotGetWallet => f.write_str("Could not find required wallet from provided input"),
        }
    }
}

impl StdError for InputError {}



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
    CouldNotGetAccount,
    CouldNotGetWallet,
    CouldNotGetBatchNonce,
    CouldNotGetBatchBroker,
    CouldNotInitSender,
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
            ConfigError::CouldNotGetAccount => f.write_str("Could not get account from config file"),
            ConfigError::CouldNotGetWallet => f.write_str("Could not get wallet from config file"),
            ConfigError::CouldNotGetBatchNonce => f.write_str("Could not get batch nonce"),
            ConfigError::CouldNotGetBatchBroker => f.write_str("Could not get batch broker"),
            ConfigError::CouldNotInitSender => f.write_str("The message you are trying to send has a sender, but it does not match any of your accounts from config file")
        }
    }
}

impl StdError for ConfigError {}
