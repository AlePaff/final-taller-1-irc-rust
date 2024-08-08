mod client_status;
pub mod message;

use super::logger::Logger;
use super::ClientsInfo;
use crate::app_errors;
use client_status::ClientStatus;
use message::command::{Command, Mode};
use message::Message;
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
/// Struct representing a conection to the server.
/// Each ClientS lives in it's own server thread.
pub struct ClientS {
    pass: Option<String>,
    nick: Option<String>,
    pub user: Option<String>,
    pub realname: Option<String>,
    stream: Option<Arc<Mutex<TcpStream>>>,
    clients: Arc<Mutex<ClientsInfo>>,
    status: ClientStatus,
    server_name: Option<String>, //if Some, is a server.
    last_nick: String,
    last_hopcount: i32,
    trusted_servers: HashMap<String, Option<String>>,
    logger: Arc<Mutex<Logger>>,
}

impl ClientS {
    /// Given the ClientsInfo lock, the new stream, the list of trusted servers and the logger lock creates a new ClientS entity
    pub fn new(
        clients: Arc<Mutex<ClientsInfo>>,
        stream: Arc<Mutex<TcpStream>>,
        trusted_servers: HashMap<String, Option<String>>,
        logger: Arc<Mutex<Logger>>,
    ) -> Result<ClientS, Box<dyn Error>> {
        match stream.lock() {
            Ok(stream) => {
                stream.set_read_timeout(Some(Duration::from_nanos(1)))?;
            }
            Err(_) => {
                return Err(Box::new(app_errors::ApplicationError(
                    "Error while locking the stream".into(),
                )))
            }
        }

        Ok(ClientS {
            pass: None,
            nick: None,
            user: None,
            realname: None,
            stream: Some(stream),
            clients,
            status: ClientStatus::Unregistered,
            server_name: None,
            last_nick: String::new(),
            last_hopcount: 0,
            trusted_servers,
            logger,
        })
    }

    /// Main loop of the client on the server.
    /// Reads the message from stream, builds it and executes the commands
    pub fn run(&mut self) -> std::io::Result<()> {
        while let Ok(line) = self.read_from_stream() {
            let message = Message::build(line).expect("Error reading from stream");
            if let Command::Invalid(_) = message.command {
                continue;
            } else {
                self.logger
                    .lock()
                    .expect("Error: log lock poisoned")
                    .write(message.clone().to_string());
            }
            if self.run_command(message).is_err() {}
        }
        Ok(())
    }

    /// Auxiliary function for reading from stream and handling error that may occur
    fn read_from_stream(&mut self) -> Result<String, Box<dyn Error>> {
        let mut line = String::new();
        let mut char = [b'\n'];

        if let Some(stream) = self.stream.clone() {
            while char[0] == b'\n' {
                thread::sleep(Duration::from_nanos(1));
                let stream = stream.lock();
                if stream.is_err() {
                    return Err(Box::new(app_errors::ApplicationError(
                        "Connection closed".into(),
                    )));
                }
                if stream
                    .expect("Error finding stream")
                    .read_exact(&mut char)
                    .is_err()
                {
                    continue;
                }
            }
            let mut stream = match stream.lock() {
                Ok(stream) => stream,
                Err(_) => {
                    return Err(Box::new(app_errors::ApplicationError(
                        "locking stream".into(),
                    )))
                }
            };
            while char[0] != b'\n' {
                line.push_str(&String::from_utf8(Vec::from(char))?);
                char = [b'\n'];
                if stream.read_exact(&mut char).is_err() {
                    continue;
                }
            }
            return Ok(line);
        }
        Err(Box::new(app_errors::ApplicationError(
            "Connection closed".into(),
        )))
    }

