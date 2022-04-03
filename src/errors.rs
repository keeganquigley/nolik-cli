use crate::constants::errors;

#[derive(PartialEq, Debug)]
pub enum Error {
    UnrecognisedCommand,
    NotEnoughArguments,
    NoArguments,
    UnrecognisedFlag,
    NoCorrespondingValue,
    RequiredKeysMissing,
    InvalidFlag,
    NonUniqueKeys,
    NoSuchKey,
}

impl Error {
    pub fn description(error: Error) -> String {
        match error {
            Error::UnrecognisedCommand => errors::UNRECOGNISED_COMMAND.to_string(),
            Error::NotEnoughArguments => errors::NOT_ENOUGH_ARGUMENTS.to_string(),
            Error::NoArguments => errors::NO_ARGUMENTS.to_string(),
            Error::UnrecognisedFlag => errors::UNRECOGNISED_FLAG.to_string(),
            Error::InvalidFlag => errors::INVALID_FLAG.to_string(),
            Error::NoCorrespondingValue => errors::NO_CORRESPONDING_VALUE.to_string(),
            Error::RequiredKeysMissing => errors::REQUIRED_KEYS_MISSING.to_string(),
            Error::NonUniqueKeys => errors::NON_UNIQUE_KEYS.to_string(),
            Error::NoSuchKey => errors::NO_SUCH_KEY.to_string(),
        }
    }
}