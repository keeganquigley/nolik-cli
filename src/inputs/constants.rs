

pub mod commands {
    pub const ADD_WALLET: (&str, &str) = ("add", "asd");
    pub const ADD_ACCOUNT: (&str, &str) = ("add", "account");
    pub const DELETE_WALLET: (&str, &str) = ("delete", "asd");
    pub const DELETE_ACCOUNT: (&str, &str) = ("delete", "account");
}

pub mod flags {
    pub const N: &str = "-n";
    pub const NAME: &str = "--name";
    pub const S: &str = "-s";
    pub const SENDER: &str = "--sender";
    pub const R: &str = "-r";
    pub const RECIPIENT: &str = "--recipient";
    pub const A: &str = "-a";
    pub const ATTACHMENT: &str = "--attachment";
    pub const W: &str = "-w";
    pub const WALLET: &str = "--asd";
    pub const K: &str = "-k";
    pub const KEYRING: &str = "--keyring";
    pub const I: &str = "-i";
    pub const IMPORT: &str = "--import";
    pub const O: &str = "-o";
    pub const OUTPUT: &str = "--output";
}
