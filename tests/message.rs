#[cfg(test)]
mod message {
    use std::{fs, process};
    use sodiumoxide::crypto::box_;
    use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
    use sp_runtime::app_crypto::wrap;
    use nolik_cli::account::{Account, AccountInput};
    use nolik_cli::cli::config::{Config, ConfigFile};
    use nolik_cli::cli::errors::InputError;
    use nolik_cli::cli::input::Input;
    use nolik_cli::message::batch::Batch;
    use nolik_cli::message::errors::MessageError;
    use nolik_cli::message::input::MessageInput;
    use nolik_cli::message::message::{EncryptedMessage, SenderOrRecipient};
    use nolik_cli::message::nonce::Nonce;
    use nolik_cli::message::recipient::Recipient;
    use nolik_cli::message::sender::Sender;
    use nolik_cli::message::utils::{base58_to_public_key, base58_to_secret_key, base64_to_nonce};

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
        let mut input = Input::new(args).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let config = Config::new(config_file.clone()).unwrap();

        let message_input = MessageInput::new(&mut input, &config).unwrap_err();

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
        let mut input = Input::new(args).unwrap();

        let message_input = MessageInput::new(&mut input, &config).unwrap();

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
        let mut input = Input::new(args).unwrap();

        let message_input = MessageInput::new(&mut input, &config).unwrap();

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
        let mut input = Input::new(args).unwrap();

        let message_input = MessageInput::new(&mut input, &config).unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            message_input,
            InputError::InvalidAddress,
        )
    }

    fn generate_message_input() -> ((PublicKey, SecretKey), (PublicKey, SecretKey), MessageInput) {
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

        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            "Gq5xd5c62w4fryJx8poYexoBJAy9JUpjir9vR4qMDF6z",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let message_input = MessageInput::new(&mut input, &config).unwrap();

        let sender_pk = base58_to_public_key(&alice.public).unwrap();
        let sender_sk = base58_to_secret_key(&alice.secret).unwrap();
        let recipient_pk = base58_to_public_key(&bob.public).unwrap();
        let recipient_sk = base58_to_secret_key(&bob.secret).unwrap();


        ((sender_pk, sender_sk), (recipient_pk, recipient_sk), message_input)
    }


    #[test]
    fn nonce_decrypted_by_sender() {
        let ((spk, ssk), (rpk, _rsk), mi) = generate_message_input();

        let encrypted_message = mi.encrypt(&spk, &rpk).unwrap();
        let initial_nonce = mi.otu.nonce.secret;
        let decrypted_nonce = encrypted_message.decrypt(&SenderOrRecipient::Sender, &ssk).unwrap().nonce;

        assert_eq!(
            initial_nonce,
            decrypted_nonce,
        )
    }


    #[test]
    fn nonce_decrypted_by_recipient() {
        let ((spk, _ssk), (rpk, rsk), mi) = generate_message_input();

        let encrypted_message = mi.encrypt(&spk, &rpk).unwrap();
        let initial_nonce = mi.otu.nonce.secret;
        let decrypted_nonce = encrypted_message.decrypt(&SenderOrRecipient::Recipient, &rsk).unwrap().nonce;

        assert_eq!(
            initial_nonce,
            decrypted_nonce,
        )
    }

    #[test]
    fn sender_decrypted_by_recipient() {
        let ((spk, _ssk), (rpk, rsk), mi) = generate_message_input();
        let encrypted_message = mi.encrypt(&spk, &rpk).unwrap();
        let decrypted_sender = encrypted_message.decrypt(&SenderOrRecipient::Recipient, &rsk).unwrap().other;

        assert_eq!(
            spk,
            decrypted_sender,
        )
    }

    #[test]
    fn recipient_decrypted_by_sender() {
        let ((spk, ssk), (rpk, _rsk), mi) = generate_message_input();

        let encrypted_message = mi.encrypt(&spk, &rpk).unwrap();
        let decrypted_recipient = encrypted_message.decrypt(&SenderOrRecipient::Sender, &ssk).unwrap().other;

        assert_eq!(
            rpk,
            decrypted_recipient,
        )
    }
}