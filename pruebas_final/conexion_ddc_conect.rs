use std::io::{self, Read, Write};
use std::net::{TcpStream, Shutdown};
use std::thread;

fn main() -> io::Result<()> {
    // Conectar al servidor DCC
    let mut stream = TcpStream::connect("127.0.0.1:9000")?;

    // Enviar solicitud de conexión DCC al servidor
    let msg = "DCC CHAT IPv4 127.0.0.1 1234\r\n";
    stream.write(msg.as_bytes())?;
    println!("Solicitud de DCC enviada");

    // Leer respuesta del servidor
    let mut buffer = [0; 512];
    stream.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..]);
    println!("Respuesta del servidor: {}", response);

    // Parsear la dirección IP y puerto del remitente
    let tokens: Vec<&str> = response.trim().split(' ').collect();
    let ip = tokens[4];
    let port = tokens[5].parse::<u16>().unwrap();

    // Conectarse al remitente
    let mut receiver = TcpStream::connect((ip, port))?;
    println!("Conectado al remitente {}:{}", ip, port);

    // Iniciar un hilo para recibir datos del remitente
    let mut receiver_clone = receiver.try_clone()?;
    thread::spawn(move || {
        loop {
            let mut buffer = [0; 512];
            match receiver_clone.read(&mut buffer) {
                Ok(0) => {
                    println!("El remitente ha cerrado la conexión");
                    break;
                }
                Ok(n) => {
                    let msg = String::from_utf8_lossy(&buffer[..n]);
                    println!("Remitente dice: {}", msg);
                }
                Err(e) => {
                    println!("Error al recibir datos del remitente: {}", e);
                    break;
                }
            }
        }
    });

    // Leer entrada del usuario y enviar datos al remitente
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let msg = input.trim().to_string();
        receiver.write(msg.as_bytes())?;
    }

    // Cerrar la conexión
    receiver.shutdown(Shutdown::Both)?;
    Ok(())
}