    /// Function responsible of executing the correct function given a message object
    fn run_command(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
        match message.command {
            Command::Pass(pass) => self.execute_pass(pass),
            Command::Nick(nick, hopcount) => self.execute_nick(nick, hopcount),
            Command::User(username, realname) => self.execute_user(username, realname),
            Command::Privmsg(receiver, msg) => self.execute_privmsg(receiver, msg, message.prefix),
            Command::Notice(receiver, msg) => self.execute_notice(receiver, msg),
            Command::Quit(msg) => self.execute_quit(message.prefix, msg),
            Command::Oper(user, password) => self.execute_oper(user, password),
            Command::Invalid(err) => self.execute_invalid(err),
            Command::Join(channels, keys) => self.execute_join(channels, keys, message.prefix),
            Command::Names(channels) => self.execute_names(channels),
            Command::Part(channels) => self.execute_part(channels, message.prefix),
            Command::Kick(channel, user, comment) => {
                self.execute_kick(channel, user, comment, message.prefix)
            }
            Command::List(channels) => self.execute_list(channels),
            Command::Invite(invited_nick, channels) => {
                self.execute_invite(channels, invited_nick, message.prefix)
            }
            Command::Topic(channel, new_topic) => self.execute_topic(channel, new_topic),
            Command::Who(mask, _) => self.execute_who(mask),
            Command::Whois(mask) => self.execute_whois(mask),
            Command::Server(name, hopcount, info) => self.execute_server(name, hopcount, info),
            Command::Squit(server_name, comment) => self.execute_squit(server_name, comment),
            Command::Away(msg) => self.execute_away(msg),
            Command::Mode(channel_name, mode, params) => {
                self.execute_mode(channel_name, mode, params, message.prefix)
            }
        }
    }

    /// Given a pass will check with the server pass.
    /// Will result in error if already registered
    fn execute_pass(&mut self, pass: String) -> Result<(), Box<dyn Error>> {
        if self.is_registered() {
            return self.return_code((app_errors::ERR_ALREADYREGISTRED, vec![]));
        }
        self.pass = Some(pass);
        Ok(())
    }

    /// Given a nick will try to set it on the server
    /// Hopcount should be used only in case ClientS is a server, otherwise is always a 0.
    /// Will check if user is already registered and if the same nick is already in use
    /// and will return the appropiate response.
    /// If USER was already executed correctly will register the user
    fn execute_nick(&mut self, new_nick: String, hopcount: i32) -> Result<(), Box<dyn Error>> {
        if self.server_name.is_some() {
            self.last_nick = new_nick.clone();
            self.last_hopcount = hopcount;
            self.nick = Some(new_nick);
            return Ok(());
        }
        if self.is_registered() {
            if let Some(nick) = self.nick.clone() {
                self.clients
                    .lock()
                    .expect("Error: poisoned clients lock during execute nick")
                    .quit_client(nick, None, self.server_name.clone())?;
            }
            self.status = ClientStatus::Unregistered;
        }
        if let Ok(mut client_guard) = self.clients.lock() {
            if client_guard.contains_client(&new_nick) {
                return self.return_code((app_errors::ERR_NICKCOLLISION, vec![new_nick]));
            }
        }
        if let Some(_user) = self.user.clone() {
            let client_nick = new_nick.clone();
            if let Some(stream) = self.stream.clone() {
                let result = self
                    .clients
                    .lock()
                    .expect("Error: poisoned clients lock during execute nick")
                    .add_client(
                        client_nick,
                        self.clone(),
                        stream,
                        self.pass.clone(),
                        hopcount,
                        self.server_name.clone(),
                    );
                match result {
                    Ok(()) => {
                        self.nick = Some(new_nick);
                        self.status = ClientStatus::Registered;
                        self.return_code((app_errors::RPL_YOUAREIN, vec![]))?;
                    }
                    Err(error) => return self.return_code(error),
                }
            }
            return Ok(());
        }
        self.nick = Some(new_nick);
        Ok(())
    }

