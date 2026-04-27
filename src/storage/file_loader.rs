use std::fs;
use serde::{Deserialize, Serialize};
use crate::domain::song::Song;
use crate::domain::playlist::Playlist;
use crate::utils::id_generator::{IdGenerator, next_id};

#[derive(Deserialize, Serialize)]
struct SongEntry {
    pub name:      String,
    pub artist:    String,
    pub genre:     String,
    pub year:      u32,
    pub file_path: String,
}

pub fn load_songs(path: &str, id_gen: &mut IdGenerator) -> Vec<Song> {
    let content = match fs::read_to_string(path) {
        Ok(c)  => c,
        Err(_) => {
            println!("Archivo '{}' no encontrado, iniciando sin canciones.", path);
            return Vec::new();
        }
    };

    let entries: Vec<SongEntry> = match serde_json::from_str(&content) {
        Ok(e)  => e,
        Err(e) => {
            println!("Error parseando {}: {}", path, e);
            return Vec::new();
        }
    };

    entries
        .into_iter()
        .map(|entry| Song {
            id:         next_id(id_gen),
            name:       entry.name,
            artist:     entry.artist,
            genre:      entry.genre,
            year:       entry.year,
            file_path:  entry.file_path.replace('\\', "/"),
            is_playing: false,
        })
        .collect()
}

pub fn save_songs(path: &str, songs: &[Song]) {
    let entries: Vec<SongEntry> = songs
        .iter()
        .map(|s| SongEntry {
            name:      s.name.clone(),
            artist:    s.artist.clone(),
            genre:     s.genre.clone(),
            year:      s.year,
            file_path: s.file_path.clone(),
        })
        .collect();

    match serde_json::to_string_pretty(&entries) {
        Ok(json) => match fs::write(path, json) {
            Ok(_)  => println!("Canciones guardadas en '{}'.", path),
            Err(e) => println!("Error guardando canciones: {}", e),
        },
        Err(e) => println!("Error serializando canciones: {}", e),
    }
}

// Cargar playlists desde JSON
pub fn load_playlists(path: &str, songs: &[Song]) -> Vec<Playlist> {
    let content = match fs::read_to_string(path) {
        Ok(c)  => c,
        Err(_) => {
            println!("Archivo '{}' no encontrado, iniciando sin playlists.", path);
            return Vec::new();
        }
    };

    // Las playlists se guardan como nombre + lista de IDs de canciones
    let entries: Vec<PlaylistEntry> = match serde_json::from_str(&content) {
        Ok(e)  => e,
        Err(e) => {
            println!("Error parseando {}: {}", path, e);
            return Vec::new();
        }
    };

    // Reconstruir las playlists buscando cada ID en la lista de canciones actual
    entries
        .into_iter()
        .map(|entry| {
            let playlist_songs = entry.song_ids
                .iter()
                .filter_map(|id| songs.iter().find(|s| s.id == *id).cloned())
                .collect();

            Playlist {
                name:  entry.name,
                songs: playlist_songs,
            }
        })
        .collect()
}

// Guardar playlists en JSON
pub fn save_playlists(path: &str, playlists: &[Playlist]) {
    let entries: Vec<PlaylistEntry> = playlists
        .iter()
        .map(|pl| PlaylistEntry {
            name:     pl.name.clone(),
            song_ids: pl.songs.iter().map(|s| s.id).collect(),
        })
        .collect();

    match serde_json::to_string_pretty(&entries) {
        Ok(json) => match fs::write(path, json) {
            Ok(_)  => println!("Playlists guardadas en '{}'.", path),
            Err(e) => println!("Error guardando playlists: {}", e),
        },
        Err(e) => println!("Error serializando playlists: {}", e),
    }
}

// Estructura para serializar playlists
// Guardamos solo los IDs de canciones, no los datos completos
// Al cargar, reconstruimos las canciones desde la lista de canciones actual
#[derive(Serialize, Deserialize)]
struct PlaylistEntry {
    pub name:     String,
    pub song_ids: Vec<u32>,
}