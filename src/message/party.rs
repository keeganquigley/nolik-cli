use sodiumoxide::crypto::box_;
use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey};
use crate::message::encryption::Encryption;
use crate::message::errors::MessageError;
use crate::message::input::MessageInput;
use crate::message::utils::base64_to_vec;


#[derive(Debug)]
pub struct Party {
    pub nonce: Nonce,
    pub others: Vec<PublicKey>,
}


impl Encryption for Party {}


impl Party {
    pub fn encrypt(&self, mi: &MessageInput, pk: &PublicKey) -> EncryptedParty {
        let nonce = Party::encrypt_data(&self.nonce.as_ref(), &mi.otu.nonce.public, pk, &mi.otu.broker.secret);

        let mut others: Vec<String> = Vec::new();
        for counterparty in &self.others {
            let other = Party::encrypt_data(&counterparty.as_ref(), &mi.otu.nonce.secret, &pk, &mi.otu.broker.secret);
            others.push(other);
        }

        EncryptedParty {
            nonce,
            others,
        }
    }

}


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedParty {
    // #[serde(rename = "n")]
    nonce: String,

    // #[serde(rename = "o")]
    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    others: Vec<String>,
}

impl Encryption for EncryptedParty {}

impl EncryptedParty {
    pub fn decrypt(&self, public_nonce: &Nonce, broker: &PublicKey, sk: &SecretKey) -> Result<Party, MessageError> {
        let encrypted_nonce = match base64_to_vec(&self.nonce) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let nonce = match Party::decrypt_data(&encrypted_nonce, &public_nonce, &broker, &sk) {
            Ok(res) => match box_::Nonce::from_slice(res.as_slice()) {
                Some(nonce) => nonce,
                None => return Err(MessageError::DecryptionError),
            }
            Err(e) => return Err(e),
        };

        let mut others: Vec<PublicKey> = Vec::new();
        for other in &self.others {
            let encrypted_counterparty = match base64_to_vec(&other) {
                Ok(res) => res,
                Err(e) => return Err(e),
            };

            let counterparty = match Party::decrypt_data(&encrypted_counterparty, &nonce, &broker, &sk) {
                Ok(res) => match box_::PublicKey::from_slice(res.as_slice()) {
                    Some(counterparty) => counterparty,
                    None => return Err(MessageError::DecryptionError),
                },
                Err(e) => return Err(e),
            };

            others.push(counterparty)
        }

        Ok(Party {
            nonce,
            others,
        })
    }
}