    /// Given a username and a realname will be set to current connection
    /// If PASS and NICK were already given will register the user.
    fn execute_user(&mut self, username: String, realname: String) -> Result<(), Box<dyn Error>> {
        let result;
        if self.server_name.is_some() {
            self.user = Some(username);
            self.realname = Some(realname);
            result = self
                .clients
                .lock()
                .expect("Error: poisoned clients lock during execute user")
                .add_client(
                    self.last_nick.clone(),
                    self.clone(),
                    self.stream.clone().ok_or("no stream for execute user")?,
                    self.pass.clone(),
                    self.last_hopcount,
                    self.server_name.clone(),
                );
            match result {
                Ok(()) => return Ok(()),
                Err(error) => return self.return_code(error),
            }
        }
        if self.is_registered() {
            return self.return_code((app_errors::ERR_ALREADYREGISTRED, vec![]));
        }
        self.user = Some(username);
        self.realname = Some(realname);

        if let Some(nick) = self.nick.clone() {
            if let Some(stream) = self.stream.clone() {
                result = self.clients.lock().expect("locking stream").add_client(
                    nick,
                    self.clone(),
                    stream,
                    self.pass.clone(),
                    0,
                    self.server_name.clone(),
                );
                match result {
                    Ok(()) => {
                        self.status = ClientStatus::Registered;
                        self.return_code((app_errors::RPL_YOUAREIN, vec![]))?;
                        return Ok(());
                    }
                    Err(error) => return self.return_code(error),
                }
            }
        }
        Ok(())
    }

    /// Given an nick will try to quit the user
    fn execute_quit(
        &mut self,
        prefix: Option<String>,
        message: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        if self.is_registered() {
            if let Some(nick) = self.nick.clone() {
                self.clients
                    .lock()
                    .expect("Error obtaining clients")
                    .quit_client(nick, message, self.server_name.clone())?
            };
        } else if self.server_name.is_some() {
            self.clients.lock().expect("locking stream").quit_client(
                prefix.ok_or("no prefix for quit")?,
                message,
                self.server_name.clone(),
            )?;
            return Ok(());
        }
        self.stream = None;
        Ok(())
    }

    /// Given a username, password combination will grant the current conection Oper abilities if
    /// combination is found in the operators log.
    fn execute_oper(&mut self, username: String, password: String) -> Result<(), Box<dyn Error>> {
        if !self.is_registered()
            || self.nick.clone().expect("Error obtaining oper nick") != username
        {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        match self
            .clients
            .lock()
            .expect("Error obtaining clients")
            .oper_login(&username, &password)
        {
            Ok(result) => {
                self.status = ClientStatus::Oper;
                self.return_code(result)
            }
            Err(error) => self.return_code(error),
        }
    }

    /// Given the nick of the receiver and the message tries to send that message
    /// If the current connection is a server, the message will be relayed.
    /// If the nick is a channel the message will be relayed to all members (minus the sender).
    /// If the nick is a user the message will be relayed.
    fn execute_privmsg(
        &mut self,
        receiver_name: String,
        msg: String,
        prefix: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() && self.server_name.is_none() {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        if self.server_name.is_some() {
            if self
                .clients
                .lock()
                .expect("Error obtaining clients")
                .send_privmsg(
                    prefix.ok_or("no prefix for privmsg")?,
                    receiver_name,
                    msg,
                    self.server_name.clone(),
                )
                .is_err()
            {}
            return Ok(());
        }
        if let Some(nick) = self.nick.clone() {
            if let Err(error) = self.clients.lock().expect("locking stream").send_privmsg(
                nick,
                receiver_name,
                msg,
                self.server_name.clone(),
            ) {
                return self.return_code(error);
            }
        }
        Ok(())
    }

    /// Given the nick and the message tries to send the NOTICE message.
    /// Uses the same internal logic of privmsg except notice doesn't returns errors.
    fn execute_notice(&mut self, receiver_name: String, msg: String) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() {
            return Ok(());
        }
        if let Some(nick) = self.nick.clone() {
            if self
                .clients
                .lock()
                .expect("Error locking obtaining clients")
                .send_privmsg(nick, receiver_name, msg, self.server_name.clone())
                .is_err()
            {
                return Ok(());
            }
        }
        Ok(())
    }

    /// Given a list of channels with an associated list of keys, tries to join the user to all the listed channels.
    fn execute_join(
        &mut self,
        channels: Vec<String>,
        channels_keys: Vec<Option<String>>,
        prefix: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        if self.server_name.is_some() {
            let nick = prefix.expect("Error obtaining prefix");
            for (i, channel) in channels.iter().enumerate() {
                if !channel.starts_with('#') && !channel.starts_with('&') {
                    continue;
                }
                let result = self
                    .clients
                    .lock()
                    .expect("error during locking")
                    .join_channel(
                        nick,
                        None,
                        channel.clone(),
                        channels_keys[i].clone(),
                        self.server_name.clone(),
                    );
                match result {
                    Ok(code) => return self.return_code(code),
                    Err(code) => return self.return_code(code),
                }
            }
        }
        if let Some(nick) = self.nick.clone() {
            //recorre la lista de canales que el usuario solicito unirse
            for (i, channel) in channels.iter().enumerate() {
                if !channel.starts_with('#') && !channel.starts_with('&') {
                    continue;
                }
                let result = self
                    .clients
                    .lock()
                    .expect("error during lock")
                    .join_channel(
                        nick,
                        self.stream.clone(),
                        channel.clone(),
                        channels_keys[i].clone(),
                        self.server_name.clone(),
                    );
                match result {
                    Ok(code) => return self.return_code(code),
                    Err(code) => return self.return_code(code),
                }
            }
        }
        Ok(())
    }

    /// Given a list of channels will return to the sender a list of all members for each channel
    fn execute_names(&mut self, channels: Vec<String>) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        self.clients
            .lock()
            .expect("Error obtaining clients during names")
            .names(channels, self.nick.clone().expect("Error executing names"));
        Ok(())
    }

