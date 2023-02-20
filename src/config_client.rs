#![allow(dead_code)]
#![allow(unused_variables)]

pub struct ConfigClient {
    //maybe path for log file in the future.
    pub address: String,
    pub port: String,
}
/// Config parses the input arguments from the client
/// such as the ip address, port, and log file path
impl ConfigClient {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<ConfigClient, &'static str> {
        args.next(); //skip the name of the program

        let address = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get an address"),
        };

        let port = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a port"),
        };

        Ok(ConfigClient { address, port })
    }
}
