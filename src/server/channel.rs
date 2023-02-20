use crate::app_errors;
use crate::server::client_s::message::command::Mode;
use crate::server::clients_info::*;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub type DefaultAndError = Result<(), ((i32, &'static str), Vec<String>)>;

#[derive(Clone)]
/// Struct representing a server channel .
pub struct Channel {
    name: String,
    topic: Option<String>,
    users: HashMap<String, Option<Arc<Mutex<TcpStream>>>>,
    //banned_users: HashSet<String>,
    operators: HashSet<String>,
    key: Option<String>,
    invites: HashSet<String>,
    invited_only: bool,
    limit: Option<usize>,
    secret: bool,
}

impl Channel {
    /// Given the channel name, it's key, the nick of the creator and the stream of the creator returns the corresponding
    /// channel object.
    pub fn new(
        name: String,
        key: Option<String>,
        creator_nick: String,
        creator_stream: Option<Arc<Mutex<TcpStream>>>,
    ) -> Channel {
        let mut operators = HashSet::new();
        let mut users = HashMap::new();
        users.insert(creator_nick.clone(), creator_stream);
        operators.insert(creator_nick);
        let topic = None;
        let invites = HashSet::new();
        let invited_only = false;

        Channel {
            name,
            topic,
            users,
            operators,
            key,
            invites,
            limit: None,
            invited_only,
            secret: false,
        }
    }

    /// Given self returns a copy of the channel key
    pub fn get_key(&mut self) -> Option<String> {
        self.key.clone()
    }

    /// Given self returns a copy of the operators of the channel
    pub fn get_operators(&mut self) -> HashSet<String> {
        self.operators.clone()
    }

    /// Given self returns a vector with a copy of each member nick
    pub fn get_names(&mut self) -> Vec<String> {
        let mut names = Vec::new();
        for name in self.users.keys() {
            names.push(name.clone());
        }
        names
    }

    /// Given self and a nick tries to remove if possible.
    /// Will also asign a new channel operator if last is removed.
    /// The new operator will be assigned arbitrarily.
    pub fn remove_if_present(&mut self, nick: &String) {
        self.users.remove(nick);
        self.operators.remove(nick);
        if self.operators.is_empty() {
            if let Some((user, _stream)) = self.users.iter().next() {
                self.operators.insert(user.clone());
            }
        }
    }

    /// Given the nick of the inited user, marks that user as invited.
    pub fn invite(&mut self, invited: &String) -> ReplyAndError {
        if self.users.contains_key(invited) {
            return Err((
                app_errors::ERR_USERONCHANNEL,
                vec![invited.clone(), self.name.clone()],
            ));
        }
        self.invites.insert(invited.clone());
        Ok((
            app_errors::RPL_INVITING,
            vec![self.name.clone(), invited.clone()],
        ))
    }

    /// Given self returns a copy of the topic.
    pub fn get_topic(&mut self, from: String) -> StringAndError {
        if !self.users.contains_key(&from) {
            return Err((app_errors::ERR_NOTONCHANNEL, vec![self.name.clone()]));
        }
        Ok(self.topic.clone())
    }

    /// Given self, the new topic and the nick of the user modifies the current topic.
    /// Only members can modify the topic
    pub fn set_topic(&mut self, new_topic: String, from: String) -> StringAndError {
        if !self.users.contains_key(&from) {
            return Err((app_errors::ERR_NOTONCHANNEL, vec![self.name.clone()]));
        }
        self.topic = Some(new_topic);
        Ok(None)
    }

    /// Given self, the sender's nick and the message sends the message to all memeber users.
    /// The sender's nick must be provided to avoid them messaging themselves
    pub fn send(
        &mut self,
        from: String,
        message: String,
    ) -> Result<(), ((i32, &'static str), Vec<String>)> {
        if !self.users.contains_key(&from) {
            return Err((app_errors::ERR_NOTONCHANNEL, vec![self.name.clone()]));
        }
        for (user, stream) in self.users.iter_mut() {
            if *user == from {
                continue;
            }
            if let Some(stream) = stream {
                Self::write_message(
                    format!(":{} PRIVMSG {} {}\n", from, self.name, message),
                    stream.clone(),
                );
            }
        }
        Ok(())
    }

    /// Given self returns whether the channel is empty
    pub fn is_empty(&mut self) -> bool {
        self.users.is_empty()
    }

    /// Given self, the potential user nick, their stream and a key tries to add the user to the channel.
    /// To be added use must provide the correct key.
    /// If the user already belongs the function will return a error.
    /// User must also be invited in caso of invite only channel.
    /// Additionally channel limit will be checked if aplicable.
    pub fn add_client(
        &mut self,
        nick: String,
        stream: Option<Arc<Mutex<TcpStream>>>,
        key: Option<String>,
    ) -> ReplyAndError {
        if self.key.is_some() && key != self.key {
            return Err((app_errors::ERR_PASSWDMISMATCH, vec![]));
        };
        if self.users.contains_key(&nick) {
            return Err((app_errors::ERR_NICKCOLLISION, vec![nick]));
        }
        if let Some(limit) = self.limit {
            if self.users.len() >= limit {
                return Err((app_errors::ERR_CHANNELISFULL, vec![self.name.clone()]));
            }
        }
        if self.invited_only && !self.invites.contains(&nick) {
            return Err((app_errors::ERR_INVITEONLYCHAN, vec![self.name.clone()]));
        }
        self.users.insert(nick, stream);
        Ok((app_errors::RPL_TOPIC, vec![]))
    }

    /// Given self returns a copy of the topic of the channel
    pub fn topic(&mut self) -> Option<String> {
        self.topic.clone()
    }

    /// Given self and a nick returns whether the user is an operator on the channel
    pub fn is_oper(&mut self, nick: &String) -> bool {
        self.operators.contains(nick)
    }

    /// Given self and a nick returns whether the user is present on the channel
    pub fn contains_client(&mut self, user: &str) -> bool {
        self.users.contains_key(user)
    }

    /// Given self returns whether the channel is secret mode
    pub fn is_secret(&mut self) -> bool {
        self.secret
    }

    /// Given self returns whether the channel is in invite only mode
    pub fn is_invite_only(&mut self) -> bool {
        self.invited_only
    }

    /// Given self returns the current limit of users on the channel
    /// The limit will be none in case of unlimited access
    pub fn get_limit(&mut self) -> Option<usize> {
        self.limit
    }

    /// Given self and a nick, tries to add the user to the list of operators
    /// If the given user is already an operator an error will be returned
    pub fn add_oper(&mut self, nick: String) -> DefaultAndError {
        if self.users.contains_key(&nick) {
            self.operators.insert(nick);
            return Ok(());
        }
        Err((app_errors::ERR_NOSUCHNICK, vec![nick]))
    }

    /// Given self and a mode changes the channel status corresponding to the secret object
    pub fn mode_secret(&mut self, mode: Mode) -> DefaultAndError {
        match mode {
            Mode::Activate(_) => self.secret = true,
            Mode::Deactivate(_) => self.secret = false,
        }
        Ok(())
    }

    /// Given self and a mode changes the channel status corresponding to the mode object
    pub fn mode_invite(&mut self, mode: Mode) -> DefaultAndError {
        match mode {
            Mode::Activate(_) => self.invited_only = true,
            Mode::Deactivate(_) => self.invited_only = false,
        }
        Ok(())
    }

    /// Given self and a mode changes the channel limit corresponding to the mode object.
    /// Channel limit can be removed by deactivate Mode type.
    pub fn set_limit(&mut self, mode: Mode, new_limit: Option<String>) -> DefaultAndError {
        match mode {
            Mode::Activate(_) => {
                if let Some(new_limit) = new_limit {
                    let new_limit: usize = match new_limit.parse() {
                        Ok(x) => x,
                        Err(_) => return Err((app_errors::ERR_UNKNOWNCOMMAND, vec![new_limit])),
                    };
                    self.limit = Some(new_limit);
                    return Ok(());
                }
                Err((
                    app_errors::ERR_NEEDMOREPARAMS,
                    vec!["MODE +/- l".to_string()],
                ))
            }
            Mode::Deactivate(_) => {
                self.limit = None;
                Ok(())
            }
        }
    }

    /// Auxiliary function to assist with sending channel messages.
    /// Given the messages and the stream tries to send the message
    fn write_message(msg: String, stream: Arc<Mutex<TcpStream>>) {
        let mut stream = match stream.lock() {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("Server error (locking stream): {err}");
                std::process::exit(1);
            }
        };

        if let Err(err) = stream.write(msg.as_bytes()) {
            eprintln!("Server error: {err}");
            std::process::exit(1);
        }
    }
}
