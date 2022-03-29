use serde::{Serialize, Deserialize};
use std::str::FromStr;
// use sp_keyring::AccountKeyring;
// use sp_runtime::{MultiAddress, MultiSignature, generic::Era};
// use parity_scale_codec::{ Compact, Encode };
use sp_core::H256;


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
struct AuthorSubmitExtrinsicSuccess {
    jsonrpc: String,
    result: String,
    id: u8,
}

#[derive(Serialize)]
struct NodeRequest {
    id: u8,
    jsonrpc: String,
    method: String,
    params: Vec<String>,
}

pub async fn get_nonce(account: &sp_runtime::AccountId32) -> Result<u32, Box<dyn std::error::Error>> {

    let client = reqwest::Client::new();
    let req = NodeRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "system_accountNextIndex".to_string(),
        params: vec![account.to_string()],
    };

    let res = client
        .post("http://localhost:9933")
        .json(&serde_json::json!(req))
        .send()
        .await;

    match res {
        Ok(res) => {
            match res.status() {
                reqwest::StatusCode::OK => {
                    match res.json::<SystemAccountNextIndexSuccess>().await {
                        Ok(parsed) => {
                            Ok(parsed.result)
                        },
                        Err(e) => Err(String::from(e.to_string()).into())
                    }
                },
                _ => Err(String::from("Something went wrong").into())
            }
        },
        Err(e) => Err(String::from(e.to_string()).into())
    }
}

pub async fn get_genesis_hash() -> Result<H256, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let req = NodeRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "chain_getBlockHash".to_string(),
        params: vec!["0".to_string()]
    };

    let res = client
        .post("http://localhost:9933")
        .json(&serde_json::json!(req))
        .send()
        .await;

    match res {
        Ok(res) => {
            match res.status() {
                reqwest::StatusCode::OK => {
                    match res.json::<ChainGetBlockHashSuccess>().await {
                        Ok(parsed) => {
                            let genesis_hash = parsed.result.replace("0x", "");
                            Ok(sp_core::H256::from_str(genesis_hash.as_ref()).unwrap())
                        },
                        Err(e) => Err(String::from(e.to_string()).into())
                    }
                },
                _ => Err(String::from("Something went wrong").into())
            }
        },
        Err(e) => Err(String::from(e.to_string()).into())
    }
}

pub async fn get_runtime_version() -> Result<StateGetRuntimeVersion, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let req = NodeRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "state_getRuntimeVersion".to_string(),
        params: vec![]
    };

    let res = client
        .post("http://localhost:9933")
        .json(&serde_json::json!(req))
        .send()
        .await;

    match res {
        Ok(res) => {
            match res.status() {
                reqwest::StatusCode::OK => {
                    match res.json::<StateGetRuntimeVersionSuccess>().await {
                        Ok(parsed) => {
                            Ok(parsed.result)
                        },
                        Err(e) => Err(String::from(e.to_string()).into())
                    }
                },
                _ => Err(String::from("Something went wrong").into())
            }
        },
        Err(e) => Err(String::from(e.to_string()).into())
    }
}

pub async fn call_extrinsic(call_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let req = NodeRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "author_submitExtrinsic".to_string(),
        params: vec![call_hex.to_string()]
    };

    let res = client
        .post("http://localhost:9933")
        .json(&serde_json::json!(req))
        .send()
        .await;

    match res {
        Ok(res) => {
            match res.status() {
                reqwest::StatusCode::OK => {
                    match res.json::<AuthorSubmitExtrinsicSuccess>().await {
                        Ok(parsed) => {
                            Ok(parsed.result)
                        },
                        Err(e) => Err(String::from(e.to_string()).into())
                    }
                },
                _ => Err(String::from("Something went wrong").into())
            }
        },
        Err(e) => Err(String::from(e.to_string()).into())
    }
}

