use serde::{Serialize, Deserialize};
use std::str::FromStr;
use sp_core::H256;
use crate::node::errors::NodeError;
use crate::node::socket::{Socket, SocketMessage};


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SystemSyncState {
    current_block: u32,
    starting_block: u32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct SystemSyncStateSuccess {
    jsonrpc: String,
    result: SystemSyncState,
    id: u8,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct SystemAccountNextIndexSuccess {
    jsonrpc: String,
    result: u32,
    id: u8,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ChainGetBlockHashSuccess {
    jsonrpc: String,
    result: String,
    id: u8,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StateGetRuntimeVersionApis {
    Int(u32),
    Str(String),
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateGetRuntimeVersion {
    apis: Vec<[StateGetRuntimeVersionApis;2]>,
    authoring_version: u32,
    impl_name: String,
    impl_version: u32,
    spec_name: String,
    pub spec_version: u32,
    pub transaction_version: u32,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct StateGetRuntimeVersionSuccess {
    jsonrpc: String,
    result: StateGetRuntimeVersion,
    id: u8,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct StateGetRuntimeMetadataSuccess {
    jsonrpc: String,
    result: String,
    id: u8,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthorSubmitExtrinsicSuccessSubscriptionId {
    jsonrpc: String,
    result: String,
    id: u8,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthorSubmitExtrinsicSuccessReady {
    jsonrpc: String,
    method: String,
    params: AuthorSubmitExtrinsicSuccessParamsReady,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthorSubmitExtrinsicSuccessBlock {
    jsonrpc: String,
    method: String,
    params: AuthorSubmitExtrinsicSuccessParamsBlock,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthorSubmitExtrinsicSuccessParamsReady {
    result: String,
    subscription: String,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthorSubmitExtrinsicSuccessParamsBlock {
    result: AuthorSubmitExtrinsicSuccessParamsResult,
    subscription: String,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AuthorSubmitExtrinsicSuccessParamsResult {
    InBlock(AuthorSubmitExtrinsicSuccessParamsResultInBlock),
    Finalised(AuthorSubmitExtrinsicSuccessParamsResultFinalized),
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthorSubmitExtrinsicSuccessParamsResultInBlock {
    in_block: String,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthorSubmitExtrinsicSuccessParamsResultFinalized {
    finalized: String,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthorSubmitExtrinsicError {
    jsonrpc: String,
    error: AuthorSubmitExtrinsicErrorBody,
    id: u8,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthorSubmitExtrinsicErrorBody {
    code: usize,
    message: String,
    data: String,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct StateGetStorageAtSuccess {
    jsonrpc: String,
    result: String,
    id: u8,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AuthorSubmitExtrinsicResult {
    SubscriptionId(AuthorSubmitExtrinsicSuccessSubscriptionId),
    Ready(AuthorSubmitExtrinsicSuccessReady),
    Block(AuthorSubmitExtrinsicSuccessBlock),
    Error(AuthorSubmitExtrinsicError),
}


#[derive(Serialize)]
pub struct NodeRequest {
    id: u8,
    jsonrpc: String,
    method: String,
    params: Vec<String>,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ChainGetBlockSuccess {
    jsonrpc: String,
    result: ChainGetBlockResult,
    id: u8,
}



#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StateGetStorageResult {
    Some(StateGetStorageSuccessSome),
    None(StateGetStorageSuccessNone),
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct StateGetStorageSuccessSome {
    jsonrpc: String,
    result: String,
    id: u8,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct StateGetStorageSuccessNone {
    jsonrpc: String,
    result: Option<u32>,
    id: u8,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ChainGetBlockResult {
    pub block: ChainGetBlock,
    justifications: Option<String>,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ChainGetBlock {
    pub extrinsics: Vec<String>,
    header: ChainGetBlockHeader,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChainGetBlockHeader {
    digest: ChainGetBlockHeaderDigest,
    extrinsics_root: String,
    number: String,
    parent_hash: String,
    state_root: String,
}


#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ChainGetBlockHeaderDigest {
    logs: Vec<String>,
}



pub async fn get_nonce(socket: &mut Socket, account: &sp_runtime::AccountId32) -> Result<u32, NodeError> {

    let req = NodeRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "system_accountNextIndex".to_string(),
        params: vec![account.to_string()],
    };

    if let Err(e) = SocketMessage::send(socket, req) {
        return Err(e);
    }

    let msg = match SocketMessage::read(socket) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let nonce: SystemAccountNextIndexSuccess = match serde_json::from_str(&msg) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return Err(NodeError::CouldNotGetAccountNonce);
        }
    };

    Ok(nonce.result)
}


pub async fn get_genesis_hash(socket: &mut Socket) -> Result<H256, NodeError> {

    let req = NodeRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "chain_getBlockHash".to_string(),
        params: vec!["0".to_string()],
    };

    if let Err(e) = SocketMessage::send(socket, req) {
        return Err(e);
    }

    let msg = match SocketMessage::read(socket) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let hash: ChainGetBlockHashSuccess = match serde_json::from_str(&msg) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return Err(NodeError::CouldNotGetAccountNonce);
        }
    };

    let genesis_hash = hash.result.replace("0x", "");

    Ok(sp_core::H256::from_str(genesis_hash.as_ref()).unwrap())
}


pub async fn get_runtime_version(socket: &mut Socket) -> Result<StateGetRuntimeVersion, NodeError> {
    let req = NodeRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "state_getRuntimeVersion".to_string(),
        params: vec![],
    };

    if let Err(e) = SocketMessage::send(socket, req) {
        return Err(e);
    }

    let msg = match SocketMessage::read(socket) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let runtime_version: StateGetRuntimeVersionSuccess = match serde_json::from_str(&msg) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return Err(NodeError::CouldNotGetAccountNonce);
        }
    };

    Ok(runtime_version.result)
}


pub async fn get_runtime_metadata(socket: &mut Socket) -> Result<String, NodeError> {
    let req = NodeRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "state_getMetadata".to_string(),
        params: vec![],
    };

    if let Err(e) = SocketMessage::send(socket, req) {
        return Err(e);
    }

    let msg = match SocketMessage::read(socket) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let runtime_metadata: StateGetRuntimeMetadataSuccess = match serde_json::from_str(&msg) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return Err(NodeError::CouldNotGetAccountNonce);
        }
    };

    Ok(runtime_metadata.result)
}


pub async fn get_block(socket: &mut Socket, block_hash: &String) -> Result<ChainGetBlockResult, NodeError> {

    let req = NodeRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "chain_getBlock".to_string(),
        params: vec![block_hash.to_string()],
    };

    if let Err(e) = SocketMessage::send(socket, req) {
        return Err(e);
    }

    let msg = match SocketMessage::read(socket) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let block: ChainGetBlockSuccess = match serde_json::from_str(&msg) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return Err(NodeError::CouldNotGetAccountNonce);
        }
    };

    Ok(block.result)
}


pub async fn get_storage_value(socket: &mut Socket, storage_key: String) -> Result<Option<String>, NodeError> {

    let req = NodeRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "state_getStorage".to_string(),
        params: vec![storage_key],
    };

    if let Err(e) = SocketMessage::send(socket, req) {
        return Err(e);
    }

    let msg = match SocketMessage::read(socket) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let res: StateGetStorageResult = match serde_json::from_str(&msg) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return Err(NodeError::CouldNotGetAccountNonce);
        }
    };


    match res {
        StateGetStorageResult::Some(res) => Ok(Some(res.result)),
        StateGetStorageResult::None(_) => Ok(None),
    }
}


pub async fn call_extrinsic(socket: &mut Socket, call_hex: &String) -> Result<String, NodeError> {

    let req = NodeRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "author_submitAndWatchExtrinsic".to_string(),
        params: vec![call_hex.to_string()],
    };

    println!("Sending request...");

    if let Err(e) = SocketMessage::send(socket, req) {
        return Err(e);
    }

    loop {

        let msg = match SocketMessage::read(socket) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let res: AuthorSubmitExtrinsicResult = match serde_json::from_str(&msg) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(NodeError::CouldNotCallExtrinsic);
            }
        };


        match res {
            AuthorSubmitExtrinsicResult::Ready(_res) => {
                println!("==> Ready");
            },
            AuthorSubmitExtrinsicResult::SubscriptionId(_res) => {},
            AuthorSubmitExtrinsicResult::Block(res) => {
                return match res.params.result {
                    AuthorSubmitExtrinsicSuccessParamsResult::InBlock(res) => {
                        println!("==> In block");
                        println!("==> Block ID: {:?}", res.in_block);
                        Ok(res.in_block)
                    },
                    AuthorSubmitExtrinsicSuccessParamsResult::Finalised(res) => {
                        println!("==> Finalized");
                        Ok(res.finalized)
                    }
                }
            },
            AuthorSubmitExtrinsicResult::Error(res) => {
                let error = format!("{}. {}", res.error.message, res.error.data);
                eprintln!("Error: {:?}", error);
                return Err(NodeError::CouldNotCallExtrinsic)
            }
        }
    }
}
