mod login;

use crate::client::ClientBuilder;
use gtk::prelude::*;
use gtk::{Builder, Button, Entry, Label, Window};
use login::*;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

/// This function runs the connect window, wich is the first window that the user sees
/// if everything goes well, it will call the login window,
/// and then the chat window (the main window of the app)
pub fn run_connect() -> Result<(), Box<dyn Error>> {
    // First we get the file content (the xml/glade file)
    let glade_src = include_str!("connect.glade");
    // Then we call the Builder call.
    let builder = Builder::from_string(glade_src);

    let window: Option<Window> = builder.object("connect_window");
    let window_clone = window.clone();

    match window_clone {
        Some(window) => window.show_all(),
        None => println!("Problems opening connect_window"),
    }

    let window_clone = window.clone().expect("Error creating a new window");
    let can_close_window = Rc::new(RefCell::new(false));
    let can_close_window_clone = can_close_window.clone();

    let ip_input: Entry = builder
        .object("ip_input")
        .expect("ip_input object not found");
    let port_input: Entry = builder
        .object("port_input")
        .expect("port_input object not found");
    let connect_button: Button = builder
        .object("connect_button")
        .expect("connect_button object not found");
    let error_label: Label = builder
        .object("error_label")
        .expect("error_label object not found");
    connect_button.connect_clicked(move |_| {
        let mut client_builder = ClientBuilder::new();
        client_builder.set_ip(ip_input.text().to_string());
        client_builder.set_port(port_input.text().to_string());
        match client_builder.get_client() {
            Ok(client) => {
                let mut can_close_window = can_close_window_clone.borrow_mut();
                *can_close_window = true;
                window_clone.close();
                run_login(client); //runs the login window
            }
            Err(line) => error_label.set_label(line.as_str()),
        }
    });

    let window_clone = window.expect("Error creating the window");
    window_clone.connect_delete_event(move |_, _| {
        if !*can_close_window.borrow() {
            gtk::main_quit();
        }
        Inhibit(false)
    });

    Ok(())
}
