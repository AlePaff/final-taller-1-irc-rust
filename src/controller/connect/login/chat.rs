use crate::client::ClientC;
use crate::client::Received::*;
use gtk::prelude::*;
use gtk::{Builder, Button, Dialog, Entry, Label, Menu, MenuItem, TextView, Window};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Runs the chat window, wich is the third window that the user sees and is the main window of the app.
pub fn run_chat(client: Arc<Mutex<ClientC>>) -> Result<(), Box<dyn Error>> {
    // First we get the file content.
    let glade_src = include_str!("irc_chat_chiquito.glade");
    let entry_dialog_src = include_str!("entry_dialog.glade");
    let double_entry_dialog_src = include_str!("double_entry_dialog.glade");

    // Then we call the Builder call.
    let builder = Builder::from_string(glade_src);
    // let opers_login_builder = Builder::from_string(opers_login_src);

    let window: Option<Window> = builder.object("window");
    let window_clone = window.clone();
    match window {
        Some(window) => window.show_all(),
        None => println!("Problems opening chat_window"),
    }
    let window = match window_clone {
        Some(window) => window,
        None => panic!("Error opening chat_window"),
    };

    // if the user closes the window, the client will send a quit message to the server
    let client_quit = client.clone();
    window.connect_delete_event(move |_, _| {
        if let Ok(mut guard) = client_quit.lock() {
            guard.send_quit();
        }
        gtk::main_quit();
        Inhibit(false)
    });
    let active_chat = Arc::new(Mutex::new(String::new()));

    //NICKS
    // get nicks and add them to the dropdown.
    let conversations = Arc::new(Mutex::new(HashMap::new()));
    // conversations is a HashMap with the format <nick, chat_history>

    //CHATS
    let chat_title: Label = builder
        .object("chat_title_label")
        .expect("chat_title object not found");
    //unirse a un channel y obtengo TextView
    let chat_display: TextView = builder
        .object("chat_display")
        .expect("chat_display object not found");

    // ENVIAR MENSAJES
    let send_entry: Entry = builder
        .object("input_box")
        .expect("input_box object not found");
    let send_button: Button = builder
        .object("send_button")
        .expect("send_button object not found");

    // enter key
    let send_entry_enter = send_entry.clone();
    let send_entry_enter_in = send_entry.clone();
    let chat_display_enter = chat_display.clone();
    let client_clone_enter = client.clone();
    let nick_conversations_enter = conversations.clone();
    let active_chat_enter = active_chat.clone();
    send_entry_enter.connect_key_press_event(move |_, key| {
        let keyval: u32 = match key.keyval().to_value().get() {
            //keyval() is for the key pressed, to_value() is for the value of the key pressed
            Ok(guard) => guard,
            Err(_) => panic!("Error geting keypress keyvalues"),
        };
        let enter_val = 65293; //65293 is the value of the enter key wich corresponds to the Enter/Return key

        if keyval == enter_val {
            send(
                send_entry_enter_in.clone(),
                chat_display_enter.clone(),
                client_clone_enter.clone(),
                nick_conversations_enter.clone(),
                active_chat_enter.clone(),
            );
        }
        Inhibit(false)
    });

    let send_active_chat = active_chat.clone();
    let client_clone_send = client.clone();
    let chat_display_send = chat_display.clone();
    let nick_conversations_send = conversations.clone();
    send_button.connect_clicked(move |_| {
        send(
            send_entry.clone(),
            chat_display_send.clone(),
            client_clone_send.clone(),
            nick_conversations_send.clone(),
            send_active_chat.clone(),
        );
    });

    /// sends a message to a chat given an: entry (a message), a TextView (the chat display),
    /// a client, a HashMap (the conversations, <nick, chat_history>), and a String (the chat destination)
    fn send(
        entry: Entry,
        chat_display: TextView,
        client: Arc<Mutex<ClientC>>,
        conversations: Arc<Mutex<HashMap<String, String>>>,
        to: Arc<Mutex<String>>,
    ) {
        let message = entry.buffer().text();
        if message.is_empty() {
            return;
        }
        let to = match to.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => panic!("Error accesing send destination lock"),
        };
        if !to.is_empty() {
            let mut conversations = match conversations.lock() {
                Ok(guard) => guard,
                Err(_) => panic!("Error accesing channel conversations lock"),
            };
            match conversations.get_mut(&to) {
                //get the chat history given the nick ("to" variable)
                Some(chat) => {
                    //if the chat exists, add the message to it
                    chat.push_str(format!("yo: {}\n", &message).as_str());
                    chat_display
                        .buffer()
                        .expect("chat_display object not found")
                        .set_text(chat);
                }
                None => {
                    //if the chat doesn't exist, create it and add the message to it
                    conversations.insert(to.clone(), format!("yo: {}\n", &message));
                }
            };
            if let Ok(mut guard) = client.lock() {
                guard.send_privmsg(to, message);
            }
        }
        entry.buffer().set_text(""); //clear the entry
                                     //conversations is a hashmap with -> the key: the chat destination, the value: the chat history
    }

    /// refreshes the chat display given a String (the current chat), a TextView (the chat display)
    /// and a HashMap (the conversations, <nick, chat_history>)
    fn refresh_chat(
        current: &Arc<Mutex<String>>,
        chat_display: &TextView,
        conversations: &Arc<Mutex<HashMap<String, String>>>,
    ) {
        let current = match current.lock() {
            Ok(guard) => guard,
            Err(_) => panic!("Error accesing current chat lock"),
        };
        if current.is_empty() {
            return;
        }
        let conversations = match conversations.lock() {
            Ok(guard) => guard,
            Err(_) => panic!("Error accesing conversations lock"),
        };
        if let Some(chat_history) = conversations.get(current.as_str()) {
            chat_display
                .buffer()
                .expect("chat_display object not found")
                .set_text(chat_history);
        }
    }

    //MENU BAR
    let menu_chats: MenuItem = builder
        .object("menu_chats")
        .expect("menu_chats object not found");
    let chats_users: MenuItem = builder
        .object("chats_users")
        .expect("chats_users object not found");
    let chats_channels: MenuItem = builder
        .object("chats_channels")
        .expect("chats_channels object not found");
    let servers_oper_login: MenuItem = builder
        .object("servers_oper_login")
        .expect("servers_oper_login object not found");
    let servers_disconnect_from: MenuItem = builder
        .object("servers_disconnect_from")
        .expect("servers_disconnect_from object not found");
    let channels_create_channel: MenuItem = builder
        .object("channels_create_channel")
        .expect("channels_create_channel object not found");
    let channels_leave: MenuItem = builder
        .object("channels_leave")
        .expect("channels_leave object not found");
    let channels_invite: MenuItem = builder
        .object("channels_invite")
        .expect("channels_invite object not found");
    let channels_kick_user: MenuItem = builder
        .object("channels_kick_user")
        .expect("channels_kick_user object not found");
    let modes_give_mod: MenuItem = builder
        .object("modes_give_mod")
        .expect("modes_give_mod object not found");
    let modes_set_limit: MenuItem = builder
        .object("modes_set_limit")
        .expect("modes_set_limit object not found");
    let modes_set_secret: MenuItem = builder
        .object("modes_set_secret")
        .expect("modes_set_secret object not found");
    let modes_unset_secret: MenuItem = builder
        .object("modes_unset_secret")
        .expect("modes_unset_secret object not found");
    let modes_set_invite: MenuItem = builder
        .object("modes_set_invite_only")
        .expect("modes_set_invite_only object not found");
    let modes_unset_invite: MenuItem = builder
        .object("modes_unset_invite_only")
        .expect("modes_unset_invite_only object not found");

    // let channels_mode: MenuItem = builder
    //     .object("channels_mode")
    //     .expect("channels_mode object not found");
    // let channels_mode: MenuItem = builder
    //     .object("channels_mode")
    //     .expect("channels_mode object not found");

    let client_clone = client.clone();

    //LOGIN AS OPER
    servers_oper_login.connect_button_press_event(move |_, _| {
        let entry_dialog_builder = Builder::from_string(entry_dialog_src);
        let dialog: Dialog = entry_dialog_builder
            .object("entry_dialog")
            .expect("Problems opening entry_dialog");
        let ok_button: Button = entry_dialog_builder
            .object("ok")
            .expect("ok object not found");
        let cancel_button: Button = entry_dialog_builder
            .object("cancel")
            .expect("cancel object not found");
        let entry: Entry = entry_dialog_builder
            .object("entry")
            .expect("entry object not found");
        dialog.show_all();
        let dialog_clone = dialog.clone();
        let entry_clone = entry.clone();
        cancel_button.connect_clicked(move |_| {
            dialog_clone.close();
            // dialog_clone.hide();
            // entry_clone.buffer().set_text("");
        });
        let dialog_clone = dialog;
        let entry_clone = entry_clone;
        let client_clone = client_clone.clone();
        ok_button.connect_clicked(move |_| {
            let pass = entry_clone.buffer().text();
            client_clone
                .lock()
                .expect("Couldn't lock client")
                .become_oper(pass);
            dialog_clone.close();
            entry.buffer().set_text("");
        });

        gtk::Inhibit(true)
    });

    //DISCONECT A SERVER
    let client_clone = client.clone();
    servers_disconnect_from.connect_button_press_event(move |_, _| {
        let entry_dialog_builder = Builder::from_string(entry_dialog_src);
        let dialog: Dialog = entry_dialog_builder
            .object("entry_dialog")
            .expect("Problems opening entry_dialog");
        let ok_button: Button = entry_dialog_builder
            .object("ok")
            .expect("ok object not found");
        let cancel_button: Button = entry_dialog_builder
            .object("cancel")
            .expect("cancel object not found");
        let entry: Entry = entry_dialog_builder
            .object("entry")
            .expect("entry object not found");
        dialog.show_all();
        let dialog_clone = dialog.clone();
        let entry_clone = entry.clone();
        cancel_button.connect_clicked(move |_| {
            dialog_clone.close();
            entry_clone.buffer().set_text("");
        });
        let dialog_clone = dialog;
        let entry_clone = entry;
        let client_clone = client_clone.clone();
        ok_button.connect_clicked(move |_| {
            let server = entry_clone.buffer().text();
            client_clone
                .lock()
                .expect("Couldn't lock client")
                .send_squit(server);
            dialog_clone.close();
            // entry.buffer().set_text("");
        });
        gtk::Inhibit(true)
    });

    //REVISAR
    let chat_display_clone = chat_display.clone();
    let channels_conversations_changed = conversations.clone();
    let nick_conversations_clone = conversations.clone();
    let client_clone_join = client.clone();

    let chat_title_clone = chat_title.clone();
    let active_chat_clone = active_chat.clone();
    let nick_conversations_refresh = conversations.clone();

    //APRETAR EL MENU CHATS
    let client_clone = client.clone();
    let client_clone_nicks = client.clone();
    let join_clone = chats_channels;
    menu_chats.connect_button_press_event(move |_, _| {
        let channels = match client_clone.lock() {
            Ok(mut client_guard) => client_guard.get_names(),
            Err(_) => panic!("Couldn't lock client"),
        };
        let menu = Menu::new();
        for channel in channels {
            let item = MenuItem::with_label(&channel);
            let channels_conversations_changed = channels_conversations_changed.clone();
            let active_chat_clone = active_chat_clone.clone();
            let chat_display_clone = chat_display_clone.clone();
            let client_clone_join = client_clone_join.clone();
            let chat_title_clone = chat_title_clone.clone();
            let item_clone = item.clone();
            item.connect_button_press_event(move |_, _| {
                let client_clone_join = client_clone_join.clone();
                let chat_display_clone = chat_display_clone.clone();
                let active_chat_clone = active_chat_clone.clone();
                if let Some(channel) = item_clone.label() {
                    client_clone_join
                        .lock()
                        .expect("Couldn't lock client")
                        .join(channel.to_string());
                    let mut conversations = match channels_conversations_changed.lock() {
                        Ok(conversations_guard) => conversations_guard,
                        Err(_) => panic!("Couldn't lock conversations"),
                    };
                    let buffer = conversations.get(channel.as_str());
                    match buffer {
                        Some(buffer) => {
                            chat_display_clone
                                .buffer()
                                .expect("chat_display object not found")
                                .set_text(buffer);

                            if let Ok(mut active_chat) = active_chat_clone.lock() {
                                active_chat.clear();
                                active_chat.push_str(&channel);
                            } else {
                                return gtk::Inhibit(true); //si falla al hacer el lock
                                                           // return Err(Box::new(app_errors::ApplicationError("lock mutex".to_string(),)));
                            }
                        }
                        None => {
                            chat_display_clone
                                .buffer()
                                .expect("chat_display object not found")
                                .set_text("");
                            if let Ok(mut active_chat) = active_chat_clone.lock() {
                                active_chat.clear();
                                active_chat.push_str(&channel);
                            } else {
                                return gtk::Inhibit(true); //si falla al hacer el lock
                            }
                            conversations.insert(channel.to_string(), String::new());
                        }
                    }
                    chat_title_clone.set_text(&channel);
                }
                Inhibit(true)
            });
            item.show();
            menu.append(&item);
        }
        join_clone.set_submenu(Some(&menu));

        let nicks = match client_clone_nicks.lock() {
            Ok(mut guard) => guard.get_server_nicks().expect("couldn't get server nicks"),
            Err(_) => panic!("Couldn't lock client"),
        };
        let mut conversations = match nick_conversations_refresh.lock() {
            Ok(guard) => guard,
            Err(_) => panic!("Couldn't lock conversations"),
        };
        let menu = Menu::new();

        for nick in nicks {
            let item = MenuItem::with_label(&nick);
            let nick_conversations_clone = nick_conversations_clone.clone();
            let active_chat_clone = active_chat_clone.clone();
            let chat_display_clone = chat_display_clone.clone();
            let chat_title_clone = chat_title_clone.clone();
            let item_clone = item.clone();
            item.connect_button_press_event(move |_, _| {
                let chat_display_clone = chat_display_clone.clone();
                let active_chat_clone = active_chat_clone.clone();
                if let Some(nick) = item_clone.label() {
                    let mut conversations = match nick_conversations_clone.lock() {
                        Ok(guard) => guard,
                        Err(_) => panic!("Couldn't lock conversations"),
                    };
                    let buffer = conversations.get(nick.as_str());
                    match buffer {
                        Some(buffer) => {
                            chat_display_clone
                                .buffer()
                                .expect("chat_display object not found")
                                .set_text(buffer);
                            if let Ok(mut active_chat) = active_chat_clone.lock() {
                                active_chat.clear();
                                active_chat.push_str(&nick);
                            } else {
                                return gtk::Inhibit(true); //si falla al hacer el lock
                            }
                        }
                        None => {
                            chat_display_clone
                                .buffer()
                                .expect("chat_display object not found")
                                .set_text("");
                            if let Ok(mut active_chat) = active_chat_clone.lock() {
                                active_chat.clear();
                                active_chat.push_str(&nick);
                            } else {
                                return gtk::Inhibit(true); //si falla al hacer el lock
                            }
                            conversations.insert(nick.to_string(), String::new());
                        }
                    }
                    chat_title_clone.set_text(&nick);
                }
                Inhibit(true)
            });
            item.show();
            menu.append(&item);
            if !conversations.contains_key(nick.as_str()) {
                conversations.insert(nick, String::new());
            }
            chats_users.set_submenu(Some(&menu));
        }
        // join_clone.show();
        gtk::Inhibit(false)
    });
    //CREAR CHANNEL
    let client_clone = client.clone();
    channels_create_channel.connect_button_press_event(move |_, _| {
        let entry_dialog_builder = Builder::from_string(entry_dialog_src);
        let dialog: Dialog = entry_dialog_builder
            .object("entry_dialog")
            .expect("Problems opening entry_dialog");
        let ok_button: Button = entry_dialog_builder
            .object("ok")
            .expect("ok object not found");
        let cancel_button: Button = entry_dialog_builder
            .object("cancel")
            .expect("cancel object not found");
        let entry: Entry = entry_dialog_builder
            .object("entry")
            .expect("entry object not found");
        dialog.show_all();
        let dialog_clone = dialog.clone();
        cancel_button.connect_clicked(move |_| {
            dialog_clone.close();
            // entry_clone.buffer().set_text("");
        });
        let dialog_clone = dialog;
        let entry_clone = entry;
        let client_clone = client_clone.clone();
        ok_button.connect_clicked(move |_| {
            let channel = entry_clone.buffer().text();
            client_clone
                .lock()
                .expect("Couldn't lock client")
                .join(channel);
            dialog_clone.close();
        });
        gtk::Inhibit(true)
    });

    //SET INVITE ONLY
    let client_clone = client.clone();
    let active_chat_clone = active_chat.clone();
    modes_set_invite.connect_button_press_event(move |_, _| {
        let client_clone = client_clone.clone();
        let active_chat_clone = active_chat_clone.clone();
        let channel = active_chat_clone
            .lock()
            .expect("Couldn't lock active chat")
            .to_string();
        client_clone
            .lock()
            .expect("Couldn't lock client")
            .set_invite_only(channel);
        gtk::Inhibit(true)
    });
    //UNSET INVITE ONLY
    let client_clone = client.clone();
    let active_chat_clone = active_chat.clone();
    modes_unset_invite.connect_button_press_event(move |_, _| {
        let client_clone = client_clone.clone();
        let active_chat_clone = active_chat_clone.clone();
        let channel = active_chat_clone
            .lock()
            .expect("Couldn't lock active chat")
            .to_string();
        client_clone
            .lock()
            .expect("Couldn't lock client")
            .unset_invite_only(channel);
        gtk::Inhibit(true)
    });
    //SET SECRET
    let client_clone = client.clone();
    let active_chat_clone = active_chat.clone();
    modes_set_secret.connect_button_press_event(move |_, _| {
        let client_clone = client_clone.clone();
        let active_chat_clone = active_chat_clone.clone();
        let channel = active_chat_clone
            .lock()
            .expect("Couldn't lock active chat")
            .to_string();
        client_clone
            .lock()
            .expect("Couldn't lock client")
            .set_secret(channel);
        gtk::Inhibit(true)
    });

    //UNSET SECRET
    let client_clone = client.clone();
    let active_chat_clone = active_chat.clone();
    modes_unset_secret.connect_button_press_event(move |_, _| {
        let client_clone = client_clone.clone();
        let active_chat_clone = active_chat_clone.clone();
        let channel = active_chat_clone
            .lock()
            .expect("Couldn't lock active chat")
            .to_string();
        client_clone
            .lock()
            .expect("Couldn't lock client")
            .unset_secret(channel);
        gtk::Inhibit(true)
    });

    //PART
    let client_clone = client.clone();
    let chat_display_clone = chat_display.clone();
    let active_chat_clone = active_chat.clone();
    let chat_title_clone = chat_title;
    let conversations_clone = conversations.clone();
    channels_leave.connect_button_press_event(move |_, _| {
        let client_clone = client_clone.clone();
        let active_chat_clone = active_chat_clone.clone();
        let channel = active_chat_clone
            .lock()
            .expect("Couldn't lock active chat")
            .to_string();
        client_clone
            .lock()
            .expect("Couldn't lock client")
            .part(channel.clone());
        if channel.starts_with('#') || channel.starts_with('&') {
            chat_title_clone.set_text("");
            active_chat_clone
                .lock()
                .expect("Couldn't lock active chat")
                .clear();
            chat_display_clone
                .buffer()
                .expect("Couldn't get chat display buffer")
                .set_text("");
            conversations_clone
                .lock()
                .expect("Couldn't lock conversations")
                .remove(&channel);
        }
        gtk::Inhibit(true)
    });

    //KICK USER
    let client_clone = client.clone();
    let active_chat_clone = active_chat.clone();
    channels_kick_user.connect_button_press_event(move |_, _| {
        let entry_dialog_builder = Builder::from_string(double_entry_dialog_src);
        let dialog: Window = entry_dialog_builder
            .object("window")
            .expect("Problems opening window");
        let ok_button: Button = entry_dialog_builder
            .object("ok")
            .expect("ok object not found");
        let cancel_button: Button = entry_dialog_builder
            .object("cancel")
            .expect("cancel object not found");
        let entry_1: Entry = entry_dialog_builder
            .object("entry_1")
            .expect("entry_1 object not found");
        let entry_2: Entry = entry_dialog_builder
            .object("entry_2")
            .expect("entry_2 object not found");
        dialog.show_all();
        let dialog_clone = dialog.clone();
        cancel_button.connect_clicked(move |_| {
            dialog_clone.close();
        });
        let dialog_clone = dialog;
        let entry_1_clone = entry_1;
        let entry_2_clone = entry_2;
        let client_clone = client_clone.clone();
        let active_chat_clone = active_chat_clone.clone();
        ok_button.connect_clicked(move |_| {
            let nick = entry_1_clone.buffer().text();
            let reason = entry_2_clone.buffer().text();
            let channel = active_chat_clone
                .lock()
                .expect("Couldn't lock active chat")
                .to_string();
            client_clone
                .lock()
                .expect("Couldn't lock client")
                .kick(channel, nick, reason);
            dialog_clone.close();
        });
        gtk::Inhibit(true)
    });
    //SET LIMIT
    let client_clone = client.clone();
    let active_chat_clone = active_chat.clone();
    modes_set_limit.connect_button_press_event(move |_, _| {
        let entry_dialog_builder = Builder::from_string(entry_dialog_src);
        let dialog: Dialog = entry_dialog_builder
            .object("entry_dialog")
            .expect("Problems opening entry_dialog");
        let ok_button: Button = entry_dialog_builder
            .object("ok")
            .expect("ok object not found");
        let cancel_button: Button = entry_dialog_builder
            .object("cancel")
            .expect("cancel object not found");
        let entry: Entry = entry_dialog_builder
            .object("entry")
            .expect("entry object not found");
        dialog.show_all();
        let dialog_clone = dialog.clone();
        cancel_button.connect_clicked(move |_| {
            dialog_clone.close();
        });
        let dialog_clone = dialog;
        let entry_clone = entry;
        let client_clone = client_clone.clone();
        let active_chat_clone = active_chat_clone.clone();
        ok_button.connect_clicked(move |_| {
            let limit: u32 = match entry_clone.buffer().text().parse() {
                Ok(x) => x,
                Err(_err) => {
                    dialog_clone.close();
                    return;
                }
            };

            let channel = active_chat_clone
                .lock()
                .expect("Couldn't lock active chat")
                .to_string();
            client_clone
                .lock()
                .expect("Couldn't lock client")
                .set_limit(channel, limit);
            dialog_clone.close();
        });
        gtk::Inhibit(true)
    });

    //GIVE CHANOP PRIVILEGES
    let client_clone = client.clone();
    let active_chat_clone = active_chat.clone();
    modes_give_mod.connect_button_press_event(move |_, _| {
        let entry_dialog_builder = Builder::from_string(entry_dialog_src);
        let dialog: Dialog = entry_dialog_builder
            .object("entry_dialog")
            .expect("Problems opening entry_dialog");
        let ok_button: Button = entry_dialog_builder
            .object("ok")
            .expect("ok object not found");
        let cancel_button: Button = entry_dialog_builder
            .object("cancel")
            .expect("cancel object not found");
        let entry: Entry = entry_dialog_builder
            .object("entry")
            .expect("entry object not found");
        dialog.show_all();
        let dialog_clone = dialog.clone();
        cancel_button.connect_clicked(move |_| {
            dialog_clone.close();
        });
        let dialog_clone = dialog;
        let entry_clone = entry;
        let client_clone = client_clone.clone();
        let active_chat_clone = active_chat_clone.clone();
        ok_button.connect_clicked(move |_| {
            let nick = entry_clone.buffer().text();
            let channel = active_chat_clone
                .lock()
                .expect("Couldn't lock active chat")
                .to_string();
            client_clone
                .lock()
                .expect("Couldn't lock client")
                .make_oper(channel, nick);
            dialog_clone.close();
        });
        gtk::Inhibit(true)
    });

    //INVITE
    let client_clone = client.clone();
    let active_chat_clone = active_chat.clone();
    channels_invite.connect_button_press_event(move |_, _| {
        let entry_dialog_builder = Builder::from_string(entry_dialog_src);
        let dialog: Dialog = entry_dialog_builder
            .object("entry_dialog")
            .expect("Problems opening entry_dialog");
        let ok_button: Button = entry_dialog_builder
            .object("ok")
            .expect("ok object not found");
        let cancel_button: Button = entry_dialog_builder
            .object("cancel")
            .expect("cancel object not found");
        let entry: Entry = entry_dialog_builder
            .object("entry")
            .expect("entry object not found");
        dialog.show_all();
        let dialog_clone = dialog.clone();
        cancel_button.connect_clicked(move |_| {
            dialog_clone.close();
        });
        let dialog_clone = dialog;
        let entry_clone = entry;
        let client_clone = client_clone.clone();
        let active_chat_clone = active_chat_clone.clone();
        ok_button.connect_clicked(move |_| {
            let nick = entry_clone.buffer().text();
            let channel = active_chat_clone
                .lock()
                .expect("Couldn't lock active chat")
                .to_string();
            client_clone
                .lock()
                .expect("Couldn't lock client")
                .invite(channel, nick);
            dialog_clone.close();
        });
        gtk::Inhibit(true)
    });

    /// given a client, a label and a hashmap of conversations
    /// it will receive messages from the server and update the label and the hashmap
    fn receive(
        client: &Arc<Mutex<ClientC>>,
        label: &Label,
        nick_conversations: &Arc<Mutex<HashMap<String, String>>>,
    ) {
        let res = client.lock().expect("Couldn't lock").read_message();
        match res {
            Msg(from, to, message) => {
                let mut nick_conversations_clone =
                    nick_conversations.lock().expect("Couldn't lock");
                let mut conversation = from.clone();
                if to.starts_with('#') || to.starts_with('&') {
                    //if it's a channel
                    conversation = to; //the conversation is the channel
                };

                match nick_conversations_clone.get_mut(&conversation) {
                    Some(chat) => {
                        chat.push_str(format!("{}: {}\n", from, &message).as_str());
                        label.set_text(
                            format!("New messages from: {}", conversation.as_str()).as_str(),
                        );
                    }
                    None => {
                        nick_conversations_clone
                            .insert(conversation.clone(), format!("{}: {}\n", from, &message));
                        label.set_text(
                            format!("New messages from: {}", conversation.as_str()).as_str(),
                        );
                    }
                };
            }
            IrcRpl(_code, message) => {
                label.set_text(message.as_str());
                thread::sleep(Duration::from_millis(10));
            }
            IrcErr(_code, message) => {
                label.set_text(message.as_str());
                thread::sleep(Duration::from_millis(10));
            }
            Unknown(message) => {
                if !message.is_empty() {
                    if let Some((_, right)) = message.split_once("KICK") {
                        if let Some((channel, rest)) = right.trim().split_once(' ') {
                            if let Some((_, reason)) = rest.trim().split_once(' ') {
                                label.set_text(
                                    format!("kicked from {}: {}", channel, reason).as_str(),
                                );
                            } else {
                                label
                                    .set_text(format!("you were kicked from {}", channel).as_str());
                            }
                        }
                    } else if let Some((_, channel)) = message.split_once("INVITED TO") {
                        label.set_text(format!("you were invited to {}", channel).as_str());
                    } else {
                        label.set_text(message.as_str());
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        };
        thread::sleep(Duration::from_millis(10));
    }

    let rpl_label: Label = builder //rpl = reply
        .object("RPL_label")
        .expect("RPL_label object not found");

    let active_chat_idle = active_chat;
    let chat_display_idle = chat_display;
    let client_idle = client;
    let conversations_idle = conversations;
    glib::idle_add_local(move || {
        refresh_chat(&active_chat_idle, &chat_display_idle, &conversations_idle);
        receive(&client_idle, &rpl_label, &conversations_idle);
        glib::Continue(true)
    });

    Ok(())
}
