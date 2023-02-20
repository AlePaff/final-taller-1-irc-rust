mod app_errors;
mod config;
mod server;

use config::Config;
use server::Server;
use std::env;
use std::process;

/// The main function for the server, which parses the command line arguments, builds the server and runs it.
fn main() {
    let config = Config::build(env::args());
    let config = match config {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Problem parsing arguments: {err}");
            process::exit(1);
        }
    };

    let mut server = match Server::build(config) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("Server error: {err}");
            process::exit(1);
        }
    };

    if let Err(err) = server.run() {
        eprintln!("Server error: {err}");
        process::exit(1);
    }
}
