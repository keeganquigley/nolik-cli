use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use sodiumoxide::crypto::box_::PublicKey;
use crate::{Account, ConfigFile, Input};
use crate::cli::errors::InputError;
use crate::cli::input::FlagKey;
use crate::message::file::File;
use crate::message::entry::Entry;
use crate::message::utils::base58_to_public_key;



#[derive(Debug)]
pub struct BatchInput {
    pub sender: Account,
    pub recipients: Vec<PublicKey>,
    pub entries: Vec<Entry>,
    pub files: Vec<File>,
}


impl BatchInput {
    pub fn new(input: &mut Input, config_file: &ConfigFile) -> Result<BatchInput, InputError> {
        let sender_key = match input.get_flag_value(FlagKey::Sender) {
            Ok(key) => key,
            Err(e) => return Err(e)
        };

        let sender = match Account::get(&config_file, sender_key) {
            Ok(account) => account,
            Err(_e) => return Err(InputError::SenderDoesNotExist),
        };

        let recipient_values = match input.get_flag_values(FlagKey::Recipient) {
            Ok(values) => values,
            Err(e) => return Err(e),
        };

        let mut recipients: Vec<PublicKey> = Vec::new();
        for value in recipient_values {
            let address = match base58_to_public_key(&value) {
                Ok(pk) => pk,
                Err(_e) => return Err(InputError::InvalidAddress)
            };

            recipients.push(address);
        }

        let mut entries: Vec<Entry> = Vec::new();
        let mut flags = input.flags.iter();
        while let Some(flag) = flags.next() {
            if let FlagKey::Key = flag.key {
                if let Some(value_flag) = flags.next() {
                    entries.push(Entry {
                        key: flag.value.to_string(),
                        value: value_flag.value.to_string(),
                    });
                }
            }
        }

        let file_values = match input.get_flag_values(FlagKey::File) {
            Ok(values) => values,
            Err(e) => {
                match e {
                    InputError::NoSuchKey => vec![],
                    _ => return Err(e),
                }
            }
        };

        let mut files: Vec<File> = Vec::new();
        for path in file_values {
            let file = Path::new(&path);
            let binary= match fs::read(file) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    return Err(InputError::CouldNotReadFileBinary);
                }
            };

            let name = match file.file_name().and_then(OsStr::to_str) {
                Some(name) => name.to_string(),
                _ => "".to_string(),
            };

            let file = File {
                binary,
                name,
            };

            files.push(file);
        }

        Ok(BatchInput {
            sender,
            recipients,
            entries,
            files,
        })
    }
}
