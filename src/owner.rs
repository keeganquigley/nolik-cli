use std::str::FromStr;
use sp_core::crypto::AccountId32;
use crate::{Account, ConfigFile, FlagKey, Input, Wallet};
use crate::cli::errors::InputError;
use sp_core::Pair;
use subxt::{ClientBuilder, DefaultConfig, ErrorMetadata, PolkadotExtrinsicParams, RawEventDetails};
use parity_scale_codec::Decode;
use futures::StreamExt;
use sp_runtime::DispatchError::Module;
use subxt::Phase::ApplyExtrinsic;
use crate::node::calls::{call_extrinsic, get_block};
use crate::node::extrinsics::add_owner;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

pub struct Owner {
    wallet: Wallet,
    account: Account,
}


impl Owner {
    pub fn new(input: &Input, config_file: &ConfigFile, password: Option<String>) -> Result<Owner, InputError> {
        let wallet_key = match input.get_flag_value(FlagKey::Wallet) {
            Ok(key) => key,
            Err(e) => return Err(e),
        };

        let wallet = match Wallet::get(&config_file, wallet_key, password) {
            Ok(wallet) => wallet,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(InputError::CouldNotAddOwner);
            }
        };

        let account_key = match input.get_flag_value(FlagKey::Account) {
            Ok(key) => key,
            Err(e) => return Err(e),
        };

        let account = match Account::get(&config_file, account_key) {
            Ok(account) => account,
            Err (e) => {
                eprintln!("Error: {}", e);
                return Err(InputError::CouldNotAddOwner);
            }
        };

        Ok(Owner {
            wallet,
            account,
        })
    }


    pub async fn add(&self) -> Result<(), InputError> {
        let (pair, _seed) = match sp_core::sr25519::Pair::from_phrase(&self.wallet.seed, self.wallet.password.as_deref()) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(InputError::CouldNotAddOwner);
            }
        };

        let owner: AccountId32 = sp_core::crypto::AccountId32::from(pair.public());

        let extrinsic_hash = match add_owner(
            owner,
            &pair,
            &self.account.public,
        ).await {
            Ok(hash) => hash,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(InputError::CouldNotAddOwner);
            }
        };

        let api = ClientBuilder::new()
            .build()
            .await
            .unwrap()
            .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<_>>>();

        let mut event_sub = api.events().subscribe().await.unwrap();

        let block_hash = match call_extrinsic(&extrinsic_hash).await {
            Ok(res) => res,
            Err(_e) => return Err(InputError::CouldNotAddOwner),
        };


        let block = match get_block(&block_hash).await {
            Ok(res) => res,
            Err(_e) => return Err(InputError::CouldNotAddOwner),
        };

        let extrinsic_index: u32 = match block.block.extrinsics.iter().position(|el| el == &extrinsic_hash) {
            Some(index) => index as u32,
            None => return Err(InputError::CouldNotAddOwner),
        };

        while let Some(events) = event_sub.next().await {

            let events = match events {
                Ok(events) => events,
                Err(_e) => return Err(InputError::CouldNotAddOwner),
            };

            let block_hash = sp_core::H256::from_str(block_hash.as_str());
            if let Err(e) = block_hash {
                eprintln!("Error: {}", e);
                return Err(InputError::CouldNotAddOwner);
            }

            if events.block_hash() == block_hash.unwrap() {
                for event in events.iter_raw() {
                    let event = match event {
                        Ok(event) => event,
                        Err(_e) => return Err(InputError::CouldNotAddOwner),
                    };

                    if event.phase.eq(&ApplyExtrinsic(extrinsic_index)) {

                        let nolik_event = event.as_event::<polkadot::nolik::events::AddOwner>().unwrap();
                        if nolik_event.is_some() {
                            println!("Owner has been successfully added to \"{}\"", self.account.alias);
                        }

                        let failed_event = event.as_event::<polkadot::system::events::ExtrinsicFailed>().unwrap();
                        if failed_event.is_some() {
                            let ev = RawEventDetails::from(event);
                            let dispatch_error = sp_runtime::DispatchError::decode(&mut &*ev.data).unwrap();
                            let locked_metadata = api.client.metadata();
                            let metadata = locked_metadata.read();

                            let module_error = match dispatch_error {
                                Module(module_error) => module_error,
                                _ => return Err(InputError::CouldNotAddOwner),
                            };

                            let error_metadata = match metadata.error(module_error.index, module_error.error) {
                                Ok(error_metadata) => ErrorMetadata::from(error_metadata.clone()),
                                Err(_e) => return Err(InputError::CouldNotAddOwner),
                            };

                            // println!("Is Failed: {:?}", error.0);
                            println!("Error: {:#?}", error_metadata.error());

                        }
                    }
                }

                break;
            }
        }

        Ok(())
    }
}
