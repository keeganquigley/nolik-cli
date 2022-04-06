use std::fmt::{Display, Formatter};
use serde::ser::StdError;

#[derive(PartialEq, Debug)]
pub enum InputError {
    UnrecognisedCommand,
    NotEnoughArguments,
    NoArguments,
    UnrecognisedFlag,
    NoCorrespondingValue,
    RequiredKeysMissing,
    InvalidFlag,
    NonUniqueKeys,
    NoSuchKey,
    PasswordsDoNotMatch,
}


impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       match self {
            InputError::UnrecognisedCommand => f.write_str("Unrecognised command"),
            InputError::NotEnoughArguments => f.write_str("Not enough arguments"),
            InputError::NoArguments => f.write_str("No arguments"),
            InputError::UnrecognisedFlag => f.write_str("Unrecognised flag"),
            InputError::InvalidFlag => f.write_str("Invalid flag"),
            InputError::NoCorrespondingValue => f.write_str("No corresponding value to provided key"),
            InputError::RequiredKeysMissing => f.write_str("Required keys are missing"),
            InputError::NonUniqueKeys => f.write_str("Non unique flags"),
            InputError::NoSuchKey => f.write_str("No such key"),
            InputError::PasswordsDoNotMatch => f.write_str("Provided passwords do not match"),
       }
    }
}

impl StdError for InputError {}
