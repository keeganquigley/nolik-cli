use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::message::nonce::{EncryptedNonce, Nonce};
use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use crate::message::errors::MessageError;
use crate::message::group::{EncryptedGroup, Group};
use crate::message::input::BatchInput;


pub struct SessionInput {}

// impl SessionInput {
//     pub fn new(bi: &BatchInput, nonce: &box_::Nonce) -> Session {
//         let nonce = Nonce::new(nonce);
//
//         let mut group = Group::new();
//         group.add(&bi.sender.public);
//         for recipient in &bi.recipients {
//             group.add(&recipient);
//         }
//
//         Session {
//             nonce,
//             group
//         }
//     }
// }


#[derive(Debug, Serialize, Deserialize)]
pub struct Session{
    pub nonce: Nonce,
    pub group: Group,
}

impl Session {
    pub fn new(bi: &BatchInput, nonce: &box_::Nonce) -> Session {
        let nonce = Nonce::new(nonce);

        let mut group = Group::new();
        group.add(&bi.sender.public);
        for recipient in &bi.recipients {
            group.add(&recipient);
        }

        Session {
            nonce,
            group
        }
    }

    pub fn encrypt(
        &self,
        public_nonce: &box_::Nonce,
        pk: &PublicKey,
        sk: &SecretKey
    ) -> EncryptedSession {
        let encrypted_nonce = self.nonce.encrypt(&public_nonce, &pk, &sk);
        let encrypted_group = self.group.encrypt(&self.nonce.0, &pk, &sk);

        EncryptedSession {
            nonce: encrypted_nonce,
            group: encrypted_group,
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedSession {
    nonce: EncryptedNonce,
    group: EncryptedGroup,
}

impl EncryptedSession {
    pub fn decrypt(
        &self,
        public_nonce: &box_::Nonce,
        pk: &PublicKey,
        sk: &SecretKey
    ) -> Result<Session, MessageError> {

        let encrypted_nonce = &self.nonce;
        let secret_nonce = match encrypted_nonce.decrypt(&public_nonce, &pk, &sk) {
            Ok(res) => res,

            Err(e) => return Err(e),
        };

        let encrypted_group = &self.group;
        let group = match encrypted_group.decrypt(&secret_nonce.0, &pk, &sk) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let session = Session {
            nonce: secret_nonce,
            group,
        };

        Ok(session)
    }
}
