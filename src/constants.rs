

pub mod commands {
    pub const ADD_WALLET: (&str, &str) = ("add", "wallet");
    pub const ADD_ACCOUNT: (&str, &str) = ("add", "account");
    pub const DELETE_WALLET: (&str, &str) = ("delete", "wallet");
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
    pub const WALLET: &str = "--wallet";
    pub const K: &str = "-k";
    pub const KEYRING: &str = "--keyring";
    pub const I: &str = "-i";
    pub const IMPORT: &str = "--import";
    pub const O: &str = "-o";
    pub const OUTPUT: &str = "--output";
}

pub mod errors {
    pub const UNRECOGNISED_COMMAND: &str = "Unrecognised command";
    pub const NOT_ENOUGH_ARGUMENTS: &str = "Not enough arguments";
    pub const NO_ARGUMENTS: &str = "No arguments";
    pub const UNRECOGNISED_FLAG: &str = "Unrecognised flag";
    pub const NO_CORRESPONDING_VALUE: &str = "No corresponding value to provided key";
    pub const REQUIRED_KEYS_MISSING: &str = "Required keys are missing";
    pub const UNKNOWN_FLAGS: &str = "Unknown flags";
    pub const INVALID_FLAG: &str = "Invalid flag";
    pub const NON_UNIQUE_KEYS: &str = "Non unique keys";
    pub const NO_SUCH_KEY: &str = "No such key";
}