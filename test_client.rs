// test_client.rs - ejecutar con: cargo script test_client.rs
// O crear un proyecto separado con este main

use std::net::TcpStream;
use std::io::{BufRead, BufReader, Write};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080")
        .expect("No se pudo conectar al servidor");

    println!("Conectado al servidor");

    // Pedir lista de canciones primero
    writeln!(stream, r#"{{"command":"ListSongs"}}"#).unwrap();

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    println!("Canciones: {}", &line[..line.len().min(200)]);
    line.clear();

    // Pedir reproducción de canción ID 1
    writeln!(stream, r#"{{"command":"Play","song_id":1}}"#).unwrap();
    println!("Comando Play enviado, esperando respuesta...");

    // Leer el header AudioStart (viene como JSON en una línea)
    reader.read_line(&mut line).unwrap();
    println!("Header recibido: {}", line.trim());
    line.clear();

    // Leer los bytes de audio y contarlos
    // No los reproducimos, solo verificamos cuántos llegan
    let mut total_bytes: usize = 0;
    let mut buf = vec![0u8; 4096];

    use std::io::Read;
    loop {
        match reader.get_mut().read(&mut buf) {
            Ok(0)    => {
                println!("Conexión cerrada por el servidor");
                break;
            }
            Ok(n)    => {
                total_bytes += n;
                print!("\rBytes recibidos: {}   ", total_bytes);
                std::io::stdout().flush().unwrap();

                // Buscar si llegó el JSON de AudioEnd mezclado
                // En una implementación real el protocolo separaría esto mejor
                if total_bytes > 39_000_000 {
                    println!("\nStream completado!");
                    break;
                }
            }
            Err(e)   => {
                println!("Error leyendo: {}", e);
                break;
            }
        }
    }

    println!("Total bytes de audio recibidos: {}", total_bytes);
}