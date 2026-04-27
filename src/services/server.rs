use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{BufRead, BufReader, Write};

use crate::domain::song::Song;
use crate::domain::playlist::Playlist;
use crate::domain::protocol::{Request, Response};
use crate::services::song_service::{
    list_songs, search_by_name, search_by_year_range, search_by_genre_ranked,
};
use crate::services::playlist_service as pl_svc;
use crate::storage::file_loader::save_playlists;
use crate::audio::decoder::decode_mp3;

pub fn start_server(
    shared_songs:     Arc<Mutex<Vec<Song>>>,
    shared_playlists: Arc<Mutex<Vec<Playlist>>>,
    address:          &str,
) {
    let listener = TcpListener::bind(address)
        .expect("No se pudo bindear el puerto");

    println!("Servidor escuchando en {}", address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let songs_clone     = Arc::clone(&shared_songs);
                let playlists_clone = Arc::clone(&shared_playlists);
                thread::spawn(move || {
                    handle_client(stream, songs_clone, playlists_clone);
                });
            }
            Err(e) => eprintln!("Error al aceptar conexión: {}", e),
        }
    }
}

fn handle_client(
    stream:           TcpStream,
    shared_songs:     Arc<Mutex<Vec<Song>>>,
    shared_playlists: Arc<Mutex<Vec<Playlist>>>,
) {
    let peer = stream.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or("desconocido".to_string());

    println!("Cliente conectado: {}", peer);

    let reader     = BufReader::new(stream.try_clone().expect("Error clonando stream"));
    let mut writer = stream;

    for line in reader.lines() {
        match line {
            Ok(json) => handle_request(&json, &shared_songs, &shared_playlists, &mut writer),
            Err(_)   => break,
        }
    }

    println!("Cliente desconectado: {}", peer);
}

fn handle_request(
    json:             &str,
    shared_songs:     &Arc<Mutex<Vec<Song>>>,
    shared_playlists: &Arc<Mutex<Vec<Playlist>>>,
    writer:           &mut TcpStream,
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
        // ── Canciones ──────────────────────────────────────────────
        Request::ListSongs => {
            let songs = shared_songs.lock().unwrap().clone();
            send_response(writer, &Response::Ok { songs: list_songs(&songs) });
        }
        Request::SearchByName { query } => {
            let songs = shared_songs.lock().unwrap().clone();
            send_response(writer, &Response::Ok {
                songs: search_by_name(&songs, &query)
            });
        }
        Request::SearchByYearRange { from, to } => {
            let songs = shared_songs.lock().unwrap().clone();
            send_response(writer, &Response::Ok {
                songs: search_by_year_range(&songs, from, to)
            });
        }
        Request::SearchByGenre { query } => {
            let songs = shared_songs.lock().unwrap().clone();
            send_response(writer, &Response::Ok {
                songs: search_by_genre_ranked(&songs, &query)
            });
        }
        Request::Play { song_id } => {
            handle_play(song_id, shared_songs, writer);
        }
        Request::Stop { song_id } => {
            handle_stop(song_id, shared_songs, writer);
        }

        // ── Playlists ──────────────────────────────────────────────
        Request::CreatePlaylist { name } => {
            let mut playlists = shared_playlists.lock().unwrap();
            match pl_svc::create(&playlists, &name) {
                Ok(new) => {
                    *playlists = new;
                    save_playlists("playlists.json", &playlists); // ← guardar
                    send_response(writer, &Response::PlaylistList {
                        playlists: pl_svc::list_names(&playlists)
                    });
                }
                Err(msg) => send_response(writer, &Response::Error { message: msg }),
            }
        }

        Request::DeletePlaylist { name } => {
            let mut playlists = shared_playlists.lock().unwrap();
            match pl_svc::delete(&playlists, &name) {
                Ok(new) => {
                    *playlists = new;
                    save_playlists("playlists.json", &playlists); // ← guardar
                    send_response(writer, &Response::PlaylistList {
                        playlists: pl_svc::list_names(&playlists)
                    });
                }
                Err(msg) => send_response(writer, &Response::Error { message: msg }),
            }
        }

        Request::ListPlaylists => {
            let playlists = shared_playlists.lock().unwrap();
            send_response(writer, &Response::PlaylistList {
                playlists: pl_svc::list_names(&playlists)
            });
        }

        Request::AddToPlaylist { playlist_name, song_id } => {
            let songs         = shared_songs.lock().unwrap().clone();
            let mut playlists = shared_playlists.lock().unwrap();

            match songs.iter().find(|s| s.id == song_id).cloned() {
                None => send_response(writer, &Response::Error {
                    message: format!("Canción {} no encontrada", song_id)
                }),
                Some(s) => match pl_svc::add_song(&playlists, &playlist_name, s) {
                    Ok(new) => {
                        *playlists = new;
                        save_playlists("playlists.json", &playlists); // ← guardar
                        match pl_svc::get(&playlists, &playlist_name) {
                            Ok(pl)   => send_response(writer, &Response::PlaylistOk { playlist: pl }),
                            Err(msg) => send_response(writer, &Response::Error { message: msg }),
                        }
                    }
                    Err(msg) => send_response(writer, &Response::Error { message: msg }),
                }
            }
        }

        Request::RemoveFromPlaylist { playlist_name, song_id } => {
            let mut playlists = shared_playlists.lock().unwrap();
            match pl_svc::remove_song(&playlists, &playlist_name, song_id) {
                Ok(new) => {
                    *playlists = new;
                    save_playlists("playlists.json", &playlists); // ← guardar
                    match pl_svc::get(&playlists, &playlist_name) {
                        Ok(pl)   => send_response(writer, &Response::PlaylistOk { playlist: pl }),
                        Err(msg) => send_response(writer, &Response::Error { message: msg }),
                    }
                }
                Err(msg) => send_response(writer, &Response::Error { message: msg }),
            }
        }

        Request::GetPlaylist { playlist_name } => {
            let playlists = shared_playlists.lock().unwrap();
            match pl_svc::get(&playlists, &playlist_name) {
                Ok(pl)   => send_response(writer, &Response::PlaylistOk { playlist: pl }),
                Err(msg) => send_response(writer, &Response::Error { message: msg }),
            }
        }

        Request::FilterPlaylist { playlist_name, field, query } => {
            let playlists = shared_playlists.lock().unwrap();
            match pl_svc::filter(&playlists, &playlist_name, &field, &query) {
                Ok(pl)   => send_response(writer, &Response::PlaylistOk { playlist: pl }),
                Err(msg) => send_response(writer, &Response::Error { message: msg }),
            }
        }

        Request::SortPlaylist { playlist_name, by } => {
            let mut playlists = shared_playlists.lock().unwrap();
            match pl_svc::sort(&playlists, &playlist_name, &by) {
                Ok(new) => {
                    *playlists = new;
                    save_playlists("playlists.json", &playlists); // ← guardar
                    match pl_svc::get(&playlists, &playlist_name) {
                        Ok(pl)   => send_response(writer, &Response::PlaylistOk { playlist: pl }),
                        Err(msg) => send_response(writer, &Response::Error { message: msg }),
                    }
                }
                Err(msg) => send_response(writer, &Response::Error { message: msg }),
            }
        }

        Request::SummarizePlaylist { playlist_name } => {
            let playlists = shared_playlists.lock().unwrap();
            match pl_svc::summarize(&playlists, &playlist_name) {
                Ok(info) => send_response(writer, &Response::PlaylistInfo { info }),
                Err(msg) => send_response(writer, &Response::Error { message: msg }),
            }
        }
    }
}

