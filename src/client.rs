#![allow(dead_code)]
#![allow(unused_variables)]

use crate::app_errors;
use std::collections::HashSet;
use std::error::Error;
use std::io::stdin;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

pub enum Received {
    Msg(String, String, String),
    IrcRpl(String, String),
    IrcErr(String, String),
    Unknown(String),
}

pub struct ClientBuilder {
    ip: Option<String>,
    port: Option<String>,
}

impl ClientBuilder {
    pub fn set_ip(&mut self, ip: String) {
        self.ip = Some(ip);
    }
    pub fn set_port(&mut self, port: String) {
        self.port = Some(port);
    }
    pub fn new() -> ClientBuilder {
        ClientBuilder {
            ip: None,
            port: None,
        }
    }
    pub fn get_client(&self) -> Result<ClientC, String> {
        match (self.ip.clone(), self.port.clone()) {
            (None, _) => Err("Ip is missing".to_string()),
            (_, None) => Err("Port is missing".to_string()),
            (Some(ip), Some(port)) => {
                return match ClientC::new(ip, port) {
                    Ok(client) => Ok(client),
                    Err(_) => Err("Ip or port incorrect!".to_string()),
                };
            }
        }
    }
}

/// ClientC is the client-side that conects to a server
pub struct ClientC {
    //logger_file_path: String,
    server: TcpStream,
    channels: HashSet<String>,
    nick: Option<String>,
}

impl ClientC {
    // #![allow(dead_code)]
    pub fn new(address: String, port: String) -> Result<ClientC, Box<dyn Error>> {
        let server = TcpStream::connect(address + ":" + &port)?;
        server.set_read_timeout(Some(Duration::from_millis(100)))?;
        let channels = HashSet::new();
        
        println!("CLIENTE CREADO!!");
        Ok(ClientC {
            //logger_file_path,
            server,
            channels,
            nick: None,
        })
    }

    /// registers a new user to the server, sending the PASS, NICK and USER commands
    pub fn register(&mut self, pass: String, nick: String, user: String) -> String {
        self.server
            .write_all(format!("PASS {}\n", pass).as_bytes())
            .expect("server write failed when writing pass");
        self.server
            .write_all(format!("NICK {}\n", nick).as_bytes())
            .expect("server write failed when writing nick");
        self.server
            .write_all(format!("USER {}\n", user).as_bytes())
            .expect("server write failed when writing user");
        self.nick = Some(nick);
        return self.read_from_stream().expect("read failed");
    }







    /// sends a privmsg, writing the PRIVMSG command to the server
    pub fn send_privmsg(&mut self, to: String, message: String) {
        self.server
            .write_all(format!("PRIVMSG {} {}\n", to, message).as_bytes())
            .expect("server write failed when writing privmsg");
    }







    pub fn make_oper(&mut self, channel: String, nick: String) {
        self.server
            .write_all(format!("MODE {} +o {}\n", channel, nick).as_bytes())
            .expect("server write failed when writing MODE +o");
    }

    pub fn set_limit(&mut self, channel: String, limit: u32) {
        self.server
            .write_all(format!("MODE {} +l {}\n", channel, limit).as_bytes())
            .expect("server write failed when writing MODE +l");
    }

    pub fn set_invite_only(&mut self, channel: String) {
        self.server
            .write_all(format!("MODE {} +i\n", channel).as_bytes())
            .expect("server write failed when writing MODE +i");
    }

    pub fn unset_invite_only(&mut self, channel: String) {
        self.server
            .write_all(format!("MODE {} -i\n", channel).as_bytes())
            .expect("server write failed when writing MODE -i");
    }

    pub fn set_secret(&mut self, channel: String) {
        self.server
            .write_all(format!("MODE {} +s\n", channel).as_bytes())
            .expect("server write failed when writing MODE +l");
    }

    pub fn unset_secret(&mut self, channel: String) {
        self.server
            .write_all(format!("MODE {} -s\n", channel).as_bytes())
            .expect("server write failed when writing MODE -l");
    }

    pub fn invite(&mut self, channel: String, nick: String) {
        self.server
            .write_all(format!("INVITE {} {}\n", nick, channel).as_bytes())
            .expect("server write failed when writing INVITE");
        self.channels.remove(&channel);
    }

    pub fn kick(&mut self, channel: String, nick: String, reason: String) {
        self.server
            .write_all(format!("KICK {} {} {}\n", channel, nick, reason).as_bytes())
            .expect("server write failed when writing PART");
        self.channels.remove(&channel);
    }