    /// Given a list of channels will return to the sender a list of all channels and their topic
    fn execute_list(&mut self, channels: Vec<String>) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        self.clients
            .lock()
            .expect("Error obtaining clients during list")
            .list(channels, self.nick.clone().expect("Error executing list"));
        Ok(())
    }

    /// Given a user and a channel will try to invite the user to the channel
    fn execute_invite(
        &mut self,
        channel: String,
        invited_nick: String,
        prefix: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() && self.server_name.is_none() {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        match self.clients.lock().expect("error during lock").send_invite(
            channel,
            invited_nick,
            self.nick.clone(),
            prefix,
            self.server_name.clone(),
        ) {
            Ok(rpl) => self.return_code(rpl),
            Err(error) => self.return_code(error),
        }
    }

    /// Given channel will try to leave the channel
    fn execute_part(
        &mut self,
        channels: Vec<String>,
        prefix: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() && self.server_name.is_none() {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        match self
            .clients
            .lock()
            .expect("Error obtaining clients during part")
            .part(
                channels,
                self.nick.clone(),
                prefix,
                self.server_name.clone(),
            ) {
            Ok(code) => self.return_code(code),
            Err(error) => self.return_code(error),
        }
    }

    /// Given a user and a channel will try to kick the user off that channel.
    /// A explanation message must also be provided
    fn execute_kick(
        &mut self,
        channel: String,
        user: String,
        comment: Option<String>,
        prefix: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() && self.server_name.is_none() {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        match self
            .clients
            .lock()
            .expect("Error obtaining clients during part")
            .kick(
                channel,
                user,
                comment,
                self.nick.clone(),
                prefix,
                self.server_name.clone(),
            ) {
            Ok(code) => self.return_code(code),
            Err(error) => self.return_code(error),
        }
    }

    /// Given a channel and a new topic will try to change the channel topic to the new one
    fn execute_topic(
        &mut self,
        channel: String,
        new_topic: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        if let Some(stream) = self.stream.clone() {
            match self
                .clients
                .lock()
                .expect("Error obtaining clients during topic")
                .topic(
                    channel,
                    new_topic,
                    self.nick.clone().expect("Error executing topic"),
                ) {
                Ok(Some(mut topic)) => {
                    topic.push('\n');
                    let mut stream = match stream.lock() {
                        Ok(stream) => stream,
                        Err(_) => {
                            return Err(Box::new(app_errors::ApplicationError(
                                "locking stream".into(),
                            )))
                        }
                    };
                    stream.write_all(topic.as_bytes())?;
                }
                Ok(None) => return Ok(()),
                Err(err) => return self.return_code(err),
            };
        }
        Ok(())
    }

    /// Returns the corresponding error of the invalid command passed
    fn execute_invalid(
        &self,
        reply: ((i32, &'static str), Vec<String>),
    ) -> Result<(), Box<dyn Error>> {
        let ((number, text), params) = reply;
        let mut reply = text;
        let mut aux;
        for param in params {
            aux = reply.replacen("{}", param.as_str(), 1);
            reply = aux.as_str();
        }
        if self.server_name.is_some() {
        } else if let Some(stream) = self.stream.clone() {
            let mut stream = match stream.lock() {
                Ok(stream) => stream,
                Err(_) => {
                    return Err(Box::new(app_errors::ApplicationError(
                        "locking stream".into(),
                    )))
                }
            };
            stream.write_all(format!("{} {}\n", number, reply).as_bytes())?;
        }
        Ok(())
    }

    /// Auxiliary code that return the code with the explainatory message
    fn return_code(&self, reply: ((i32, &'static str), Vec<String>)) -> Result<(), Box<dyn Error>> {
        if let Some(stream) = self.stream.clone() {
            let mut stream = match stream.lock() {
                Ok(stream) => stream,
                Err(_) => {
                    return Err(Box::new(app_errors::ApplicationError(
                        "locking stream".into(),
                    )))
                }
            };
            let ((number, text), params) = reply;
            let mut reply = text;
            let mut aux;
            for param in params {
                aux = reply.replacen("{}", param.as_str(), 1);
                reply = aux.as_str();
            }
            stream.write_all(format!("{} {}\n", number, reply).as_bytes())?;
        }
        Ok(())
    }

    /// Given self will check if the current conection is registered
    fn is_registered(&self) -> bool {
        ClientStatus::Unregistered != self.status
    }

    /// Given a mask will try to execute the who command
    fn execute_who(&self, mask: String) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        self.clients
            .lock()
            .expect("Error obtaining clients")
            .who(mask, self.nick.clone().expect("Error executing who"))?;
        Ok(())
    }

    /// Given a mask will try to execute the who is command
    fn execute_whois(&self, mask: String) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        self.clients
            .lock()
            .expect("Error obtainging clients during whois")
            .whois(mask, self.nick.clone().expect("Error executing whois"))?;
        Ok(())
    }

    /// Given the new servername with their hopcount will try to register that server.
    /// If the connection is a server will also relay the command with an increase in hopcount
    fn execute_server(
        &mut self,
        name: String,
        hopcount: i32,
        info: String,
    ) -> Result<(), Box<dyn Error>> {
        if self.is_registered() {
            return self.return_code((app_errors::ERR_ALREADYREGISTRED, vec![]));
        }
        if let Some(server_name) = self.server_name.clone() {
            if server_name == name {
                return self.return_code((app_errors::ERR_SERVERCOLLISION, vec![name]));
            }
        }
        if hopcount == 1 {
            //registrando hijo nuevo
            if !self.trusted_servers.contains_key(&name) {
                return self.return_code((app_errors::ERR_UNTRUSTEDSERVER, vec![name]));
            }
            match self
                .clients
                .lock()
                .expect("error during lock")
                .try_add_server(
                    name.clone(),
                    self.pass.clone(),
                    self.stream.clone(),
                    hopcount,
                    info,
                    name.clone(),
                ) {
                Ok(_) => self.server_name = Some(name),
                Err(reply) => return self.return_code(reply),
            }
        } else {
            //registrando vecinos lejanos
            match self
                .clients
                .lock()
                .expect("error during lock")
                .try_add_server(
                    name,
                    self.pass.clone(),
                    None,
                    hopcount,
                    info,
                    self.server_name.clone().ok_or_else(|| {
                        app_errors::ApplicationError("server name not found".into())
                    })?,
                ) {
                Ok(_) => return Ok(()),
                Err(((code, text), _params)) => println!("{code} :{text}"), //TODO: AGREGAR PARAMETROS
            }
        }

        Ok(())
    }

    /// Given the server name will try to sever the connection to that server.
    /// A comment with the reason can be provided
    fn execute_squit(
        &mut self,
        server_name: String,
        comment: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        if self.is_registered() {
            //si soy un cliente
            let nick_copy = self.nick.clone().ok_or("no nick")?;
            if let Err(code) = self.clients.lock().expect("error during lock").squit(
                nick_copy,
                server_name,
                comment,
            ) {
                return self.return_code(code);
            }
        } else if let Some(my_name) = self.server_name.clone() {
            // si me avisa un server
            if let Err(code) = self.clients.lock().expect("error during lock").squit(
                self.server_name.clone().ok_or("no server name")?,
                server_name.clone(),
                comment,
            ) {
                return self.return_code(code);
            }
            if my_name == server_name {
                self.stream = None;
            }
        }
        Ok(())
    }

    /// Given the name and password of another server will try to connect the client to the parent server
    pub fn set_parent(&mut self, server_name: Option<String>, password: Option<String>) {
        self.pass = password;
        self.server_name = server_name.clone();
        if let Some(server_name) = server_name {
            let mut client_guard = match self.clients.lock() {
                Ok(guard) => guard,
                Err(_) => panic!("locking stream"),
            };
            if let Err(((code, text), _params)) = client_guard.try_add_server(
                server_name.clone(),
                self.pass.clone(),
                self.stream.clone(),
                1,
                "info".to_string(),
                server_name,
            ) {
                println!("{code} :{text}");
            }
        }
    }

    /// Given a message (optionally), will try to execute the away message.
    fn execute_away(&mut self, msg: Option<String>) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        let nick_copy = self.nick.clone().ok_or("no nick")?;
        self.clients
            .lock()
            .expect("error during lock")
            .away(nick_copy, msg.clone())?;
        match msg {
            Some(_) => self.return_code((app_errors::RPL_NOWAWAY, vec![])),
            None => self.return_code((app_errors::RPL_UNAWAY, vec![])),
        }
    }

    /// Auxiliary function for executing the oper command in the case of the oper mode.
    /// Must only be called from execute oper
    fn execute_mode_oper(
        &mut self,
        channel_name: String,
        mode: Mode,
        params: Option<String>,
        prefix: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        if let Mode::Deactivate(_) = mode {
            return self.return_code((app_errors::ERR_UNKNOWNMODE, vec!["-o".to_string()]));
        }
        if let Some(new_oper) = params {
            match self.clients.lock().expect("Couldn't lock").mode_oper(
                self.nick.clone(),
                channel_name,
                new_oper,
                prefix,
                self.server_name.clone(),
            ) {
                Ok(rpl) => return self.return_code(rpl),
                Err(err) => return self.return_code(err),
            }
        }
        self.return_code((app_errors::ERR_NEEDMOREPARAMS, vec!["MODE +o".to_string()]))
    }

    /// Given a nick and a mode object, the mode command will try to be executed.
    /// The posible mode options include o, l, s and i.
    fn execute_mode(
        &mut self,
        channel_name: String,
        mode: Mode,
        params: Option<String>,
        prefix: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        if !self.is_registered() && self.server_name.is_none() {
            return self.return_code((app_errors::ERR_NOLOGIN, vec![]));
        }
        match mode {
            Mode::Activate('o') | Mode::Deactivate('o') => {
                self.execute_mode_oper(channel_name, mode, params, prefix)
            }
            Mode::Activate('l') | Mode::Deactivate('l') => {
                match self.clients.lock().expect("Couldn't lock").mode_limit(
                    self.nick.clone(),
                    channel_name,
                    mode,
                    params,
                    prefix,
                    self.server_name.clone(),
                ) {
                    Ok(code) => self.return_code(code),
                    Err(code) => self.return_code(code),
                }
            }
            Mode::Activate('s') | Mode::Deactivate('s') => {
                match self.clients.lock().expect("Couldn't lock").mode_secret(
                    self.nick.clone(),
                    channel_name,
                    mode,
                    prefix,
                    self.server_name.clone(),
                ) {
                    Ok(code) => self.return_code(code),
                    Err(code) => self.return_code(code),
                }
            }
            Mode::Activate('i') | Mode::Deactivate('i') => {
                match self.clients.lock().expect("Couldn't lock").mode_invite(
                    self.nick.clone(),
                    channel_name,
                    mode,
                    prefix,
                    self.server_name.clone(),
                ) {
                    Ok(code) => self.return_code(code),
                    Err(code) => self.return_code(code),
                }
            }
            Mode::Activate(mode) | Mode::Deactivate(mode) => {
                self.return_code((app_errors::ERR_UNKNOWNCOMMAND, vec![mode.to_string()]))
            }
        }
    }
}

impl PartialEq for ClientS {
    fn eq(&self, other: &Self) -> bool {
        self.nick == other.nick && self.user == other.user
    }
}
