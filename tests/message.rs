#[cfg(test)]
mod message {
    use std::fs;
    use async_std;
    use blake2::Digest;
    use blake2::digest::Update;
    use sodiumoxide::crypto::box_;
    use nolik_cli::account::{Account, AccountInput};
    use nolik_cli::cli::config::ConfigFile;
    use nolik_cli::cli::input::Input;
    use nolik_cli::message::input::BatchInput;
    use nolik_cli::cli::errors::InputError;
    use nolik_cli::message::batch::Batch;
    use nolik_cli::message::message::EncryptedMessage;
    use nolik_cli::message::session::{Session};
    use nolik_cli::message::utils::{base64_to_nonce, base64_to_public_key};

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
        let message_input = BatchInput::new(&mut input, &config_file).unwrap_err();

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
            "--alias",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, account).unwrap();

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

        let message_input = BatchInput::new(&mut input, &config_file).unwrap();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            message_input.sender.alias,
            "alice".to_string(),
        );
    }

    #[test]
    fn sender_address_exist() {
        let arr = [
            "add",
            "account",
            "--alias",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap();
        let account_address =account.public.clone();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, account).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            format!("{}", bs58::encode(&account_address).into_string()).as_str(),
            "--recipient",
            "Gq5xd5c62w4fryJx8poYexoBJAy9JUpjir9vR4qMDF6z",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let message_input = BatchInput::new(&mut input, &config_file).unwrap();

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
            "--alias",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, account).unwrap();


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

        let message_input = BatchInput::new(&mut input, &config_file).unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            message_input,
            InputError::InvalidAddress,
        )
    }

    async fn generate_message_input() -> (Vec<Account>, BatchInput) {
        let arr = [
            "add",
            "account",
            "--alias",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let alice = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, alice.clone()).unwrap();

        let arr = [
            "add",
            "account",
            "--alias",
            "bob",
            "--import",
            "CBj4K14XmQNbMuTu9PUtZchkMyyry4ua3GKG15tgk3xM"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let bob = Account::new(account_input).unwrap();

        let arr = [
            "add",
            "account",
            "--alias",
            "carol",
            "--import",
            "GjAEDCgMg55ByJysg6pfAuAJnVjFoaVFfTbgTrXefWcD"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let carol = Account::new(account_input).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--recipient",
            format!("{}", bs58::encode(&carol.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
            "--key",
            "message",
            "--value",
            "test",
            "--file",
            "/Users/amrbz/Desktop/test.txt"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let bi = BatchInput::new(&mut input, &config_file).unwrap();
        fs::remove_file(config_file.path).unwrap();

        let recipients: Vec<Account> = vec![bob, carol];

        (recipients, bi)
    }


    #[async_std::test]
    async fn message_decrypted_by_sender() {
        let (_recipient, bi) = generate_message_input().await;

        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();
        let ipfs_data = ipfs_file.get().await.unwrap();

        let public_nonce = base64_to_nonce(&ipfs_data.nonce).unwrap();
        let broker = base64_to_public_key(&ipfs_data.broker).unwrap();

        let decrypted_sessions: Vec<Session> = ipfs_data.sessions
            .iter()
            .filter_map(|es| es.decrypt(&public_nonce, &broker, &bi.sender.secret).ok())
            .collect();

        let first_session = decrypted_sessions.first().unwrap();
        let first_address = first_session.group.0.first().unwrap();

        assert_eq!(
            first_session.nonce.0,
            secret_nonce,
        );

        assert_eq!(
            first_address.0,
            bi.sender.public,
        );

        let recipients = first_session.group.get_recipients();
        let any_recipient = recipients.first().unwrap();

        let mut parties = blake2::Blake2s256::new();
        Update::update(&mut parties, &first_address.0.as_ref());
        Update::update(&mut parties, &any_recipient.as_ref());
        let parties_hash = base64::encode(parties.finalize().to_vec());

        let encrypted_messages = ipfs_data.messages
            .iter()
            .filter(|em| em.parties == parties_hash)
            .collect::<Vec<&EncryptedMessage>>();

        let encrypted_message = encrypted_messages.first().unwrap();
        let decrypted_message = encrypted_message.decrypt(first_session, any_recipient, &bi.sender.secret).unwrap();

        assert_eq!(
            decrypted_message.entries.first().unwrap().key,
            bi.entries.first().unwrap().key,
        );

        assert_eq!(
            decrypted_message.entries.first().unwrap().value,
            bi.entries.first().unwrap().value,
        );
    }


    #[async_std::test]
    async fn message_decrypted_by_recipients() {
        let (recipients, bi) = generate_message_input().await;


        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();

        let ipfs_data = ipfs_file.get().await.unwrap();

        let public_nonce = base64_to_nonce(&ipfs_data.nonce).unwrap();
        let broker = base64_to_public_key(&ipfs_data.broker).unwrap();

        for recipient in &recipients {
            let decrypted_sessions: Vec<Session> = ipfs_data.sessions
                .iter()
                .filter_map(|es| es.decrypt(&public_nonce, &broker, &recipient.secret).ok())
                .collect::<Vec<Session>>();

            let first_session = decrypted_sessions.first().unwrap();
            let first_address = first_session.group.0.first().unwrap();

            assert_eq!(
                first_session.nonce.0,
                secret_nonce,
            );

            assert_eq!(
                first_address.0,
                bi.sender.public,
            );

            assert_eq!(
                first_session.group.0.iter().any(|el| el.0 == recipient.public),
                true,
            );

            let mut parties = blake2::Blake2s256::new();
            Update::update(&mut parties, &first_address.0.as_ref());
            Update::update(&mut parties, &recipient.public.as_ref());
            let parties_hash = base64::encode(parties.finalize().to_vec());

            let encrypted_messages = ipfs_data.messages
                .iter()
                .filter(|em| em.parties == parties_hash)
                .collect::<Vec<&EncryptedMessage>>();

            let encrypted_message = encrypted_messages.first().unwrap();
            let decrypted_message = encrypted_message.decrypt(first_session, &first_address.0, &recipient.secret).unwrap();

            assert_eq!(
                decrypted_message.entries.first().unwrap().key,
                bi.entries.first().unwrap().key,
            );

            assert_eq!(
                decrypted_message.entries.first().unwrap().value,
                bi.entries.first().unwrap().value,
            );
        }
    }

    #[async_std::test]
    async fn confirmed_batch_hash() {
        let (.., bi) = generate_message_input().await;

        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();

        let ipfs_data = ipfs_file.get().await.unwrap();

        let batch_hash = Batch::get_batch_hash(&bi, &secret_nonce);
        assert_eq!(
            batch_hash,
            ipfs_data.hash,
        )
    }
}