    pub fn part(&mut self, channel: String) {
        self.server
            .write_all(format!("PART {}\n", channel).as_bytes())
            .expect("server write failed when writing PART");
        self.channels.remove(&channel);
    }

    pub fn become_oper(&mut self, pass: String) {
        if let Some(nick) = self.nick.clone() {
            self.server
                .write_all(format!("OPER {} {}\n", nick, pass).as_bytes())
                .expect("server write failed when writing privmsg");
        }
    }

    pub fn send_squit(&mut self, server: String) {
        self.server
            .write_all(format!("SQUIT {}\n", server).as_bytes())
            .expect("server write failed when writing privmsg");
    }

    pub fn send_quit(&mut self) {
        self.server
            .write_all("QUIT\n".as_bytes())
            .expect("server write failed when writing quit");
    }

    /// gets all nicks from the server, returning a HashSet of nicks
    pub fn get_server_nicks(&mut self) -> Result<HashSet<String>, Box<dyn Error>> {
        self.server.write_all("WHO *\n".to_string().as_bytes())?; // get all nicks from server
        let response = self.read_from_stream()?; // read response (nicks)
        println!("response: {}", response);
        let mut nicks = HashSet::new();
        let mut clients: Vec<String> = response.split(';').map(|value| value.to_string()).collect();
        clients.pop(); // remove last element (empty string)
        for client in clients {
            let pair: Vec<String> = client.split(':').map(|value| value.to_string()).collect();
            let pair: Vec<String> = pair
                .get(0) // get first element (nick)
                .ok_or("Error getting server nicks")?
                .split(' ')
                .map(|value| value.to_string())
                .collect();
            if let Some(x) = pair.get(1) {
                nicks.insert(x.to_string());
            }
        }
        Ok(nicks)
    }
    pub fn get_names(&mut self) -> Vec<String> {
        self.server
            .write_all("NAMES\n".to_string().as_bytes())
            .expect("server write failed when writing names");
        let channels = Vec::new();
        let response = self.try_read_from_stream();
        if response.is_err() {
            return channels;
        }
        let mut channels = Vec::new();
        let mut server_channels: Vec<String> = match response {
            Ok(response) => response.split(';').map(|value| value.to_string()).collect(),
            Err(_) => return channels, //en caso de error devolvemos el vector vacio
        };
        server_channels.pop();
        for channel in server_channels {
            let pair: Vec<String> = channel.split(':').map(|value| value.to_string()).collect();
            if let Some(x) = pair.get(0) {
                channels.push(x.to_string());
            }
        }
        channels
    }

    /// given a channel string join the client to the channel, sending the JOIN command to the server
    pub fn join(&mut self, channel: String) {
        if !self.is_in_channel(channel.clone()) {
            self.server
                .write_all(format!("JOIN {}\n", channel).as_bytes())
                .expect("server write failed when writing join");
            self.channels.insert(channel);
        }
    }

    /// auxiliar function to check if the channel is in the channels hashset
    fn is_in_channel(&mut self, channel: String) -> bool {
        self.channels.contains(&channel)
    }

    /// read_message reads a message from the server and returns a Received enum
    pub fn read_message(&mut self) -> Received {
        if let Ok(mut message) = self.try_read_from_stream() {
            if message.starts_with(':') {
                //if the message starts with a colon, it is a command
                message.remove(0);
                if let Some((sender, message)) = message.split_once("PRIVMSG") {
                    let (to, msg) = match message.trim().split_once(' ') {
                        Some(x) => x,
                        None => return Received::Unknown(message.to_string()),
                    };
                    return Received::Msg(
                        sender.trim().to_string(),
                        to.trim().to_string(),
                        msg.to_string(),
                    );
                // } else if let Some((sender, message)) = message.split_once("PRIVMSG") {
                //     let (to, msg) = match message.trim().split_once(' ') {
                //         Some(x) => x,
                //         None => return Received::Unknown(message.to_string()),
                //     };
                //     return Received::Msg(
                //         sender.trim().to_string(),
                //         to.trim().to_string(),
                //         msg.to_string(),
                //     );
                } else {
                    return Received::Unknown(message);
                }
            } else if message.starts_with('4') {
                //if the message starts with a 4, it is an error
                if let Some((code, message)) = message.split_once(':') {
                    return Received::IrcErr(code.trim().to_string(), message.trim().to_string());
                }
            } else if message.starts_with('3') || message.starts_with('2') {
                //if the message starts with a 3 or 2, it is a reply
                if let Some((code, message)) = message.split_once(':') {
                    return Received::IrcRpl(code.trim().to_string(), message.trim().to_string());
                }
            }
        }
        Received::Unknown(String::new())
    }

