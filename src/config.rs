#![allow(dead_code)]
#![allow(unused_variables)]

use crate::app_errors;
use std::error::Error;
use std::fs;

pub struct Config {
    pub name: String,
    pub address: String,
    pub port: String,
    pub log_path: String,
    pub operators_path: String, //path to the file with the operators of the server/s
    pub trusted_servers_path: String,
    pub password: Option<String>,
    pub parent_name: Option<String>, //refers to a server conected to this one
    pub parent_ip: Option<String>,
    pub parent_port: Option<String>,
    pub parent_pwd: Option<String>,
}
/// Config parses the input arguments from the server
/// such as the ip address, port, and log file path
impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, Box<dyn Error>> {
        args.next(); //skip the name of the program

        let config_file = match args.next() {
            Some(arg) => arg,
            None => {
                return Err(Box::new(app_errors::ApplicationError(
                    "Didn't get a config file".into(),
                )))
            }
        };

        let parent_name = args.next();
        let parent_ip = args.next();
        let parent_port = args.next();
        let parent_pwd = args.next();

        if parent_ip.is_some() && parent_port.is_none() {
            return Err(Box::new(app_errors::ApplicationError(
                "Expected parent port since parent ip was provided".into(),
            )));
        }

        let contents = fs::read_to_string(config_file)?;
        let mut port = "".to_string();
        let mut address = "".to_string();
        let mut name = "".to_string();
        let mut password = None;
        let mut trusted_servers_path = "".to_string();
        let mut operators_path = "".to_string();
        let mut log_path = "".to_string();

        for line in contents.lines() {
            let l_split: Vec<String> = line
                .trim()
                .split(':')
                .map(|value| value.to_string())
                .collect();
            if l_split.len() != 2 {
                //si no es un par clave:valor lanza error
                return Err(Box::new(app_errors::ApplicationError(
                    "Invalid config format.".into(),
                )));
            }
            let l_key = match l_split.get(0) {
                Some(value) => value,
                None => {
                    return Err(Box::new(app_errors::ApplicationError(
                        "Invalid config format.".into(),
                    )))
                }
            };
            let l_value = match l_split.get(1) {
                Some(value) => value,
                None => {
                    return Err(Box::new(app_errors::ApplicationError(
                        "Invalid config format.".into(),
                    )))
                }
            };
            match l_key.as_str() {
                "port" => port = l_value.to_string(),
                "ip" => address = l_value.to_string(),
                "name" => name = l_value.to_string(),
                "password" => password = Some(l_value.to_string()),
                "trusted_servers_path" => trusted_servers_path = l_value.to_string(),
                "operators_path" => operators_path = l_value.to_string(),
                "log_path" => log_path = l_value.to_string(),
                _ => {
                    return Err(Box::new(app_errors::ApplicationError(
                        "Error reading config file.".into(),
                    )))
                }
            }
        }
        if name.is_empty() {
            return Err(Box::new(app_errors::ApplicationError(
                "Error: no server name provided".into(),
            )));
        }
        if address.is_empty() {
            return Err(Box::new(app_errors::ApplicationError(
                "Error: no address provided".into(),
            )));
        }
        if port.is_empty() {
            return Err(Box::new(app_errors::ApplicationError(
                "Error: no port provided".into(),
            )));
        }
        if operators_path.is_empty() {
            return Err(Box::new(app_errors::ApplicationError(
                "Error: no operators file provided".into(),
            )));
        }
        if trusted_servers_path.is_empty() {
            return Err(Box::new(app_errors::ApplicationError(
                "Error: no trusted servers file provided".into(),
            )));
        }
        if log_path.is_empty() {
            return Err(Box::new(app_errors::ApplicationError(
                "Error: no log file provided".into(),
            )));
        }
        Ok(Config {
            name,
            address,
            port,
            log_path,
            operators_path,
            trusted_servers_path,
            password,
            parent_name,
            parent_ip,
            parent_port,
            parent_pwd,
        })
    }
}

#[cfg(test)]
mod config_test {

    #[test]
    fn config_for_client_returns_error_if_no_config_file() {
        let args = vec!["server".to_string()];
        let config = super::Config::build(args.into_iter());
        assert!(config.is_err());
    }

    #[test]
    fn config_for_client_returns_error_if_no_port() {
        let args = vec!["server".to_string(), "config.csv".to_string()];
        let config = super::Config::build(args.into_iter());
        assert!(config.is_err());
    }

    #[test]
    fn config_for_client_returns_error_if_no_address() {
        let args = vec![
            "server".to_string(),
            "config.csv".to_string(),
            "port".to_string(),
        ];
        let config = super::Config::build(args.into_iter());
        assert!(config.is_err());
    }

    #[test]
    fn config_for_client_returns_error_if_no_name() {
        let args = vec![
            "server".to_string(),
            "config.csv".to_string(),
            "port".to_string(),
            "address".to_string(),
        ];
        let config = super::Config::build(args.into_iter());
        assert!(config.is_err());
    }

    #[test]
    fn config_for_client_returns_error_if_no_operators_path() {
        let args = vec![
            "server".to_string(),
            "config.csv".to_string(),
            "port".to_string(),
            "address".to_string(),
            "name".to_string(),
        ];
        let config = super::Config::build(args.into_iter());
        assert!(config.is_err());
    }

    #[test]
    fn config_for_client_returns_error_if_no_trusted_servers_path() {
        let args = vec![
            "server".to_string(),
            "config.csv".to_string(),
            "port".to_string(),
            "address".to_string(),
            "name".to_string(),
            "operators_path".to_string(),
        ];
        let config = super::Config::build(args.into_iter());
        assert!(config.is_err());
    }

    // #[test]
    // fn invalid_config_format_returns_error() {
    //     let config_file_test_path = test_files::invalid_config_file();
    //     let config = super::Config::build(args.into_iter());
    //     assert!(config.is_err());
    // }

    // #[test]
    // fn valid_config_format_returns_config() {
    //     let config_file_test_path = test_files::valid_config_file();
    //     let config = super::Config::build(args.into_iter());
    //     assert!(config.is_ok());
    // }
}
