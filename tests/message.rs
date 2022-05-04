#[cfg(test)]
// #[cfg(all(test, feature = "serde"))]
mod message {
    use std::fs;
    use async_std;
    use nolik_cli::account::{Account, AccountInput};
    use nolik_cli::cli::config::ConfigFile;
    use nolik_cli::cli::errors::InputError;
    use nolik_cli::cli::input::Input;
    use nolik_cli::message::input::MessageInput;
    use nolik_cli::message::ipfs::IpfsInput;
    use blake2::Digest;
    use blake2::digest::Update;

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
        let message_input = MessageInput::new(&mut input, config_file).unwrap_err();

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
        Account::add(config_file.clone(), account).unwrap();

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

        let message_input = MessageInput::new(&mut input, config_file.clone()).unwrap();

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
        let account_address =account.public.clone();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(config_file.clone(), account).unwrap();

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

        let message_input = MessageInput::new(&mut input, config_file.clone()).unwrap();

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
        Account::add(config_file.clone(), account).unwrap();


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

        let message_input = MessageInput::new(&mut input, config_file.clone()).unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            message_input,
            InputError::InvalidAddress,
        )
    }

    async fn generate_message_input() -> (Account, Vec<Account>, MessageInput) {
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
        Account::add(config_file.clone(), alice.clone()).unwrap();

        let arr = [
            "add",
            "account",
            "--name",
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
            "--name",
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
            "4yQcPKKoC7hvcYRS497vKqRZHYy2vsyGbVWz7JSXNWNu",
            "--recipient",
            "2YfopzWtXSXyx2zNKpM2HCpJHhgozDUtverm5LGsmqVT",
            "--key",
            "subject",
            "--value",
            "hello",
            "--key",
            "message",
            "--value",
            "test",
            "--blob",
            "/Users/amrbz/Desktop/test.txt"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let mi = MessageInput::new(&mut input, config_file.clone()).unwrap();
        // let encrypted_message = mi.encrypt(&rpk).unwrap();
        // let ipfs_hash = encrypted_message.save().await.unwrap();

        // let ipfs_input = IpfsInput::new(&ipfs_hash);
        // let sor = SenderOrRecipient::Sender((&alice.public, &alice.secret));
        // let decrypted_message = ipfs_input.get_ipfs_data(&sor).await.unwrap();
        // let ipfs_data = IpfsData::new(decrypted_message, ipfs_hash);
        // let file_path = ipfs_data.save().unwrap();
        //
        // let file_contents = fs::read_to_string(&file_path).unwrap();
        // let file_data: IpfsData = toml::from_str(&file_contents).unwrap();
        // let last_data = file_data.data.last().unwrap();
        fs::remove_file(config_file.path).unwrap();

        let sender = alice;
        let recipients = vec![bob, carol];

        (sender, recipients, mi)
    }


    #[async_std::test]
    async fn nonce_decrypted_by_sender() {
        let (sender, recipients, mi) = generate_message_input().await;

        for r in &recipients {
            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let decrypted_message = encrypted_ipfs_data.decrypt(&sender).unwrap();

            assert_eq!(
                mi.otu.nonce.secret,
                decrypted_message.nonce,
            )
        }
    }


    #[async_std::test]
    async fn nonce_decrypted_by_recipients() {
        let (.., recipients, mi) = generate_message_input().await;

        for r in &recipients {
            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let decrypted_message = encrypted_ipfs_data.decrypt(&r).unwrap();

            assert_eq!(
                mi.otu.nonce.secret,
                decrypted_message.nonce,
            )
        }
    }


    #[async_std::test]
    async fn sender_decrypted_by_sender() {
        let (sender, recipients, mi) = generate_message_input().await;

        for r in &recipients {
            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let decrypted_message = encrypted_ipfs_data.decrypt(&sender).unwrap();

            assert_eq!(
                mi.sender.public,
                decrypted_message.from
            )
        }
    }


    #[async_std::test]
    async fn sender_decrypted_by_recipients() {
        let (.., recipients, mi) = generate_message_input().await;

        for r in &recipients {
            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let decrypted_message = encrypted_ipfs_data.decrypt(&r).unwrap();

            assert_eq!(
                mi.sender.public,
                decrypted_message.from,
            )
        }
    }


    #[async_std::test]
    async fn parties_decrypted_by_sender() {
        let (sender, recipients, mut mi) = generate_message_input().await;

        for r in &recipients {
            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let mut decrypted_message = encrypted_ipfs_data.decrypt(&sender).unwrap();

            assert_eq!(
                mi.recipients.sort(),
                decrypted_message.to.sort(),
            )
        }
    }


    #[async_std::test]
    async fn parties_decrypted_by_recipients() {
        let (.., recipients, mut mi) = generate_message_input().await;

        for r in &recipients {
            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let mut decrypted_message = encrypted_ipfs_data.decrypt(&r).unwrap();

            assert_eq!(
                mi.recipients.sort(),
                decrypted_message.to.sort(),
            )
        }
    }


    #[async_std::test]
    async fn entry_decrypted_by_sender() {
        let (sender, recipients, mi) = generate_message_input().await;

        let last_init_entry = mi.entries.last().unwrap();
        for r in &recipients {
            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let decrypted_message = encrypted_ipfs_data.decrypt(&sender).unwrap();
            let last_decrypted_entry = decrypted_message.entries.last().unwrap();

            assert_eq!(
                (&last_init_entry.key, &last_init_entry.value),
                (&last_decrypted_entry.key, &last_decrypted_entry.value)
            )
        }
    }


    #[async_std::test]
    async fn entry_decrypted_by_recipients() {
        let (.., recipients, mi) = generate_message_input().await;

        let last_init_entry = mi.entries.last().unwrap();
        for r in &recipients {
            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let decrypted_message = encrypted_ipfs_data.decrypt(&r).unwrap();
            let last_decrypted_entry = decrypted_message.entries.last().unwrap();

            assert_eq!(
                (&last_init_entry.key, &last_init_entry.value),
                (&last_decrypted_entry.key, &last_decrypted_entry.value)
            )
        }
    }

    #[async_std::test]
    async fn blob_decrypted_by_sender() {
        let (sender, recipients, mi) = generate_message_input().await;

        let last_init_blob = mi.blobs.last().unwrap();
        for r in &recipients {
            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let decrypted_message = encrypted_ipfs_data.decrypt(&sender).unwrap();
            let last_decrypted_blob = decrypted_message.blobs.last().unwrap();

            assert_eq!(
                (&last_init_blob.binary, &last_init_blob.name),
                (&last_decrypted_blob.binary, &last_decrypted_blob.name)
            )
        }
    }


    #[async_std::test]
    async fn blob_decrypted_by_recipients() {
        let (.., recipients, mi) = generate_message_input().await;

        let last_init_entry = mi.entries.last().unwrap();
        for r in &recipients {
            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let decrypted_message = encrypted_ipfs_data.decrypt(&r).unwrap();
            let last_decrypted_entry = decrypted_message.entries.last().unwrap();

            assert_eq!(
                (&last_init_entry.key, &last_init_entry.value),
                (&last_decrypted_entry.key, &last_decrypted_entry.value)
            )
        }
    }


    #[async_std::test]
    async fn hash_confirmed_by_sender() {
        let (sender, recipients, mi) = generate_message_input().await;

        for r in &recipients {
            let mut hasher = blake2::Blake2s256::new();
            Update::update(&mut hasher, (&mi.otu.nonce.public).as_ref());
            Update::update(&mut hasher, (&mi.otu.nonce.secret).as_ref());
            Update::update(&mut hasher, (&mi.otu.broker.public).as_ref());
            Update::update(&mut hasher, (&mi.sender.public).as_ref());
            Update::update(&mut hasher, (&r.public).as_ref());

            for entry in &mi.entries {
                Update::update(&mut hasher, (&entry.key).as_ref());
                Update::update(&mut hasher, (&entry.value).as_ref());
            }

            for blob in &mi.blobs {
                Update::update(&mut hasher, &blob.binary);
                Update::update(&mut hasher, (&blob.name).as_ref());
            }

            let hash = base64::encode(hasher.finalize().to_vec());

            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let decrypted_message = encrypted_ipfs_data.decrypt(&sender).unwrap();

            assert_eq!(
                hash,
                decrypted_message.hash,
            )
        }
    }


    #[async_std::test]
    async fn hash_confirmed_by_recipients() {
        let (.., recipients, mi) = generate_message_input().await;

        for r in &recipients {
            let mut hasher = blake2::Blake2s256::new();
            Update::update(&mut hasher, (&mi.otu.nonce.public).as_ref());
            Update::update(&mut hasher, (&mi.otu.nonce.secret).as_ref());
            Update::update(&mut hasher, (&mi.otu.broker.public).as_ref());
            Update::update(&mut hasher, (&mi.sender.public).as_ref());
            Update::update(&mut hasher, (&r.public).as_ref());

            for entry in &mi.entries {
                Update::update(&mut hasher, (&entry.key).as_ref());
                Update::update(&mut hasher, (&entry.value).as_ref());
            }

            for blob in &mi.blobs {
                Update::update(&mut hasher, &blob.binary);
                Update::update(&mut hasher, (&blob.name).as_ref());
            }

            let hash = base64::encode(hasher.finalize().to_vec());

            let encrypted_message = mi.encrypt(&r.public).unwrap();
            let ipfs_hash = encrypted_message.save().await.unwrap();

            let ipfs_input = IpfsInput::new(ipfs_hash);
            let mut encrypted_ipfs_data = ipfs_input.get().await.unwrap();
            let decrypted_message = encrypted_ipfs_data.decrypt(&r).unwrap();

            assert_eq!(
                hash,
                decrypted_message.hash,
            )
        }
    }

    // #[async_std::test]
    // async fn nonce_decrypted_by_recipients() {
    //     let (sender, recipients, _ipfs_hash) = generate_message_input().await;
    //
    //     for recipient in recipients {
    //         let encrypted_message = mi.encrypt(&alice.public, &alice.secret, &rpk).unwrap();
    //         let initial_nonce = mi.otu.nonce.secret;
    //         let decrypted_nonce = encrypted_message.decrypt(&SenderOrRecipient::Recipient((&rpk, &rsk))).unwrap().nonce;
    //
    //         assert_eq!(
    //             initial_nonce,
    //             decrypted_nonce,
    //         )
    //     }
    // }

    // #[test]
    // fn sender_decrypted_by_recipients() {
    //     let (alice, bob, carol, mi) = generate_message_input();
    //     let recipients = std::iter::zip(&mi.recipients,[&bob.secret, &carol.secret]);
    //
    //     for (rpk, rsk) in recipients {
    //         let encrypted_message = mi.encrypt(&alice.public, &alice.secret, &rpk).unwrap();
    //         let decrypted_sender = encrypted_message.decrypt(&SenderOrRecipient::Recipient((&rpk, &rsk))).unwrap().from;
    //
    //         assert_eq!(
    //             alice.public,
    //             decrypted_sender,
    //         )
    //     }
    // }
    //
    // #[test]
    // fn recipient_decrypted_by_sender() {
    //     let (alice, _bob, _carol, mi) = generate_message_input();
    //
    //     for rpk in &mi.recipients {
    //         let encrypted_message = mi.encrypt(&alice.public, &alice.secret, &rpk).unwrap();
    //         let decrypted_recipient = encrypted_message.decrypt(&SenderOrRecipient::Sender((&alice.public, &alice.secret))).unwrap().to;
    //
    //         assert_eq!(
    //             rpk,
    //             &decrypted_recipient,
    //         )
    //     }
    // }
    //
    // #[test]
    // fn data_decrypted_by_sender() {
    //     let (alice, _bob, _carol, mi) = generate_message_input();
    //
    //     for rpk in &mi.recipients {
    //         let encrypted_message = mi.encrypt(&alice.public, &alice.secret, &rpk).unwrap();
    //         let decrypted_data_inputs = encrypted_message.decrypt(&SenderOrRecipient::Sender((&alice.public, &alice.secret))).unwrap().data;
    //
    //         let decrypted_data = decrypted_data_inputs.last().unwrap();
    //         assert_eq!(
    //             (&String::from("message"), &String::from("test")),
    //             (&decrypted_data.key, &decrypted_data.value),
    //         )
    //     }
    // }
    //
    // #[test]
    // fn data_decrypted_by_recipient() {
    //     let (alice, bob, carol, mi) = generate_message_input();
    //
    //     let recipients = std::iter::zip(&mi.recipients,[&bob.secret, &carol.secret]);
    //
    //     for (rpk, rsk) in recipients {
    //         let encrypted_message = mi.encrypt(&alice.public, &alice.secret, &rpk).unwrap();
    //         let decrypted_data_inputs = encrypted_message.decrypt(&SenderOrRecipient::Recipient((&rpk, &rsk))).unwrap().data;
    //         let decrypted_data = decrypted_data_inputs.last().unwrap();
    //
    //         assert_eq!(
    //             (&String::from("message"), &String::from("test")),
    //             (&decrypted_data.key, &decrypted_data.value),
    //         )
    //     }
    // }
    //
    // #[async_std::test]
    // async fn saving_ipfs_file_by_sender() {
    //     let (alice, bob, carol, mi) = generate_message_input();
    //     let recipients = std::iter::zip(&mi.recipients,[&bob.secret, &carol.secret]);
    //
    //     for (rpk, ..) in recipients {
    //         let encrypted_message = mi.encrypt(&alice.public, &alice.secret, &rpk).unwrap();
    //         let ipfs_hash = encrypted_message.save().await.unwrap();
    //         let ipfs_input = IpfsInput::new(&ipfs_hash);
    //         let sor = SenderOrRecipient::Sender((&alice.public, &alice.secret));
    //         let decrypted_message = ipfs_input.get_ipfs_data(&sor).await.unwrap();
    //         let ipfs_data = IpfsData::new(decrypted_message, ipfs_hash);
    //         let file_path = ipfs_data.save().unwrap();
    //
    //         let file_contents = fs::read_to_string(&file_path).unwrap();
    //         let file_data: IpfsData = toml::from_str(&file_contents).unwrap();
    //         let last_data = file_data.data.last().unwrap();
    //
    //         fs::remove_file(file_path).unwrap();
    //
    //         assert_eq!(
    //             (&String::from("message"), &String::from("test")),
    //             (&last_data.key, &last_data.value),
    //         )
    //     }
    // }
    //
    // #[async_std::test]
    // async fn saving_ipfs_file_by_recipients() {
    //     let (alice, bob, carol, mi) = generate_message_input();
    //     let recipients = std::iter::zip(&mi.recipients,[&bob.secret, &carol.secret]);
    //
    //     for (rpk, rsk) in recipients {
    //         let encrypted_message = mi.encrypt(&alice.public, &alice.secret, &rpk).unwrap();
    //         let ipfs_hash = encrypted_message.save().await.unwrap();
    //         let ipfs_input = IpfsInput::new(&ipfs_hash);
    //
    //         let sor = SenderOrRecipient::Recipient((&rpk, &rsk));
    //         let decrypted_message = ipfs_input.get_ipfs_data(&sor).await.unwrap();
    //         let ipfs_data = IpfsData::new(decrypted_message, ipfs_hash);
    //         let file_path = ipfs_data.save().unwrap();
    //
    //         let file_contents = fs::read_to_string(&file_path).unwrap();
    //         let file_data: IpfsData = toml::from_str(&file_contents).unwrap();
    //         let last_data = file_data.data.last().unwrap();
    //
    //         fs::remove_file(file_path).unwrap();
    //
    //         assert_eq!(
    //             (&String::from("message"), &String::from("test")),
    //             (&last_data.key, &last_data.value),
    //         )
    //     }
    // }
}