fn handle_play(
    song_id:      u32,
    shared_songs: &Arc<Mutex<Vec<Song>>>,
    writer:       &mut TcpStream,
) {
    let file_path = {
        let mut songs = shared_songs.lock().unwrap();
        let song = songs.iter_mut().find(|s| s.id == song_id);
        match song {
            None => {
                send_response(writer, &Response::Error {
                    message: format!("Canción con ID {} no encontrada", song_id)
                });
                return;
            }
            Some(s) => { s.is_playing = true; s.file_path.clone() }
        }
    };

    println!("Decodificando: {}", file_path);
    let decoded = match decode_mp3(&file_path) {
        Ok(d)  => d,
        Err(e) => {
            if let Ok(mut songs) = shared_songs.lock() {
                if let Some(s) = songs.iter_mut().find(|s| s.id == song_id) {
                    s.is_playing = false;
                }
            }
            send_response(writer, &Response::Error { message: e });
            return;
        }
    };

    println!("Audio decodificado: {} bytes, {}Hz, {} canales",
             decoded.samples.len(), decoded.info.sample_rate, decoded.info.channels);

    send_response(writer, &Response::AudioStart {
        sample_rate: decoded.info.sample_rate,
        channels:    decoded.info.channels,
        bits:        decoded.info.bits,
        total_bytes: decoded.samples.len(),
    });

    for chunk in decoded.samples.chunks(4096) {
        if writer.write_all(chunk).is_err() {
            println!("Error enviando audio, cliente desconectado");
            break;
        }
    }

    if let Ok(mut songs) = shared_songs.lock() {
        if let Some(s) = songs.iter_mut().find(|s| s.id == song_id) {
            s.is_playing = false;
        }
    }

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

fn send_response(writer: &mut TcpStream, response: &Response) {
    let json = serde_json::to_string(response)
        .unwrap_or_else(|_| r#"{"status":"Error","message":"serialize error"}"#.to_string());
    if let Err(e) = writeln!(writer, "{}", json) {
        eprintln!("Error enviando respuesta: {}", e);
    }
}