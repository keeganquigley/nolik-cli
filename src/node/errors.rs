use std::fmt::{Display, Formatter};
use serde::de::StdError;

#[derive(PartialEq, Debug)]
pub enum NodeError {
    CouldNotGetAccountNonce,
    CouldNotGetGenesisHash,
    CouldNotGetRuntimeVersion,
    CouldNotCallExtrinsic,
    CouldNotGetMetadata,
}


impl Display for NodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeError::CouldNotGetAccountNonce => f.write_str("Could not get blockchain account nonce"),
            NodeError::CouldNotGetGenesisHash => f. write_str("Could not get genesis hash"),
            NodeError::CouldNotGetRuntimeVersion => f.write_str("Could not get runtime version"),
            NodeError::CouldNotCallExtrinsic => f.write_str("Could not call extrinsic"),
            NodeError::CouldNotGetMetadata => f.write_str("Could not get node metadata"),
        }
    }
}

impl StdError for NodeError {}
