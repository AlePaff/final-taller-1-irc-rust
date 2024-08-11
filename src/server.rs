pub mod channel;
pub mod client_s;
pub mod clients_info;
pub mod logger;
use crate::server::logger::Logger;
use client_s::ClientS;
pub use clients_info::ClientsInfo;

use crate::app_errors;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

pub struct Server {
    clients: Arc<Mutex<ClientsInfo>>,
    listener: TcpListener,
    parent_name: Option<String>,
    parent: Option<Arc<Mutex<TcpStream>>>,
    password: Option<String>,
    trusted_servers: HashMap<String, Option<String>>,
    log: Arc<Mutex<Logger>>,
}

/// Server is the main struct of the server. Initializes new conections and allows a communication in the network.
/// beetwen the clients and other servers
impl Server {
    pub fn build(config: crate::config::Config) -> Result<Server, Box<dyn Error>> {
        let operators = Self::build_operators(config.operators_path)?;
        let trusted_servers = Self::build_trusted_servers(config.trusted_servers_path)?;
        let parent_name = config.parent_name;
        let password = config.password.clone();
        let clients = Arc::new(Mutex::new(ClientsInfo::new(
            config.name.clone(),
            config.password,
            operators,
        )));
        let log = Arc::new(Mutex::new(Logger::build(config.log_path)));
        log.lock()
            .expect("Error creating log lock")
            .write("\n=============== SERVER EXECUTED ===============".to_string());
        // arc allow multiple threads to access the same data and mutex allow only one thread to access the data at a time
        let listener = TcpListener::bind(config.address + ":" + &config.port)?;
        // TcpListener is a type that listens for incoming TCP connections.

        let mut parent_connection = Self::connect_to_parent(config.parent_ip, config.parent_port)?;
        //si no hay parent, parent_connection es None

        Self::register_to_parent(&mut parent_connection, config.parent_pwd, config.name)?;

        let mut parent = None;

        if let Some(connection) = parent_connection {
            parent = Some(Arc::new(Mutex::new(connection)))
        }

        Ok(Server {
            clients,
            listener,
            parent_name,
            parent,
            password,
            trusted_servers,
            log,
        })
    }

