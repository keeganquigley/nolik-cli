
use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::message::encryption::Encryption;
use crate::message::errors::MessageError;
use crate::message::utils::base64_to_vec;
use crate::message::party::{EncryptedParty, Party};



#[derive(Debug, Serialize, Deserialize)]
pub struct Group(pub Vec<Party>);


impl Group {
    pub fn new() -> Group {
        let parties: Vec<Party> = Vec::new();
        Group(parties)
    }

    pub fn add(&mut self, pk: &PublicKey) {
        self.0.push(Party(*pk));
    }

    pub fn encrypt(&self, nonce: &box_::Nonce, pk: &PublicKey, sk: &SecretKey) -> EncryptedGroup {
        let mut encrypted_group = Vec::new();
        for party in &self.0 {
            let encrypted_party = Party::encrypt_data(party.0.as_ref(), &nonce, pk, sk);
            encrypted_group.push(encrypted_party);
        }

        EncryptedGroup(encrypted_group)
    }

    // pub fn get_counterparties(&self, pk: &PublicKey) -> Parties {
    //     Parties(self.0.iter().filter(|p| p.0.ne(pk)).map(|p| *p).collect())
    // }

    pub fn get_sender(&self) -> PublicKey {
        self.0.first().unwrap().0
    }

    pub fn get_recipients(&self) -> Vec<PublicKey> {
        self.0.iter().skip(1).map(|p| p.0).collect()
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedGroup(Vec<String>);


impl EncryptedGroup {
    pub fn new(encrypted_group: &Vec<String>) -> EncryptedGroup {
        EncryptedGroup(encrypted_group.iter().map(|ep| ep.to_string()).collect())
    }

    pub fn decrypt(&self, nonce: &box_::Nonce, pk: &PublicKey, sk: &SecretKey) -> Result<Group, MessageError> {

        let mut group: Vec<Party> = Vec::new();
        for ep in &self.0 {
            let encrypted_party = match base64_to_vec(ep) {
                Ok(res) => res,
                Err(e) => return Err(e),
            };

            let party = match EncryptedParty::decrypt_data(&encrypted_party, &nonce, pk, sk) {
                Ok(res) => match PublicKey::from_slice(res.as_slice()) {
                    Some(party) => Party(party),
                    None => return Err(MessageError::DecryptionError),
                }
                Err(e) => return Err(e),
            };

            group.push(party);
        }

        Ok(Group(group))
    }
}
