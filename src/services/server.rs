use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{BufRead, BufReader, Write};

use crate::domain::song::Song;
use crate::domain::protocol::{Request, Response};
use crate::services::song_service::{
    list_songs, search_by_name, search_by_artist, search_by_year,
};

// Ahora recibe Arc directamente en vez de Vec
pub fn start_server(shared_songs: Arc<Mutex<Vec<Song>>>, address: &str) {
    let listener = TcpListener::bind(address)
        .expect("No se pudo bindear el puerto");

    println!("Servidor escuchando en {}", address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let songs_clone = Arc::clone(&shared_songs);
                thread::spawn(move || {
                    handle_client(stream, songs_clone);
                });
            }
            Err(e) => eprintln!("Error al aceptar conexión: {}", e),
        }
    }
}

fn handle_client(stream: TcpStream, shared_songs: Arc<Mutex<Vec<Song>>>) {
    let peer = stream.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or("desconocido".to_string());

    println!("Cliente conectado: {}", peer);

    let reader = BufReader::new(stream.try_clone().expect("Error clonando stream"));
    let mut writer = stream;

    for line in reader.lines() {
        match line {
            Ok(json) => {
                let response = process_request(&json, &shared_songs);
                let response_json = serde_json::to_string(&response)
                    .unwrap_or_else(|_| r#"{"status":"Error","message":"serialize error"}"#.to_string());

                if writeln!(writer, "{}", response_json).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    println!("Cliente desconectado: {}", peer);
}

fn process_request(json: &str, shared_songs: &Arc<Mutex<Vec<Song>>>) -> Response {
    let request: Request = match serde_json::from_str(json) {
        Ok(r)  => r,
        Err(e) => return Response::Error { message: format!("JSON inválido: {}", e) },
    };

    let songs = shared_songs.lock().expect("Mutex envenenado");

    match request {
        Request::ListSongs =>
            Response::Ok { songs: list_songs(&songs) },
        Request::SearchByName { query } =>
            Response::Ok { songs: search_by_name(&songs, &query) },
        Request::SearchByArtist { query } =>
            Response::Ok { songs: search_by_artist(&songs, &query) },
        Request::SearchByYear { year } =>
            Response::Ok { songs: search_by_year(&songs, year) },
    }
}