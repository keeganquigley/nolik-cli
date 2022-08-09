use std::{env, process};
use nolik_cli::cli::input::Input;
use async_std;
use colored::Colorize;

#[async_std::main]
async fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let input = Input::new(args.iter()).unwrap_or_else(|err| {
        eprintln!("Error on parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = nolik_cli::run(input).await {
        let err = format!("Application error: {}", e);
        eprintln!("{}", err.bright_red());
        process::exit(1);
    }
}