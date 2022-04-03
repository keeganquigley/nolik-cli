mod constants;
mod wallet;
mod flag_validation;
pub mod errors;


use constants::{commands, flags};
use std::error::Error;
use std::slice::Iter;
use sp_keyring::AccountKeyring;
use crate::flag_validation::Rules;
use errors::Error as ErrorType;
use crate::wallet::Wallet;


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FlagKey {
    Recipient,
    Sender,
    Wallet,
    Attachment,
    Import,
    Keyring,
    Output,
    Name,
}


#[derive(Debug)]
enum Command {
    AddWallet,
    AddAccount,
    DeleteWallet,
    DeleteAccount,
}

#[derive(Debug, PartialEq)]
struct Flag {
    key: FlagKey,
    value: String,
}

type Flags = Vec<Flag>;

#[derive(Debug)]
pub struct Config {
    command: Command,
    flags: Flags,
    rules: Rules,
}

impl Config {
    pub fn new(mut args: Iter<String>) -> Result<Config, ErrorType> {
        // args.next();

        let (command, rules) = match args.next() {
            Some(command) => {
                match args.next() {
                    Some(arg) => {
                        match (command.as_str(), arg.as_str()) {
                            commands::ADD_WALLET => (Command::AddWallet, Rules::add_wallet()),
                            commands::ADD_ACCOUNT => (Command::AddAccount, Rules::add_account()),
                            commands::DELETE_WALLET => (Command::DeleteWallet, Rules::delete_wallet()),
                            commands::DELETE_ACCOUNT => (Command::DeleteAccount, Rules::delete_account()),
                            _ => return Err(ErrorType::UnrecognisedCommand)
                        }
                    },
                    None => return Err(ErrorType::NotEnoughArguments)
                }
            },
            None => return Err(ErrorType::NoArguments)
        };

        let mut flags: Vec<Flag> = Vec::new();
        while let Some(key) = args.next() {
            match args.next() {
                Some(value) => {
                    let flag_key: FlagKey = match key.as_str() {
                        flags::NAME | flags::N => FlagKey::Name,
                        flags::SENDER | flags::S => FlagKey::Sender,
                        flags::RECIPIENT | flags::R => FlagKey::Recipient,
                        flags::ATTACHMENT | flags::A => FlagKey::Attachment,
                        flags::WALLET | flags::W => FlagKey::Wallet,
                        flags::IMPORT | flags::I => FlagKey::Import,
                        flags::KEYRING | flags::K => FlagKey::Keyring,
                        flags::OUTPUT | flags::O => FlagKey::Output,
                        _ => return Err(ErrorType::UnrecognisedFlag)
                    };

                    let flag = Flag {
                        key: flag_key,
                        value: value.to_string(),
                    };

                    flags.push(flag);
                },
                None => return Err(ErrorType::NoCorrespondingValue)
            };
        };

        let config = Config {
            command,
            flags,
            rules,
        };

        match validate_flags(&config) {
            Ok(_) => Ok(config),
            Err (e) => return Err(e),
        }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.command {
        Command::AddWallet => {
            let name = get_flag_values(FlagKey::Name, config.flags)
                .unwrap()
                .last()
                .unwrap()
                .to_string();

            let pair = AccountKeyring::Alice.pair();
            let wallet = Wallet::new(pair, name);

            Wallet::add(wallet);
            // println!("Wallet {:?}", wallet);
        }
        Command::DeleteWallet => {}
        Command::AddAccount => {}
        Command::DeleteAccount => {}
    }
    Ok(())
}


fn validate_flags(config: &Config) -> Result<(), ErrorType> {
    if let Err(e) = validate_on_missing_keys(config) { return Err(e) }
    if let Err(e) = validate_invalid_flags(config) { return Err(e) }
    if let Err(e) = validate_on_non_unique_keys(config) { return Err(e) }

    Ok(())
}

fn validate_on_missing_keys(config: &Config) -> Result<(), ErrorType> {
    let missing_keys: Vec<FlagKey> = config.rules.required_keys
        .iter()
        .filter(|key|
            config.flags.iter().filter(|flag| &&flag.key == key).count() == 0
        )
        .map(|key| *key)
        .collect();

    for key in &missing_keys {
        eprintln!("Missing key: {:?}", key);
    }

    match missing_keys.len() {
        0 => Ok(()),
        _ => return Err(ErrorType::RequiredKeysMissing)
    }
}

fn validate_invalid_flags(config: &Config) -> Result<(), ErrorType> {
    let invalid_keys: Vec<FlagKey> = config.flags
        .iter()
        .filter(|flag|
            config.rules.valid_keys.iter().any(|el| *el == flag.key) == false
        )
        .map(|flag| flag.key)
        .collect();

    for key in &invalid_keys {
        eprintln!("Invalid flag: {:?}", key);
    }

    match invalid_keys.len() {
        0 => Ok(()),
        _ => return Err(ErrorType::InvalidFlag)
    }
}

fn validate_on_non_unique_keys(config: &Config) -> Result<(), ErrorType> {
    let non_unique_keys: Vec<FlagKey> = config.rules.unique_keys
        .iter()
        .filter(|key|
            config.flags.iter().filter(|flag| &&flag.key == key).count() > 1
        )
        .map(|key| *key)
        .collect();

    for key in &non_unique_keys {
        eprintln!("Non-unique flag: {:?}", key);
    }

    match non_unique_keys.len() {
        0 => Ok(()),
        _ => return Err(ErrorType::NonUniqueKeys)
    }
}

fn get_flag_values(flag_key: FlagKey, flags: Flags) -> Result<Vec<String>, ErrorType> {
    let mut values: Vec<String> = Vec::new();
    for flag in flags {
        if flag.key == flag_key {
            values.push(flag.value);
        }
    }

    match values.len() {
        0 => return Err(ErrorType::NoSuchKey),
        _ => Ok(values),
    }
}

#[cfg(test)]
mod parser {
    use super::*;

    #[test]
    fn unrecognised_command() {
        let arr = ["unrecognised", "command"].map(|el| el.to_string());;
        let args = arr.iter();
        assert_eq!(
            ErrorType::UnrecognisedCommand,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn not_enough_arguments() {
        let arr = ["argument".to_string()];
        let args = arr.iter();
        assert_eq!(
            ErrorType::NotEnoughArguments,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn no_arguments() {
        let arr = [];
        let args = arr.iter();
        assert_eq!(
            ErrorType::NoArguments,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn no_corresponding_value() {
        let arr = ["add", "wallet", "--name"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            ErrorType::NoCorrespondingValue,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn required_key_missing() {
        let arr = ["add", "wallet"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            ErrorType::RequiredKeysMissing,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn invalid_flags() {
        let arr = ["add", "wallet", "--name", "alice", "--output", "value"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            ErrorType::InvalidFlag,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn non_unique_key() {
        let arr = ["add", "wallet", "--name", "alice", "--name", "alice"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            ErrorType::NonUniqueKeys,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn non_unique_key_short_flags() {
        let arr = ["add", "wallet", "-n", "alice", "-n", "alice"].map(|el| el.to_string());
        let args = arr.iter();
        assert_eq!(
            ErrorType::NonUniqueKeys,
            Config::new(args).unwrap_err()
        );
    }

    #[test]
    fn returns_flag_value() {
        let mut flags: Flags = Vec::new();
        let flag = Flag {
            key: FlagKey::Name,
            value: "alice".to_string(),
        };

        flags.push(flag);

        assert_eq!(
            vec![String::from("alice")],
            get_flag_values(FlagKey::Name, flags).unwrap(),
        );
    }
}
