use std::{env, process};
use nolik_cli::Config;

fn main() {

    let args = env::args().skip(1).collect::<Vec<String>>();
    let config = Config::new(args.iter()).unwrap_or_else(|err| {
        eprintln!("Error on parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = nolik_cli::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}