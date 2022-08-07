

pub mod commands {
    pub const ADD_WALLET: (&str, &str) = ("add", "wallet");
    pub const ADD_ACCOUNT: (&str, &str) = ("add", "account");
    pub const ADD_OWNER: (&str, &str) = ("add", "owner");
    pub const SEND_MESSAGE: (&str, &str) = ("send", "message");
    pub const GET_MESSAGES: (&str, &str) = ("get", "messages");
    pub const GET_COINS: (&str, &str) = ("get", "coins");
    pub const UPDATE_WHITELIST: (&str, &str) = ("update", "whitelist");
    pub const UPDATE_BLACKLIST: (&str, &str) = ("update", "blacklist");

    pub const ADD_CONTACT: (&str, &str) = ("add", "contact");
}

pub mod flags {
    pub const A: &str = "-a";
    pub const ACCOUNT: &str = "--account";
    pub const ALIAS: &str = "--alias";
    pub const I: &str = "-i";
    pub const IMPORT: &str = "--import";
    pub const S: &str = "-s";
    pub const SENDER: &str = "--sender";
    pub const R: &str = "-r";
    pub const RECIPIENT: &str = "--recipient";
    pub const K: &str = "-k";
    pub const KEY: &str = "--key";
    pub const V: &str = "-v";
    pub const VALUE: &str = "--value";
    pub const B: &str = "-b";
    pub const BLOB: &str = "--blob";
    pub const W: &str = "-w";
    pub const WALLET: &str = "--wallet";
}
