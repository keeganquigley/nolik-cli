#[cfg(test)]
mod message {
    use std::fs;
    use sodiumoxide::crypto::box_;
    use nolik_cli::config::{errors::ConfigError, Config, ConfigFile, ConfigData};
    use nolik_cli::account::{Account, AccountInput};
    use nolik_cli::inputs::errors::InputError;
    use nolik_cli::message::{Batch, BatchFile, errors::MessageError, Message, MessageInput};
    // use std::fs;
    // use std::io::prelude::*;
    use nolik_cli::inputs::Input;

    #[test]
    fn required_arguments_are_not_provided() {
        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let message_input = Input::new(args).unwrap_err();
        assert_eq!(
            message_input,
            InputError::RequiredKeysMissing
        );
    }

    #[test]
    fn broken_key_value_arguments() {
        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            "bob",
            "--key",
            "message",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let message_input = Input::new(args).unwrap_err();

        assert_eq!(
            message_input,
            InputError::NoCorrespondingValue
        );
    }

    #[test]
    fn sender_does_not_exist() {
        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            "bob",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let config = Config::new(config_file.clone()).unwrap();

        let message_input = MessageInput::new(input, config).unwrap_err();

        assert_eq!(
            message_input,
            InputError::SenderDoesNotExist,
        );
    }


    #[test]
    fn sender_name_exist() {
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
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_account(account).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            "Gq5xd5c62w4fryJx8poYexoBJAy9JUpjir9vR4qMDF6z",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let message_input = MessageInput::new(input, config).unwrap();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            message_input.sender.name,
            "alice".to_string(),
        );
    }

    #[test]
    fn sender_address_exist() {
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
        let mut config = Config::new(config_file.clone()).unwrap();

        let account_address =account.public.clone();
        config.add_account(account).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            format!("{}", &account_address).as_str(),
            "--recipient",
            "Gq5xd5c62w4fryJx8poYexoBJAy9JUpjir9vR4qMDF6z",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let message_input = MessageInput::new(input, config).unwrap();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            message_input.sender.public,
            account_address,
        );
    }


    #[test]
    fn broken_recipient_address() {
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
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_account(account).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            "@q5xd5c62w4fryJx8poYexoBJAy9JUpjir9vR4qMDF6z",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let message_input = MessageInput::new(input, config).unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            message_input,
            InputError::InvalidAddress,
        );
    }

    #[test]
    fn message_nonce_decrypted() {
        let arr = [
            "add",
            "account",
            "--name",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let alice = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let mut config = Config::new(config_file.clone()).unwrap();

        config.add_account(alice.clone()).unwrap();

        let arr = [
            "add",
            "account",
            "--name",
            "bob",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let bob = Account::new(account_input).unwrap();

        let bob_sk_decoded = bs58::decode(&bob.secret)
            .into_vec()
            .unwrap();
        let bob_sk = box_::SecretKey::from_slice(bob_sk_decoded.as_slice()).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bob.public).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let message_input = MessageInput::new(input, config).unwrap();

        let initial_nonce = message_input.secret_nonce;

        let batch = Batch::new(message_input).unwrap();
        let batch_file = BatchFile::new(&batch);

        batch.save(&batch_file).unwrap();

        let message = batch.messages.last().unwrap();
        let decrypted_nonce = message.decrypt_nonce(bob_sk).unwrap();

        fs::remove_file(config_file.path).unwrap();
        // fs::remove_file(batch_file.path).unwrap();

        assert_eq!(
            initial_nonce,
            decrypted_nonce,
        );
    }
}