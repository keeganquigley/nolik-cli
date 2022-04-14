use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    ciphertext: String,
    hash: String,
}
