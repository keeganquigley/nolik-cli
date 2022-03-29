use tui::{widgets::ListState};
use crate::Menu;
use crate::menu::{MenuItem, MenuAction, MenuGroup, Account};


#[derive(Clone, Debug)]
pub enum ComposeFocus {
    Recipient,
    Message,
}

#[derive(Clone, Debug)]
pub struct Pane {
    pub path: Vec<MenuItem>,
    pub full_menu: Vec<MenuItem>,
    pub pane_menu: Vec<MenuItem>,
    pub state: ListState,
    pub group: MenuGroup,
    pub action: MenuAction,
    pub account_name: String,
    pub generate_account: bool,
    pub import_account: bool,
    pub import_seed: String,
    pub account_info: bool,
    pub compose_message: bool,
    pub compose_focus: Option<ComposeFocus>,
    pub recipient: String,
    pub recipient_error: bool,
    pub message: String,
    pub message_error: bool,
    pub show_inbox: bool,
    pub show_sent: bool,
    pub account_address: String,
    pub add_to_whitelist: bool,
    pub add_to_blacklist: bool,
}

impl Pane {
    pub fn new() -> Pane {
        Pane {
            path: vec![],
            full_menu: Menu::new().items,
            pane_menu: Pane::filter_menu(Menu::new().items, MenuGroup::Root),
            state: ListState::default(),
            group: MenuGroup::Root,
            action: MenuAction::ShowIdentitiesUp,
            account_name: String::from(""),
            generate_account: false,
            import_account: false,
            import_seed: String::from(""),
            account_info: false,
            compose_message: false,
            compose_focus: None,
            recipient: String::from(""),
            recipient_error: false,
            message: String::from(""),
            message_error: false,
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
                if i >= self.pane_menu.len() - 1 {
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
                    self.pane_menu.len() - 1
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

            let item = self.pane_menu.get(index).unwrap();

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
                MenuAction::ImportAccount(_) => {
                    self.action = item.clone().action;
                    self.import_account = true;
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

            self.update_pane_menu();
        }
    }

    pub fn update_full_menu(&mut self, full_menu: Vec<MenuItem>) {
        self.full_menu = full_menu;
        self.update_pane_menu();
    }

    pub fn update_pane_menu(&mut self) {
        let menu_items = Pane::filter_menu(
            self.full_menu.clone(),
            self.group.clone());
        self.pane_menu = menu_items;
    }

    pub fn filter_menu(menu_items: Vec<MenuItem>, group: MenuGroup) -> Vec<MenuItem> {
        menu_items.iter().filter(|item| item.group.eq(&group)).cloned().collect()
    }

    pub fn on_account_save(&mut self, account: Account) {
        self.action = MenuAction::ShowAccountDown(account.clone());
        self.group = MenuGroup::Account(account.clone());

        let item = MenuItem {
            title: self.account_name.clone(),
            action: self.action.clone(),
            group: self.group.clone(),
        };
        let mut path = self.path.clone();
        path.push(item.to_owned());
        self.path = path;

        self.generate_account = false;
        self.account_name = String::from("");
        self.state.select(None);

        self.update_pane_menu();
    }

    pub fn on_save_to_whitelist(&mut self, account: Account) {
        self.action = MenuAction::ShowWhiteList(account);
        self.add_to_whitelist = false;
        self.account_address = String::from("");
        self.state.select(None);
    }

    pub fn on_save_to_blacklist(&mut self, account: Account) {
        self.action = MenuAction::ShowBlackList(account);
        self.add_to_blacklist = false;
        self.account_address = String::from("");
        self.state.select(None);
    }
}