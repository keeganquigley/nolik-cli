#[cfg(test)]
mod account {

    use nolik_cli::account::{Account, AccountInput};
    use std::fs;
    use std::io::prelude::*;
    use nolik_cli::cli::config::{Config, ConfigData, ConfigFile};
    use nolik_cli::cli::errors::ConfigError;
    use nolik_cli::cli::input::Input;


    #[test]
    fn create_new_account() {
        let arr = [
            "add",
            "account",
            "--name",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, account).unwrap();

        let contents = fs::read_to_string(&config_file.path).unwrap();
        let toml_data: ConfigData = toml::from_str(contents.as_str()).unwrap();

        let new_account_len= toml_data.accounts
            .iter()
            .filter(|account| account.name == "alice".to_string())
            .count();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            new_account_len,
            1,
        );
    }

    #[test]
    fn import_new_account() {
        let arr = [
            "add",
            "account",
            "--name",
            "alice",
            "--import",
            "EJ4kZ655xhqRjjYwmf6cgz5k5ZgY2c5uz4Z2kqG7Z1Xs",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, account).unwrap();

        let contents = fs::read_to_string(&config_file.path).unwrap();
        let toml_data: ConfigData = toml::from_str(contents.as_str()).unwrap();

        let new_account_len = toml_data.accounts
            .iter()
            .filter(|account| account.seed == "EJ4kZ655xhqRjjYwmf6cgz5k5ZgY2c5uz4Z2kqG7Z1Xs".to_string())
            .count();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            new_account_len,
            1,
        );
    }


    #[test]
    fn create_new_non_unique_name_wallet() {
        let arr = [
            "add",
            "account",
            "--name",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input_a = Input::new(args).unwrap();

        let account_input = AccountInput::new(input_a).unwrap();
        let account = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, account).unwrap();

        let arr = [
            "add",
            "account",
            "--name",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input_b = Input::new(args).unwrap();

        let account_input = AccountInput::new(input_b).unwrap();
        let account = Account::new(account_input).unwrap();

        let should_err = Account::add(&config_file, account).unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            ConfigError::AccountNameIsNotUnique,
            should_err,
        );
    }

    #[test]
    fn create_new_non_unique_phrase_wallet() {
        let arr = [
            "add",
            "account",
            "--name",
            "alice",
            "--import",
            "EJ4kZ655xhqRjjYwmf6cgz5k5ZgY2c5uz4Z2kqG7Z1Xs",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input_a = Input::new(args).unwrap();

        let account_input = AccountInput::new(input_a).unwrap();
        let account = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, account).unwrap();

        let arr = [
            "add",
            "wallet",
            "--name",
            "bob",
            "--import",
            "EJ4kZ655xhqRjjYwmf6cgz5k5ZgY2c5uz4Z2kqG7Z1Xs",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input_b = Input::new(args).unwrap();

        let account_input = AccountInput::new(input_b).unwrap();
        let account = Account::new(account_input).unwrap();

        let should_err = Account::add(&config_file, account).unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            ConfigError::AccountAlreadyExists,
            should_err,
        );
    }

    #[test]
    fn could_not_parse_secret_key() {
        let arr = [
            "add",
            "wallet",
            "--name",
            "alice",
            "--import",
            "#EJ4kZ655xhqRjjYwmf6cgz5k5ZgY2c5uz4Z2kqG7Z1Xs",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap_err();

        assert_eq!(
            ConfigError::CouldNotParseAccountSecretKey,
            account,
        );
    }


    #[test]
    fn broken_config_file() {
        let arr = [
            "add",
            "wallet",
            "--name",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();


        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, account).unwrap();

        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&config_file.path)
            .unwrap();

        write!(file, "Some unexpected data\n").unwrap();

        let toml_data: ConfigError = Config::new(&config_file).unwrap_err();

        fs::remove_file(config_file.path).unwrap();


        assert_eq!(
            ConfigError::CouldNotParseConfigFile,
            toml_data,
        );
    }
}