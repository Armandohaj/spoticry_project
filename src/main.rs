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
    pub mod playlist_service;
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
use domain::playlist::Playlist;
use utils::id_generator::{new_id_generator, next_id};

use storage::file_loader::{load_songs, save_songs, load_playlists, save_playlists};
use services::server::start_server;
use services::song_service::*;
use services::playlist_service as pl_svc;
use cli::menu::{read_input, show_menu, normalize_path};

fn main() {
    let mut id_gen = new_id_generator();
    let songs = load_songs("canciones.json", &mut id_gen);
    println!("Canciones cargadas: {}", songs.len());

    // Cargar playlists usando las canciones ya cargadas para reconstruirlas
    let playlists = load_playlists("playlists.json", &songs);
    println!("Playlists cargadas: {}", playlists.len());

    let shared_id_gen    = Arc::new(Mutex::new(id_gen));
    let shared_songs     = Arc::new(Mutex::new(songs));
    let shared_playlists = Arc::new(Mutex::new(playlists));

    let songs_clone     = Arc::clone(&shared_songs);
    let playlists_clone = Arc::clone(&shared_playlists);

    thread::spawn(move || {
        start_server(songs_clone, playlists_clone, "0.0.0.0:8080");
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
                let biblioteca = read_input("Ruta de la carpeta biblioteca (Enter para 'biblioteca'): ");
                let biblioteca = if biblioteca.is_empty() { "biblioteca".to_string() } else { biblioteca };

                // Leer archivos de la carpeta biblioteca
                let archivos: Vec<String> = std::fs::read_dir(&biblioteca)
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .map(|e| e.path())
                            .filter(|p| p.is_file())
                            .filter(|p| {
                                p.extension()
                                    .and_then(|e| e.to_str())
                                    .map(|e| e.to_lowercase() == "mp3")
                                    .unwrap_or(false)
                            })
                            .filter_map(|p| p.to_str().map(|s| s.replace('\\', "/")))
                            .collect()
                    })
                    .unwrap_or_default();

                if archivos.is_empty() {
                    println!("No hay MP3 en la carpeta '{}'.", biblioteca);
                    continue;
                }

                // Mostrar archivos disponibles
                println!("\nArchivos disponibles en '{}':", biblioteca);
                archivos.iter().enumerate().for_each(|(i, ruta)| {
                    let nombre = ruta.split('/').last().unwrap_or(ruta);
                    println!("  {}. {}", i + 1, nombre);
                });

                let idx: usize = read_input("Número del archivo: ").parse().unwrap_or(0);
                if idx == 0 || idx > archivos.len() {
                    println!("Número inválido.");
                    continue;
                }

                let origen = &archivos[idx - 1];
                let nombre_archivo = origen.split('/').last().unwrap_or("cancion.mp3");
                let destino = format!("songs/{}", nombre_archivo);

                // Copiar el archivo a songs/
                if let Err(e) = std::fs::create_dir_all("songs") {
                    println!("Error creando carpeta songs/: {}", e);
                    continue;
                }

                if let Err(e) = std::fs::copy(origen, &destino) {
                    println!("Error copiando archivo: {}", e);
                    continue;
                }

                println!("Archivo copiado a '{}'.", destino);

                // Pedir metadatos
                let name      = read_input("Nombre de la canción: ");
                let artist    = read_input("Artista: ");
                let genre     = read_input("Género: ");
                let year: u32 = read_input("Año: ").parse().unwrap_or(0);
                let id        = next_id(&mut shared_id_gen.lock().unwrap());

                let song = Song {
                    id,
                    name,
                    artist,
                    genre,
                    year,
                    file_path: destino,
                    is_playing: false,
                };

                let mut songs = shared_songs.lock().unwrap();
                songs.push(song);
                save_songs("canciones.json", &songs);
                println!("Canción agregada con ID {}.", id);
            }
            "3" => {
                let id: u32 = read_input("ID: ").parse().unwrap_or(0);
                let mut songs = shared_songs.lock().unwrap();

                // Buscar la canción antes de eliminar para obtener su file_path
                let file_path = songs.iter()
                    .find(|s| s.id == id)
                    .map(|s| s.file_path.clone());

                let original_len = songs.len();
                *songs = remove_song(songs.clone(), id);

                if songs.len() < original_len {
                    save_songs("canciones.json", &songs);

                    // Eliminar el archivo físico de songs/
                    if let Some(path) = file_path {
                        match std::fs::remove_file(&path) {
                            Ok(_)  => println!("Archivo '{}' eliminado.", path),
                            Err(e) => println!("Canción eliminada del servidor pero no se pudo borrar el archivo: {}", e),
                        }
                    }

                    println!("Canción eliminada.");
                } else {
                    println!("No se pudo eliminar (está reproduciéndose o no existe).");
                }
            }
            "4" => {
                let query = read_input("Nombre o palabra clave: ");
                let songs = shared_songs.lock().unwrap();
                print_songs(&search_by_name(&songs, &query));
            }
            "5" => {
                let from: u32 = read_input("Año desde: ").parse().unwrap_or(0);
                let to: u32   = read_input("Año hasta: ").parse().unwrap_or(9999);
                let songs = shared_songs.lock().unwrap();
                print_songs(&search_by_year_range(&songs, from, to));
            }
            "6" => {
                let query = read_input("Género: ");
                let songs = shared_songs.lock().unwrap();
                print_songs(&search_by_genre_ranked(&songs, &query));
            }
            "7" => {
                println!("\n--- Playlists ---");
                println!("a. Crear playlist");
                println!("b. Listar playlists");
                println!("c. Ver playlist");
                println!("d. Agregar canción a playlist");
                println!("e. Eliminar canción de playlist");
                println!("f. Filtrar canciones en playlist");
                println!("g. Ordenar playlist");
                println!("h. Resumen de playlist");
                println!("i. Eliminar playlist");

                let sub = read_input("Sub-opción: ");
                match sub.as_str() {
                    "a" => {
                        let name = read_input("Nombre de la playlist: ");
                        let mut playlists = shared_playlists.lock().unwrap();
                        match pl_svc::create(&playlists, &name) {
                            Ok(new)  => {
                                *playlists = new;
                                save_playlists("playlists.json", &playlists); // ← guardar
                                println!("Playlist '{}' creada.", name);
                            }
                            Err(msg) => println!("Error: {}", msg),
                        }
                    }
                    "b" => {
                        let playlists = shared_playlists.lock().unwrap();
                        let names = pl_svc::list_names(&playlists);
                        if names.is_empty() {
                            println!("No hay playlists.");
                        } else {
                            names.iter().for_each(|n| println!("- {}", n));
                        }
                    }
                    "c" => {
                        let name = read_input("Nombre de la playlist: ");
                        let playlists = shared_playlists.lock().unwrap();
                        match pl_svc::get(&playlists, &name) {
                            Ok(pl) => {
                                println!("Playlist '{}':", pl.name);
                                if pl.songs.is_empty() {
                                    println!("  (vacía)");
                                } else {
                                    pl.songs.iter().for_each(|s| {
                                        println!("  [{}] {} - {}", s.id, s.name, s.artist);
                                    });
                                }
                            }
                            Err(msg) => println!("Error: {}", msg),
                        }
                    }
                    "d" => {
                        let name    = read_input("Nombre de la playlist: ");
                        let id: u32 = read_input("ID de la canción: ").parse().unwrap_or(0);
                        let songs   = shared_songs.lock().unwrap().clone();
                        let mut playlists = shared_playlists.lock().unwrap();
                        match songs.iter().find(|s| s.id == id).cloned() {
                            None    => println!("Canción no encontrada."),
                            Some(s) => match pl_svc::add_song(&playlists, &name, s) {
                                Ok(new)  => {
                                    *playlists = new;
                                    save_playlists("playlists.json", &playlists); // ← guardar
                                    println!("Canción agregada.");
                                }
                                Err(msg) => println!("Error: {}", msg),
                            }
                        }
                    }
                    "e" => {
                        let name    = read_input("Nombre de la playlist: ");
                        let id: u32 = read_input("ID de la canción: ").parse().unwrap_or(0);
                        let mut playlists = shared_playlists.lock().unwrap();
                        match pl_svc::remove_song(&playlists, &name, id) {
                            Ok(new)  => {
                                *playlists = new;
                                save_playlists("playlists.json", &playlists); // ← guardar
                                println!("Canción eliminada de la playlist.");
                            }
                            Err(msg) => println!("Error: {}", msg),
                        }
                    }
                    "f" => {
                        let name  = read_input("Nombre de la playlist: ");
                        let field = read_input("Filtrar por (name/artist/genre/year): ");
                        let query = read_input("Valor: ");
                        let playlists = shared_playlists.lock().unwrap();
                        match pl_svc::filter(&playlists, &name, &field, &query) {
                            Ok(pl) => {
                                println!("Resultado del filtro:");
                                if pl.songs.is_empty() {
                                    println!("  (sin resultados)");
                                } else {
                                    pl.songs.iter().for_each(|s| {
                                        println!("  [{}] {} - {}", s.id, s.name, s.artist);
                                    });
                                }
                            }
                            Err(msg) => println!("Error: {}", msg),
                        }
                    }
                    "g" => {
                        let name = read_input("Nombre de la playlist: ");
                        let by   = read_input("Ordenar por (name/year): ");
                        let mut playlists = shared_playlists.lock().unwrap();
                        match pl_svc::sort(&playlists, &name, &by) {
                            Ok(new)  => {
                                *playlists = new;
                                save_playlists("playlists.json", &playlists); // ← guardar
                                println!("Playlist ordenada.");
                            }
                            Err(msg) => println!("Error: {}", msg),
                        }
                    }
                    "h" => {
                        let name = read_input("Nombre de la playlist: ");
                        let playlists = shared_playlists.lock().unwrap();
                        match pl_svc::summarize(&playlists, &name) {
                            Ok(info) => println!("{}", info),
                            Err(msg) => println!("Error: {}", msg),
                        }
                    }
                    "i" => {
                        let name = read_input("Nombre de la playlist a eliminar: ");
                        let mut playlists = shared_playlists.lock().unwrap();
                        match pl_svc::delete(&playlists, &name) {
                            Ok(new)  => {
                                *playlists = new;
                                save_playlists("playlists.json", &playlists); // ← guardar
                                println!("Playlist eliminada.");
                            }
                            Err(msg) => println!("Error: {}", msg),
                        }
                    }
                    _ => println!("Sub-opción inválida."),
                }
            }
            "8" => {
                println!("Saliendo...");
                break;
            }
            _ => println!("Opción inválida"),
        }
    }
}