#[cfg(test)]
mod wallet {

    use nolik_cli::wallet::{Wallet, WalletInput};
    use std::fs;
    use std::io::prelude::*;
    use nolik_cli::cli::config::{Config, ConfigData, ConfigFile};
    use nolik_cli::cli::errors::ConfigError;
    use nolik_cli::cli::input::Input;

    #[test]
    fn create_new_wallet() {
        let arr = [
            "add",
            "wallet",
            "--name",
            "alice",
            "--with-password",
            "no"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let wallet_input = WalletInput::new(input).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_wallet(wallet).unwrap();

        let contents = fs::read_to_string(&config_file.path).unwrap();
        let toml_data: ConfigData = toml::from_str(contents.as_str()).unwrap();

        let new_wallet_len = toml_data.wallets
            .iter()
            .filter(|wallet| wallet.name == "alice".to_string())
            .count();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            new_wallet_len,
            1,
        );
    }

    #[test]
    fn import_new_wallet() {
        let arr = [
            "add",
            "wallet",
            "--name",
            "alice",
            "--import",
            "4ecF8kHC5xfAf6FLNKkc1KnQk6KAXwub1HbpZE7Xe6nhhneHzNb8rDxCSk3r8zC1VHjE5b8EcGDtN9WXxxEJyuWh4XN5r8oxpgjQiUu7hTT",
            "--with-password",
            "no"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let wallet_input = WalletInput::new(input).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_wallet(wallet).unwrap();

        let contents = fs::read_to_string(&config_file.path).unwrap();
        let toml_data: ConfigData = toml::from_str(contents.as_str()).unwrap();

        let new_wallet_len = toml_data.wallets
            .iter()
            .filter(|wallet| wallet.seed == "purse quiz priority zero raccoon uphold flat observe resemble meadow teach pen".to_string())
            .count();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            new_wallet_len,
            1,
        );
    }


    #[test]
    fn create_new_non_unique_name_wallet() {
        let arr = [
            "add",
            "wallet",
            "--name",
            "alice",
            "--with-password",
            "no"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input_a = Input::new(args).unwrap();

        let wallet_input = WalletInput::new(input_a).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_wallet(wallet).unwrap();

        let arr = [
            "add",
            "wallet",
            "--name",
            "alice",
            "--with-password",
            "no"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input_b = Input::new(args).unwrap();

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
        let arr = [
            "add",
            "wallet",
            "--name",
            "alice",
            "--import",
            "4ecF8kHC5xfAf6FLNKkc1KnQk6KAXwub1HbpZE7Xe6nhhneHzNb8rDxCSk3r8zC1VHjE5b8EcGDtN9WXxxEJyuWh4XN5r8oxpgjQiUu7hTT",
            "--with-password",
            "no"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input_a = Input::new(args).unwrap();

        let wallet_input = WalletInput::new(input_a).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_wallet(wallet).unwrap();

        let arr = [
            "add",
            "wallet",
            "--name",
            "bob",
            "--import",
            "4ecF8kHC5xfAf6FLNKkc1KnQk6KAXwub1HbpZE7Xe6nhhneHzNb8rDxCSk3r8zC1VHjE5b8EcGDtN9WXxxEJyuWh4XN5r8oxpgjQiUu7hTT",
            "--with-password",
            "no"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input_b = Input::new(args).unwrap();

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
        let arr = [
            "add",
            "wallet",
            "--name",
            "alice",
            "--import",
            "#ecF8kHC5xfAf6FLNKkc1KnQk6KAXwub1HbpZE7Xe6nhhneHzNb8rDxCSk3r8zC1VHjE5b8EcGDtN9WXxxEJyuWh4XN5r8oxpgjQiUu7hTT",
            "--with-password",
            "no"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let wallet_input = WalletInput::new(input).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap_err();

        assert_eq!(
            ConfigError::CouldNotParseSeed,
            wallet,
        );
    }


    #[test]
    fn broken_config_file() {
        let arr = ["add", "wallet", "--name", "alice", "--with-password", "no"].map(|el| el.to_string());
        let args = arr.iter();
        let input = Input::new(args).unwrap();

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