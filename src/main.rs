use std::{env, process};
use nolik_cli::cli::input::Input;

fn main() {

    let args = env::args().skip(1).collect::<Vec<String>>();
    let input = Input::new(args.iter()).unwrap_or_else(|err| {
        eprintln!("Error on parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = nolik_cli::run(input) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}