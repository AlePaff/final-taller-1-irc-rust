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

    // clona y usa rc con box simplemente para poder apretar Enter luego de escribir el puerto
    let ip_input_clone = ip_input.clone();
    let port_input_clone = port_input.clone();
    let error_label_clone = error_label.clone();

    let handle_connect = Rc::new(RefCell::new(Box::new(move || {
        let mut client_builder = ClientBuilder::new();
        client_builder.set_ip(ip_input_clone.text().to_string());
        client_builder.set_port(port_input_clone.text().to_string());
        match client_builder.get_client() {
            Ok(client) => {
                let mut can_close_window = can_close_window_clone.borrow_mut();
                *can_close_window = true;
                window_clone.close();
                run_login(client);
            }
            Err(line) => error_label_clone.set_label(line.as_str()),
        }
    }) as Box<dyn FnMut()>));

    // si hago click en el boton enviar
    let handle_connect_clone = handle_connect.clone();
    connect_button.connect_clicked(move |_| {
        (handle_connect_clone.borrow_mut())();
    });
    // permitir apretar Enter (connect_activate)
    let handle_connect_clone = handle_connect.clone();
    port_input.connect_activate(move |_| {
        (handle_connect_clone.borrow_mut())();
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
