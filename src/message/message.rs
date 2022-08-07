use std::io::Cursor;
use sodiumoxide::crypto::box_::{Nonce, PublicKey, SecretKey};
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use crate::message::entry::{Entry, EncryptedEntry};
use serde_derive::{Serialize, Deserialize};
use crate::Account;
use crate::message::blob::{Blob, EncryptedBlob};
use crate::message::errors::MessageError;
use crate::message::party::EncryptedParty;
use crate::message::utils::{base64_to_nonce, base64_to_public_key};
use blake2::Digest;
use crate::message::ipfs::{IpfsFile};


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    // #[serde(rename = "n")]
    pub nonce: String,

    // #[serde(rename = "b")]
    pub broker: String,

    // #[serde(rename = "s")]
    pub sender: String,

    // #[serde(rename = "h")]
    pub hash: String,

    // #[serde(rename = "p")]
    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub parties: Vec<EncryptedParty>,

    // #[serde(rename = "e")]
    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub entries: Vec<EncryptedEntry>,

    // #[serde(rename = "b")]
    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub blobs: Vec<EncryptedBlob>,

}


impl EncryptedMessage {
    pub fn decrypt(&mut self, account: &Account) -> Result<Message, MessageError> {
        let public_nonce = match base64_to_nonce(&self.nonce) {
            Ok(nonce) => nonce,
            Err(e) => return Err(e),
        };

        let broker = match base64_to_public_key(&self.broker) {
            Ok(broker) => broker,
            Err(e) => return Err(e),
        };

        let (nonce, parties) =  match Self::decrypt_secret_nonce(&mut self.parties,&public_nonce, &broker, &account.secret) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };


