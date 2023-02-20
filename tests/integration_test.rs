mod common;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use irc_2c_2022::server::{client_s::ClientS, logger::Logger};
use std::sync::Mutex;

#[test]
fn test_add_user_functions_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8087").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8087").expect("")));

    let client = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream,
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");

    assert!(server
        .lock()
        .expect("")
        .contains_client(&"nico".to_string()));
}

#[test]
fn test_add_user_returns_error_with_incorrect_password() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8086").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8086").expect("")));

    let client = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream,
            Some("incorecta".to_string()),
            0,
            None,
        )
        .expect_err("");
}

#[test]
fn test_add_user_returns_error_with_nick_collision() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8088").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8088").expect("")));

    let client = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client.clone(),
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream,
            Some("hola".to_string()),
            0,
            None,
        )
        .expect_err("");
}

#[test]
fn test_privmsg_to_nonexistant_user_returns_error() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8097").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8097").expect("")));

    let client = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream,
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");

    server
        .lock()
        .expect("")
        .send_privmsg(
            "nico".to_string(),
            "juan".to_string(),
            "Hola\n".to_string(),
            None,
        )
        .expect_err("");
}

#[test]
fn test_quit_user_functions_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8089").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8089").expect("")));

    let client = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream,
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .quit_client("nico".to_string(), None, None)
        .expect("");

    assert!(!server
        .lock()
        .expect("")
        .contains_client(&"nico".to_string()))
}

#[test]
fn test_create_channel_functions_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8090").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8090").expect("")));

    let client = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "nico".to_string(),
            Some(stream),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    assert!(server
        .lock()
        .expect("")
        .contains_channel(&"#channel".to_string()))
}

#[test]
fn test_delete_channels_function_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8091").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8091").expect("")));

    let client = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "nico".to_string(),
            Some(stream),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .part(
            vec!["#channel".to_string()],
            Some("nico".to_string()),
            None,
            None,
        )
        .expect("");
    assert!(!server
        .lock()
        .expect("")
        .contains_channel(&"#channel".to_string()))
}

#[test]
fn test_delete_channels_work_with_multiple_clients() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8092").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8092").expect("")));

    let client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");
    let client2 = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .add_client(
            "juan".to_string(),
            client2,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "nico".to_string(),
            Some(stream.clone()),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "juan".to_string(),
            Some(stream),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .part(
            vec!["#channel".to_string()],
            Some("nico".to_string()),
            None,
            None,
        )
        .expect("");
    assert!(server
        .lock()
        .expect("")
        .contains_channel(&"#channel".to_string()));
    server
        .lock()
        .expect("")
        .part(
            vec!["#channel".to_string()],
            Some("juan".to_string()),
            None,
            None,
        )
        .expect("");
    assert!(!server
        .lock()
        .expect("")
        .contains_channel(&"#channel".to_string()));
}

#[test]
fn test_correct_oper_functions_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8093").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8093").expect("")));

    let client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    match server
        .lock()
        .expect("")
        .oper_login(&"juan".to_string(), &"botter".to_string())
    {
        Ok(_) => return,
        Err(_) => panic!("Error login operator"),
    };
}

#[test]
fn test_incorrect_oper_functions_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8094").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8094").expect("")));

    let client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    match server
        .lock()
        .expect("")
        .oper_login(&"nico".to_string(), &"amigo".to_string())
    {
        Ok(_) => panic!("Operator should be invalid"),
        Err(_) => return,
    };
}

#[test]
fn test_private_message_functions_beetween_users() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let listener = TcpListener::bind("localhost:8095").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8095").expect("")));

    let client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");
    let client2 = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .add_client(
            "juan".to_string(),
            client2,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    let client_stream = listener.incoming().nth(0).expect("").expect("");
    server
        .lock()
        .expect("")
        .send_privmsg(
            "juan".to_string(),
            "nico".to_string(),
            "Hola\n".to_string(),
            None,
        )
        .expect("");
    let mut buf = String::new();
    let mut reader = BufReader::new(client_stream);
    reader.read_line(&mut buf).expect("");
    assert_eq!(buf, ":juan PRIVMSG nico Hola\n");
}

#[test]
fn test_private_message_functions_beetween_users_in_channel() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let listener = TcpListener::bind("localhost:8096").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8096").expect("")));

    let client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");
    let client2 = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .add_client(
            "juan".to_string(),
            client2,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "nico".to_string(),
            Some(stream.clone()),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "juan".to_string(),
            Some(stream),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    let client_stream = listener.incoming().nth(0).expect("").expect("");
    server
        .lock()
        .expect("")
        .send_privmsg(
            "juan".to_string(),
            "#channel".to_string(),
            "Hola\n".to_string(),
            None,
        )
        .expect("");
    let mut buf = String::new();
    let mut reader = BufReader::new(client_stream);
    reader.read_line(&mut buf).expect("");
    assert_eq!(buf, ":juan PRIVMSG #channel Hola\n");
}

#[test]
fn test_names_functions_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let listener = TcpListener::bind("localhost:8089").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8089").expect("")));

    let client = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client.clone(),
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "nico".to_string(),
            Some(stream.clone()),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .names(vec!["#channel".to_string()], "nico".to_string());

    let client_stream = listener.incoming().nth(0).expect("").expect("");

    let mut buf = String::new();
    let mut reader = BufReader::new(client_stream);
    reader.read_line(&mut buf).expect("");
    assert_eq!(buf, "#channel: nico;\n");
}

