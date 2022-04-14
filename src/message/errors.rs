use std::fmt::{Display, Formatter};
use serde::de::StdError;

#[derive(PartialEq, Debug)]
pub enum MessageError {
    CouldNotSaveBatchFile,
    CouldNotCreateDataDir,
    DecryptionError,
    CouldNotDecryptAddress,
}


impl Display for MessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageError::CouldNotSaveBatchFile => f.write_str("Could not save composed message"),
            MessageError::CouldNotCreateDataDir => f.write_str("Could not create Date directory"),
            MessageError::DecryptionError => f.write_str("Decryption error"),
            MessageError::CouldNotDecryptAddress => f.write_str("Could not decrypt provided address")
        }
    }
}

impl StdError for MessageError {}