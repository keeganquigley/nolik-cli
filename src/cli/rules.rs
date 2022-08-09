use crate::cli::errors::InputError;
use crate::cli::input::{FlagKey, Flags};
use crate::Command;

#[derive(Debug)]
pub struct Rules {
    pub valid_keys: Vec<FlagKey>,
    pub required_keys: Vec<FlagKey>,
    pub unique_keys: Vec<FlagKey>,
}

impl Rules {
    pub fn new(command: &Command) -> Rules {
        match command {
            Command::AddWallet =>
                Rules {
                    valid_keys: vec![
                        FlagKey::Alias,
                        FlagKey::Import,
                    ],
                    required_keys: vec![
                        FlagKey::Alias,
                    ],
                    unique_keys: vec![
                        FlagKey::Alias,
                        FlagKey::Import,
                    ],
                },
            Command::AddAccount =>
                Rules {
                    valid_keys: vec![
                        FlagKey::Alias,
                        FlagKey::Import,
                    ],
                    required_keys: vec![
                        FlagKey::Alias,
                    ],
                    unique_keys: vec![
                        FlagKey::Alias,
                        FlagKey::Import,
                    ],
                },
            Command::AddOwner =>
                Rules {
                    valid_keys: vec![
                        FlagKey::Wallet,
                        FlagKey::Account,
                    ],
                    required_keys: vec![
                        FlagKey::Wallet,
                        FlagKey::Account,
                    ],
                    unique_keys: vec![
                        FlagKey::Wallet,
                        FlagKey::Account,
                    ]
                },
            Command::UpdateWhitelist =>
                Rules {
                    valid_keys: vec![
                        FlagKey::For,
                        FlagKey::Add,
                        FlagKey::Wallet,
                    ],
                    required_keys: vec![
                        FlagKey::For,
                        FlagKey::Add,
                        FlagKey::Wallet,
                    ],
                    unique_keys: vec![
                        FlagKey::For,
                        FlagKey::Add,
                        FlagKey::Wallet,
                    ],
                },
            Command::SendMessage =>
                Rules {
                    valid_keys: vec![
                        FlagKey::Wallet,
                        FlagKey::Sender,
                        FlagKey::Recipient,
                        FlagKey::Key,
                        FlagKey::Value,
                        FlagKey::Blob,
                    ],
                    required_keys: vec![
                        FlagKey::Wallet,
                        FlagKey::Sender,
                        FlagKey::Recipient,
                    ],
                    unique_keys: vec![
                        FlagKey::Wallet,
                        FlagKey::Sender,
                    ],
                },
            Command::GetMessages =>
                Rules {
                    valid_keys: vec![
                        FlagKey::Account,
                    ],
                    required_keys: vec![
                        FlagKey::Account,
                    ],
                    unique_keys: vec![
                        FlagKey::Account,
                    ]
                },
            Command::GetCoins =>
                Rules {
                    valid_keys: vec![
                        FlagKey::Wallet,
                    ],
                    required_keys: vec![
                        FlagKey::Wallet
                    ],
                    unique_keys: vec![
                        FlagKey::Wallet,
                    ]
                },
        }
    }

    pub fn validate_on_missing_keys(self, flags: &Flags) -> Result<(), InputError> {
        let missing_keys: Vec<FlagKey> = self.required_keys
            .iter()
            .filter(|key|
                flags.iter().filter(|flag| &&flag.key == key).count() == 0
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

    pub fn validate_invalid_flags(self, flags: &Flags) -> Result<(), InputError> {
        let invalid_keys: Vec<FlagKey> = flags
            .iter()
            .filter(|flag|
                self.valid_keys.iter().any(|el| *el == flag.key) == false
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

    pub fn validate_on_non_unique_keys(self, flags: &Flags) -> Result<(), InputError> {
        let non_unique_keys: Vec<FlagKey> = self.unique_keys
            .iter()
            .filter(|key|
                flags.iter().filter(|flag| &&flag.key == key).count() > 1
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

    pub fn validate_key_value_flags(self, flags: &Flags) -> Result<(), InputError> {
        let mut flags = flags.iter();
        while let Some(key_flag) = &flags.next() {
            if key_flag.key == FlagKey::Key {
                match &flags.next() {
                    Some(value_flag) => {
                        match value_flag.key {
                            FlagKey::Value => continue,
                            _ => {
                                eprintln!("A key without value: {}", key_flag.value);
                                return Err(InputError::NoCorrespondingValue);
                            }
                        }
                    },
                    None => {
                        eprintln!("A key without value: {}", key_flag.value);
                        return Err(InputError::NoCorrespondingValue);
                    }
                }
            }
        }
        Ok(())
    }
}


