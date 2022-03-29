use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use sp_keyring::AccountKeyring;
use crate::rpc::{extrinsics, calls};

#[derive(Clone, Eq, PartialEq, Debug, Copy)]
pub struct Identity {
    pub pair: AccountKeyring
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Account {
    pub pair: (PublicKey, SecretKey),
    identity: Identity,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum MenuGroup {
    Root,
    Identity(Identity),
    Account(Account),
    WhiteList(Account),
    BlackList(Account),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum MenuAction {
    ShowIdentitiesUp,
    ShowAccountsUp(Identity),
    ShowAccountsDown(Identity),
    GenerateAccount(Identity),
    ImportAccount(Identity),
    ShowAccountUp(Account),
    ShowAccountDown(Account),
    ComposeMessage(Account),
    EditRecipient(Account),
    EditMessage(Account),
    ShowAccountInfo(Account),
    ShowInbox(Account),
    ShowSent(Account),
    ShowWhiteList(Account),
    ShowBlackList(Account),
    AddToWhiteList(Account),
    AddToBlackList(Account),
    ShowWhiteListItem(Account),
    ShowBlackListItem(Account),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MenuItem {
    pub title: String,
    pub group: MenuGroup,
    pub action: MenuAction,
}

#[derive(Clone, Debug)]
pub struct Menu {
    pub items: Vec<MenuItem>,
}

impl Menu {
    pub fn identity_menu_items(title: String, identity: Identity) -> Vec<MenuItem> {
        vec![
            MenuItem {
                title,
                group: MenuGroup::Root,
                action: MenuAction::ShowAccountsDown(identity.clone()),
            },
            MenuItem {
                title: String::from(".."),
                group: MenuGroup::Identity(identity.clone()),
                action: MenuAction::ShowIdentitiesUp,
            },
            MenuItem {
                title: String::from("Generate Account"),
                group: MenuGroup::Identity(identity.clone()),
                action: MenuAction::GenerateAccount(identity.clone()),
            },
            MenuItem {
                title: String::from("Import Account"),
                group: MenuGroup::Identity(identity.clone()),
                action: MenuAction::ImportAccount(identity),
            },
        ]
    }

    pub fn account_menu_items(title: String, account: Account) -> Vec<MenuItem> {
        vec![
            MenuItem {
                title,
                group: MenuGroup::Identity(account.identity.clone()),
                action: MenuAction::ShowAccountDown(account.clone()),
            },
            MenuItem {
                title: String::from(".."),
                group: MenuGroup::Account(account.clone()),
                action: MenuAction::ShowAccountsUp(account.identity),
            },
            MenuItem {
                title: String::from("Compose Message"),
                group: MenuGroup::Account(account.clone()),
                action: MenuAction::ComposeMessage(account.clone()),
            },
            MenuItem {
                title: String::from("Inbox"),
                group: MenuGroup::Account(account.clone()),
                action: MenuAction::ShowInbox(account.clone()),
            },
            MenuItem {
                title: String::from("Sent"),
                group: MenuGroup::Account(account.clone()),
                action: MenuAction::ShowSent(account.clone()),
            },
            MenuItem {
                title: String::from("Whitelist"),
                group: MenuGroup::Account(account.clone()),
                action: MenuAction::ShowWhiteList(account.clone()),
            },
            MenuItem {
                title: String::from("Blacklist"),
                group: MenuGroup::Account(account.clone()),
                action: MenuAction::ShowBlackList(account.clone()),
            },
            MenuItem {
                title: String::from("Info"),
                group: MenuGroup::Account(account.clone()),
                action: MenuAction::ShowAccountInfo(account.clone()),
            },
            MenuItem {
                title: String::from(".."),
                group: MenuGroup::WhiteList(account.clone()),
                action: MenuAction::ShowAccountUp(account.clone()),
            },
            MenuItem {
                title: String::from("Add to Whitelist"),
                group: MenuGroup::WhiteList(account.clone()),
                action: MenuAction::AddToWhiteList(account.clone()),
            },
            MenuItem {
                title: String::from(".."),
                group: MenuGroup::BlackList(account.clone()),
                action: MenuAction::ShowAccountUp(account.clone()),
            },
            MenuItem {
                title: String::from("Add to Blacklist"),
                group: MenuGroup::BlackList(account.clone()),
                action: MenuAction::AddToBlackList(account.clone()),
            },
        ]
    }

    pub fn whitelist_items(title: String, account: Account) -> Vec<MenuItem> {
        vec![
            MenuItem {
                title,
                group: MenuGroup::WhiteList(account.clone()),
                action: MenuAction::ShowWhiteListItem(account.clone()),
            },
        ]
    }

    pub fn blacklist_items(title: String, account: Account) -> Vec<MenuItem> {
        vec![
            MenuItem {
                title,
                group: MenuGroup::BlackList(account.clone()),
                action: MenuAction::ShowBlackListItem(account.clone()),
            },
        ]
    }

    pub fn new() -> Menu {
        let mut menu_items: Vec<MenuItem> = vec![];
        for item in [
            ("Alice", AccountKeyring::Alice),
            ("Bob", AccountKeyring::Bob),
            ("Charlie", AccountKeyring::Charlie),
            ("Dave", AccountKeyring::Dave),
        ] {
            let menu_item = Menu::identity_menu_items(
                String::from(item.0),
                Identity { pair: item.1 },
            );
            menu_items.extend(menu_item);
        }

        Menu {
            items: menu_items,
        }
    }

    pub async fn save_account(&mut self, title: String, identity: Identity) -> Result<Account, Box<dyn std::error::Error>> {
        let (public_key, secret_key) = box_::gen_keypair();
        let account = Account {
            identity: identity.clone(),
            pair: (public_key, secret_key),
        };


        let address: Vec<u8> = sp_core::blake2_128(public_key.as_ref()).as_ref().into();
        let xt = extrinsics::add_owner(identity.pair.clone(), address).await;

        let call = calls::call_extrinsic(xt).await;

        match call {
            Ok(_res) => {
                let menu_item = Menu::account_menu_items(
                    title,
                    account.clone(),
                );

                let mut menu_items = self.items.clone();
                menu_items.extend(menu_item);
                self.items = menu_items;

                Ok(account)
            },
            Err(e) => Err(String::from(e.to_string()).into())
        }
    }

    // pub fn new_account(title: String, identity: Identity) -> Vec<MenuItem> {
    //     let (public_key, secret_key) = box_::gen_keypair();
    //     let account = Account {
    //         identity,
    //         pair: (public_key, secret_key),
    //     };
    //
    //     Menu::account_menu_items(
    //         title,
    //         account,
    //     )
    // }

    pub async fn save_to_whitelist(&mut self, title: String, account: Account) -> Result<(), Box<dyn std::error::Error>> {

        let address_bytes: Vec<u8> = bs58::decode(title.clone()).into_vec().unwrap();

        let add_to: Vec<u8> = sp_core::blake2_128(account.pair.0.as_ref()).as_ref().into();
        let new_address: Vec<u8> = sp_core::blake2_128(address_bytes.as_ref()).as_ref().into();
        let xt = extrinsics::add_to_whitelist(account.identity.pair.clone(), add_to, new_address).await;

        let call = calls::call_extrinsic(xt).await;

        match call {
            Ok(_res) => {

                let menu_item = Menu::whitelist_items(
                    title,
                    account,
                );

                let mut menu_items = self.items.clone();
                menu_items.extend(menu_item);
                self.items = menu_items;

                Ok(())
            },
            Err(e) => Err(String::from(e.to_string()).into())
        }
    }

    pub async fn save_to_blacklist(&mut self, title: String, account: Account) -> Result<(), Box<dyn std::error::Error>>{

        let address_bytes: Vec<u8> = bs58::decode(title.clone()).into_vec().unwrap();

        let add_to: Vec<u8> = sp_core::blake2_128(account.pair.0.as_ref()).as_ref().into();
        let new_address: Vec<u8> = sp_core::blake2_128(address_bytes.as_ref()).as_ref().into();
        let xt = extrinsics::add_to_blacklist(account.identity.pair.clone(), add_to, new_address).await;

        let call = calls::call_extrinsic(xt).await;

        match call {
            Ok(_res) => {

                let menu_item = Menu::blacklist_items(
                    title,
                    account,
                );

                let mut menu_items = self.items.clone();
                menu_items.extend(menu_item);
                self.items = menu_items;

                Ok(())
            },
            Err(e) => Err(String::from(e.to_string()).into())
        }
    }
}