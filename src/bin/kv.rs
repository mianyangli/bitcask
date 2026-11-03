use std::env;

use clap::{Parser, Subcommand};
use kvs::{KvStore, Result};
use std::process;

#[derive(Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Get { key: String },
    Set { key: String, value: String },
    Rm { key: String },
}

fn route(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Get { key } => handle_get(key),
        Commands::Set { key, value } => handle_set(key, value),
        Commands::Rm { key } => handle_rm(key),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    route(cli)
}

fn handle_get(_key: String) -> Result<()> {
    let store = KvStore::open(env::current_dir()?)?;
    match store.get(_key)? {
        Some(value) => println!("{}", value),
        None => println!("Key not found")
    }

    Ok(())
}

fn handle_set(_key: String, _value: String) -> Result<()> {
    let mut store = KvStore::open(env::current_dir()?)?;
    store.set(_key, _value)
}

fn handle_rm(_key: String) -> Result<()> {
    let mut store = KvStore::open(env::current_dir()?)?;
    if store.remove(_key).is_err() {
        println!("Key not found");
        process::exit(1);
    }
    Ok(())
}
