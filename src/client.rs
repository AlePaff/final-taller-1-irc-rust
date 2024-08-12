#![allow(dead_code)]
#![allow(unused_variables)]

use crate::app_errors;
use std::collections::HashSet;
use std::error::Error;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, stdin};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

const PACKET_SIZE: usize = 256; // 4 bytes (32 bits)


#[derive(Debug, Clone)]
pub struct DCCMessage {
    to: String,
    from: String,
    message: String,
    is_read: bool,
}

impl DCCMessage {
    pub fn new(to: String, from: String, message: String, is_read: bool) -> Self {
        DCCMessage {
            to,
            from,
            message,
            is_read,
        }
    }
    pub fn to(&self) -> &str {
        &self.to
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn is_read(&self) -> bool {
        self.is_read
    }

    pub fn set_is_read(&mut self, value: bool) {
        self.is_read = value;
    }
}


#[derive(Clone)]
pub enum Received {
    Msg(String, String, String),
    IrcRpl(String, String),
    IrcErr(String, String),
    Unknown(String),
}

// utilizado para el cliente con interfaz gráfica. TODO: Se podría adaptar al CLI
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
    dcc_chat: Option<TcpStream>,
}

impl ClientC {
    // #![allow(dead_code)]
    pub fn new(address: String, port: String) -> Result<ClientC, Box<dyn Error>> {
        let server = TcpStream::connect(address + ":" + &port)?;
        server.set_read_timeout(Some(Duration::from_millis(100)))?;
        let channels = HashSet::new();
        
        println!("Cliente creado GTK!");
        Ok(ClientC {
            //logger_file_path,
            server,
            channels,
            nick: None,
            dcc_chat: None,
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


    /// Lado de quien quiere iniciar la conexión
    /// sends a DCC CHAT request, writing the PRIVMSG command with a CTCP message to the server
    pub fn send_dcc_chat(&mut self, to: String) {
        
        // Crear el socket de escucha en la ip y puerto del sender
        // let listener = TcpListener::bind("localhost:7676").expect("Failed to bind to port");
        let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind to port");      // escucha a toodas las posibles conexiones entrantes
        let local_addr = listener.local_addr().expect("Failed to get local address");

        // Obtener la IP y el puerto
        let ip = local_addr.ip().to_string();
        let port = local_addr.port().to_string();

        // Enviar el mensaje DCC CHAT a través de IRC
        let message_dcc = format!("\x01DCC CHAT chat {} {}\x01", ip, port);
        self.send_privmsg(to, message_dcc);

        // let mut sender = self.server.try_clone()?;
        // let sender = thread::spawn(move || ClientC::sender(&mut sender));
        // Clonar la referencia para enviarla al hilo
        // let self_clone = Arc::clone(&self);
        // std::thread::spawn(move || {
        println!("Listening for incoming DCC CHAT connection on {}:{}", ip, port);

        // Aceptar una conexión entrante
        if let Ok((socket, addr)) = listener.accept() {
            println!("Accepted DCC CHAT connection from {}", addr);
            
            // Cerrar el socket del listener
            drop(listener);

            // Establecer el socket de chat
            self.dcc_chat = Some(socket.try_clone().expect("Failed to clone socket"));

            // Clonar el socket para uso en el hilo
            let mut chat_socket = self.dcc_chat.as_ref().expect("Chat socket not initialized").try_clone().expect("Failed to clone chat socket");
            let reader = io::BufReader::new(chat_socket.try_clone().expect("Failed to clone chat socket"));
            let stdin = io::stdin();
            
            // Lanzar un hilo para leer mensajes desde el socket
            let handle = thread::spawn(move || {
                for line in reader.lines() {
                    match line {
                        Ok(message) => {
                            if message.trim() == "DCC CLOSE" {
                                println!("[Remote] Connection closed by remote.");
                                break;
                            } else {
                                println!("[Remote] {}", message);
                            }
                        }
                        Err(e) => eprintln!("Error reading from socket: {}", e),
                    }
                }
            });

            // Leer desde la entrada estándar y enviar al socket
            for line in stdin.lock().lines() {
                let input = line.unwrap_or_default();
                if input.trim() == "DCC CLOSE" {
                    if let Err(e) = chat_socket.write_all(b"DCC CLOSE\n") {
                        eprintln!("Failed to send close message: {}", e);
                    }
                    break;
                }
                if let Err(e) = chat_socket.write_all(input.as_bytes()) {
                    eprintln!("Failed to send message: {}", e);
                    break;
                }
                if let Err(e) = chat_socket.write_all(b"\n") {
                    eprintln!("Failed to send newline: {}", e);
                    break;
                }
            }

            // Esperar a que el hilo de lectura termine
            if let Err(e) = chat_socket.shutdown(Shutdown::Both) {
                eprintln!("Error shutting down socket: {}", e);
            }
            handle.join().expect("Thread panicked");
        } else {
            eprintln!("Failed to accept connection");
        }
        // });
    }

    /// El cliente acepta una conexión P2P, se parsea del mensaje la ip y puerto
    /// ya se sabe que es un mensaje DCC. Se conecta al TCP Stream
    pub fn handle_dcc_chat_session(&mut self, dcc_response: Arc<Mutex<DCCMessage>>) {
        let mut dcc_response_lock = dcc_response.lock().expect("Couldn't lock dcc_response");
        let DCCMessage {ref from, ref to, ref message, ref is_read} = *dcc_response_lock;

        // Quitar los caracteres de inicio y final (\x01)
        let trimmed_message = message.trim_start_matches('\x01').trim_end_matches('\x01');

        // Dividir el mensaje por espacios
        let parts: Vec<&str> = trimmed_message.split_whitespace().collect();

        // Asegurarse de que hay suficientes partes
        if parts.len() >= 4 && parts[0] == "DCC" && parts[1] == "CHAT" && parts[2] == "chat" {
            let ip = parts[3];
            let port = parts[4];
            
            println!("Extracted IP: {}", ip);
            println!("Extracted Port: {}", port);

            // Conectar al IP y puerto suministrado
            match TcpStream::connect(format!("{}:{}", ip, port)) {
                Ok(chat_socket) => {
                    println!("Successfully connected to {}:{}", ip, port);
                    self.dcc_chat = Some(chat_socket.try_clone().expect("Failed to clone chat_socket"));
    
                    // Lanzar un hilo para leer mensajes desde el socket
                    let mut chat_socket = self.dcc_chat.as_ref().expect("Chat socket not initialized").try_clone().expect("Failed to clone chat_socket");
                    let reader = io::BufReader::new(chat_socket.try_clone().expect("Failed to clone chat_socket"));
                    let stdin = io::stdin();
                    let handle = thread::spawn(move || {
                        for line in reader.lines() {
                            match line {
                                Ok(message) => {
                                    if message.trim() == "DCC CLOSE" {
                                        println!("[Remote] Chat closed.");
                                        break;
                                    }
                                    println!("[Remote] {}", message);
                                }
                                Err(e) => eprintln!("Error reading from socket: {}", e),
                            }
                        }
                    });
    
                    // Leer desde la entrada estándar y enviar al socket
                    for line in stdin.lock().lines() {
                        match line {
                            Ok(input) => {
                                if input.trim() == "DCC CLOSE" {
                                    if let Err(e) = chat_socket.write_all(b"DCC CLOSE\n") {
                                        eprintln!("Failed to send close message: {}", e);
                                    }
                                    break;
                                }
                                if let Err(e) = chat_socket.write_all(input.as_bytes()) {
                                    eprintln!("Failed to send message: {}", e);
                                    break;
                                }
                                if let Err(e) = chat_socket.write_all(b"\n") {
                                    eprintln!("Failed to send newline: {}", e);
                                    break;
                                }
                            }
                            Err(e) => eprintln!("Error reading from stdin: {}", e),
                        }
                    }
    
                    // Esperar a que el hilo de lectura termine
                    if let Err(e) = chat_socket.shutdown(Shutdown::Both) {
                        eprintln!("Error shutting down socket: {}", e);
                    }
                    handle.join().expect("Thread panicked");
                }
                Err(e) => {
                    eprintln!("Failed to connect to {}:{}. Error: {}", ip, port, e);
                }    
            }
        } else {
            println!("Message does not match expected format: {}", message);
        }
        // dejarlo vacío para otra posible conexion
        *dcc_response_lock = DCCMessage::new(String::new(), String::new(), String::new(), false);
    }
    


    // envia mensaje SEND por privmsg y espera a establecer una conexión
    pub fn send_dcc_send_message(&mut self, to: String, file_path: String) {
        // Crear el socket de escucha en la ip y puerto del sender
        let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind to port");
        let local_addr = listener.local_addr().expect("Failed to get local address");

        // Obtener la IP y el puerto
        let ip = local_addr.ip().to_string();
        let port = local_addr.port().to_string();

        let path = Path::new(&file_path);
        let mut file = File::open(&path).expect("Failed to open file");
        let filesize = path.metadata().expect("Failed to get file metadata").len().to_string();
        
        let filename = path.file_name().unwrap_or_else(|| "unknown".as_ref()).to_string_lossy();
        
        // Enviar el mensaje DCC SEND a través de IRC
        let message_dcc = format!("\x01DCC SEND {} {} {} {}\x01", filename, ip, port, filesize);
        println!("MENSAJE A ENVIAR: {}", message_dcc);
        self.send_privmsg(to, message_dcc);

        println!("Listening for incoming DCC SEND connection on {}:{}", ip, port);

        if let Ok((socket, addr)) = listener.accept() {
            println!("Accepted DCC SEND connection from {}", addr);

            // Enviar el archivo en paquetes y esperar confirmaciones
            if let Err(e) = ClientC::send_file_in_packets(&mut file, socket) {
                eprintln!("Error sending file: {}", e);
            }

            println!("File transfer completed successfully.");

        } else {
            eprintln!("Failed to accept connection");
        }
    }


    // Función para enviar datos en paquetes y esperar confirmación
    fn send_file_in_packets(file: &mut File, mut stream: TcpStream) -> Result<(), std::io::Error> {
        let mut buffer = [0u8; PACKET_SIZE];
        let mut total_bytes_sent = 0;

        loop {
            // Leer un paquete del archivo
            let bytes_read = file.read(&mut buffer)?;

            if bytes_read == 0 {
                // Archivo completamente leído
                break;
            }

            // Enviar el paquete
            stream.write_all(&buffer[..bytes_read])?;
            stream.flush()?;

            // Esperar confirmación de recepción del paquete
            let mut ack_buffer = [0u8; 4];
            stream.read_exact(&mut ack_buffer)?;
            let ack_bytes_received = u32::from_be_bytes(ack_buffer);

            if ack_bytes_received != total_bytes_sent as u32 + bytes_read as u32 {
                eprintln!("Acknowledgement mismatch: expected {}, got {}", total_bytes_sent + bytes_read, ack_bytes_received);
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Acknowledgement mismatch"));
            }

            total_bytes_sent += bytes_read;
            println!("bytes_read / total_bytes_sent: {} / {}. ack_bytes_received: {} ", bytes_read, total_bytes_sent, ack_bytes_received);

        }

        // Esperar confirmación final (para el último paquete)
        let mut final_ack_buffer = [0u8; 4];
        stream.read_exact(&mut final_ack_buffer)?;
        let final_ack_bytes_received = u32::from_be_bytes(final_ack_buffer);

        if final_ack_bytes_received != total_bytes_sent as u32 {
            eprintln!("Final acknowledgement mismatch: expected {}, got {}", total_bytes_sent, final_ack_bytes_received);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Final acknowledgement mismatch"));
        }

        Ok(())
    }


    /// Lado de quien recibe la conexión para la transferencia de archivos
    pub fn handle_dcc_send_files(&mut self, dcc_response: Arc<Mutex<DCCMessage>>){
        let mut dcc_response_lock = dcc_response.lock().expect("Couldn't lock dcc_response");
        let DCCMessage {ref from, ref to, ref message, ref is_read} = *dcc_response_lock;

        // Quitar los caracteres de inicio y final (\x01)
        let trimmed_message = message.trim_start_matches('\x01').trim_end_matches('\x01');

        // Dividir el mensaje por espacios
        let parts: Vec<&str> = trimmed_message.split_whitespace().collect();

        // Asegurarse de que hay suficientes partes
        if parts.len() >= 5 && parts[0] == "DCC" && parts[1] == "SEND" {
            let filename = parts[2];
            let ip = parts[3];
            let port = parts[4];
            let filesize_str = parts[5];
            let filesize: u64 = match filesize_str.parse() {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Error parsing filesize: {}", e);
                    return;
                }
            };
            
            println!("Extracted filename: {}, IP: {}, Port: {}, filesize: {}", filename, ip, port, filesize);

            // Conectar al IP y puerto suministrado
            // Conectar al IP y puerto suministrado
            match TcpStream::connect(format!("{}:{}", ip, port)) {
                Ok(mut stream) => {
                    println!("Successfully connected to {}:{}", ip, port);

                    // Crear la carpeta 'receptor' si no existe
                    let receptor_dir = Path::new("receptor");
                    if !receptor_dir.exists() {
                        create_dir_all(receptor_dir).expect("Failed to create 'receptor' directory");
                    }

                    // Construir la ruta completa del archivo dentro de la carpeta 'receptor'
                    let path = receptor_dir.join(filename);
                    let mut file = OpenOptions::new().create(true).write(true).open(&path)
                        .expect("Failed to open file for writing");

                    let mut total_bytes_received: u64 = 0;
                    let mut buffer = [0u8; PACKET_SIZE];

                    while total_bytes_received < filesize {
                        // Leer el paquete de la conexión
                        let bytes_read = stream.read(&mut buffer).expect("error en el buffer");

                        if bytes_read == 0 {
                            // Si no se leyeron datos, se asume que la conexión se cerró
                            break;
                        }

                        // Escribir el paquete en el archivo
                        file.write_all(&buffer[..bytes_read]).expect("error en el buffer");
                        total_bytes_received += bytes_read as u64;

                        // Enviar confirmación de recepción
                        let ack = (total_bytes_received as u32).to_be_bytes();
                        stream.write_all(&ack).expect("error en el buffer");
                        stream.flush().expect("error at flush stream");

                        println!("bytes_read / total_bytes_received: {} / {}. File size: {} ", bytes_read, total_bytes_received, filesize);
                    }

                    println!("File transfer completed successfully.");

                    // Cerrar la conexión
                    stream.shutdown(Shutdown::Both).expect("Failed to shut down the connection");
                }
                Err(e) => {
                    eprintln!("Failed to connect to {}:{}. Error: {}", ip, port, e);
                }
            }
        } else {
            println!("Message does not match expected format: {}", message);
        }
        // dejarlo vacío para otra posible conexion
        *dcc_response_lock = DCCMessage::new(String::new(), String::new(), String::new(), false);
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
            dcc_chat: None,
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

    /// writes a line to the stream
    fn write_to(stream: &mut dyn Write, buffer: String) -> std::io::Result<()> {
        stream.write_all(buffer.as_bytes())?;
        stream.write_all("\n".as_bytes())?;
        Ok(())
    }
}
