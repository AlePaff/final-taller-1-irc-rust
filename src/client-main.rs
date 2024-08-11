mod app_errors;
mod client;
mod config_client;

use client::ClientC;
use config_client::ConfigClient;
use std::env;
use std::process;

/// The main function for the client, which parses the command line arguments, builds the client and runs it.
fn main() {
    eprintln!("Iniciando por consola...");

    let config = match ConfigClient::build(env::args()) {
        Err(err) => {
            eprintln!("Problem parsing arguments: {err}");
            process::exit(1);
        }
        Ok(x) => x,
    };
    let mut client = match ClientC::build(config) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("client error: {err}");
            process::exit(1);
        }
    };

    if let Err(err) = client.run() {
        eprintln!("client error: {err}");
        process::exit(1);
    }
}
