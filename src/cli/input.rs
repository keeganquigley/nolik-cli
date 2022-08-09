use std::slice::Iter;
use crate::cli::constants::{commands, flags};
use crate::cli::errors::InputError;
use crate::cli::rules::Rules;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FlagKey {
    Alias,
    Account,
    Import,
    Sender,
    Recipient,
    Key,
    Value,
    Blob,
    Wallet,
    IpfsId,
    Add,
    For,
}


#[derive(Debug)]
pub enum Command {
    AddWallet,
    AddAccount,
    AddOwner,
    UpdateWhitelist,
    SendMessage,
    GetMessages,
    GetCoins,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Flag {
    pub key: FlagKey,
    pub value: String,
}

pub type Flags = Vec<Flag>;

#[derive(Debug)]
pub struct Input {
    pub command: Command,
    pub flags: Flags,
}

impl Input {
    pub fn new(mut args: Iter<String>) -> Result<Input, InputError> {
        let command = match args.next() {
            Some(command) => {
                match args.next() {
                    Some(arg) => {
                        match (command.as_str(), arg.as_str()) {
                            commands::ADD_WALLET => Command::AddWallet,
                            commands::ADD_ACCOUNT => Command::AddAccount,
                            commands::ADD_OWNER => Command::AddOwner,
                            commands::UPDATE_WHITELIST => Command::UpdateWhitelist,
                            commands::SEND_MESSAGE => Command::SendMessage,
                            commands::GET_MESSAGES => Command::GetMessages,
                            commands::GET_COINS => Command::GetCoins,
                            _ => return Err(InputError::UnrecognisedCommand)
                        }
                    },
                    None => return Err(InputError::NotEnoughArguments)
                }
            },
            None => return Err(InputError::NoArguments)
        };

        let mut flags: Flags = Vec::new();
        while let Some(key) = args.next() {
            match args.next() {
                Some(value) => {
                    let flag_key: FlagKey = match key.as_str() {
                        flags::ALIAS => FlagKey::Alias,
                        flags::ACCOUNT | flags::A => FlagKey::Account,
                        flags::IMPORT | flags::I => FlagKey::Import,
                        flags::SENDER | flags::S => FlagKey::Sender,
                        flags::RECIPIENT | flags::R => FlagKey::Recipient,
                        flags::KEY | flags::K => FlagKey::Key,
                        flags::VALUE | flags::V => FlagKey::Value,
                        flags::BLOB | flags::B => FlagKey::Blob,
                        flags::WALLET | flags::W => FlagKey::Wallet,
                        flags::ADD => FlagKey::Add,
                        flags::FOR => FlagKey::For,
                        _ => return Err(InputError::UnrecognisedFlag)
                    };

                    let flag = Flag {
                        key: flag_key,
                        value: value.to_string(),
                    };

                    flags.push(flag);
                },
                None => {
                    eprintln!("Flag without value: {}", key);
                    return Err(InputError::NoValueForFlag);
                }
            };
        };

        if let Err(e) = Rules::new(&command).validate_on_missing_keys(&flags) {
            return Err(e);
        }

        if let Err(e) = Rules::new(&command).validate_invalid_flags(&flags) {
            return Err(e);
        }

        if let Err(e) = Rules::new(&command).validate_on_non_unique_keys(&flags) {
            return Err(e);
        }

        if let Err(e) = Rules::new(&command).validate_key_value_flags(&flags) {
            return Err(e);
        }

        Ok(Input { command, flags })
    }

    pub fn get_flag_values(&self, flag_key: FlagKey) -> Result<Vec<String>, InputError> {
        let values : Vec<String> = self.flags
            .iter()
            .filter(|flag| flag.key == flag_key)
            .map(|flag| flag.value.clone())
            .collect();

        match values.len() {
            0 => {
                println!("No such key: {:?}", flag_key);
                return Err(InputError::NoSuchKey)
            },
            _ => Ok(values)
        }
    }

    pub fn get_flag_value(&self, flag_key: FlagKey) -> Result<String, InputError> {
        match self.get_flag_values(flag_key) {
            Ok(values) => Ok(values.last().unwrap().to_string()),
            Err(e) => return Err(e),
        }
    }
}