    /// is the main loop of the server, it accepts new connections and
    /// creates a new thread for each one
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut threads = vec![]; // vector of threads
        if let Some(parent) = self.parent.clone() {
            let clients = self.clients.clone(); //with clone create a new reference to ClientsInfo
            self.handle_server(parent, clients)?;
        }
        // accept connections and process them, spawning a new thread for each one
        for stream in self.listener.incoming() {
            let stream = stream?;
            let clients = self.clients.clone(); //with clone create a new reference to ClientsInfo
            self.handle_client(Arc::new(Mutex::new(stream)), clients, &mut threads)?;
        }
        // wait for all threads to finish
        for child in threads {
            match child.join() {
                Ok(_) => continue,
                Err(_) => println!("Thread panicked"),
            }
        }
        Ok(())
    }

    /// Registers to the parent server sending the PASS (if there is one) and SERVER commands
    fn register_to_parent(
        connection: &mut Option<TcpStream>,
        pwd: Option<String>,
        server_name: String,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(connection) = connection {
            if let Some(pwd) = pwd {
                connection.write_all(format!("PASS {}\n", pwd).as_bytes())?; //it sends the PASS command to the parent server
            }
            connection.write_all(format!("SERVER {} 1 info", server_name).as_bytes())?; //it sends the SERVER command to the parent server
            let ans = Self::read_from_stream(connection)?;
            println!("{ans}");
            if ans.starts_with('4') {
                // 4xx are the http error codes for client errors (in this context the parent server is the client)
                return Err(Box::new(app_errors::ApplicationError(ans)));
            }
        }
        Ok(())
    }

    /// Returns the conection to the parent server if there is one
    fn connect_to_parent(
        ip: Option<String>,
        port: Option<String>,
    ) -> Result<Option<TcpStream>, Box<dyn Error>> {
        //si hay parent (ip y puerto) se conecta, sino devuelve None
        if let (Some(ip), Some(port)) = (ip, port) {
            let connection = TcpStream::connect(ip + ":" + &port)?;
            Ok(Some(connection))
        } else {
            Ok(None)
        }
    }

    /// Reads a line from the stream and returns it as a string
    fn read_from_stream(connection: &mut TcpStream) -> Result<String, Box<dyn Error>> {
        let mut line = String::new();
        let mut char = [b'\n'];

        while char[0] == b'\n' {
            // read until the first non-empty line
            if connection.read_exact(&mut char).is_err() {
                continue;
            }
        }
        while char[0] != b'\n' {
            // read until the end of the line
            line.push_str(&String::from_utf8(Vec::from(char))?);
            char = [b'\n'];
            if connection.read_exact(&mut char).is_err() {
                continue;
            }
        }
        Ok(line)
    }

    /// Creates a new client and runs it in a new thread
    fn handle_client(
        &self,
        stream: Arc<Mutex<TcpStream>>,
        clients: Arc<Mutex<ClientsInfo>>,
        threads: &mut Vec<JoinHandle<Result<(), std::io::Error>>>,
    ) -> std::io::Result<()> {
        let mut client = ClientS::new(
            clients,
            stream,
            self.trusted_servers.clone(),
            self.log.clone(),
        )
        .expect("Error creating a new client");
        threads.push(thread::spawn(move || client.run()));
        Ok(())
    }

    /// for every server conected to a parent server it sends the list of clients to the parent server
    /// and creates a new thread for each one
    fn handle_server(
        &self,
        stream: Arc<Mutex<TcpStream>>,
        clients: Arc<Mutex<ClientsInfo>>,
    ) -> std::io::Result<()> {
        let mut client = ClientS::new(
            clients,
            stream,
            self.trusted_servers.clone(),
            self.log.clone(),
        )
        .expect("Error creating a new server conection");
        client.set_parent(self.parent_name.clone(), self.password.clone());
        thread::spawn(move || client.run());
        Ok(())
    }

    /// Given a path to a csv file, it returns a hashmap with the operators of the server
    fn build_operators(path_operators: String) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let contents = fs::read_to_string(path_operators)?;
        let mut operators = HashMap::new();

        //parse the csv file and adds the operators to the hashmap
        for line in contents.lines() {
            let mut l_split: Vec<String> = line
                .trim()
                .split(',')
                .map(|value| value.to_string())
                .collect();
            if l_split.len() != 2 {
                // if the line is not in the format "operator, password"
                return Err(Box::new(app_errors::ApplicationError(
                    "Invalid operators format.".into(),
                )));
            }
            operators.insert(l_split.remove(0), l_split.remove(0));
        }
        Ok(operators)
    }

    /// Given a path to a csv file, it returns a hashmap with the trusted servers of the server
    /// with the format "server_name: password"
    fn build_trusted_servers(
        path_trusted_servers: String,
    ) -> Result<HashMap<String, Option<String>>, Box<dyn Error>> {
        let contents = fs::read_to_string(path_trusted_servers)?;
        let mut trusted_servers = HashMap::new();

        //parse the csv file and adds the operators to the hashmap
        for line in contents.lines() {
            let mut l_split: Vec<String> = line
                .trim()
                .split(',')
                .map(|value| value.to_string())
                .collect();
            if l_split.len() != 2 {
                return Err(Box::new(app_errors::ApplicationError(
                    "Invalid trusted servers format.".into(),
                )));
            }
            let curr_name = l_split.remove(0);
            if curr_name.is_empty() {
                return Err(Box::new(app_errors::ApplicationError(
                    "Invalid trusted server name.".into(),
                )));
            }
            let curr_pass = l_split.remove(0);
            if curr_pass.is_empty() {
                trusted_servers.insert(curr_name, None);
            } else {
                trusted_servers.insert(curr_name, Some(curr_pass));
            }
        }
        Ok(trusted_servers)
    }
}

#[cfg(test)]
mod server_test {
    // use crate::app_errors;
    // use crate::server::clients_info::ClientsInfo;
    use crate::config::Config;
    use crate::server::Server;

    // use std::collections::HashMap;

    // pub fn setup_clients_info() -> ClientsInfo {
    //     let mut opers = HashMap::new();
    //     opers.insert("juan".to_string(), "botter".to_string());
    //     return ClientsInfo::new("tests".to_string(), Some("hola".to_string()), opers);
    // }

