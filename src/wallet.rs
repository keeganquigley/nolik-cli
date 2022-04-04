use sp_core::sr25519;
use dirs;
use std::fs;
use std::io::ErrorKind;

pub struct Wallet {
    pair: sr25519::Pair,
    name: String,
}

struct Config {
    wallets: Vec<Wallet>,
    default: usize,
}

impl Wallet {
    pub fn new(pair: sr25519::Pair, name: String) -> Wallet {
        Wallet {
            pair,
            name,
        }
    }

    pub fn add(wallet: Wallet) -> Result<(), &'static str> {
        let home_dir = dirs::home_dir().unwrap();
        let home_path = home_dir.as_path();
        let nolik_dir = home_path.join(".nolik");

        println!("NOLIKDIR {:?}", nolik_dir);

        match fs::create_dir(nolik_dir) {
            Ok(res) => {
                println!("RES {:?}", res);
            },
            Err(e) => {
                if e.kind() == ErrorKind::AlreadyExists {
                    eprintln!("Directory alreay exists");
                }
            }
        }
        Ok(())
    }

    pub fn delete(wallet: Wallet) -> Result<(), &'static str> {
        Ok(())
    }
}