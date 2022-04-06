#[cfg(test)]
mod wallet {

    use nolik_cli::config::{errors::ConfigError, Config, ConfigFile, ConfigData};
    use nolik_cli::wallet::{Wallet, WalletInput};
    use std::fs;
    use std::io::Cursor;
    use std::io::prelude::*;
    use nolik_cli::inputs::{Command, Flag, FlagKey, Input};


    #[test]
    fn create_new_wallet() {
        let input = Input {
            command: Command::AddWallet,
            flags: vec![
                Flag {
                    key: FlagKey::Name,
                    value: "alice".to_string(),
                },
                Flag {
                    key: FlagKey::WithPassword,
                    value: "no".to_string(),
                }
            ]
        };

        let wallet_input = WalletInput::new(input).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_wallet(wallet).unwrap();

        let contents = fs::read_to_string(&config_file.path).unwrap();
        let toml_data: ConfigData = toml::from_str(contents.as_str()).unwrap();

        let new_wallet: Vec<Wallet> = toml_data.wallets
            .iter()
            .filter(|wallet| wallet.name == "alice".to_string())
            .map(|wallet| wallet.clone())
            .collect();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            new_wallet.len(),
            1
        );
    }

    #[test]
    fn import_new_wallet() {
        let input = Input {
            command: Command::AddWallet,
            flags: vec![
                Flag {
                    key: FlagKey::Name,
                    value: "alice".to_string(),
                },
                Flag {
                    key: FlagKey::Import,
                    value: "4ecF8kHC5xfAf6FLNKkc1KnQk6KAXwub1HbpZE7Xe6nhhneHzNb8rDxCSk3r8zC1VHjE5b8EcGDtN9WXxxEJyuWh4XN5r8oxpgjQiUu7hTT".to_string(),
                },
                Flag {
                    key: FlagKey::WithPassword,
                    value: "no".to_string(),
                }
            ]
        };

        let wallet_input = WalletInput::new(input).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_wallet(wallet).unwrap();

        let contents = fs::read_to_string(&config_file.path).unwrap();
        let toml_data: ConfigData = toml::from_str(contents.as_str()).unwrap();

        let new_wallet: Vec<Wallet> = toml_data.wallets
            .iter()
            .filter(|wallet| wallet.seed == "purse quiz priority zero raccoon uphold flat observe resemble meadow teach pen".to_string())
            .map(|wallet| wallet.clone())
            .collect();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            new_wallet.len(),
            1
        );
    }


    #[test]
    fn create_new_non_unique_name_wallet() {
        let input_a = Input {
            command: Command::AddWallet,
            flags: vec![
                Flag {
                    key: FlagKey::Name,
                    value: "alice".to_string(),
                },
                Flag {
                    key: FlagKey::WithPassword,
                    value: "no".to_string(),
                }
            ]
        };

        let input_b = Input {
            command: Command::AddWallet,
            flags: vec![
                Flag {
                    key: FlagKey::Name,
                    value: "alice".to_string(),
                },
                Flag {
                    key: FlagKey::WithPassword,
                    value: "no".to_string(),
                }
            ]
        };

        let wallet_input = WalletInput::new(input_a).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_wallet(wallet).unwrap();


        let wallet_input = WalletInput::new(input_b).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let should_err = config.add_wallet(wallet).unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            ConfigError::WalletNameIsNotUnique,
            should_err,
        );
    }

    #[test]
    fn create_new_non_unique_phrase_wallet() {
        let input_a = Input {
            command: Command::AddWallet,
            flags: vec![
                Flag {
                    key: FlagKey::Name,
                    value: "alice".to_string(),
                },
                Flag {
                    key: FlagKey::Import,
                    value: "4ecF8kHC5xfAf6FLNKkc1KnQk6KAXwub1HbpZE7Xe6nhhneHzNb8rDxCSk3r8zC1VHjE5b8EcGDtN9WXxxEJyuWh4XN5r8oxpgjQiUu7hTT".to_string(),
                },
                Flag {
                    key: FlagKey::WithPassword,
                    value: "no".to_string(),
                }
            ]
        };

        let wallet_input = WalletInput::new(input_a).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_wallet(wallet).unwrap();


        let input_b = Input {
            command: Command::AddWallet,
            flags: vec![
                Flag {
                    key: FlagKey::Name,
                    value: "bob".to_string(),
                },
                Flag {
                    key: FlagKey::Import,
                    value: "4ecF8kHC5xfAf6FLNKkc1KnQk6KAXwub1HbpZE7Xe6nhhneHzNb8rDxCSk3r8zC1VHjE5b8EcGDtN9WXxxEJyuWh4XN5r8oxpgjQiUu7hTT".to_string(),
                },
                Flag {
                    key: FlagKey::WithPassword,
                    value: "no".to_string(),
                }
            ]
        };

        let wallet_input = WalletInput::new(input_b).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let should_err = config.add_wallet(wallet).unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            ConfigError::WalletAlreadyExists,
            should_err,
        );
    }

    #[test]
    fn could_not_parse_seed() {
        let input = Input {
            command: Command::AddWallet,
            flags: vec![
                Flag {
                    key: FlagKey::Name,
                    value: "alice".to_string(),
                },
                Flag {
                    key: FlagKey::Import,
                    value: "@ecF8kHC5xfAf6FLNKkc1KnQk6KAXwub1HbpZE7Xe6nhhneHzNb8rDxCSk3r8zC1VHjE5b8EcGDtN9WXxxEJyuWh4XN5r8oxpgjQiUu7hTT".to_string(),
                },
                Flag {
                    key: FlagKey::WithPassword,
                    value: "no".to_string(),
                }
            ]
        };


        let wallet_input = WalletInput::new(input).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap_err();

        assert_eq!(
            ConfigError::CouldNotParseSeed,
            wallet,
        );
    }


    #[test]
    fn broken_config_file() {
        let input = Input {
            command: Command::AddWallet,
            flags: vec![
                Flag {
                    key: FlagKey::Name,
                    value: "alice".to_string(),
                },
                Flag {
                    key: FlagKey::Import,
                    value: "4ecF8kHC5xfAf6FLNKkc1KnQk6KAXwub1HbpZE7Xe6nhhneHzNb8rDxCSk3r8zC1VHjE5b8EcGDtN9WXxxEJyuWh4XN5r8oxpgjQiUu7hTT".to_string(),
                },
                Flag {
                    key: FlagKey::WithPassword,
                    value: "no".to_string(),
                }
            ]
        };


        let wallet_input = WalletInput::new(input).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_wallet(wallet).unwrap();

        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&config_file.path)
            .unwrap();

        write!(file, "Some unexpected data\n").unwrap();

        let contents = fs::read_to_string(&config_file.path).unwrap();
        let toml_data: ConfigError = Config::parse_config_data(contents).unwrap_err();

        fs::remove_file(config_file.path).unwrap();


        assert_eq!(
            ConfigError::CouldNotParseConfigFile,
            toml_data,
        );
    }
}