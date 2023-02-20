# Proyecto IRC (Internet Rust Chat)

Este proyecto es un chat de internet utilizando el [protocolo IRC](https://es.wikipedia.org/wiki/Internet_Relay_Chat) (Internet Relay Chat) escrito en Rust según el [RFC 1459](https://www.rfc-editor.org/rfc/rfc1459)

## Instrucciones 

Correr el servidor con
```bash
cargo run --bin=server -- server_x_config.csv [{neighbour_name} {neighbour_ip} {neighbour_port} {neighbour_pass}]
```
Lo que está entre corchetes son parámetros opcionales si se quiere conectar un servidor a un servidor vecino

* Si el vecino no tiene password, no se debe ingresar nada en `neighbour_pass`

* `server_x_config` es el archivo de configuración del servidor (contiene el nombre, password, ip, puerto, archivo a servidores de confianza, archivo a operadores de servidor).

Correr el cliente con
```bash
#con interfaz gráfica (GUI)
cargo run --bin=client-gtk

#sin interfaz grafica (CLI)
cargo run --bin=client -- {ip} {port} 
```

## Ejemplos de uso

* Hacer una conexión de un servidor a un servidor vecino:
```bash
#server 1
cargo run --bin=server -- server_uno_config.csv
#server 2 conectado a server 1
cargo run --bin=server -- server_dos_config.csv server_uno localhost 7878 1111
```

* Usar el cliente (sin interfaz grafica) una vez ejecutado en la terminal
```
PASS 1234
USER pepe PedroRodriguez                //nombre de usuario nombre real
NICK dragon                             //registra el identificador del usuario, su id
PRIVMSG nacho:Hola, como estas?         //envia un mensaje privado a un usuario con id nacho
QUIT me voy a comer                     //se desconecta dejando un mensaje
```


### Integrantes
* Juan Botter
* Gastón  Avila Cabrera
* Nicolas Amigo
* Alejandro Paff
