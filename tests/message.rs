#[cfg(test)]
mod message {
    use std::fs;
    use std::io::Write;
    use async_std;
    use blake2::Digest;
    use blake2::digest::Update;
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use sodiumoxide::crypto::box_;
    use sp_core::crypto::AccountId32;
    use sp_keyring::AccountKeyring;
    use nolik_cli::account::{Account, AccountInput, AccountOutput};
    use nolik_cli::blacklist::Blacklist;
    use nolik_cli::cli::config::{Config, ConfigFile};
    use nolik_cli::cli::input::Input;
    use nolik_cli::message::input::BatchInput;
    use nolik_cli::cli::errors::{ConfigError, InputError};
    use nolik_cli::message::batch::Batch;
    use nolik_cli::message::index::{Index, IndexFile, IndexMessage};
    use nolik_cli::message::ipfs::{IpfsFile, IpfsInput};
    use nolik_cli::message::message::EncryptedMessage;
    use nolik_cli::message::session::{Session};
    use nolik_cli::message::utils::{base64_to_nonce, base64_to_public_key};
    use nolik_cli::node::errors::NodeError;
    use nolik_cli::node::events::{BalanceTransferEvent, NodeEvent};
    use nolik_cli::node::extrinsics::BalancesTransfer;
    use nolik_cli::owner::Owner;
    use nolik_cli::wallet::{Wallet, WalletInput};
    use nolik_cli::whitelist::Whitelist;

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
        Account::add(&config_file, &account).unwrap();

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
        let account_address = account.public.clone();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, &account).unwrap();

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
        Account::add(&config_file, &account).unwrap();


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
        Account::add(&config_file, &alice).unwrap();

        let arr = [
            "add",
            "account",
            "--alias",
            "bob",
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
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let carol = Account::new(account_input).unwrap();

        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let file_name = format!("temp_{}_text.txt", s);

        let home_dir = dirs::home_dir().unwrap();
        let home_path = home_dir.as_path();
        let nolik_dir = home_path.join(".nolik");
        let text_file = nolik_dir.join(file_name.as_str());

        if let Ok(mut file) = fs::File::create(&text_file) {
            let contents = "Hello World";
            file.write_all(contents.as_ref()).unwrap();
        }


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
            format!("{}", &text_file.clone().into_os_string().into_string().unwrap()).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let bi = BatchInput::new(&mut input, &config_file).unwrap();
        let recipients: Vec<Account> = vec![bob, carol];

        fs::remove_file(config_file.path).unwrap();
        fs::remove_file(text_file).unwrap();

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

        let batch_hash = Batch::hash(&bi, &secret_nonce);
        assert_eq!(
            batch_hash,
            ipfs_data.hash,
        )
    }


    async fn init_sending() -> ConfigFile {
        let config_file: ConfigFile = ConfigFile::temp();

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
        Account::add(&config_file, &alice).unwrap();


        let arr = [
            "add",
            "account",
            "--alias",
            "bob",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let bob = Account::new(account_input).unwrap();
        Account::add(&config_file, &bob).unwrap();


        let arr = [
            "add",
            "account",
            "--alias",
            "carol",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let carol = Account::new(account_input).unwrap();
        Account::add(&config_file, &carol).unwrap();


        let arr = [
            "add",
            "wallet",
            "--alias",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let wallet_input = WalletInput::new(&input, Some(String::from("pass"))).unwrap();
        let wallet_a = Wallet::new(wallet_input).unwrap();
        Wallet::add(&config_file, &wallet_a).unwrap();


        let arr = [
            "add",
            "wallet",
            "--alias",
            "wallet_b",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let wallet_input = WalletInput::new(&input, Some(String::from("pass"))).unwrap();
        let wallet_b = Wallet::new(wallet_input).unwrap();
        Wallet::add(&config_file, &wallet_b).unwrap();


        let sender = AccountKeyring::Alice;
        let recipient = AccountId32::from(wallet_a.public);

        let extrinsic = BalancesTransfer::new(&config_file, &sender, &recipient).unwrap();
        let extrinsic_hash = extrinsic.hash::<BalancesTransfer>().await.unwrap();
        let event = BalanceTransferEvent;
        event.submit(&config_file, &extrinsic_hash).await.unwrap();

        let recipient = AccountId32::from(wallet_b.public);
        let extrinsic = BalancesTransfer::new(&config_file, &sender, &recipient).unwrap();
        let extrinsic_hash = extrinsic.hash::<BalancesTransfer>().await.unwrap();
        let event = BalanceTransferEvent;
        event.submit(&config_file, &extrinsic_hash).await.unwrap();


        let arr = [
            "add",
            "owner",
            "--account",
            format!("{}", &alice.alias).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str()
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let owner = Owner::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        owner.add(&config_file).await.unwrap();


        let arr = [
            "add",
            "owner",
            "--account",
            format!("{}", &bob.alias).as_str(),
            "--wallet",
            format!("{}", wallet_b.alias).as_str()
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let owner = Owner::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        owner.add(&config_file).await.unwrap();

        config_file
    }


    #[async_std::test]
    async fn send_to_two_recipients() {

        let config_file = init_sending().await;

        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();


        let file_name = format!("temp_{}_text.txt", s);
        let home_dir = dirs::home_dir().unwrap();
        let home_path = home_dir.as_path();
        let nolik_dir = home_path.join(".nolik");
        let text_file = nolik_dir.join(file_name.as_str());

        if let Ok(mut file) = fs::File::create(&text_file) {
            let contents = "Hello World";
            file.write_all(contents.as_ref()).unwrap();
        }

        let bob = Account::get(&config_file, String::from("bob")).unwrap();
        let carol = Account::get(&config_file, String::from("carol")).unwrap();

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
            format!("{}", &text_file.clone().into_os_string().into_string().unwrap()).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let bi = BatchInput::new(&mut input, &config_file).unwrap();


        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();


        let arr = [
            "send",
            "message",
            "--hash",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();

        let batch = ipfs_input.ipfs_file.get().await.unwrap();
        let (sender, recipients) = batch.parties(&config_file).unwrap();

        for pk in recipients {
            let res = ipfs_input.ipfs_file.send(&config_file, &sender, &pk, &ipfs_input.wallet).await.is_ok();
            assert_eq!(res, true);
        }


        fs::remove_file(config_file.path).unwrap();
        fs::remove_file(text_file).unwrap();
    }


    #[async_std::test]
    async fn send_if_in_blacklist() {

        let config_file = init_sending().await;

        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let bob = Account::get(&config_file, String::from("bob")).unwrap();


        let arr = [
            "update",
            "blacklist",
            "--for",
            format!("{}", bob.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(alice.public).into_string()).as_str(),
            "--wallet",
            "wallet_b"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let blacklist = Blacklist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        blacklist.update(&config_file).await.unwrap();


        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let bi = BatchInput::new(&mut input, &config_file).unwrap();


        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();


        let arr = [
            "send",
            "message",
            "--hash",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();


        let batch = ipfs_input.ipfs_file.get().await.unwrap();
        let (sender, recipients) = batch.parties(&config_file).unwrap();

        for pk in recipients {
            let res = ipfs_input.ipfs_file.send(&config_file, &sender, &pk, &ipfs_input.wallet).await.unwrap_err();
            assert_eq!(res, NodeError::PalletAddressInBlacklist);
        }

        fs::remove_file(config_file.path).unwrap();
    }


    #[async_std::test]
    async fn send_if_in_whitelist() {

        let config_file = init_sending().await;

        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let bob = Account::get(&config_file, String::from("bob")).unwrap();


        let arr = [
            "update",
            "whitelist",
            "--for",
            format!("{}", bob.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(alice.public).into_string()).as_str(),
            "--wallet",
            "wallet_b"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let whitelist = Whitelist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        whitelist.update(&config_file).await.unwrap();


        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let bi = BatchInput::new(&mut input, &config_file).unwrap();


        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();


        let arr = [
            "send",
            "message",
            "--hash",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();


        let batch = ipfs_input.ipfs_file.get().await.unwrap();
        let (sender, recipients) = batch.parties(&config_file).unwrap();

        for pk in recipients {
            let res = ipfs_file.send(&config_file, &sender, &pk, &ipfs_input.wallet).await.is_ok();
            assert_eq!(res, true);
        }

        fs::remove_file(config_file.path).unwrap();
    }


    #[async_std::test]
    async fn send_if_not_in_whitelist() {

        let config_file = init_sending().await;

        let bob = Account::get(&config_file, String::from("bob")).unwrap();
        let carol = Account::get(&config_file, String::from("carol")).unwrap();


        let arr = [
            "update",
            "whitelist",
            "--for",
            format!("{}", bob.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(carol.public).into_string()).as_str(),
            "--wallet",
            "wallet_b"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let whitelist = Whitelist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        whitelist.update(&config_file).await.unwrap();


        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let bi = BatchInput::new(&mut input, &config_file).unwrap();


        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();


        let arr = [
            "send",
            "message",
            "--hash",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();


        let batch = ipfs_input.ipfs_file.get().await.unwrap();
        let (sender, recipients) = batch.parties(&config_file).unwrap();

        for pk in recipients {
            let res = ipfs_file.send(&config_file, &sender, &pk, &ipfs_input.wallet).await.unwrap_err();
            assert_eq!(res, NodeError::PalletAddressNotInWhitelist);
        }

        fs::remove_file(config_file.path).unwrap();
    }



    #[async_std::test]
    async fn send_if_could_not_init_sender() {

        let config_file = init_sending().await;

        let bob = Account::get(&config_file, String::from("bob")).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();


        let bi = BatchInput::new(&mut input, &config_file).unwrap();

        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();


        let arr = [
            "send",
            "message",
            "--hash",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();

        fs::remove_file(config_file.path).unwrap();


        let config_file: ConfigFile = ConfigFile::temp();
        let config = Config::new(&config_file).unwrap();
        config.save().unwrap();

        let batch = ipfs_input.ipfs_file.get().await.unwrap();
        let res = batch.parties(&config_file).unwrap_err();

        assert_eq!(res, ConfigError::CouldNotInitSender);

        fs::remove_file(config_file.path).unwrap();
    }


    #[async_std::test]
    async fn incrementing_message_count_on_chain() {
        let config_file = init_sending().await;

        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let bob = Account::get(&config_file, String::from("bob")).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();


        let bi = BatchInput::new(&mut input, &config_file).unwrap();

        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();


        let arr = [
            "send",
            "message",
            "--hash",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();

        let batch = ipfs_input.ipfs_file.get().await.unwrap();
        let (sender, recipients) = batch.parties(&config_file).unwrap();

        for pk in recipients {
            ipfs_file.send(&config_file, &sender, &pk, &ipfs_input.wallet).await.unwrap();
        }

        let nonce_alice_a = alice.index(&config_file).await.unwrap();
        assert_eq!(nonce_alice_a.is_some(), true);

        let nonce_bob_a = bob.index(&config_file).await.unwrap();
        assert_eq!(nonce_bob_a.is_some(), true);


        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();


        let bi = BatchInput::new(&mut input, &config_file).unwrap();

        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();


        let arr = [
            "send",
            "message",
            "--hash",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();

        let batch = ipfs_input.ipfs_file.get().await.unwrap();
        let (sender, recipients) = batch.parties(&config_file).unwrap();

        for pk in recipients {
            ipfs_file.send(&config_file, &sender, &pk, &ipfs_input.wallet).await.unwrap();
        }

        let nonce_alice_b = alice.index(&config_file).await.unwrap();
        let nonce_bob_b = bob.index(&config_file).await.unwrap();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(nonce_alice_b.unwrap(), nonce_alice_a.unwrap() + 1);
        assert_eq!(nonce_bob_b.unwrap(), nonce_bob_a.unwrap() + 1);
    }


    #[async_std::test]
    async fn no_account_record_on_chain() {
        let config_file = init_sending().await;

        let alice = Account::get(&config_file, String::from("alice")).unwrap();

        let nonce = alice.index(&config_file).await.unwrap();
        assert_eq!(nonce.is_none(), true);

        fs::remove_file(config_file.path).unwrap();
    }


    #[async_std::test]
    async fn check_message_ipfs_hash() {
        let config_file = init_sending().await;

        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let bob = Account::get(&config_file, String::from("bob")).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();


        let bi = BatchInput::new(&mut input, &config_file).unwrap();

        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();


        let arr = [
            "send",
            "message",
            "--hash",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();

        let batch = ipfs_input.ipfs_file.get().await.unwrap();
        let (sender, recipients) = batch.parties(&config_file).unwrap();

        for pk in recipients {
            ipfs_file.send(&config_file, &sender, &pk, &ipfs_input.wallet).await.unwrap();
        }

        let nonce_alice = alice.index(&config_file).await.unwrap().unwrap();
        let nonce_bob = bob.index(&config_file).await.unwrap().unwrap();

        let message_alice = alice.message(&config_file, nonce_alice).await.unwrap().unwrap();
        let message_bob = alice.message(&config_file, nonce_bob).await.unwrap().unwrap();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(message_alice, message_bob);
        assert_eq!(message_alice, ipfs_file.0);
    }


    #[async_std::test]
    async fn get_decrypt_and_save_ipfs_data() {
        let config_file = init_sending().await;
        let config = Config::new(&config_file).unwrap();

        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let bob = Account::get(&config_file, String::from("bob")).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();


        let bi = BatchInput::new(&mut input, &config_file).unwrap();

        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();


        let arr = [
            "send",
            "message",
            "--hash",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();

        let batch = ipfs_input.ipfs_file.get().await.unwrap();
        let (sender, recipients) = batch.parties(&config_file).unwrap();

        for pk in &recipients {
            ipfs_file.send(&config_file, &sender, &pk, &ipfs_input.wallet).await.unwrap();
        }


        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "world",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();


        let bi = BatchInput::new(&mut input, &config_file).unwrap();

        // let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();


        let arr = [
            "send",
            "message",
            "--hash",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();

        let batch = ipfs_input.ipfs_file.get().await.unwrap();
        let (sender, recipients) = batch.parties(&config_file).unwrap();

        for pk in &recipients {
            ipfs_file.send(&config_file, &sender, &pk, &ipfs_input.wallet).await.unwrap();
        }



        let index_file = IndexFile::temp();
        let mut index = Index::new(&index_file).unwrap();


        let local_alice_indexes: Vec<AccountOutput> = config.data.accounts
            .iter()
            .filter(|ao| ao.public == bs58::encode(bi.sender.public).into_string())
            .map(|el| el.clone())
            .collect();

        let local_alice_index = local_alice_indexes.first().unwrap().index;

        let nonce_alice = alice.index(&config_file).await.unwrap().unwrap();
        // let nonce_bob = bob.index(&config_file).await.unwrap().unwrap();

        for i in local_alice_index..nonce_alice {

            let message_alice = alice.message(&config_file, i + 1).await.unwrap().unwrap();

            let ipfs_file = IpfsFile::new(message_alice.clone());
            let ipfs_data = ipfs_file.get().await.unwrap();

            let public_nonce = base64_to_nonce(&ipfs_data.nonce).unwrap();
            let broker = base64_to_public_key(&ipfs_data.broker).unwrap();

            let decrypted_sessions: Vec<Session> = ipfs_data.sessions
                .iter()
                .filter_map(|es| es.decrypt(&public_nonce, &broker, &bi.sender.secret).ok())
                .collect();

            let first_session = decrypted_sessions.first().unwrap();
            let first_address = first_session.group.0.first().unwrap();

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

            let new_index = i + 1;
            let index_message = IndexMessage::new(&decrypted_message, &alice.public, new_index, ipfs_file.0);
            index.data.messages.push(index_message);
            index.save().unwrap();


            let index_after_save = Index::new(&index_file).unwrap();
            let saved_message = index_after_save.data.messages[i as usize].clone();


            assert_eq!(new_index, saved_message.index);
            assert_eq!(bs58::encode(secret_nonce).into_string(), saved_message.nonce);
            assert_eq!(bs58::encode(bi.sender.public).into_string(), saved_message.from);
            assert_eq!(bs58::encode(bi.recipients.first().unwrap()).into_string(), saved_message.to.first().unwrap().clone());
            assert_eq!(match i { 0 => String::from("subject"), _ => String::from("subject")}, saved_message.entries.first().unwrap().key);
            assert_eq!(match i { 0 => String::from("hello"), _ => String::from("world")}, saved_message.entries.first().unwrap().value);
        }


        fs::remove_file(config_file.path).unwrap();
        fs::remove_file(index_file.path).unwrap();
    }
}