use std::fs;
use crate::domain::song::Song;
use crate::utils::id_generator::{IdGenerator, next_id};
use serde::Deserialize;

// Estructura temporal para leer el JSON
// No tiene id ni is_playing porque esos los asignamos nosotros
#[derive(Deserialize)]
struct SongEntry {
    pub name:      String,
    pub artist:    String,
    pub genre:     String,
    pub year:      u32,
    pub file_path: String,
}

pub fn load_songs(path: &str, id_gen: &mut IdGenerator) -> Vec<Song> {
    // Leer el contenido del archivo JSON
    let content = match fs::read_to_string(path) {
        Ok(c)  => c,
        Err(_) => {
            println!("Archivo '{}' no encontrado, iniciando sin canciones.", path);
            return Vec::new();
        }
    };

    // Parsear el JSON a un Vec de SongEntry
    let entries: Vec<SongEntry> = match serde_json::from_str(&content) {
        Ok(e)  => e,
        Err(e) => {
            println!("Error parseando {}: {}", path, e);
            return Vec::new();
        }
    };

    // Convertir cada SongEntry a Song asignando ID automáticamente
    entries
        .into_iter()
        .map(|entry| Song {
            id:        next_id(id_gen),
            name:      entry.name,
            artist:    entry.artist,
            genre:     entry.genre,
            year:      entry.year,
            file_path: entry.file_path,
            is_playing: false,
        })
        .collect()
}