pub mod inputs;
mod rpc;
mod wallet;

use std::error::Error;
use std::slice::Iter;
use sp_keyring::AccountKeyring;
use inputs::{
    constants::{commands, flags},
    errors::InputError,
    rules::Rules,
};
use wallet::Wallet;


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
pub struct Flag {
    pub key: FlagKey,
    pub value: String,
}

pub type Flags = Vec<Flag>;

#[derive(Debug)]
pub struct Config {
    command: Command,
    flags: Flags,
    rules: Rules,
}

impl Config {
    pub fn new(mut args: Iter<String>) -> Result<Config, InputError> {
        let (command, rules) = match args.next() {
            Some(command) => {
                match args.next() {
                    Some(arg) => {
                        match (command.as_str(), arg.as_str()) {
                            commands::ADD_WALLET => (Command::AddWallet, Rules::add_wallet()),
                            commands::ADD_ACCOUNT => (Command::AddAccount, Rules::add_account()),
                            commands::DELETE_WALLET => (Command::DeleteWallet, Rules::delete_wallet()),
                            commands::DELETE_ACCOUNT => (Command::DeleteAccount, Rules::delete_account()),
                            _ => return Err(InputError::UnrecognisedCommand)
                        }
                    },
                    None => return Err(InputError::NotEnoughArguments)
                }
            },
            None => return Err(InputError::NoArguments)
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
                        _ => return Err(InputError::UnrecognisedFlag)
                    };

                    let flag = Flag {
                        key: flag_key,
                        value: value.to_string(),
                    };

                    flags.push(flag);
                },
                None => return Err(InputError::NoCorrespondingValue)
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
            // println!("Wallet {:?}", asd);
        }
        Command::DeleteWallet => {}
        Command::AddAccount => {}
        Command::DeleteAccount => {}
    }
    Ok(())
}


fn validate_flags(config: &Config) -> Result<(), InputError> {
    if let Err(e) = validate_on_missing_keys(config) { return Err(e) }
    if let Err(e) = validate_invalid_flags(config) { return Err(e) }
    if let Err(e) = validate_on_non_unique_keys(config) { return Err(e) }

    Ok(())
}

fn validate_on_missing_keys(config: &Config) -> Result<(), InputError> {
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
        _ => return Err(InputError::RequiredKeysMissing)
    }
}

fn validate_invalid_flags(config: &Config) -> Result<(), InputError> {
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
        _ => return Err(InputError::InvalidFlag)
    }
}

fn validate_on_non_unique_keys(config: &Config) -> Result<(), InputError> {
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
        _ => return Err(InputError::NonUniqueKeys)
    }
}

pub fn get_flag_values(flag_key: FlagKey, flags: Flags) -> Result<Vec<String>, InputError> {
    let mut values: Vec<String> = Vec::new();
    for flag in flags {
        if flag.key == flag_key {
            values.push(flag.value);
        }
    }

    match values.len() {
        0 => return Err(InputError::NoSuchKey),
        _ => Ok(values),
    }
}