        let (from, mut to) = match Self::get_from_to(&self, &account, &nonce, &parties) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };


        let is_sender = from == account.public;

        let entries = match is_sender {
            true => {
                match Self::try_decrypt_entries(&self.entries, &nonce, &mut to.clone(), &account.secret) {
                    Ok(entries) => entries,
                    Err(e) => return Err(e),
                }
            }
            false => match Self::decrypt_entries(&self.entries, &nonce, &from, &account.secret) {
                Ok(entries) => entries,
                Err(e) => return Err(e),
            }
        };


        let blobs = match is_sender {
            true => {
                match Self::try_decrypt_blobs(&self.blobs, &nonce, &mut to, &account.secret) {
                    Ok(entries) => entries,
                    Err(e) => return Err(e),
                }
            }
            false => match Self::decrypt_blobs(&self.blobs, &nonce, &from, &account.secret) {
                Ok(entries) => entries,
                Err(e) => return Err(e),
            }
        };


        Ok(Message {
            nonce,
            from,
            to,
            entries,
            blobs,
            hash: self.hash.to_string(),
        })
    }



    fn decrypt_secret_nonce(parties: &mut Vec<EncryptedParty>, public_nonce: &Nonce, broker: &PublicKey, sk: &SecretKey) -> Result<(Nonce, Vec<PublicKey>), MessageError> {
        let party  = match parties.pop() {
            Some(party) => party,
            None => return Err(MessageError::DecryptionError),
        };

        match party.decrypt(&public_nonce, &broker, &sk) {
            Ok(party) => Ok((party.nonce, party.others)),
            Err(_e) => Self::decrypt_secret_nonce(parties, public_nonce, broker, sk),
        }
    }


    fn try_decrypt_entries(encrypted_entries: &Vec<EncryptedEntry>, nonce: &Nonce, pks: &mut Vec<PublicKey>, sk: &SecretKey) -> Result<Vec<Entry>, MessageError> {
        let pk = match pks.pop() {
            Some(pk) => pk,
            None => return Err(MessageError::DecryptionError),
        };

        match Self::decrypt_entries(&encrypted_entries, &nonce, &pk, &sk) {
            Ok(entries) => Ok(entries),
            Err(_e) => Self::try_decrypt_entries(encrypted_entries, nonce, pks, sk),
        }
    }


    fn decrypt_entries(encrypted_entries: &Vec<EncryptedEntry>, nonce: &Nonce, pk: &PublicKey, sk: &SecretKey) -> Result<Vec<Entry>, MessageError> {
        encrypted_entries
            .iter()
            .map(|el|
                match el.decrypt(&nonce, &pk, &sk) {
                    Ok(entry) => Ok(entry),
                    Err(e) => return Err(e)
                }
            )
            .collect()
    }


    fn try_decrypt_blobs(encrypted_blobs: &Vec<EncryptedBlob>, nonce: &Nonce, pks: &mut Vec<PublicKey>, sk: &SecretKey) -> Result<Vec<Blob>, MessageError> {
        let pk = match pks.pop() {
            Some(pk) => pk,
            None => return Err(MessageError::DecryptionError),
        };

        match Self::decrypt_blobs(&encrypted_blobs, &nonce, &pk, &sk) {
            Ok(entries) => Ok(entries),
            Err(_e) => Self::try_decrypt_blobs(encrypted_blobs, nonce, pks, sk),
        }
    }


    fn decrypt_blobs(encrypted_blobs: &Vec<EncryptedBlob>, nonce: &Nonce, pk: &PublicKey, sk: &SecretKey) -> Result<Vec<Blob>, MessageError> {
        encrypted_blobs
            .iter()
            .map(|el|
                match el.decrypt(&nonce, &pk, &sk) {
                    Ok(entry) => Ok(entry),
                    Err(e) => return Err(e)
                }
            )
            .collect()
    }


    fn get_from_to(&self, account: &Account, nonce: &Nonce, parties: &Vec<PublicKey>) -> Result<(PublicKey, Vec<PublicKey>), MessageError> {
        let mut all_parties: Vec<PublicKey> = Vec::new();
        all_parties.push(account.public);
        for party in parties {
            all_parties.push(party.to_owned());
        }

        for pk in &all_parties {
            let hash = Self::get_pk_salted_hash(&pk, &nonce);
            if hash == self.sender {
                let recipients = all_parties.iter().filter(|&el| el.ne(&pk)).map(|pk| *pk).collect();
                return Ok((pk.to_owned(), recipients))
            }
        }

        Err(MessageError::DecryptionError)
    }


    fn get_pk_salted_hash(pk: &PublicKey, nonce: &Nonce) -> String {
        let mut hasher = blake2::Blake2s256::new();
        hasher.update(pk);
        hasher.update(nonce);
        base64::encode(hasher.finalize().to_vec())
    }


    pub async fn save(&self) -> Result<IpfsFile, MessageError> {
        let contents = match toml::to_string(&self) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(MessageError::CouldNotCreateTomlFileContents);
            },
        };

        let client = IpfsClient::default();
        let data = Cursor::new(contents);
        match client.add(data).await {
            Ok(res) => {
                match client.pin_add(&res.hash, true).await {
                    Ok(_res) => {
                        println!("File has been saved to the IPFS network with ID: {:?}", res.hash);
                    },
                    Err(e) => {
                        eprintln!("Error on pinning IPFS file: {:#?}", e);
                        return Err(MessageError::CouldNotAddFileToIPFS)
                    }
                }
                Ok(IpfsFile {
                    ipfs_id: res.hash
                })
            },
            Err(e) => {
                eprintln!("Error on adding file to IPFS: {:#?}", e);
                return Err(MessageError::CouldNotAddFileToIPFS)
            }
        }
    }


    // pub fn encode_decrypted_ipfs_data(sor: &SenderOrRecipient, ipfs_hash: &String) {
    //
    //     // let handle = async_std::task::spawn(async {
    //     //     let em = Self::get_encrypted_message(&ipfs_hash).await;
    //     //     em.unwrap()
    //     // });
    //
    //     let handle = std::thread::spawn(move || {
    //         // Self::get_encrypted_message(&ipfs_hash)
    //     });
    //
    //     handle.join().unwrap();
    //
    //
    //     // handle.join().unwrap()
    // }
}



#[derive(Debug)]
pub struct Message {
    pub nonce: Nonce,
    pub from: PublicKey,
    pub to: Vec<PublicKey>,
    pub entries: Vec<Entry>,
    pub blobs: Vec<Blob>,
    pub hash: String,
}


impl Message {
    pub fn save(&self) {

    }
}
// trait VectorItems<T, U> where T: Encryption {
//     fn decrypt(items: Vec<T>, nonce: &Nonce, pk: &PublicKey, sk: &SecretKey) -> Result<Vec<U>, MessageError> {
//         let mut decrypted_items: Vec<U> = Vec::new();
//         for item in items {
//             match item.decrypt(&nonce, &pk, &sk) {
//                 Ok(decrypted_item) => {
//                     decrypted_items.push(decrypted_item);
//                 }
//                 Err(e) => return Err(e),
//             };
//         }
//
//         Ok(decrypted_items)
//     }
// }
//
//
// impl VectorItems<EncryptedBlob, Blob> for EncryptedBlob { }
// impl VectorItems<EncryptedEntry, Entry> for EncryptedEntry { }
