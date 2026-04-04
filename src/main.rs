mod audio {
    pub mod decoder;
}

mod domain {
    pub mod song;
    pub mod playlist;
    pub mod protocol;
}
mod services {
    pub mod song_service;
    pub mod server;
}
mod storage {
    pub mod file_loader;
}
mod utils {
    pub mod id_generator;
}
mod cli {
    pub mod menu;
}

use std::sync::{Arc, Mutex};
use std::thread;

use domain::song::Song;
use utils::id_generator::{new_id_generator, next_id};
use storage::file_loader::load_songs;
use services::server::start_server;
use services::song_service::*;
use cli::menu::*;

fn main() {
    let mut id_gen = new_id_generator();
    let songs = load_songs("canciones.json", &mut id_gen);

    println!("Canciones cargadas: {}", songs.len());

    // Compartir el generador para que nunca se repitan IDs
    let shared_id_gen  = Arc::new(Mutex::new(id_gen));
    let shared_songs   = Arc::new(Mutex::new(songs));

    let songs_for_server = Arc::clone(&shared_songs);
    thread::spawn(move || {
        start_server(songs_for_server, "127.0.0.1:8080");
    });

    loop {
        show_menu();
        let option = read_input("Opción: ");

        match option.as_str() {
            "1" => {
                let songs = shared_songs.lock().unwrap();
                print_songs(&songs);
            }

            "2" => {
                let name      = read_input("Nombre: ");
                let artist    = read_input("Artista: ");
                let genre     = read_input("Género: ");
                let year: u32 = read_input("Año: ").parse().unwrap_or(0);
                let path      = read_input("Ruta del archivo: ");

                // Usar el mismo generador que usó load_songs
                let id = next_id(&mut shared_id_gen.lock().unwrap());

                let song = Song {
                    id,
                    name,
                    artist,
                    genre,
                    year,
                    file_path: path,
                    is_playing: false,
                };

                shared_songs.lock().unwrap().push(song);
                println!("Canción agregada con ID {}.", id);
            }

            "3" => {
                let id: u32 = read_input("ID: ").parse().unwrap_or(0);
                let mut songs = shared_songs.lock().unwrap();
                let original_len = songs.len();
                *songs = remove_song(songs.clone(), id);
                if songs.len() < original_len {
                    println!("Canción eliminada.");
                } else {
                    println!("No se pudo eliminar (está reproduciéndose o no existe).");
                }
            }

            "4" => {
                let query = read_input("Nombre: ");
                let songs = shared_songs.lock().unwrap();
                print_songs(&search_by_name(&songs, &query));
            }

            "5" => {
                let query = read_input("Artista: ");
                let songs = shared_songs.lock().unwrap();
                print_songs(&search_by_artist(&songs, &query));
            }

            "6" => {
                let query = read_input("Género: ");
                let songs = shared_songs.lock().unwrap();
                print_songs(&search_by_genre(&songs, &query));
            }

            "7" => {
                let year: u32 = read_input("Año: ").parse().unwrap_or(0);
                let songs = shared_songs.lock().unwrap();
                print_songs(&search_by_year(&songs, year));
            }

            "8" => {
                println!("Saliendo...");
                break;
            }

            _ => println!("Opción inválida"),
        }
    }
}