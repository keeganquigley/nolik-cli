use std::str::FromStr;
use sp_runtime::DispatchError::Module;
use subxt::{ClientBuilder, DefaultConfig, ErrorMetadata, Event, PolkadotExtrinsicParams, RawEventDetails};
use subxt::Phase::ApplyExtrinsic;
use parity_scale_codec::Decode;
use crate::cli::constants::pallet_errors;
use crate::node::calls::{call_extrinsic, get_block};
use crate::NodeError;
use async_trait::async_trait;
use futures::StreamExt;


#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}


#[async_trait]
pub trait NodeEvent {
    type S: Event;
    type F: Event;

    async fn submit(&self, extrinsic_hash: &String) -> Result<(), NodeError> {
        let api = ClientBuilder::new()
            .build()
            .await
            .unwrap()
            .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<_>>>();

        let mut event_sub = api.events().subscribe().await.unwrap();

        let block_hash = match call_extrinsic(&extrinsic_hash).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error {}", e);
                return Err(NodeError::CouldNotSubmitEvent);
            },
        };

        let block = match get_block(&block_hash).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error {}", e);
                return Err(NodeError::CouldNotSubmitEvent);
            },
        };

        let extrinsic_index: u32 = match block.block.extrinsics.iter().position(|el| el == extrinsic_hash) {
            Some(index) => index as u32,
            None => return Err(NodeError::CouldNotSubmitEvent),
        };

        while let Some(events) = event_sub.next().await {

            let events = match events {
                Ok(events) => events,
                Err(_e) => return Err(NodeError::CouldNotSubmitEvent)
            };

            let block_hash = sp_core::H256::from_str(block_hash.as_str());
            if let Err(e) = block_hash {
                eprintln!("Error: {}", e);
                return Err(NodeError::CouldNotSubmitEvent);
            }

            if events.block_hash() == block_hash.unwrap() {
                for event in events.iter_raw() {
                    let event = match event {
                        Ok(event) => event,
                        Err(_e) =>  return Err(NodeError::CouldNotSubmitEvent),
                    };

                    if event.phase.eq(&ApplyExtrinsic(extrinsic_index)) {
                        let success_event = event.as_event::<Self::S>().unwrap();

                        if success_event.is_some() {
                            return Ok(());
                        }

                        let failed_event = event.as_event::<Self::F>().unwrap();
                        if failed_event.is_some() {
                            let ev = RawEventDetails::from(event);
                            let dispatch_error = sp_runtime::DispatchError::decode(&mut &*ev.data).unwrap();
                            let locked_metadata = api.client.metadata();
                            let metadata = locked_metadata.read();

                            let module_error = match dispatch_error {
                                Module(module_error) => module_error,
                                _ => return Err(NodeError::CouldNotSubmitEvent),
                            };

                            let error_metadata = match metadata.error(module_error.index, module_error.error) {
                                Ok(error_metadata) => ErrorMetadata::from(error_metadata.clone()),
                                Err(_e) =>  return Err(NodeError::CouldNotSubmitEvent),
                            };

                            let pallet_error = match error_metadata.error() {
                                pallet_errors::ERROR_ACCOUNT_IN_OWNERS => NodeError::PalletAccountInOwners,
                                pallet_errors::ERROR_ADDRESS_NOT_OWNED => NodeError::PalletAddressNotOwned,
                                pallet_errors::ERROR_ALREADY_IN_WHITELIST => NodeError::PalletAlreadyInWhitelist,
                                pallet_errors::ERROR_ALREADY_IN_BLACKLIST => NodeError::PalletAlreadyInBlacklist,
                                pallet_errors::ERROR_SAME_ADDRESS => NodeError::PalletSameAddress,
                                pallet_errors::ERROR_ADDRESS_IN_BLACKLIST => NodeError::PalletAddressInBlacklist,
                                pallet_errors::ERROR_ADDRESS_NOT_IN_WHITELIST => NodeError::PalletAddressNotInWhitelist,
                                _ => NodeError::PalletUnknownError,
                            };

                            return Err(pallet_error);
                        }
                    }
                }

                break;
            }
        }

        Err(NodeError::CouldNotSubmitEvent)
    }
}


#[derive(Debug)]
pub struct AddOwnerEvent;

impl NodeEvent for AddOwnerEvent {
    type S = polkadot::nolik::events::AddOwner;
    type F = polkadot::system::events::ExtrinsicFailed;
}


#[derive(Debug)]
pub struct AddToWhitelist;

impl NodeEvent for AddToWhitelist {
    type S = polkadot::nolik::events::AddWhiteList;
    type F = polkadot::system::events::ExtrinsicFailed;
}


#[derive(Debug)]
pub struct AddToBlacklist;

impl NodeEvent for AddToBlacklist {
    type S = polkadot::nolik::events::AddBlackList;
    type F = polkadot::system::events::ExtrinsicFailed;
}


#[derive(Debug)]
pub struct SendMessage;

impl NodeEvent for SendMessage {
    type S = polkadot::nolik::events::MessageSent;
    type F = polkadot::system::events::ExtrinsicFailed;
}


#[derive(Debug)]
pub struct BalanceTransferEvent;

impl NodeEvent for BalanceTransferEvent {
    type S = polkadot::balances::events::Transfer;
    type F = polkadot::system::events::ExtrinsicFailed;
}



