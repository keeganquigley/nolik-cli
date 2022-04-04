use crate::FlagKey;

#[derive(Debug)]
pub struct Rules {
    pub valid_keys: Vec<FlagKey>,
    pub required_keys: Vec<FlagKey>,
    pub unique_keys: Vec<FlagKey>,
}

impl Rules {
    pub fn add_wallet() -> Rules {
        Rules {
            valid_keys: vec![FlagKey::Name],
            required_keys: vec![FlagKey::Name],
            unique_keys: vec![FlagKey::Name],
        }
    }

    pub fn delete_wallet() -> Rules {
        Rules {
            valid_keys: vec![FlagKey::Name],
            required_keys: vec![FlagKey::Name],
            unique_keys: vec![FlagKey::Name],
        }
    }

    pub fn add_account() -> Rules {
        Rules {
            valid_keys: vec![FlagKey::Name],
            required_keys: vec![FlagKey::Name],
            unique_keys: vec![FlagKey::Name],
        }
    }

    pub fn delete_account() -> Rules {
        Rules {
            valid_keys: vec![FlagKey::Name],
            required_keys: vec![FlagKey::Name],
            unique_keys: vec![FlagKey::Name],
        }
    }
}