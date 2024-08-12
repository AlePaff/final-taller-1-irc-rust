use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use std::env;

const PACKET_SIZE: usize = 4; // 4 bytes (32 bits)
const FILE_PATH: &str = "receptor/sample_file.txt";

fn main() -> std::io::Result<()> {
    // Canal para comunicar la dirección del servidor al cliente
    let (tx, rx) = mpsc::channel();

    
println!("Current directory: {}", env::current_dir().unwrap().display());


    // Inicia el servidor en un hilo
    let server = thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to address");
        let local_addr = listener.local_addr().expect("Failed to get local address");

        // Enviar la dirección del servidor al cliente
        tx.send(local_addr).expect("Failed to send server address");

        println!("Server listening on {}", local_addr);

        // Espera una conexión entrante
        if let Ok((mut stream, _addr)) = listener.accept() {
            println!("Client connected");

            loop {
                let mut buffer = [0u8; PACKET_SIZE];
                match stream.read_exact(&mut buffer) {
                    Ok(_) => {
                        // Imprimir datos recibidos y enviar ACK
                        println!("Received data: {:?}", buffer);
                        stream.write_all(&buffer).expect("Failed to send ACK");
                    }
                    Err(e) => {
                        eprintln!("Error reading from stream: {}", e);
                        break;
                    }
                }
            }
        } else {
            eprintln!("Failed to accept connection");
        }
    });

    // Esperar para asegurar que el servidor está escuchando
    thread::sleep(Duration::from_secs(1));

    // Inicia el cliente
    let client = thread::spawn(move || {
        // Obtener la dirección del servidor del canal
        let server_addr = rx.recv().expect("Failed to receive server address");
        let mut stream = TcpStream::connect(server_addr).expect("Failed to connect to server");

        // Abre el archivo y envía su contenido en paquetes de 32 bits
        let path = Path::new(FILE_PATH);
        let mut file = File::open(&path).expect("Failed to open file");

        let mut buffer = [0u8; PACKET_SIZE];
        loop {
            match file.read_exact(&mut buffer) {
                Ok(_) => {
                    stream.write_all(&buffer).expect("Failed to send data");
                    // Espera ACK
                    let mut ack = [0u8; PACKET_SIZE];
                    stream.read_exact(&mut ack).expect("Failed to receive ACK");
                    println!("Received ACK: {:?}", ack);
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Fin del archivo
                    break;
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    break;
                }
            }
        }
    });

    // Espera a que los hilos terminen
    server.join().expect("Server thread panicked");
    client.join().expect("Client thread panicked");

    Ok(())
}
