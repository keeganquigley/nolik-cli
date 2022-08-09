use std::fmt::{Display, Formatter};
use serde::de::StdError;

#[derive(PartialEq, Debug)]
pub enum NodeError {
    CouldNotGetAccountNonce,
    CouldNotGetGenesisHash,
    CouldNotGetRuntimeVersion,
    CouldNotCallExtrinsic,
    CouldNotGetMetadata,
    CouldNotSubmitEvent,
    CouldNotGetCallIndex,
}


impl Display for NodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeError::CouldNotGetAccountNonce => f.write_str("Could not get blockchain account nonce"),
            NodeError::CouldNotGetGenesisHash => f. write_str("Could not get genesis hash"),
            NodeError::CouldNotGetRuntimeVersion => f.write_str("Could not get runtime version"),
            NodeError::CouldNotCallExtrinsic => f.write_str("Could not call extrinsic"),
            NodeError::CouldNotGetMetadata => f.write_str("Could not get node metadata"),
            NodeError::CouldNotSubmitEvent => f.write_str("Could not submit event"),
            NodeError::CouldNotGetCallIndex => f.write_str("Could not get call index")
        }
    }
}

impl StdError for NodeError {}


#[derive(PartialEq, Debug)]
pub enum PalletError {
    AddressNotOwned,
    AccountInOwners,
    SameAddress,
    AlreadyInWhiteList,
    UnknownError,
}

impl Display for PalletError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PalletError::AccountInOwners => f.write_str("Account is already owned by this wallet"),
            PalletError::AddressNotOwned => f.write_str("Account is not owned by this wallet"),
            PalletError::SameAddress => f.write_str("Trying to add your own address"),
            PalletError::AlreadyInWhiteList => f.write_str("Address is already in Whitelist"),
            PalletError::UnknownError => f.write_str("Unknown error"),
        }
    }
}

impl StdError for PalletError {}