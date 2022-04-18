use std::fmt::{Display, Formatter, Write};
use serde::de::StdError;

#[derive(PartialEq, Debug)]
pub enum MessageError {
    CouldNotSaveBatchFile,
    CouldNotCreateDataDir,
    DecryptionError,
    CouldNotDecryptAddress,
    CouldNotCreateTomlFileContents,
    CouldNotAddFileToIPFS,
    CouldNotSaveContentsToLocalFile,
    CouldNotCreateLocalFile,
    CouldNotAddBootstrapPeers,
}


impl Display for MessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageError::CouldNotSaveBatchFile => f.write_str("Could not save composed message"),
            MessageError::CouldNotCreateDataDir => f.write_str("Could not create Date directory"),
            MessageError::DecryptionError => f.write_str("Decryption error"),
            MessageError::CouldNotDecryptAddress => f.write_str("Could not decrypt provided address"),
            MessageError::CouldNotCreateTomlFileContents => f.write_str("Could not create TOML file contents"),
            MessageError::CouldNotAddFileToIPFS => f.write_str("Could not add file to IPFS"),
            MessageError::CouldNotSaveContentsToLocalFile => f.write_str("Could not save TOML content to local file"),
            MessageError::CouldNotCreateLocalFile => f.write_str("Could not create a local file"),
            MessageError::CouldNotAddBootstrapPeers => f.write_str("Could not add bootstrap peers")
        }
    }
}

impl StdError for MessageError {}