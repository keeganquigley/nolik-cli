#[cfg(test)]
mod owner {
    use nolik_cli::account::{Account, AccountInput};
    use nolik_cli::cli::config::ConfigFile;
    use nolik_cli::cli::input::Input;
    use nolik_cli::owner::Owner;
    use nolik_cli::wallet::{Wallet, WalletInput};

    async fn create_new_owner() -> Owner {

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
        Account::add(&config_file, account.clone()).unwrap();


        let arr = [
            "add",
            "wallet",
            "--name",
            "personal",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let password = String::from("pass");

        let wallet_input = WalletInput::new(input, Some(password)).unwrap();
        let wallet = Wallet::new(wallet_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Wallet::add(config_file.clone(), wallet.clone()).unwrap();

        Owner {
            account,
            wallet,
        }
    }
}
