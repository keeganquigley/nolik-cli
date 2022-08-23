use serde_derive::{Serialize, Deserialize};
use sodiumoxide::crypto::box_::PublicKey;
use crate::message::encryption::Encryption;


#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Party(pub PublicKey);
impl Encryption for Party {}


#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedParty(pub String);
impl Encryption for EncryptedParty {}