    #[allow(dead_code)]
    pub fn setup_config_server() -> Config {
        let config = Config {
            name: "server_uno".to_string(),
            password: Some("1111".to_string()),
            port: "7878".to_string(),
            address: "localhost".to_string(),
            log_path: "tests/test_files/log_file_1".to_string(),
            operators_path: "tests/test_files/valid_operators".to_string(),
            trusted_servers_path: "tests/test_files/valid_trusted_servers".to_string(),
            parent_name: None,
            parent_ip: None,
            parent_port: None,
            parent_pwd: None,
        };
        return config;
    }

    #[allow(dead_code)]
    pub fn setup_config_server_parent() -> Config {
        let config = Config {
            name: "server_dos".to_string(),
            password: Some("2222".to_string()),
            port: "7879".to_string(),
            address: "localhost".to_string(),
            log_path: "tests/test_files/log_file_2".to_string(),
            operators_path: "tests/test_files/valid_operators".to_string(),
            trusted_servers_path: "tests/test_files/valid_trusted_servers".to_string(),
            parent_name: Some("server_uno".to_string()),
            parent_ip: Some("localhost".to_string()),
            parent_port: Some("7878".to_string()),
            parent_pwd: Some("1111".to_string()),
        };
        return config;
    }

    // #[test]
    // fn read_from_stream_returns_line() {
    //     let _listener = TcpListener::bind("localhost:7878").expect("");
    //     let mut stream = TcpStream::connect("localhost:7878").expect("");
    //     stream.write_all("\nhello\n".as_bytes()).expect("");
    //     assert_eq!(Server::read_from_stream(&mut stream).expect(""), "hello");
    // }

    #[test]
    fn server_builds_operators_key_is_correct() {
        let opers = Server::build_operators("tests/test_files/valid_operators".to_string());
        assert!(opers.is_ok());
        assert!(opers
            .expect("fail result")
            .contains_key(&"nico".to_string()));
    }
    #[test]
    fn server_builds_operators_value_is_correct() {
        let opers = Server::build_operators("tests/test_files/valid_operators".to_string());
        assert_eq!(
            opers
                .expect("fail result")
                .get(&"nico".to_string())
                .expect("fail get"),
            "123"
        );
    }

    #[test]
    fn trusted_servers_with_valid_path_returns_corresponding_keys() {
        let trusted_servers =
            Server::build_trusted_servers("tests/test_files/valid_trusted_servers".to_string());
        assert!(trusted_servers.is_ok());
        assert_eq!(
            *trusted_servers
                .expect("fail result")
                .get(&"server_uno".to_string())
                .expect("fail get"),
            Some("1111".to_string())
        );
    }

    #[test]
    fn invalid_trusted_server_file_results_in_error() {
        let trusted_servers =
            Server::build_trusted_servers("tests/test_files/invalid_csv_file".to_string());
        assert!(trusted_servers.is_err());
    }

    /*
    #[test]
    fn invalid_operators_file_results_in_error(){
        let opers = Server::build_operators("tests/test_files/invalid_csv_file".to_string());
        assert!(opers.is_err());
    }

    #[test]
    fn create_a_server_given_a_correct_config_gives_ok() {
        let config = setup_config_server();
        let server = Server::build(config);
        assert!(server.is_ok());
    }

    // #[test]
    // fn build_server_gives_ok_and_conects_to_parent(){
    //     let config_s = setup_config_server();
    //     let server_s = Server::build(config_s);

    //     let config_p = setup_config_server_parent();
    //     let server_p = Server::build(config_p);

    //     assert!(server_s.is_ok());
    //     assert!(server_p.is_ok());

        // assert_eq!(server_s.expect("").parent_name, None);
        // assert_eq!(server_p.expect("").parent_name, Some("server_uno".to_string()));
        // assert!(server_p.unwrap().clients.lock().expect("lock").contains_client(&"nico".to_string()));
        // assert!(server_s.unwrap().clients.lock().expect("lock").contains_client(&"nico".to_string()));
    // }

    // #[test]
    // fn server_and_parent_have_same_clients(){
    //     let config_s = setup_config_server();
    //     let server_s = Server::build(config_s);

    //     let config_p = setup_config_server_parent();
    //     let server_p = Server::build(config_p);

    //     assert!(server_s.unwrap().clients.lock().expect("lock").contains_client(&"nico".to_string()));
    //     assert!(server_p.unwrap().clients.lock().expect("lock").contains_client(&"nico".to_string()));
    // }
    */
}
