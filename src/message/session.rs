use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::message::nonce::{EncryptedNonce, Nonce};
use crate::MessageInput;
use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use crate::message::errors::MessageError;
use crate::message::group::{EncryptedGroup, Group};


pub struct SessionInput {}

impl SessionInput {
    pub fn new(mi: &MessageInput) -> Session {
        let nonce = Nonce::new(mi.otu.nonce.secret);

        let mut group = Group::new();
        group.add(&mi.sender.public);
        for recipient in &mi.recipients {
            group.add(&recipient);
        }

        Session {
            nonce,
            group
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Session{
    pub nonce: Nonce,
    pub group: Group,
}

impl Session {
    pub fn new(nonce: Nonce, group: Group) -> Session {
        Session {
            nonce,
            group
        }
    }

    pub fn encrypt(&self, mi: &MessageInput, pk: &PublicKey) -> EncryptedSession {
        let encrypted_nonce = self.nonce.encrypt(&mi.otu.nonce.public, &pk, &mi.otu.broker.secret);

        let encrypted_group = self.group.encrypt(&mi.otu.nonce.secret, &pk, &mi.otu.broker.secret);
        // let counterparties = self.group.get_counterparties(pk);
        // let encrypted_counterparties = counterparties.encrypt(&mi.otu.nonce.secret, &pk, &mi.otu.broker.secret);

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

        let session = Session::new(secret_nonce, group);
        Ok(session)
    }
}
