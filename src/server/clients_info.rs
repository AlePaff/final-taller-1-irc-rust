use super::channel::Channel;
use super::client_s::ClientS;
use crate::app_errors::{self, ApplicationError};
use crate::server::client_s::message::command::Mode;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ForeignServer(Option<Arc<Mutex<TcpStream>>>, i32, String, String); // stream, hopcount (distance), name, 1st_server_in_path
pub struct ForeignClient(Arc<Mutex<TcpStream>>, i32, Option<String>, Option<String>); // stream, hopcount, server_name, away_msg

pub struct ClientsInfo {
    server_name: String,
    users: HashMap<String, ClientS>,
    streams: HashMap<String, ForeignClient>,
    channels: HashMap<String, Channel>,
    server_operators: HashMap<String, String>,
    active_opers: HashSet<String>,
    server_password: Option<String>,
    servers: HashMap<String, ForeignServer>,
}

// new error codes (that are too long to be written in the code)
pub type DefaultAndError = Result<(), ((i32, &'static str), Vec<String>)>;
pub type ReplyAndError =
    Result<((i32, &'static str), Vec<String>), ((i32, &'static str), Vec<String>)>;
pub type StringAndError = Result<Option<String>, ((i32, &'static str), Vec<String>)>;

/// clients info is the responsible of storing all the information about the clients, channels and servers
/// and allow all the comunication between a server and its clients.
/// it delegates the comunication with the clients to the client_s module
impl ClientsInfo {
    pub fn new(
        server_name: String,
        server_password: Option<String>,
        server_operators: HashMap<String, String>,
    ) -> ClientsInfo {
        ClientsInfo {
            server_name,
            users: HashMap::new(),
            streams: HashMap::new(),
            channels: HashMap::new(),
            server_operators,
            active_opers: HashSet::new(),
            server_password,
            servers: HashMap::new(),
        }
    }

    pub fn send_privmsg(
        &mut self,
        from: String,
        to: String,
        msg: String,
        server_name: Option<String>,
    ) -> DefaultAndError {
        // para enviar mensajes broadcast en el server
        // Ej. enviar a todos los  $*.fi.uba el mensaje 'server en mantenimietno'
        if to.starts_with('&') | to.starts_with('#') {
            // if it is a channel
            if !self.channels.contains_key(&to) {
                return Err((app_errors::ERR_NOSUCHNICK, vec![to]));
            }
            let channel = self
                .channels
                .get_mut(&to)
                .expect("Error getting reciver message during privmsg");
            channel.send(from.clone(), msg.clone())?;

            //INFORMO A LOS VECINOS
            for (neighbour_name, server) in self.servers.iter() {
                let ForeignServer(stream, _hopcount, _info, _path) = server;
                if server_name.is_some()
                    && *neighbour_name == server_name.clone().expect("Error: server name is none")
                {
                    continue;
                }
                if let Some(stream) = stream.clone() {
                    stream
                        .lock()
                        .expect("Error locking stream during privmsg")
                        .write_all(format!(":{} PRIVMSG {} {}\n", from, to, msg).as_bytes())
                        .expect("Error writing to server");
                }
            }
            return Ok(());
        }
        // si no tiene destinatario
        if !self.users.contains_key(&to) {
            return Err((app_errors::ERR_NOSUCHNICK, vec![to]));
        }

        // 
        let ForeignClient(user, _hopcount, _server, away_msg) = match self.streams.get(&to) {
            Some(user) => user,
            None => return Err((app_errors::ERR_NOSUCHNICK, vec![to])),
        };

        Self::write_message(format!(":{} PRIVMSG {} {}\n", from, to, msg), user.clone());
        if let Some(away_msg) = away_msg {
            let ForeignClient(origin, _, _, _) = match self.streams.get(&from) {
                Some(origin) => origin,
                None => return Err((app_errors::ERR_NOSUCHNICK, vec![from])),
            };
            Self::write_message(
                format!(":{} PRIVMSG {} {}\n", to, from, away_msg),
                origin.clone(),
            );
        }
        Ok(())
    }

    pub fn contains_client(&mut self, nick: &String) -> bool {
        self.streams.contains_key(nick)
    }

    pub fn contains_channel(&mut self, name: &String) -> bool {
        self.channels.contains_key(name)
    }

    pub fn oper_login(&mut self, nick: &String, password: &String) -> ReplyAndError {
        if let Some(pass) = self.server_operators.get(nick) {
            if password == pass {
                self.active_opers.insert(nick.clone());
                return Ok((app_errors::RPL_YOUREOPER, vec![]));
            }
        }
        Err((app_errors::ERR_PASSWDMISMATCH, vec![]))
    }

    pub fn add_client(
        &mut self,
        nick: String,
        client: ClientS,
        stream: Arc<Mutex<TcpStream>>,
        password: Option<String>,
        hopcount: i32,
        server_name: Option<String>,
    ) -> DefaultAndError {
        if self.server_password.is_some() && password != self.server_password {
            return Err((app_errors::ERR_PASSWDMISMATCH, vec![]));
        };
        if self.contains_client(&nick) {
            return Err((app_errors::ERR_NICKCOLLISION, vec![nick]));
        }
        self.users.insert(nick.clone(), client.clone());
        self.streams.insert(
            nick.clone(),
            ForeignClient(stream, hopcount, server_name.clone(), None),
        );
        for (neighbour_name, foreign_server) in self.servers.iter_mut() {
            let ForeignServer(stream, hopcount, _info, _path) = foreign_server;

            if let Some(stream) = stream {
                if let Some(sender) = server_name.clone() {
                    if sender == *neighbour_name {
                        continue;
                    }
                }
                stream
                    .lock()
                    .expect("Error: server stream is none")
                    .write_all(
                        format!(
                            ":{} NICK {} {}\n",
                            self.server_name.clone(),
                            nick,
                            *hopcount + 1
                        )
                        .as_bytes(),
                    )
                    .expect("Error writing to server");
                stream
                    .lock()
                    .expect("Error: server stream is none")
                    .write_all(
                        format!(
                            ":{} USER {} {}\n",
                            nick,
                            client
                                .user
                                .clone()
                                .expect("Error: client user is none when adding client"),
                            client
                                .realname
                                .clone()
                                .expect("Error: client realname is none when adding client")
                        )
                        .as_bytes(),
                    )
                    .expect("Error writing to server");
            }
        }
        Ok(())
    }

    pub fn names(&mut self, channels: Vec<String>, from: String) {
        let ForeignClient(stream, _hopcount, _server, _away_msg) = self
            .streams
            .get_mut(&from)
            .expect("Error obtainging streams");
        if channels.is_empty() {
            //TODO: refactor
            for (channel_name, channel) in self.channels.iter_mut() {
                if channel.is_secret() && !channel.contains_client(&from) {
                    continue;
                }
                let names = channel.get_names();
                Self::write_message(
                    format!("{}: {};", channel_name, names.join(" ")),
                    stream.clone(),
                );
            }
            Self::write_message("\n".to_string(), stream.clone());
            return;
        }
        for channel_name in channels {
            match self.channels.get_mut(&channel_name) {
                Some(channel) => {
                    let names = channel.get_names();
                    Self::write_message(
                        format!("{}: {};", channel_name, names.join(" ")),
                        stream.clone(),
                    );
                }
                None => continue,
            }
            Self::write_message("\n".to_string(), stream.clone());
        }
    }

    pub fn list(&mut self, channels: Vec<String>, from: String) {
        let ForeignClient(stream, _hopcount, _server, _away_msg) = self
            .streams
            .get_mut(&from)
            .expect("Error obtainging streams");
        if channels.is_empty() {
            for (channel_name, channel) in self.channels.iter_mut() {
                if let Ok(Some(topic)) = channel.get_topic(from.clone()) {
                    Self::write_message(format!("{}: {}\n", channel_name, topic), stream.clone());
                }
            }
            return;
        }
        for channel_name in channels {
            match self.channels.get_mut(&channel_name) {
                Some(channel) => {
                    if let Ok(Some(topic)) = channel.get_topic(from.clone()) {
                        Self::write_message(
                            format!("{}: {}\n", channel_name, topic),
                            stream.clone(),
                        );
                    }
                }
                None => continue,
            }
        }
    }
    pub fn kick(
        &mut self,
        channel_name: String,
        kicked: String,
        comment: Option<String>,
        nick: Option<String>,
        prefix: Option<String>,
        sender: Option<String>,
    ) -> ReplyAndError {
        let unwrapped_prefix;
        if let Some(prefix) = prefix {
            unwrapped_prefix = prefix;
        } else {
            unwrapped_prefix = nick.clone().expect("No nick 'KICK'");
        }
        let mut unwrapped_comment = String::new();
        if let Some(comment) = comment {
            unwrapped_comment = comment;
        }
        match self.channels.get_mut(&channel_name) {
            Some(channel) => {
                if sender.is_none() {
                    if let Some(nick) = nick {
                        if !channel.is_oper(&nick) {
                            return Err((app_errors::ERR_NOCHANPRIVILEGES, vec![]));
                        }
                    }
                }
                if !channel.contains_client(&kicked) {
                    return Err((app_errors::ERR_NOSUCHNICK, vec![kicked]));
                }
                if let Some(ForeignClient(kicked_stream, kicked_hopcount, _, _)) =
                    self.streams.get_mut(&kicked)
                {
                    if *kicked_hopcount == 0 {
                        kicked_stream
                            .lock()
                            .expect("Problem writting to stream 'KICK'")
                            .write_all(
                                format!(
                                    ":{} KICK {} {} {}\n",
                                    unwrapped_prefix, channel_name, kicked, unwrapped_comment
                                )
                                .as_bytes(),
                            )
                            .expect("Problem writting to stream 'KICK'");
                    }
                }
                channel.remove_if_present(&kicked);
                if channel.is_empty() {
                    self.channels.remove(&channel_name);
                }
                if let Err(code) = self.notify_servers(
                    format!(
                        ":{} KICK {} {} {}\n",
                        unwrapped_prefix, channel_name, kicked, unwrapped_comment
                    ),
                    sender,
                ) {
                    return Err(code);
                }
            }
            None => {
                return Err((app_errors::ERR_NOSUCHCHANNEL, vec![channel_name]));
            }
        }
        Ok((app_errors::RPL_SUCCESS, vec![]))
    }

    pub fn send_invite(
        &mut self,
        channel_name: String,
        invited: String,
        nick: Option<String>,
        prefix: Option<String>,
        sender: Option<String>,
    ) -> ReplyAndError {
        if !self.users.contains_key(&invited) {
            return Err((app_errors::ERR_NOSUCHNICK, vec![invited]));
        }
        let unwrapped_prefix;
        if let Some(prefix) = prefix {
            unwrapped_prefix = prefix;
        } else {
            unwrapped_prefix = nick.clone().expect("No nick 'KICK'");
        }
        match self.channels.get_mut(&channel_name) {
            Some(channel) => {
                if sender.is_none() {
                    if let Some(nick) = nick {
                        if !channel.is_oper(&nick) {
                            return Err((app_errors::ERR_NOCHANPRIVILEGES, vec![]));
                        }
                    }
                }
                match channel.invite(&invited) {
                    Ok(code) => {
                        if let Some(ForeignClient(invited_stream, invited_hopcount, _, _)) =
                            self.streams.get_mut(&invited)
                        {
                            if *invited_hopcount == 0 {
                                invited_stream
                                    .lock()
                                    .expect("Problem writting to stream 'KICK'")
                                    .write_all(
                                        format!(
                                            ":{} INVITED TO {} \n",
                                            unwrapped_prefix, channel_name
                                        )
                                        .as_bytes(),
                                    )
                                    .expect("Problem writting to stream 'KICK'");
                            }
                        }
                        match self.notify_servers(
                            format!(
                                ":{} INVITE {} {}\n",
                                unwrapped_prefix, invited, channel_name
                            ),
                            sender,
                        ) {
                            Ok(_) => Ok(code),
                            Err(code) => Err(code),
                        }
                    }
                    Err(code) => Err(code),
                }
            }
            None => Err((app_errors::ERR_NOSUCHCHANNEL, vec![channel_name])),
        }
    }

    pub fn topic(
        &mut self,
        channel_name: String,
        new_topic: Option<String>,
        from: String,
    ) -> StringAndError {
        if let Some(channel) = self.channels.get_mut(&channel_name) {
            match new_topic {
                Some(new_topic) => channel.set_topic(new_topic, from),
                None => channel.get_topic(from),
            }
        } else {
            Err((app_errors::ERR_NOSUCHCHANNEL, vec![channel_name]))
        }
    }
    pub fn quit_client(
        &mut self,
        nick: String,
        message: Option<String>,
        issuer: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        if !self.streams.contains_key(&nick) {
            return Err(Box::new(ApplicationError("Client not found.".into())));
        }
        self.streams.remove_entry(&nick);
        self.users.remove_entry(&nick);
        println!("Quitting client {}", nick);
        let mut empty_chans = Vec::new();
        for (channel_name, channel) in self.channels.iter_mut() {
            channel.remove_if_present(&nick);
            if channel.is_empty() {
                empty_chans.push(channel_name.clone());
            }
        }
        for channel_name in empty_chans {
            self.channels.remove(&channel_name);
        }
        let mut msg = String::new();
        if let Some(message) = message {
            msg = message;
        }
        for (_neighbour_name, foreign_server) in self.servers.iter_mut() {
            let ForeignServer(neighbour_stream, _neighbour_hopcount, _neighbour_info, path) =
                foreign_server;
            if let Some(neighbour_stream) = neighbour_stream {
                if issuer.is_some()
                    && *path
                        == issuer
                            .clone()
                            .expect("Error: Issuer is none when quiting client")
                {
                    continue;
                }
                neighbour_stream
                    .lock()
                    .expect("Error: poisoed lock when writing to neighbour servers")
                    .write_all(format!(":{} QUIT {}\n", nick, msg).as_bytes())
                    .expect("Error writing to server");
            }
        }
        //TODO: remover al cliente de todos los channels
        Ok(())
    }

    pub fn join_channel(
        &mut self,
        user_nick: String,
        user_stream: Option<Arc<Mutex<TcpStream>>>,
        channel_name: String,
        key: Option<String>,
        server_name: Option<String>,
    ) -> ReplyAndError {
        if !self.contains_channel(&channel_name) {
            self.channels.insert(
                channel_name.clone(),
                Channel::new(channel_name.clone(), key, user_nick.clone(), user_stream),
            );
            // return Ok((
            //     app_errors::RPL_TOPIC,
            //     vec![channel_name.clone(), "".to_string()],
            // ));
        } else {
            let channel = self
                .channels
                .get_mut(&channel_name)
                .expect("Error retrieving channel during join channel");
            channel.add_client(user_nick.clone(), user_stream, key)?;
        }
        let channel = self
            .channels
            .get_mut(&channel_name)
            .expect("Error retrieving channel during join channel");
        //INFORMO A LOS VECINOS
        for (neighbour_name, server) in self.servers.iter() {
            let ForeignServer(stream, _hopcount, _info, _path) = server;
            if server_name.is_some()
                && *neighbour_name
                    == server_name
                        .clone()
                        .expect("Error: server name is none during join channel")
            {
                continue;
            }
            if let Some(stream) = stream.clone() {
                stream
                    .lock()
                    .expect("Error: stream is none during join channel")
                    .write_all(format!(":{} JOIN {}\n", user_nick, channel_name).as_bytes())
                    .expect("Error writing to server");
            }
        }
        Ok((
            app_errors::RPL_TOPIC,
            vec![
                channel_name,
                match channel
                    .get_topic(user_nick)
                    .expect("Error obtaining channel topic")
                {
                    Some(x) => x,
                    None => "".to_string(),
                },
            ],
        ))
    }

    pub fn part(
        &mut self,
        channels: Vec<String>,
        nick: Option<String>,
        prefix: Option<String>,
        sender: Option<String>,
    ) -> ReplyAndError {
        let to_remove = if sender.is_some() {
            prefix.ok_or((app_errors::ERR_NEEDMOREPARAMS, vec!["PART".to_string()]))?
        } else {
            nick.ok_or((app_errors::ERR_NEEDMOREPARAMS, vec!["PART".to_string()]))?
        };
        for channel_name in channels.iter() {
            if !self.channels.contains_key(channel_name) {
                return Err((app_errors::ERR_NOSUCHCHANNEL, vec![channel_name.clone()]));
            }
        }
        for channel_name in channels {
            match self.channels.get_mut(&channel_name) {
                Some(channel) => {
                    channel.remove_if_present(&to_remove);
                    if channel.is_empty() {
                        self.channels.remove(&channel_name);
                    }
                    if let Err(code) = self.notify_servers(
                        format!(":{} PART {}\n", to_remove, channel_name),
                        sender.clone(),
                    ) {
                        return Err(code);
                    }
                }
                None => {
                    return Err((app_errors::ERR_NOSUCHCHANNEL, vec![channel_name]));
                }
            }
        }
        Ok((app_errors::RPL_SUCCESS, vec![]))
    }

    pub fn who(&mut self, mut mask: String, from: String) -> Result<(), Box<dyn Error>> {
        let ForeignClient(stream, _hopcount, _server, _away_msg) = self
            .streams
            .get_mut(&from)
            .expect("Error obtaining streams during who");
        let mut stream = match stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                return Err(Box::new(app_errors::ApplicationError(
                    "locking stream".into(),
                )))
            }
        };
        if mask.contains('*') {
            if mask.starts_with('*') {
                mask.remove(0);
                for (user, client) in self.users.iter() {
                    if user.ends_with(mask.as_str()) {
                        stream.write_all(
                            format!(
                                "{} {}: {};",
                                client.user.clone().ok_or("No user")?,
                                user,
                                client.realname.clone().ok_or("No realname")?
                            )
                            .as_bytes(),
                        )?;
                    }
                }
            } else if mask.ends_with('*') {
                mask.remove(mask.len() - 1);
                for (user, client) in self.users.iter() {
                    if user.starts_with(mask.as_str()) {
                        stream.write_all(
                            format!(
                                "{} {}: {};",
                                client.user.clone().ok_or("No user")?,
                                user,
                                client.realname.clone().ok_or("No realname")?
                            )
                            .as_bytes(),
                        )?;
                    }
                }
            }
        }
        for (user, client) in self.users.iter() {
            if user.as_str() == mask.as_str() {
                stream.write_all(
                    format!(
                        "{} {}: {};",
                        client.user.clone().ok_or("No user")?,
                        user,
                        client.realname.clone().ok_or("No realname")?
                    )
                    .as_bytes(),
                )?;
            }
        }
        stream.write_all("\n".to_string().as_bytes())?;

        Ok(())
    }

    pub fn whois(&mut self, mask: String, from: String) -> Result<(), Box<dyn Error>> {
        let ForeignClient(stream, _hopcount, _server, _away_msg) = self
            .streams
            .get_mut(&from)
            .expect("Error obtaining streams during whois");
        let mut stream = match stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                return Err(Box::new(app_errors::ApplicationError(
                    "locking stream".into(),
                )))
            }
        };

        if self.channels.contains_key(&mask) {
            stream.write_all(
                format!(
                    "{} : {}\n",
                    mask,
                    match self
                        .channels
                        .get_mut(&mask)
                        .expect("Error getting channel during whois")
                        .topic()
                    {
                        Some(x) => x,
                        None => "".to_string(),
                    }
                )
                .as_bytes(),
            )?;
            return Ok(());
        }
        if self.users.contains_key(&mask) {
            stream.write_all(
                format!(
                    "{} {}: {}\n",
                    self.users
                        .get_mut(&mask)
                        .expect("Error getting user during whois")
                        .user
                        .clone()
                        .expect("Error: user not found during whois"),
                    mask,
                    self.users
                        .get_mut(&mask)
                        .expect("Error getting user during whois")
                        .realname
                        .clone()
                        .expect("Error: realname not found during whois"),
                )
                .as_bytes(),
            )?;
            return Ok(());
        }
        stream.write_all("431 :No nickname given".as_bytes())?;
        Ok(())
    }

    // escribe mensajes en el stream (ej. cuando se envia PRIVMSG a otro usuario)
    fn write_message(msg: String, stream: Arc<Mutex<TcpStream>>) {
        let mut stream = match stream.lock() {
            Ok(stream) => stream,
            Err(_) => panic!("locking stream"), //mejor panic que return silencioso (por ahora)
        };
        if let Err(err) = stream.write(msg.as_bytes()) {
            eprintln!("Server error: {err}");
            std::process::exit(1);
        }
    }

    pub fn try_add_server(
        &mut self,
        name: String,
        pass: Option<String>,
        stream: Option<Arc<Mutex<TcpStream>>>,
        hopcount: i32,
        info: String,
        server_name: String,
    ) -> ReplyAndError {
        if self.server_password.is_some() && pass != self.server_password {
            return Err((app_errors::ERR_PASSWDMISMATCH, vec![]));
        }
        if self.servers.contains_key(&name) {
            return Err((app_errors::ERR_SERVERCOLLISION, vec![name]));
        }
        // if hopcount == 1 {
        if let Some(stream) = stream.clone() {
            {
                let mut stream = stream
                    .lock()
                    .expect("Error: server lock poisoned during try add server");
                // envía un mensaje al padre indicando, que se pudo conectar
                stream
                    .write_all("200 :Succesfully Connected\n".as_bytes())
                    .expect("Error writing to server");
                println!("Registrando nuevo server: {}", name);
                // broadcast comando SERVER a todos los servidores de la red, acerca del nuevo server
                for (neighbour_name, foreign_server) in self.servers.iter_mut() {
                    let ForeignServer(neighbour_stream, neighbour_hopcount, neighbour_info, _path) =
                        foreign_server;
                    stream
                        .write_all(
                            format!(
                                "SERVER {} {} {}\n",
                                neighbour_name,
                                (*neighbour_hopcount + 1),
                                neighbour_info
                            )
                            .as_bytes(),
                        )
                        .expect("Error writing to server");
                    if let Some(neighbour_stream) = neighbour_stream {
                        neighbour_stream
                            .lock()
                            .expect("Error: neighbour stream poisoned during try add server")
                            .write_all(
                                format!("SERVER {} {} {}\n", name, (hopcount + 1), info).as_bytes(),
                            )
                            .expect("Error writing to server");
                    }
                }
                // broadcast de NICK y USER a todos los servidores del usuario nuevo
                for (nick, ForeignClient(_stream, hopcount, _server, _away_msg)) in
                    self.streams.iter_mut()
                {
                    // NICK para indicar que tan lejos esta el usuario de su servidor
                    stream
                        .write_all(
                            format!(
                                ":{} NICK {} {}\n",
                                self.server_name.clone(),
                                nick,
                                *hopcount + 1
                            )
                            .as_bytes(),
                        )
                        .expect("Error writing to server");
                    let user = self
                        .users
                        .get(nick)
                        .expect("Error obtaining user during try add server");
                    // USER para indicar nuevo usuario en la red
                    stream
                        .write_all(
                            format!(
                                ":{} USER {} {}\n",
                                nick,
                                user.user
                                    .clone()
                                    .expect("Error: user's user is none during try add server"),
                                user.realname
                                    .clone()
                                    .expect("Error: user's realname is none during try add server")
                            )
                            .as_bytes(),
                        )
                        .expect("Error writing to server");
                }
                // broadcast de los canales, y operadores ?
                for (channel_name, channel) in self.channels.iter_mut() {
                    let operators = channel.get_operators();
                    let mut key = String::new();
                    if let Some(channel_key) = channel.get_key() {
                        key = channel_key;
                    }
                    for nick in channel.get_names() {
                        stream
                            .write_all(
                                format!(":{} JOIN {} {}\n", nick, channel_name, key).as_bytes(),
                            )
                            .expect("Error writing to server");
                    }
                    for operator in operators.iter() {
                        stream
                            .write_all(
                                format!(
                                    ":{} MODE {} +o {}\n",
                                    self.server_name.clone(),
                                    channel_name,
                                    operator
                                )
                                .as_bytes(),
                            )
                            .expect("Error writing to server");
                    }
                    if let Some(limit) = channel.get_limit() {
                        stream
                            .write_all(
                                format!(
                                    ":{} MODE {} +l {}\n",
                                    self.server_name.clone(),
                                    channel_name,
                                    limit
                                )
                                .as_bytes(),
                            )
                            .expect("Error writing to server");
                    }
                    if channel.is_secret() {
                        stream
                            .write_all(
                                format!(
                                    ":{} MODE {} +s\n",
                                    self.server_name.clone(),
                                    channel_name,
                                )
                                .as_bytes(),
                            )
                            .expect("Error writing to server");
                    }
                    if channel.is_invite_only() {
                        stream
                            .write_all(
                                format!(
                                    ":{} MODE {} +i\n",
                                    self.server_name.clone(),
                                    channel_name,
                                )
                                .as_bytes(),
                            )
                            .expect("Error writing to server");
                    }
                }
            }
        } else {
            for (
                neighbour_name,
                ForeignServer(neighbour_stream, _neighbour_hopcount, _neighbour_info, _path),
            ) in self.servers.iter_mut()
            {
                if let Some(neighbour_stream) = neighbour_stream {
                    if server_name == *neighbour_name {
                        continue;
                    }
                    neighbour_stream
                        .lock()
                        .expect("Error: neighbour lock poisoned during try add server")
                        .write_all(
                            format!("SERVER {} {} {}\n", name, (hopcount + 1), info).as_bytes(),
                        )
                        .expect("Error writing to server");
                }
            }
        }
        // agrega el nuevo server
        self.servers
            .insert(name, ForeignServer(stream, hopcount, info, server_name));
        Ok((app_errors::RPL_SUCCESS, vec![]))
    }

    pub fn squit(
        &mut self,
        issuer: String,
        server_name: String,
        comment: Option<String>,
    ) -> ReplyAndError {
        if !self.active_opers.contains(&issuer) && !self.servers.contains_key(&issuer) {
            return Err((app_errors::ERR_NOPRIVILEGES, vec![]));
        }
        if !self.servers.contains_key(&server_name) {
            return Err((app_errors::ERR_NOSUCHSERVER, vec![server_name]));
        }
        if self.active_opers.contains(&issuer) {
            if let Some(ForeignServer(
                neighbour_stream,
                _neighbour_hopcount,
                _neighbour_info,
                _path,
            )) = self.servers.clone().get(&server_name)
            {
                match neighbour_stream {
                    Some(stream) => {
                        let mut disjoint_clients = Vec::new();
                        for (
                            joint_nick,
                            ForeignClient(_joint_stream, _joint_hopcount, joint_server, _away_msg),
                        ) in self.streams.iter_mut()
                        {
                            if joint_server.is_none()
                                || (joint_server.is_some()
                                    && joint_server
                                        .clone()
                                        .expect("Error getting joint server during squit")
                                        != server_name)
                            {
                                stream
                                    .lock()
                                    .expect("Error getting stream during squit")
                                    .write_all(
                                        format!(":{} QUIT disconnected by server.\n", joint_nick)
                                            .as_bytes(),
                                    )
                                    .expect("Error writing to server");
                            } else {
                                disjoint_clients.push(joint_nick.clone());
                            }
                        }
                        let mut to_be_notified = Vec::new();
                        let mut joint_servers = Vec::new();
                        let mut disjoint_servers = Vec::new();
                        for (server, ForeignServer(stream, _hopcount, _info, path)) in
                            self.servers.iter()
                        {
                            if *path == server_name {
                                disjoint_servers.push(server.clone());
                            } else if let Some(stream) = stream {
                                to_be_notified.push(stream.clone());
                                joint_servers.push(server.clone());
                            } else {
                                joint_servers.push(server.clone());
                            }
                        }
                        let mut server_comment = String::new();
                        if let Some(comment) = comment {
                            server_comment = comment;
                        }
                        for server in joint_servers {
                            stream
                                .lock()
                                .expect("Error: poisoned lock during squit")
                                .write_all(
                                    format!(
                                        ":{} SQUIT {} {}\n",
                                        self.server_name, server, server_comment
                                    )
                                    .as_bytes(),
                                )
                                .expect("Error writing to server");
                        }
                        stream
                            .lock()
                            .expect("Error: poisoned lock during squit")
                            .write_all(
                                format!(
                                    ":{} SQUIT {} {}\n",
                                    self.server_name, self.server_name, server_comment
                                )
                                .as_bytes(),
                            )
                            .expect("Error writing to server");

                        for stream in to_be_notified {
                            let mut stream = stream
                                .lock()
                                .expect("Error: poisoned to be notified lock during squit");
                            for server in disjoint_servers.iter() {
                                stream
                                    .write_all(
                                        format!(":{} SQUIT {}\n", self.server_name, server)
                                            .as_bytes(),
                                    )
                                    .expect("Error writing to server");
                                self.servers.remove(server);
                            }

                            for nick in disjoint_clients.iter() {
                                stream
                                    .write_all(format!(":{} QUIT\n", nick).as_bytes())
                                    .expect("Error writing to server");
                            }
                        }
                        for nick in disjoint_clients.iter() {
                            self.users.remove_entry(nick);
                            self.streams.remove_entry(nick);
                            let mut empty_chans = Vec::new();
                            for (channel_name, channel) in self.channels.iter_mut() {
                                channel.remove_if_present(nick);
                                if channel.is_empty() {
                                    empty_chans.push(channel_name.clone());
                                }
                            }
                            for channel_name in empty_chans {
                                self.channels.remove(&channel_name);
                            }
                        }
                        for server in disjoint_servers.iter() {
                            self.servers.remove(server);
                        }
                    }
                    None => return Err((app_errors::ERR_NOSUCHSERVER, vec![server_name])),
                }
            }
        } else {
            for (
                neighbour_name,
                ForeignServer(neighbour_stream, _neighbour_hopcount, _neighbour_info, _path),
            ) in self.servers.iter_mut()
            {
                if let Some(neighbour_stream) = neighbour_stream {
                    if issuer == *neighbour_name {
                        continue;
                    }
                    let mut server_comment = String::new();
                    if let Some(comment) = comment.clone() {
                        server_comment = comment;
                    }
                    neighbour_stream
                        .lock()
                        .expect("Error: poisoned neighbour stream during squit")
                        .write_all(format!("SQUIT {} {}\n", server_name, server_comment).as_bytes())
                        .expect("Error writing to server");
                }
            }
            self.servers.remove(&server_name);
        }
        Ok((app_errors::RPL_SUCCESS, vec![]))
    }
    pub fn away(&mut self, nick: String, msg: Option<String>) -> Result<(), Box<dyn Error>> {
        let ForeignClient(stream, hopcount, server, _away_msg) = match self.streams.get(&nick) {
            Some(client) => client,
            None => {
                return Err(Box::new(app_errors::ApplicationError(
                    "nick not found".to_string(),
                )))
            }
        };
        self.streams.insert(
            nick,
            ForeignClient(stream.clone(), *hopcount, server.clone(), msg),
        );
        Ok(())
    }

    fn notify_servers(&mut self, msg: String, sender: Option<String>) -> DefaultAndError {
        for (neighbour_name, ForeignServer(stream, _hopcount, _info, _path)) in
            self.servers.iter_mut()
        {
            if let Some(stream) = stream {
                if let Some(sender) = sender.clone() {
                    if sender == *neighbour_name {
                        continue;
                    }
                }
                stream
                    .lock()
                    .expect("Error: poisoned neighbour stream during notify")
                    .write_all(msg.as_bytes())
                    .expect("Error writing to server");
            }
        }
        Ok(())
    }

    fn mode_oper_add(
        &mut self,
        channel_name: String,
        new_oper: String,
        prefix: Option<String>,
        sender: Option<String>,
    ) -> ReplyAndError {
        let mut unwrapped_prefix = self.server_name.clone();
        if let Some(prefix) = prefix {
            unwrapped_prefix = prefix;
        }
        match self
            .channels
            .get_mut(&channel_name)
            .unwrap()
            .add_oper(new_oper.clone())
        {
            Ok(()) => {
                match self.notify_servers(
                    format!(
                        ":{} MODE {} +o {}\n",
                        unwrapped_prefix, channel_name, new_oper
                    ),
                    sender,
                ) {
                    Ok(_) => Ok((app_errors::RPL_SUCCESS, vec![])),
                    Err(code) => Err(code),
                }
            }
            Err(err) => Err(err),
        }
    }
    fn set_limit(
        &mut self,
        channel_name: String,
        mode: Mode,
        new_limit: Option<String>,
        prefix: Option<String>,
        sender: Option<String>,
    ) -> ReplyAndError {
        let mut unwrapped_prefix = self.server_name.clone();
        if let Some(prefix) = prefix {
            unwrapped_prefix = prefix;
        }
        let mut unwrapped_limit = String::new();
        if let Some(limit) = new_limit.clone() {
            unwrapped_limit = limit;
        }
        match self
            .channels
            .get_mut(&channel_name)
            .unwrap()
            .set_limit(mode.clone(), new_limit)
        {
            Ok(()) => {
                match self.notify_servers(
                    format!(
                        ":{} MODE {} {} {}\n",
                        unwrapped_prefix,
                        channel_name,
                        mode.to_mode_string(),
                        unwrapped_limit
                    ),
                    sender,
                ) {
                    Ok(_) => Ok((app_errors::RPL_SUCCESS, vec![])),
                    Err(code) => Err(code),
                }
            }
            Err(err) => Err(err),
        }
    }
    fn set_mode_secret(
        &mut self,
        channel_name: String,
        mode: Mode,
        prefix: Option<String>,
        sender: Option<String>,
    ) -> ReplyAndError {
        let mut unwrapped_prefix = self.server_name.clone();
        if let Some(prefix) = prefix {
            unwrapped_prefix = prefix;
        }
        match self
            .channels
            .get_mut(&channel_name)
            .unwrap()
            .mode_secret(mode.clone())
        {
            Ok(()) => {
                match self.notify_servers(
                    format!(
                        ":{} MODE {} {}\n",
                        unwrapped_prefix,
                        channel_name,
                        mode.to_mode_string(),
                    ),
                    sender,
                ) {
                    Ok(_) => Ok((app_errors::RPL_SUCCESS, vec![])),
                    Err(code) => Err(code),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub fn mode_oper(
        &mut self,
        nick: Option<String>,
        channel_name: String,
        new_oper: String,
        prefix: Option<String>,
        sender: Option<String>,
    ) -> ReplyAndError {
        if let Some(channel) = self.channels.clone().get_mut(&channel_name) {
            if sender.is_some() {
                return self.mode_oper_add(channel_name, new_oper, prefix, sender);
            }
            if let Some(nick) = nick {
                if channel.is_oper(&nick) {
                    return self.mode_oper_add(channel_name, new_oper, prefix, sender);
                }
                return Err((app_errors::ERR_NOCHANPRIVILEGES, vec![]));
            }
        }
        Err((app_errors::ERR_NOSUCHCHANNEL, vec![channel_name]))
    }
    pub fn mode_limit(
        &mut self,
        nick: Option<String>,
        channel_name: String,
        mode: Mode,
        new_limit: Option<String>,
        prefix: Option<String>,
        sender: Option<String>,
    ) -> ReplyAndError {
        if let Some(channel) = self.channels.clone().get_mut(&channel_name) {
            if sender.is_some() {
                return self.set_limit(channel_name, mode, new_limit, prefix, sender);
            }
            if let Some(nick) = nick {
                if channel.is_oper(&nick) {
                    return self.set_limit(channel_name, mode, new_limit, prefix, sender);
                }
                return Err((app_errors::ERR_NOCHANPRIVILEGES, vec![]));
            }
        }
        Err((app_errors::ERR_NOSUCHCHANNEL, vec![channel_name]))
    }
    pub fn mode_secret(
        &mut self,
        nick: Option<String>,
        channel_name: String,
        mode: Mode,
        prefix: Option<String>,
        sender: Option<String>,
    ) -> ReplyAndError {
        if let Some(channel) = self.channels.clone().get_mut(&channel_name) {
            if sender.is_some() {
                return self.set_mode_secret(channel_name, mode, prefix, sender);
            }
            if let Some(nick) = nick {
                if channel.is_oper(&nick) {
                    return self.set_mode_secret(channel_name, mode, prefix, sender);
                }
                return Err((app_errors::ERR_NOCHANPRIVILEGES, vec![]));
            }
        }
        Err((app_errors::ERR_NOSUCHCHANNEL, vec![channel_name]))
    }
    fn set_mode_invite(
        &mut self,
        channel_name: String,
        mode: Mode,
        prefix: Option<String>,
        sender: Option<String>,
    ) -> ReplyAndError {
        let mut unwrapped_prefix = self.server_name.clone();
        if let Some(prefix) = prefix {
            unwrapped_prefix = prefix;
        }
        match self
            .channels
            .get_mut(&channel_name)
            .unwrap()
            .mode_invite(mode.clone())
        {
            Ok(()) => {
                match self.notify_servers(
                    format!(
                        ":{} MODE {} {}\n",
                        unwrapped_prefix,
                        channel_name,
                        mode.to_mode_string(),
                    ),
                    sender,
                ) {
                    Ok(_) => Ok((app_errors::RPL_SUCCESS, vec![])),
                    Err(code) => Err(code),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub fn mode_invite(
        &mut self,
        nick: Option<String>,
        channel_name: String,
        mode: Mode,
        prefix: Option<String>,
        sender: Option<String>,
    ) -> ReplyAndError {
        if let Some(channel) = self.channels.clone().get_mut(&channel_name) {
            if sender.is_some() {
                return self.set_mode_invite(channel_name, mode, prefix, sender);
            }
            if let Some(nick) = nick {
                if channel.is_oper(&nick) {
                    return self.set_mode_invite(channel_name, mode, prefix, sender);
                }
                return Err((app_errors::ERR_NOCHANPRIVILEGES, vec![]));
            }
        }
        Err((app_errors::ERR_NOSUCHCHANNEL, vec![channel_name]))
    }
}
