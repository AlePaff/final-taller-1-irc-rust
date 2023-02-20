mod chat;

use crate::client::ClientC;
use chat::*;
use gtk::prelude::*;
use gtk::{Builder, Button, Entry, Label, Window};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Runs the login window, wich is the second window that the user sees.
/// it allows to register a new user
pub fn run_login(client: ClientC) {
    // First we get the file content.
    let glade_src = include_str!("login.glade");
    // Then we call the Builder call.
    let builder = Builder::from_string(glade_src);

    let window: Option<Window> = builder.object("login_window");
    let window_clone = window.clone();

    match window_clone {
        Some(window) => window.show_all(),
        None => println!("Problems opening login_window"),
    }

    let can_close_window = Rc::new(RefCell::new(false));
    let can_close_window_clone = can_close_window.clone();

    //input fields and button
    let nick_input: Entry = builder
        .object("nick_input")
        .expect("nick_input object not found");
    let user_input: Entry = builder
        .object("user_input")
        .expect("user_input object not found");
    let pass_input: Entry = builder
        .object("pass_input")
        .expect("pass_input object not found");
    let login_button: Button = builder
        .object("login_button")
        .expect("login_button object not found");
    let error_label: Label = builder
        .object("error_label")
        .expect("error_label object not found");
    let client = Arc::new(Mutex::new(client));
    let button_client = client.clone();
    let window_clone = window.clone().expect("Error creating window");

    login_button.connect_clicked(move |_| {
        let result = button_client
            .lock()
            .expect("Error creating login button")
            .register(
                pass_input.text().to_string(),
                nick_input.text().to_string(),
                user_input.text().to_string(),
            );
        match result.starts_with('2') {
            //2xx are the http codes for success
            true => {
                let mut can_close_window = can_close_window_clone.borrow_mut();
                *can_close_window = true;
                window_clone.close();
                run_chat(client.clone()).expect("Error running run_chat"); //runs the chat window
            }
            _ => {
                error_label.set_label(result.as_str());
            }
        }
    });

    // This is the code that runs when the window is closed.
    let window_clone = window.expect("Error creating window");
    window_clone.connect_delete_event(move |_, _| {
        if !*can_close_window.borrow() {
            gtk::main_quit();
        }
        Inhibit(false)
    });
}
