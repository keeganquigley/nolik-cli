use serde_derive::{Serialize, Deserialize};
use crate::message::errors::MessageError;
use crate::message::input::MessageInput;
use crate::message::message::EncryptedMessage;
use crate::message::utils::{base58_to_public_key, base58_to_secret_key};

#[derive(Debug, Serialize, Deserialize)]
pub struct Batch {
    pub messages: Vec<EncryptedMessage>,
}


impl Batch {
    pub fn new(message_input: MessageInput) -> Result<Batch, MessageError> {
        let mut messages: Vec<EncryptedMessage> = Vec::new();
        let sender_pk = match base58_to_public_key(&message_input.sender.public) {
            Ok(pk) => pk,
            Err(e) => return Err(e),
        };

        let sender_sk = match base58_to_secret_key(&message_input.sender.secret) {
            Ok(pk) => pk,
            Err(e) => return Err(e),
        };

        for recipient_pk in &message_input.recipients {
            let message = match message_input.encrypt(&sender_pk, &sender_sk, &recipient_pk) {
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
