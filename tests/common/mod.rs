use irc_2c_2022::server::ClientsInfo;
use std::collections::HashMap;

pub fn setup() -> ClientsInfo {
    let mut opers = HashMap::new();
    opers.insert("juan".to_string(), "botter".to_string());
    return ClientsInfo::new("tests".to_string(), Some("hola".to_string()), opers);
}
