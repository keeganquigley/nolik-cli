// use std::path::PathBuf;
use serde_derive::{Serialize, Deserialize};
use crate::message::errors::MessageError;
use crate::message::input::MessageInput;
use crate::message::message::EncryptedMessage;
// use crate::message::utils::{base58_to_public_key, base58_to_secret_key};


// #[derive(Debug)]
// pub struct BatchFile {
//     path: PathBuf,
//     dir: PathBuf,
// }
//
//
// impl BatchFile {
//     pub fn new(batch: &Batch) -> BatchFile {
//         let contents = toml::to_string(&batch).unwrap();
//         let file_base64 = base64::encode(contents.as_bytes());
//         let file_hash = sp_core::twox_128(file_base64.as_bytes());
//         let file_hash_string = format!("{}", hex::encode(file_hash));
//
//         let home_dir = dirs::home_dir().unwrap();
//         let home_path = home_dir.as_path();
//         let batch_dir = home_path.join(".nolik").join("export");
//         let file_name = format!("{}.toml", file_hash_string);
//         let batch_file = batch_dir.join(file_name);
//
//         BatchFile {
//             path: batch_file,
//             dir: batch_dir,
//         }
//     }
// }


#[derive(Debug)]
pub struct Batch {
    // #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub messages: Vec<EncryptedMessage>,
}


impl Batch {
    pub fn new(mi: MessageInput) -> Result<Batch, MessageError> {
        let mut messages: Vec<EncryptedMessage> = Vec::new();
        // let sender_pk = match base58_to_public_key(&message_input.sender.public) {
        //     Ok(pk) => pk,
        //     Err(e) => return Err(e),
        // };
        //
        // let sender_sk = match base58_to_secret_key(&message_input.sender.secret) {
        //     Ok(pk) => pk,
        //     Err(e) => return Err(e),
        // };

        for recipient_pk in &mi.recipients {
            let message = match mi.encrypt(&recipient_pk) {
                Ok(message) => message,
                Err(e) => return Err(e),
            };

            messages.push(message);
        }

        Ok(Batch { messages })
    }


    pub fn save(&self) -> Result<(), MessageError> {
        Ok(())
    }
}
