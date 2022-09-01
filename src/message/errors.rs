use std::fmt::{Display, Formatter};
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
    CouldNotReadIpfsData,
    CouldNotDecryptAnyOfParties,
    CouldNotSendMessage,
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
            MessageError::CouldNotAddBootstrapPeers => f.write_str("Could not add bootstrap peers"),
            MessageError::CouldNotReadIpfsData => f.write_str("Could not read IPFS data"),
            MessageError::CouldNotDecryptAnyOfParties => f.write_str("Could not decrypt any of message parties"),
            MessageError::CouldNotSendMessage => f.write_str("Could not send the message"),
        }
    }
}

impl StdError for MessageError {}


#[derive(PartialEq, Debug)]
pub enum IndexError {
    CouldNotCreateIndexDir,
    CouldNotCreateIndexFile,
    CouldNotReadIndexFile,
    CouldNotParseIndexFile,
}


impl Display for IndexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexError::CouldNotCreateIndexDir => f.write_str("Could not create Index dir"),
            IndexError::CouldNotCreateIndexFile => f.write_str("Could not create Index file"),
            IndexError::CouldNotReadIndexFile => f.write_str("Could not read Index file"),
            IndexError::CouldNotParseIndexFile => f.write_str("Could not parse Index file"),
        }
    }
}

impl StdError for IndexError {}