#[test]
fn test_list_functions_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let listener = TcpListener::bind("localhost:8098").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8098").expect("")));

    let client = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client.clone(),
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "nico".to_string(),
            Some(stream.clone()),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .topic(
            "#channel".to_string(),
            Some("new topic".to_string()),
            "nico".to_string(),
        )
        .expect("");
    server
        .lock()
        .expect("")
        .list(vec!["#channel".to_string()], "nico".to_string());

    let client_stream = listener.incoming().nth(0).expect("").expect("");

    let mut buf = String::new();
    let mut reader = BufReader::new(client_stream);
    reader.read_line(&mut buf).expect("");
    assert_eq!(buf, "#channel: new topic\n");
}

#[test]
fn test_kick_functions_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let listener = TcpListener::bind("localhost:8099").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8099").expect("")));

    let client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");
    let client2 = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .add_client(
            "juan".to_string(),
            client2,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "nico".to_string(),
            Some(stream.clone()),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "juan".to_string(),
            Some(stream),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .kick(
            "#channel".to_string(),
            "juan".to_string(),
            Some("mensaje".to_string()),
            Some("nico".to_string()),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .names(vec!["#channel".to_string()], "nico".to_string());

    let client_stream = listener.incoming().nth(0).expect("").expect("");

    let mut buf = String::new();
    let mut reader = BufReader::new(client_stream);
    reader.read_line(&mut buf).expect("");
    buf.clear();
    reader.read_line(&mut buf).expect("");
    assert_eq!(buf, "#channel: nico;\n");
}

#[test]
fn test_invite_functions_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8100").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8100").expect("")));

    let client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");
    let client2 = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .add_client(
            "juan".to_string(),
            client2,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "nico".to_string(),
            Some(stream.clone()),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .send_invite(
            "#channel".to_string(),
            "juan".to_string(),
            Some("nico".to_string()),
            None,
            None,
        )
        .expect("");
}

#[test]
fn test_invite_fails_without_oper_status() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8101").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8101").expect("")));

    let client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");
    let client2 = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");
    let client3 = ClientS::new(server.clone(), stream.clone(), HashMap::new(), logger).expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .add_client(
            "juan".to_string(),
            client2,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .add_client(
            "pedro".to_string(),
            client3,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "nico".to_string(),
            Some(stream.clone()),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "juan".to_string(),
            Some(stream.clone()),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .send_invite(
            "#channel".to_string(),
            "pedro".to_string(),
            Some("juan".to_string()),
            None,
            None,
        )
        .expect_err("");
}

#[test]
fn test_invite_fails_if_nick_doesnt_exist() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8102").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8102").expect("")));

    let client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");
    let client2 = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .add_client(
            "juan".to_string(),
            client2,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "nico".to_string(),
            Some(stream.clone()),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "juan".to_string(),
            Some(stream.clone()),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .send_invite(
            "#channel".to_string(),
            "pedro".to_string(),
            Some("juan".to_string()),
            None,
            None,
        )
        .expect_err("");
}

#[test]
fn test_topic_functions_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8103").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8103").expect("")));

    let client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .join_channel(
            "nico".to_string(),
            Some(stream.clone()),
            "#channel".to_string(),
            None,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .topic(
            "#channel".to_string(),
            Some("new_topic".to_string()),
            "nico".to_string(),
        )
        .expect("");
}

#[test]
fn test_topic_fails_if_not_channel() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let _listener = TcpListener::bind("localhost:8104").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8104").expect("")));

    let client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");

    server
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .topic(
            "#channel".to_string(),
            Some("new_topic".to_string()),
            "nico".to_string(),
        )
        .expect_err("");
}

#[test]
fn test_who_works_with_one_client() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let listener = TcpListener::bind("localhost:8105").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8105").expect("")));

    let mut client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");

    client.realname = Some("juan botter".to_string());
    client.user = Some("juan".to_string());

    server.clone()
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .who("*".to_string(), "nico".to_string()).expect("");

        let client_stream = listener.incoming().nth(0).expect("").expect("");

        let mut buf = String::new();
        let mut reader = BufReader::new(client_stream);
        reader.read_line(&mut buf).expect("");
        assert_eq!(buf, "juan nico: juan botter;\n");
    
}

#[test]
fn test_whois_functions_correctly() {
    let server = Arc::new(Mutex::new(common::setup()));

    let logger = Arc::new(Mutex::new(Logger::build("logs/log1.txt".to_string())));

    let listener = TcpListener::bind("localhost:8107").expect("");
    let stream = Arc::new(Mutex::new(TcpStream::connect("localhost:8107").expect("")));

    let mut client = ClientS::new(
        server.clone(),
        stream.clone(),
        HashMap::new(),
        logger.clone(),
    )
    .expect("");

    client.realname = Some("juan botter".to_string());
    client.user = Some("juan".to_string());

    server.clone()
        .lock()
        .expect("")
        .add_client(
            "nico".to_string(),
            client,
            stream.clone(),
            Some("hola".to_string()),
            0,
            None,
        )
        .expect("");
    server
        .lock()
        .expect("")
        .whois("nico".to_string(), "nico".to_string()).expect("");

        let client_stream = listener.incoming().nth(0).expect("").expect("");

        let mut buf = String::new();
        let mut reader = BufReader::new(client_stream);
        reader.read_line(&mut buf).expect("");
        assert_eq!(buf, "juan nico: juan botter\n");
    
}
