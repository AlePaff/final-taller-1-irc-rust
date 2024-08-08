//! Abre un puerto TCP en el puerto asignado por argv.
//! Escribe las lineas recibidas a stdout y las manda mediante el socket.

use std::env::args;
use std::io::{BufRead, BufReader, Read};
use std::net::{TcpListener, TcpStream};

static SERVER_ARGS: usize = 2;

fn main() -> Result<(), ()> {
    let argv = args().collect::<Vec<String>>();     //pide argumentos por la linea de comendos
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("{:?} <host> <puerto>", app_name);
        return Err(());
    }

    let address = "0.0.0.0:".to_owned() + &argv[1];     //el 0.0.0.0 es para que escuche en todas las interfaces de red de mi computadora
    server_run(&address).unwrap();
    Ok(())
}

fn server_run(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    // accept devuelve una tupla (TcpStream, std::net::SocketAddr)
    let connection = listener.accept()?;            //con esto solo puedo hacer un cliente, un servidor y un solo cliente
    let mut client_stream : TcpStream = connection.0;       //elijo el primer elemento de la tupla
    // TcpStream implementa el trait Read, así que podemos trabajar como si fuera un archivo
    handle_client(&mut client_stream)?;
    Ok(())
}

fn handle_client(stream: &mut dyn Read) -> std::io::Result<()> {
    //dyn porque el tipo de dato es dinamico
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();
    // iteramos las lineas que recibimos de nuestro cliente
    while let Some(line) = lines.next() {       //se invoca cada vez que obtenga una linea. Deja de recibir cuando se cierra la conexion con el cliente
        println!("Recibido: {:?}", line);
    }
    Ok(())
}