    /// Funcionalidad de la entrega anterior (CLI). No tiene todas las funcionalidades de la entrega final implementadas
    pub fn build(config: super::config_client::ConfigClient) -> Result<ClientC, Box<dyn Error>> {
        //let logger_file_path = config.log_path;
        let server = TcpStream::connect(config.address + ":" + &config.port)?;

        let channels = HashSet::new();
        Ok(ClientC {
            //logger_file_path,
            server,
            channels,
            nick: None,
        })
    }

    /// similar to read_from_stream but waits for the server to send a message
    fn try_read_from_stream(&mut self) -> Result<String, Box<dyn Error>> {
        let mut line = String::new();
        let mut char = [b'\n'];

        thread::sleep(Duration::from_millis(10)); // wait for the server to send the message
        if self.server.read_exact(&mut char).is_err() {
            return Err(Box::new(app_errors::ApplicationError(
                "nothing to read".to_string(),
            )));
        }
        while char[0] != b'\n' {
            line.push_str(&String::from_utf8(char.to_vec())?); //convertir el array de bytes a string
            char = [b'\n'];
            if self.server.read_exact(&mut char).is_err() {
                continue;
            }
        }
        Ok(line)
    }

    /// Reads a line from the stream (conected server) and returns it as a string
    fn read_from_stream(&mut self) -> Result<String, Box<dyn Error>> {
        let mut line = String::new();
        let mut char = [b'\n'];

        while char[0] == b'\n' {
            if self.server.read_exact(&mut char).is_err() {
                continue;
            }
        }
        while char[0] != b'\n' {
            line.push_str(&String::from_utf8(char.to_vec())?);

            char = [b'\n'];
            if self.server.read_exact(&mut char).is_err() {
                continue;
            }
        }
        Ok(line)
    }

    /// runs the client with a sender and a listener thread
    /// sender is the thread that sends messages to the server
    /// and listener is the thread that receives messages from the server
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        //try_clone() devuelve un nuevo TcpStream que referencia al mismo socket que el TcpStream original.
        let mut sender = self.server.try_clone()?;
        let sender = thread::spawn(move || ClientC::sender(&mut sender));

        let mut listener = self.server.try_clone()?;
        let listener = thread::spawn(move || ClientC::listener(&mut listener));

        // join waits for the thread to finish
        if sender.join().is_err() {
            return Err(Box::new(app_errors::ApplicationError(
                "Error in sender thread.".into(),
            )));
        }
        if listener.join().is_err() {
            return Err(Box::new(app_errors::ApplicationError(
                "Error in listener thread.".into(),
            )));
        }
        Ok(())
    }

    /// returns true if the line starts with QUIT and false otherwise
    fn is_quit_message(line: String) -> bool {
        match line.split(' ').next() {
            Some(x) => x == "QUIT",
            None => false,
        }
    }

    /// reads from stdin and sends the messages to the server (writes in TcpStream) until the user types QUIT
    fn sender(sender: &mut dyn Write) -> std::io::Result<()> {
        let reader = BufReader::new(stdin());
        for line in reader.lines().flatten() {
            ClientC::write_to(sender, line.clone())?;
            if Self::is_quit_message(line) {
                return Ok(());
            }
        }
        Ok(())
    }

    /// reads from the server (from TcpStream) and prints the message to stdout
    fn listener(listener: &mut dyn Read) -> std::io::Result<()> {
        let reader = BufReader::new(listener);
        for line in reader.lines().flatten() {
            println!("{}", line);
        }
        Ok(())
    }

    /*     /// reads a line from the stream
    fn read_from(stream: &mut dyn Read) -> std::io::Result<String> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line[..line.len() - 1].to_string())
        //Ok(line.to_string())
    } */

    /// writes a line to the stream
    fn write_to(stream: &mut dyn Write, buffer: String) -> std::io::Result<()> {
        stream.write_all(buffer.as_bytes())?;
        stream.write_all("\n".as_bytes())?;
        Ok(())
    }
}
