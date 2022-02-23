use tui::{widgets::ListState, };
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use sp_keyring::AccountKeyring;


#[derive(Clone, Eq, PartialEq)]
pub struct Identity {
    pub pair: AccountKeyring
}

#[derive(Clone, Eq, PartialEq)]
pub struct Account {
    pub pair: (PublicKey, SecretKey),
    identity: Identity,
}

#[derive(Clone, Eq, PartialEq)]
pub enum MenuGroup {
    None,
    Root,
    Identity(Identity),
    Account(Account),
    WhiteList(Account),
    BlackList(Account),
}

#[derive(Clone, Eq, PartialEq)]
pub enum MenuAction {
    ShowIdentitiesUp,
    ShowAccountsUp(Identity),
    ShowAccountsDown(Identity),
    GenerateAccount(Identity),
    ImportAccount(Identity),
    ShowAccountUp(Account),
    ShowAccountDown(Account),
    ComposeMessage(Account),
    ShowAccountInfo(Account),
    ShowInbox(Account),
    ShowSent(Account),
    ShowWhiteList(Account),
    ShowBlackList(Account),
    AddToWhiteList(Account),
    AddToBlackList(Account),
    ShowWhiteListItem(Account),
    ShowBlackListItem(Account,)
}

#[derive(Clone, Eq, PartialEq)]
pub struct MenuItem {
    pub title: String,
    pub group: MenuGroup,
    pub action: MenuAction,
}

#[derive(Clone)]
pub struct Pane {
    pub path: Vec<MenuItem>,
    items: Vec<MenuItem>,
    pub menu: Vec<MenuItem>,
    pub state: ListState,
    pub group: MenuGroup,
    pub action: MenuAction,
    pub account_name: String,
    pub generate_account: bool,
    pub account_info: bool,
    pub compose_message: bool,
    pub show_inbox: bool,
    pub show_sent: bool,
    pub account_address: String,
    pub add_to_whitelist: bool,
    pub add_to_blacklist: bool,
}

