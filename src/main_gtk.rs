mod app_errors;
mod client;
mod config_client;
mod controller;

/// The main function for the client with the GTK interface.
fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    controller::run_interface();

    gtk::main();
}
