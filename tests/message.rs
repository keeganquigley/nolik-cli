#[cfg(test)]
// #[cfg(all(test, feature = "serde"))]
mod message {
    use std::fs;
    use async_std;
    use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
    use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
    use nolik_cli::account::{Account, AccountInput};
    use nolik_cli::cli::config::{Config, ConfigFile};
    use nolik_cli::cli::errors::InputError;
    use nolik_cli::cli::input::Input;
    use nolik_cli::message::input::MessageInput;
    use nolik_cli::message::message::{EncryptedMessage, SenderOrRecipient};
    use nolik_cli::message::utils::{base58_to_public_key, base58_to_secret_key};
    use futures_util::TryStreamExt;

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
            "--key",
            "subject",
            "--value",
            "hello",
            "--key",
            "message",
            "--value",
            "test",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let message_input = MessageInput::new(&mut input, &config).unwrap();

        let sender_pk = base58_to_public_key(&alice.public).unwrap();
        let sender_sk = base58_to_secret_key(&alice.secret).unwrap();
        let recipient_pk = base58_to_public_key(&bob.public).unwrap();
        let recipient_sk = base58_to_secret_key(&bob.secret).unwrap();

        fs::remove_file(config_file.path).unwrap();

        ((sender_pk, sender_sk), (recipient_pk, recipient_sk), message_input)
    }


    #[test]
    fn nonce_decrypted_by_sender() {
        let ((spk, ssk), (rpk, _rsk), mi) = generate_message_input();

        let encrypted_message = mi.encrypt(&spk, &ssk, &rpk).unwrap();
        let initial_nonce = mi.otu.nonce.secret;
        let decrypted_nonce = encrypted_message.decrypt(&SenderOrRecipient::Sender(ssk)).unwrap().nonce;

        assert_eq!(
            initial_nonce,
            decrypted_nonce,
        )
    }


    #[test]
    fn nonce_decrypted_by_recipient() {
        let ((spk, ssk), (rpk, rsk), mi) = generate_message_input();

        let encrypted_message = mi.encrypt(&spk, &ssk, &rpk).unwrap();
        let initial_nonce = mi.otu.nonce.secret;
        let decrypted_nonce = encrypted_message.decrypt(&SenderOrRecipient::Recipient(rsk)).unwrap().nonce;

        assert_eq!(
            initial_nonce,
            decrypted_nonce,
        )
    }

    #[test]
    fn sender_decrypted_by_recipient() {
        let ((spk, ssk), (rpk, rsk), mi) = generate_message_input();
        let encrypted_message = mi.encrypt(&spk, &ssk, &rpk).unwrap();
        let decrypted_sender = encrypted_message.decrypt(&SenderOrRecipient::Recipient(rsk)).unwrap().address;

        assert_eq!(
            spk,
            decrypted_sender,
        )
    }

    #[test]
    fn recipient_decrypted_by_sender() {
        let ((spk, ssk), (rpk, _rsk), mi) = generate_message_input();

        let encrypted_message = mi.encrypt(&spk, &ssk, &rpk).unwrap();
        let decrypted_recipient = encrypted_message.decrypt(&SenderOrRecipient::Sender(ssk)).unwrap().address;

        assert_eq!(
            rpk,
            decrypted_recipient,
        )
    }

    #[test]
    fn data_decrypted_by_sender() {
        let ((spk, ssk), (rpk, _rsk), mi) = generate_message_input();

        let encrypted_message = mi.encrypt(&spk, &ssk, &rpk).unwrap();
        let decrypted_data_inputs = encrypted_message.decrypt(&SenderOrRecipient::Sender(ssk)).unwrap().data;

        let (initial_key, initial_value) = mi.data.last().unwrap();
        let decrypted_data = decrypted_data_inputs.last().unwrap();

        assert_eq!(
            (initial_key, initial_value),
            (&decrypted_data.key, &decrypted_data.value),
        )
    }

    #[test]
    fn data_decrypted_by_recipient() {
        let ((spk, ssk), (rpk, rsk), mi) = generate_message_input();

        let encrypted_message = mi.encrypt(&spk, &ssk, &rpk).unwrap();
        let decrypted_data_inputs = encrypted_message.decrypt(&SenderOrRecipient::Recipient(rsk)).unwrap().data;

        let (initial_key, initial_value) = mi.data.last().unwrap();
        let decrypted_data = decrypted_data_inputs.last().unwrap();

        assert_eq!(
            (initial_key, initial_value),
            (&decrypted_data.key, &decrypted_data.value),
        )
    }

    #[async_std::test]
    async fn saving_file_to_ipfs() {
        let ((spk, ssk), (rpk, _rsk), mi) = generate_message_input();
        let encrypted_message = mi.encrypt(&spk, &ssk, &rpk).unwrap();

        let ipfs_hash = encrypted_message.save().await.unwrap();
        let client = IpfsClient::default();
        let asd = client.cat(&ipfs_hash).map_ok(|chunk| chunk.to_vec()).try_concat().await.unwrap();
        // let zxc: EncryptedMessage = toml::from_slice(asd.as_slice()).unwrap();
        let dd = String::from_utf8(asd).unwrap();
        let cc: EncryptedMessage = toml::from_str(dd.as_str()).unwrap();
        println!("RES {:#?}", cc);

    }
}