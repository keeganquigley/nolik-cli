use crate::FlagKey;
use crate::inputs::errors::InputError;
use crate::Command;
use crate::inputs::Flags;

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
                    valid_keys: vec![FlagKey::Name, FlagKey::Import, FlagKey::WithPassword],
                    required_keys: vec![FlagKey::Name],
                    unique_keys: vec![FlagKey::Name, FlagKey::Import, FlagKey::WithPassword],
                },
            Command::AddAccount =>
                Rules {
                    valid_keys: vec![FlagKey::Name, FlagKey::Import],
                    required_keys: vec![FlagKey::Name],
                    unique_keys: vec![FlagKey::Name, FlagKey::Import],
                },
            Command::DeleteWallet | Command::DeleteAccount =>
                Rules {
                    valid_keys: vec![FlagKey::Name],
                    required_keys: vec![FlagKey::Name],
                    unique_keys: vec![FlagKey::Name],
                }
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
}


