use std::time::Duration;
use futures_util::TryStreamExt;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use async_recursion::async_recursion;
use sodiumoxide::crypto::box_::PublicKey;
use crate::{Batch, ConfigFile, FlagKey, Input, NodeError, NodeEvent, Wallet};
// use crate::node::extrinsics::send_message;
use crate::message::errors::MessageError;
use crate::node::events::SendMessage;
use colored::Colorize;
use crate::cli::errors::InputError;
use crate::node::extrinsics::NolikSendMessage;


pub struct IpfsInput {
    pub ipfs_file: IpfsFile,
    pub wallet: Wallet,
}

impl IpfsInput {
    pub fn new(config_file: &ConfigFile, input: &Input, password: Option<String>) -> Result<IpfsInput, InputError> {
        let wallet_alias = match input.get_flag_value(FlagKey::Wallet) {
            Ok(name) => name,
            Err(e) => return Err(e),
        };

        let ipfs_id = match input.get_flag_value(FlagKey::IpfsId) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };

        let wallet = match Wallet::get(&config_file, wallet_alias, password){
            Ok(res) => res,
            Err(_e) => return Err(InputError::CouldNotGetWallet)
        };

        let ipfs_file = IpfsFile::new(ipfs_id);

        Ok(IpfsInput {
            ipfs_file,
            wallet,
        })
    }
}


pub struct IpfsFile(pub String);

impl IpfsFile {
    pub fn new(id: String) -> IpfsFile {
        IpfsFile(id)
    }


    #[async_recursion(?Send)]
    pub async fn get(&self) -> Result<Batch, MessageError> {
        let client = IpfsClient::default();
        let data = match client.cat(&self.0).map_ok(|chunk| chunk.to_vec()).try_concat().await {
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
    }


    pub async fn send(&self, config_file: &ConfigFile, sender: &PublicKey, recipient: &PublicKey, wallet: &Wallet) -> Result<(), NodeError> {

        let pair = match wallet.get_pair() {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(NodeError::CouldNotSubmitEvent);
            }
        };

        let extrinsic = match NolikSendMessage::new(&config_file, &pair, &sender, &recipient, &self.0) {
            Ok(res) => res,
            Err(_e) => return Err(NodeError::CouldNotSubmitEvent),
        };

        let extrinsic_hash = match extrinsic.hash::<NolikSendMessage>().await {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let event = SendMessage;
        match event.submit(&config_file, &extrinsic_hash).await {
            Ok(_res) => {
                let to = bs58::encode(recipient).into_string();
                let res = format!("Message has been sent to \"{}\"", to);
                println!("{}", res.bright_green());
                Ok(())
            },
            Err(e) => return Err(e),
        }
    }
}