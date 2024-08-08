use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, BufWriter, Write};

fn handle_dcc_connection(mut stream: TcpStream) {
    // Crear un buffer de entrada para leer mensajes del cliente DCC
    let reader = BufReader::new(stream.try_clone().unwrap());

    // Crear un buffer de salida para enviar mensajes al cliente DCC
    let mut writer = BufWriter::new(stream.try_clone().unwrap());

    // Mandar un mensaje de bienvenida
    let welcome_message = "Bienvenido a mi conexión DCC p2p!\n";
    writer.write_all(welcome_message.as_bytes()).unwrap();
    writer.flush().unwrap();

    // Leer y procesar mensajes del cliente DCC
    for line in reader.lines() {
        let message = line.unwrap();
        // Procesar el mensaje
        println!("Mensaje recibido del cliente DCC: {}", message);
    }

    // Cerrar la conexión DCC
    writer.write_all(b"Adios!").unwrap();
    writer.flush().unwrap();
}

fn main() {
    // Crear un socket pasivo para escuchar conexiones DCC entrantes
    let listener = TcpListener::bind("127.0.0.1:9000").unwrap();
    println!("Escuchando en {}", "127.0.0.1:9000");

    // Aceptar conexiones DCC entrantes en un bucle infinito
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Nueva conexión DCC entrante!");
                handle_dcc_connection(stream);
            }
            Err(e) => {
                eprintln!("Error al aceptar la conexión DCC: {}", e);
            }
        }
    }
}