use std::time::Duration;
use futures_util::TryStreamExt;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use async_recursion::async_recursion;
use crate::message::errors::MessageError;
use sodiumoxide::crypto::box_::PublicKey;
use sp_core::crypto::AccountId32;
use sp_core::Pair;
// use crate::cli::input::FlagKey;
use crate::{Batch, Wallet};
// use crate::cli::errors::{InputError};
use crate::node::calls::call_extrinsic;
use crate::node::extrinsics::send_message;


// pub struct IpfsInput {}
//
//
// impl IpfsInput {
//     pub fn new(input: &Input) -> Result<IpfsFile, InputError> {
//         let ipfs_id = match input.get_flag_value(FlagKey::IpfsId) {
//             Ok(name) => name,
//             Err(e) => return Err(e)
//         };
//
//         Ok(IpfsFile {
//             ipfs_id,
//         })
//     }
// }


pub struct IpfsFile {
    pub(crate) ipfs_id: String,
}


impl IpfsFile {
    pub fn new(ipfs_id: String) -> IpfsFile {
        IpfsFile {
            ipfs_id,
        }
    }


    #[async_recursion(?Send)]
    pub async fn get(&self) -> Result<Batch, MessageError> {
        let client = IpfsClient::default();
        let data = match client.cat(&self.ipfs_id).map_ok(|chunk| chunk.to_vec()).try_concat().await {
            Ok(res) => {
                match String::from_utf8(res) {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Error on getting IPFS data: {:?}", e);
                        return Err(MessageError::CouldNotReadIpfsData)
                    }
                }
            }
            Err(e) => {
                eprintln!("Error on getting IPFS data: {:?}", e);
                eprintln!("Trying again...");
                async_std::task::sleep(Duration::from_secs(2)).await;
                return Self::get(self).await
            }
        };

        let batch: Batch = match toml::from_str(data.as_str()) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error on parsing IPFS data: {:?}", e);
                return Err(MessageError::CouldNotReadIpfsData);
            }
        };

        Ok(batch)

        // let encrypted_message: EncryptedMessage = match toml::from_str(data.as_str()) {
        //     Ok(res) => res,
        //     Err(e) => {
        //         eprintln!("Error on parsing IPFS data: {:?}", e);
        //         return Err(MessageError::CouldNotReadIpfsData)
        //     }
        // };
        // Ok(encrypted_message)
    }


    pub async fn send(&self, wallet: &Wallet, sender: &PublicKey, recipient: &PublicKey) -> Result<(), MessageError> {

        let (pair, _seed) = match sp_core::sr25519::Pair::from_phrase(&wallet.seed, wallet.password.as_deref()) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(MessageError::CouldNotSendMessage);
            }
        };

        let owner: AccountId32 = sp_core::crypto::AccountId32::from(pair.public());

        let extrinsic_hash = match send_message(
            owner,
            &pair,
            &sender,
            &recipient,
            &self.ipfs_id,
        ).await {
            Ok(hash) => hash,

            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(MessageError::CouldNotSendMessage);
            }
        };

        match call_extrinsic(&extrinsic_hash).await {
            Ok(tx) => {
                println!("Successfully sent the message");
                println!("Transaction ID: {:?}", tx);
                Ok(())
            },
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(MessageError::CouldNotSendMessage)
            },
        }
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct IpfsData {
//     pub from: String,
//     pub to: Vec<String>,
//     pub nonce: String,
//     pub ipfs_hash: String,
//
//     #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
//     pub entries: Vec<Entry>,
// }


// impl IpfsData {
//     pub fn new(message: Message, ipfs_hash: String) -> IpfsData {
//         IpfsData {
//             from: bs58::encode(message.from).into_string(),
//             to: message.to.iter().map(|&el| bs58::encode(el).into_string()).collect(),
//             nonce: base64::encode(message.nonce),
//             ipfs_hash,
//             entries: message.entries,
//         }
//     }
//
//     pub fn save(&self) -> Result<PathBuf, MessageError> {
//         let ipfs_file = IpfsFile::new(&self);
//
//         if let false = ipfs_file.dir.exists() {
//             if let Err(e) = fs::create_dir(&ipfs_file.dir) {
//                 eprintln!("Error: {}", e);
//                 return Err(MessageError::CouldNotCreateDataDir)
//             }
//         }
//
//         match fs::File::create(&ipfs_file.path) {
//             Ok(mut file) => {
//                 let contents = match toml::to_string(&self) {
//                     Ok(contents) => contents,
//                     Err(e) => {
//                         eprintln!("Error: {}", e);
//                         return Err(MessageError::CouldNotCreateLocalFile)
//                     },
//                 };
//                 match file.write_all(contents.as_ref()) {
//                     Ok(_) => Ok(ipfs_file.path),
//                     Err(e) => {
//                         eprintln!("Error: {}", e);
//                         return Err(MessageError::CouldNotSaveContentsToLocalFile)
//                     }
//                 }
//             },
//             Err(e) => {
//                 eprintln!("Error: {}", e);
//                 return Err(MessageError::CouldNotCreateLocalFile)
//             }
//         }
//     }
// }


// pub struct IpfsFile {
//     path: PathBuf,
//     dir: PathBuf,
// }
//
//
// impl IpfsFile {
//     pub fn new(ipfs_data: &IpfsData) -> IpfsFile {
//         let contents = toml::to_string(&ipfs_data).unwrap();
//         let file_base64 = base64::encode(contents.as_bytes());
//         let file_hash = sp_core::twox_128(file_base64.as_bytes());
//
//         let home_dir = dirs::home_dir().unwrap();
//         let home_path = home_dir.as_path();
//         let ipfs_dir = home_path.join(".nolik").join("data");
//         let file_name = format!("{}.toml", hex::encode(file_hash));
//         let ipfs_file = ipfs_dir.join(file_name);
//
//         IpfsFile {
//             path: ipfs_file,
//             dir: ipfs_dir,
//         }
//     }
// }