impl Pane {
    fn identity_menu_items(title: String, identity: Identity) -> Vec<MenuItem> {
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

    fn account_menu_items(title: String, identity: Identity, account: Account) -> Vec<MenuItem> {
        vec![
            MenuItem {
                title,
                group: MenuGroup::Identity(identity.clone()),
                action: MenuAction::ShowAccountDown(account.clone()),
            },
            MenuItem {
                title: String::from(".."),
                group: MenuGroup::Account(account.clone()),
                action: MenuAction::ShowAccountsUp(identity),
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

    fn whitelist_items(title: String, account: Account) -> Vec<MenuItem> {
        vec![
            MenuItem {
                title,
                group: MenuGroup::WhiteList(account.clone()),
                action: MenuAction::ShowWhiteListItem(account.clone()),
            },
        ]
    }

    fn blacklist_items(title: String, account: Account) -> Vec<MenuItem> {
        vec![
            MenuItem {
                title,
                group: MenuGroup::BlackList(account.clone()),
                action: MenuAction::ShowBlackListItem(account.clone()),
            },
        ]
    }

    pub fn new() -> Pane {
        let mut menu_items: Vec<MenuItem> = vec![];
        for item in [
            ("Alice", AccountKeyring::Alice),
            ("Bob", AccountKeyring::Bob),
            ("Charlie", AccountKeyring::Charlie),
            ("Dave", AccountKeyring::Dave),
        ] {
            let menu_item = Pane::identity_menu_items(
                String::from(item.0),
                Identity { pair: item.1 },
            );
            menu_items.extend(menu_item);
        }

        Pane {
            path: vec![
                MenuItem {
                    title: String::from("Select Identity"),
                    group: MenuGroup::None,
                    action: MenuAction::ShowIdentitiesUp,
                }
            ],
            items: menu_items,
            menu: vec![
                MenuItem {
                    title: String::from("Alice"),
                    group: MenuGroup::Root,
                    action: MenuAction::ShowAccountsDown(Identity {
                        pair: AccountKeyring::Alice,
                    }),
                },
                MenuItem {
                    title: String::from("Bob"),
                    group: MenuGroup::Root,
                    action: MenuAction::ShowAccountsDown(Identity {
                        pair: AccountKeyring::Bob,
                    }),
                },
                MenuItem {
                    title: String::from("Charlie"),
                    group: MenuGroup::Root,
                    action: MenuAction::ShowAccountsDown(Identity {
                        pair: AccountKeyring::Charlie,
                    }),
                },
                MenuItem {
                    title: String::from("Dave"),
                    group: MenuGroup::Root,
                    action: MenuAction::ShowAccountsDown(Identity {
                        pair: AccountKeyring::Dave,
                    }),
                },
            ],
            state: ListState::default(),
            group: MenuGroup::Root,
            action: MenuAction::ShowIdentitiesUp,
            account_name: String::from(""),
            generate_account: false,
            account_info: false,
            compose_message: false,
            show_inbox: false,
            show_sent: false,
            account_address: String::from(""),
            add_to_whitelist: false,
            add_to_blacklist: false,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.menu.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.menu.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select(&mut self) {
        if let Some(index) = self.state.selected() {

            let item = self.menu.get(index).unwrap();

            match item.action.clone() {
                MenuAction::ShowIdentitiesUp => {
                    let mut path = self.path.clone();
                    path.pop();
                    self.path = path;
                },
                MenuAction::ShowAccountsUp(_) => {
                    let mut path = self.path.clone();
                    path.pop();
                    self.path = path;
                },
                MenuAction::GenerateAccount(_) => {
                    self.action = item.clone().action;
                    self.generate_account = true;
                },
                MenuAction::ShowAccountInfo(_) => {
                    self.action = item.clone().action;
                    self.account_info = true;
                },
                MenuAction::ShowAccountUp(_) => {
                    let mut path = self.path.clone();
                    path.pop();
                    self.path = path;
                },
                MenuAction::ShowAccountDown(_) => {
                    let mut path = self.path.clone();
                    path.pop();
                    self.path = path;
                },
                MenuAction::ComposeMessage(_) => {
                    self.action = item.clone().action;
                    self.compose_message = true;
                },
                MenuAction::ShowInbox(_) => {
                    self.action = item.clone().action;
                    self.show_inbox = true;
                },
                MenuAction::ShowSent(_) => {
                    self.action = item.clone().action;
                    self.show_sent = true;
                },
                MenuAction::AddToWhiteList(_) => {
                    self.action = item.clone().action;
                    self.add_to_whitelist = true;
                },
                MenuAction::AddToBlackList(_) => {
                    self.action = item.clone().action;
                    self.add_to_blacklist = true;
                },
                MenuAction::ShowWhiteListItem(_) => {
                    // self.action = item.clone().action;
                    // self.add_to_blacklist = true;
                },
                MenuAction::ShowBlackListItem(_) => {

                },
                _ => {
                    let mut path = self.path.clone();
                    path.push(item.clone());
                    self.path = path;
                    self.state.select(None);
                }
            }

            match item.action.clone() {
                MenuAction::ShowIdentitiesUp => self.group = MenuGroup::Root,
                MenuAction::ShowAccountsDown(identity) => self.group = MenuGroup::Identity(identity),
                MenuAction::ShowAccountsUp(identity) => self.group = MenuGroup::Identity(identity),
                MenuAction::ShowAccountUp(account) => self.group = MenuGroup::Account(account),
                MenuAction::ShowAccountDown(account) => self.group = MenuGroup::Account(account),
                MenuAction::ShowWhiteList(account) => self.group = MenuGroup::WhiteList(account),
                MenuAction::ShowBlackList(account) => self.group = MenuGroup::BlackList(account),
                _ => {},
            }

            let menu = self.filter_menu(self.group.clone());
            self.menu = menu;
        }
    }

    fn filter_menu(&self, group: MenuGroup) -> Vec<MenuItem> {
        self.items.iter().filter(|item| item.group.eq(&group)).cloned().collect()
    }

    pub fn save_account(&mut self) {
        if let MenuGroup::Identity(identity ) = self.group.clone() {
            let (public_key, secret_key) = box_::gen_keypair();
            let account = Account {
                identity: identity.clone(),
                pair: (public_key, secret_key),
            };

            let menu_item = Pane::account_menu_items(
                self.account_name.clone(),
                identity.clone(),
                account.clone(),
            );

            let mut menu_items = self.items.clone();
            menu_items.extend(menu_item);
            self.items = menu_items;

            self.group = MenuGroup::Account(account.clone());
            let menu = self.filter_menu(self.group.clone());
            self.menu = menu;

            let item = MenuItem {
                title: self.account_name.clone(),
                group: MenuGroup::Identity(identity),
                action: MenuAction::ShowAccountDown(account.clone()),
            };

            let mut path = self.path.clone();
            path.push(item);
            self.path = path;

            self.account_name = String::from("");
            self.generate_account = false;
            self.action = MenuAction::ShowAccountDown(account);
            self.state.select(None);
        }
    }

    pub fn save_to_whitelist(&mut self) {
        if let MenuGroup::WhiteList(account) = self.group.clone() {
            let menu_item = Pane::whitelist_items(
                self.account_address.clone(),
                account.clone(),
            );

            let mut menu_items = self.items.clone();
            menu_items.extend(menu_item);
            self.items = menu_items;

            self.group = MenuGroup::WhiteList(account.clone());
            let menu = self.filter_menu(self.group.clone());
            self.menu = menu;

            self.account_address = String::from("");
            self.add_to_whitelist = false;
            self.action = MenuAction::ShowAccountDown(account.clone());
            self.state.select(None);
        }
    }

    pub fn save_to_blacklist(&mut self) {
        if let MenuGroup::BlackList(account) = self.group.clone() {
            let menu_item = Pane::blacklist_items(
                self.account_address.clone(),
                account.clone(),
            );

            let mut menu_items = self.items.clone();
            menu_items.extend(menu_item);
            self.items = menu_items;

            self.group = MenuGroup::BlackList(account.clone());
            let menu = self.filter_menu(self.group.clone());
            self.menu = menu;

            self.account_address = String::from("");
            self.add_to_blacklist = false;
            self.action = MenuAction::ShowAccountDown(account.clone());
            self.state.select(None);
        }
    }
}