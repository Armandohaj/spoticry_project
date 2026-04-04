use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{BufRead, BufReader, Write};

use crate::domain::song::Song;
use crate::domain::protocol::{Request, Response};
use crate::services::song_service::{
    list_songs, search_by_name, search_by_artist, search_by_year,
};
use crate::audio::decoder::decode_mp3;

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

    let reader     = BufReader::new(stream.try_clone().expect("Error clonando stream"));
    let mut writer = stream;

    for line in reader.lines() {
        match line {
            Ok(json) => {
                handle_request(&json, &shared_songs, &mut writer);
            }
            Err(_) => break,
        }
    }

    println!("Cliente desconectado: {}", peer);
}

// Separamos el manejo en su propia función porque Play necesita
// enviar múltiples respuestas (header + bytes), no solo una
fn handle_request(
    json:          &str,
    shared_songs:  &Arc<Mutex<Vec<Song>>>,
    writer:        &mut TcpStream,
) {
    let request: Request = match serde_json::from_str(json) {
        Ok(r)  => r,
        Err(e) => {
            send_response(writer, &Response::Error {
                message: format!("JSON inválido: {}", e)
            });
            return;
        }
    };

    match request {
        Request::ListSongs => {
            let songs = shared_songs.lock().unwrap().clone();
            send_response(writer, &Response::Ok { songs: list_songs(&songs) });
        }

        Request::SearchByName { query } => {
            let songs = shared_songs.lock().unwrap().clone();
            send_response(writer, &Response::Ok { songs: search_by_name(&songs, &query) });
        }

        Request::SearchByArtist { query } => {
            let songs = shared_songs.lock().unwrap().clone();
            send_response(writer, &Response::Ok { songs: search_by_artist(&songs, &query) });
        }

        Request::SearchByYear { year } => {
            let songs = shared_songs.lock().unwrap().clone();
            send_response(writer, &Response::Ok { songs: search_by_year(&songs, year) });
        }

        Request::Play { song_id } => {
            handle_play(song_id, shared_songs, writer);
        }

        Request::Stop { song_id } => {
            handle_stop(song_id, shared_songs, writer);
        }
    }
}

fn handle_play(
    song_id:      u32,
    shared_songs: &Arc<Mutex<Vec<Song>>>,
    writer:       &mut TcpStream,
) {
    // Buscar la canción y obtener su file_path
    // Hacemos esto en un bloque separado para liberar el lock antes de decodificar
    let file_path = {
        let mut songs = shared_songs.lock().unwrap();

        // Buscar la canción por ID
        let song = songs.iter_mut().find(|s| s.id == song_id);

        match song {
            None => {
                send_response(writer, &Response::Error {
                    message: format!("Canción con ID {} no encontrada", song_id)
                });
                return;
            }
            Some(s) => {
                // Marcar como en reproducción
                s.is_playing = true;
                s.file_path.clone()
            }
        }
    }; // ← lock se libera aquí

    // Decodificar el MP3 (puede tardar varios segundos)
    println!("Decodificando: {}", file_path);
    let decoded = match decode_mp3(&file_path) {
        Ok(d)  => d,
        Err(e) => {
            // Si falla, desmarcar is_playing
            if let Ok(mut songs) = shared_songs.lock() {
                if let Some(s) = songs.iter_mut().find(|s| s.id == song_id) {
                    s.is_playing = false;
                }
            }
            send_response(writer, &Response::Error { message: e });
            return;
        }
    };

    println!(
        "Audio decodificado: {} bytes, {}Hz, {} canales",
        decoded.samples.len(),
        decoded.info.sample_rate,
        decoded.info.channels
    );

    // Enviar header con información del audio
    send_response(writer, &Response::AudioStart {
        sample_rate: decoded.info.sample_rate,
        channels:    decoded.info.channels,
        bits:        decoded.info.bits,
    });

    // Enviar los bytes de audio en bloques de 4096 bytes
    // No se envía todo de una vez porque puede ser muy grande (30MB+)
    let chunk_size = 4096;
    for chunk in decoded.samples.chunks(chunk_size) {
        if writer.write_all(chunk).is_err() {
            println!("Error enviando audio, cliente desconectado");
            break;
        }
    }

    // Marcar como no reproduciendo al terminar
    if let Ok(mut songs) = shared_songs.lock() {
        if let Some(s) = songs.iter_mut().find(|s| s.id == song_id) {
            s.is_playing = false;
        }
    }

    // Notificar al cliente que terminó
    send_response(writer, &Response::AudioEnd);
    println!("Stream de audio finalizado para canción {}", song_id);
}

fn handle_stop(
    song_id:      u32,
    shared_songs: &Arc<Mutex<Vec<Song>>>,
    writer:       &mut TcpStream,
) {
    let mut songs = shared_songs.lock().unwrap();
    if let Some(s) = songs.iter_mut().find(|s| s.id == song_id) {
        s.is_playing = false;
        send_response(writer, &Response::Ok { songs: vec![s.clone()] });
    } else {
        send_response(writer, &Response::Error {
            message: format!("Canción {} no encontrada", song_id)
        });
    }
}

// Función auxiliar para serializar y enviar una respuesta JSON
fn send_response(writer: &mut TcpStream, response: &Response) {
    let json = serde_json::to_string(response)
        .unwrap_or_else(|_| r#"{"status":"Error","message":"serialize error"}"#.to_string());

    if let Err(e) = writeln!(writer, "{}", json) {
        eprintln!("Error enviando respuesta: {}", e);
    }
}