use std::fs;
use std::io::Write;
use std::path::PathBuf;
use serde_derive::{Serialize, Deserialize};
use rand::{distributions::Alphanumeric, Rng};
use sodiumoxide::crypto::box_::PublicKey;
use crate::message::entry::Entry;
use crate::message::errors::IndexError;
use crate::message::file::File;
use crate::message::message::Message;


#[derive(Debug, Clone)]
pub struct IndexFile {
    pub path: PathBuf,
    dir: PathBuf,
}

impl IndexFile {
    pub fn new() -> IndexFile {
        let home_dir = dirs::home_dir().unwrap();
        let home_path = home_dir.as_path();
        let nolik_dir = home_path.join(".nolik");
        let config_file = nolik_dir.join("index.toml");

        IndexFile  {
            path: config_file,
            dir: nolik_dir,
        }
    }

    pub fn temp() -> IndexFile {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let file_name = format!("temp_{}_index.toml", s);
        let home_dir = dirs::home_dir().unwrap();
        let home_path = home_dir.as_path();
        let nolik_dir = home_path.join(".nolik");
        let config_file = nolik_dir.join(file_name);

        IndexFile {
            path: config_file,
            dir: nolik_dir,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexMessage {
    pub index: u32,
    pub public: String,
    pub hash: String,
    pub nonce: String,
    pub from: String,
    pub to: Vec<String>,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub entries: Vec<Entry>,

    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub files: Vec<IndexFileLink>,
}


impl IndexMessage {
    pub fn new(m: &Message, pk: &PublicKey, index: u32, hash: String) -> IndexMessage {

        let public = bs58::encode(pk).into_string();
        let nonce = bs58::encode(m.nonce).into_string();
        let from = bs58::encode(m.sender).into_string();

        let mut to: Vec<String> = Vec::new();
        for pk in &m.recipients {
            let recipient = bs58::encode(pk).into_string();
            to.push(recipient);
        }

        let entries = m.entries.clone();
        let mut files: Vec<IndexFileLink> = Vec::new();
        for file in m.files.iter() {
            let link = IndexFileLink::new(&file, &hash);
            files.push(link);
        }

        IndexMessage {
            index,
            public,
            hash,
            nonce,
            from,
            to,
            entries,
            files,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexFileLink(String);


impl IndexFileLink {
    pub fn new(file: &File, hash: &String) -> IndexFileLink {
        let path = format!("./{}/{}", hash, file.name);
        IndexFileLink(path)
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct IndexData {
    #[serde(skip_serializing_if="Vec::is_empty", default="Vec::new")]
    pub messages: Vec<IndexMessage>,
}


#[derive(Debug)]
pub struct Index {
    pub file: IndexFile,
    pub data: IndexData,
}

impl Index {
    pub fn new(index_file: &IndexFile) -> Result<Index, IndexError> {
        if let false = &index_file.path.exists() {
            return Ok(Index {
                file: index_file.to_owned(),
                data: IndexData {
                    messages: vec![],
                }
            });
        }

        if let Err(e) = fs::read_to_string(&index_file.path) {
            eprintln!("Error: {}", e);
            return Err(IndexError::CouldNotReadIndexFile)
        }

        let contents: String = fs::read_to_string(&index_file.path).unwrap();
        match toml::from_str(contents.as_str()) {
            Ok(index_data) => Ok(Index {
                file: index_file.to_owned(),
                data: index_data,
            }),
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(IndexError::CouldNotParseIndexFile);
            },
        }
    }

    pub fn save(&self) -> Result<(), IndexError> {
        if let false = self.file.dir.exists() {
            if let Err(e) = fs::create_dir(&self.file.dir) {
                eprintln!("Error: {}", e);
                return Err(IndexError::CouldNotCreateIndexDir)
            }
        }

        match fs::File::create(&self.file.path) {
            Ok(mut file) => {
                let contents = match toml::to_string(&self.data) {
                    Ok(contents) => contents,
                    Err(e) => {
                        eprintln!("DATA: {:?}", &self.data);
                        eprintln!("Error: {}", e);
                        return Err(IndexError::CouldNotCreateIndexFile);
                    },
                };
                match file.write_all(contents.as_ref()) {
                    Ok(_) => {
                        Ok(())
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return Err(IndexError::CouldNotCreateIndexFile);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(IndexError::CouldNotCreateIndexFile);
            }
        }
    }
}


