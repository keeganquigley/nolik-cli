#[cfg(test)]
mod blacklist {
    use std::fs;
    use sp_core::crypto::AccountId32;
    use sp_keyring::AccountKeyring;
    use nolik_cli::account::{Account, AccountInput};
    use nolik_cli::blacklist::Blacklist;
    use nolik_cli::cli::config::ConfigFile;
    use nolik_cli::cli::input::Input;
    use nolik_cli::node::errors::NodeError;
    use nolik_cli::node::events::{BalanceTransferEvent, NodeEvent};
    use nolik_cli::node::extrinsics::balance_transfer;
    use nolik_cli::owner::Owner;
    use nolik_cli::wallet::{Wallet, WalletInput};
    use nolik_cli::whitelist::Whitelist;

    async fn create_new_config_file() -> ConfigFile {

        let config_file: ConfigFile = ConfigFile::temp();

        let arr = [
            "add",
            "account",
            "--alias",
            "alice"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap();

        Account::add(&config_file, &account).unwrap();


        let arr = [
            "add",
            "account",
            "--alias",
            "bob"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap();

        Account::add(&config_file, &account).unwrap();


        let arr = [
            "add",
            "wallet",
            "--alias",
            "wallet_a"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let password = String::from("pass");

        let wallet_input = WalletInput::new(&input, Some(password)).unwrap();
        let wallet_a = Wallet::new(wallet_input).unwrap();

        Wallet::add(&config_file, &wallet_a).unwrap();


        let arr = [
            "add",
            "wallet",
            "--alias",
            "wallet_b"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let password = String::from("pass");

        let wallet_input = WalletInput::new(&input, Some(password)).unwrap();
        let wallet_b = Wallet::new(wallet_input).unwrap();

        Wallet::add(&config_file, &wallet_b).unwrap();


        let sender = AccountKeyring::Alice;

        let recipient = AccountId32::from(wallet_a.public);
        let extrinsic_hash = balance_transfer(sender, &recipient).await.unwrap();
        let event = BalanceTransferEvent;
        event.submit(&extrinsic_hash).await.unwrap();

        let recipient = AccountId32::from(wallet_b.public);
        let extrinsic_hash = balance_transfer(sender, &recipient).await.unwrap();
        let event = BalanceTransferEvent;
        event.submit(&extrinsic_hash).await.unwrap();

        config_file
    }


    #[async_std::test]
    async fn add_to_blacklist() {
        let config_file = create_new_config_file().await;
        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let bob = Account::get(&config_file, String::from("bob")).unwrap();
        let wallet_a = Wallet::get(&config_file, String::from("wallet_a"), Some(String::from("pass"))).unwrap();


        let arr = [
            "add",
            "owner",
            "--account",
            format!("{}", alice.alias).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str()
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let owner = Owner::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        owner.add().await.unwrap();


        let arr = [
            "update",
            "blacklist",
            "--for",
            format!("{}", alice.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(bob.public).into_string()).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let blacklist = Blacklist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        let res = blacklist.update().await.is_ok();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            res,
            true,
        );
    }

    #[async_std::test]
    async fn add_to_blacklist_same_address() {
        let config_file = create_new_config_file().await;
        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let wallet_a = Wallet::get(&config_file, String::from("wallet_a"), Some(String::from("pass"))).unwrap();


        let arr = [
            "add",
            "owner",
            "--account",
            format!("{}", alice.alias).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str()
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let owner = Owner::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        owner.add().await.unwrap();


        let arr = [
            "update",
            "blacklist",
            "--for",
            format!("{}", alice.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(alice.public).into_string()).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let blacklist = Blacklist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        let res = blacklist.update().await.unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            res,
            NodeError::PalletSameAddress,
        );
    }


    #[async_std::test]
    async fn add_to_blacklist_of_not_owned_address() {
        let config_file = create_new_config_file().await;
        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let bob = Account::get(&config_file, String::from("bob")).unwrap();
        let wallet_a = Wallet::get(&config_file, String::from("wallet_a"), Some(String::from("pass"))).unwrap();
        let wallet_b = Wallet::get(&config_file, String::from("wallet_b"), Some(String::from("pass"))).unwrap();


        let arr = [
            "add",
            "owner",
            "--account",
            format!("{}", alice.alias).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str()
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let owner = Owner::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        owner.add().await.unwrap();


        let arr = [
            "update",
            "blacklist",
            "--for",
            format!("{}", alice.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(bob.public).into_string()).as_str(),
            "--wallet",
            format!("{}", wallet_b.alias).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let blacklist = Blacklist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        let res = blacklist.update().await.unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            res,
            NodeError::PalletAddressNotOwned,
        );
    }


    #[async_std::test]
    async fn add_to_blacklist_already_added_address() {
        let config_file = create_new_config_file().await;
        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let bob = Account::get(&config_file, String::from("bob")).unwrap();
        let wallet_a = Wallet::get(&config_file, String::from("wallet_a"), Some(String::from("pass"))).unwrap();


        let arr = [
            "add",
            "owner",
            "--account",
            format!("{}", alice.alias).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str()
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let owner = Owner::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        owner.add().await.unwrap();


        let arr = [
            "update",
            "blacklist",
            "--for",
            format!("{}", alice.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(bob.public).into_string()).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let blacklist = Blacklist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        blacklist.update().await.unwrap();


        let arr = [
            "update",
            "blacklist",
            "--for",
            format!("{}", alice.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(bob.public).into_string()).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let blacklist = Blacklist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        let res = blacklist.update().await.unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            res,
            NodeError::PalletAlreadyInBlacklist,
        );
    }

    #[async_std::test]
    async fn add_to_blacklist_of_address_already_added_to_whitelist() {
        let config_file = create_new_config_file().await;
        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let bob = Account::get(&config_file, String::from("bob")).unwrap();
        let wallet_a = Wallet::get(&config_file, String::from("wallet_a"), Some(String::from("pass"))).unwrap();


        let arr = [
            "add",
            "owner",
            "--account",
            format!("{}", alice.alias).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str()
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let owner = Owner::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        owner.add().await.unwrap();


        let arr = [
            "update",
            "whitelist",
            "--for",
            format!("{}", alice.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(bob.public).into_string()).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let whitelist = Whitelist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        whitelist.update().await.unwrap();


        let arr = [
            "update",
            "blacklist",
            "--for",
            format!("{}", alice.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(bob.public).into_string()).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let blacklist = Blacklist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        let res = blacklist.update().await.unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            res,
            NodeError::PalletAlreadyInWhitelist,
        );
    }
}
