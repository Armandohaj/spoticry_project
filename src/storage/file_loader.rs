use std::fs;
use crate::domain::song::Song;
use crate::utils::id_generator::{IdGenerator, next_id};

pub fn load_songs(path: &str, id_gen: &mut IdGenerator) -> Vec<Song> {
    let entries = match fs::read_dir(path) {
        Ok(e)  => e,
        Err(_) => {
            println!("Carpeta '{}' no encontrada, iniciando sin canciones.", path);
            return Vec::new();
        }
    };

    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name      = path.file_stem()?.to_str()?.to_string();
            let file_path = path.to_str()?.to_string();
            Some(Song {
                id: next_id(id_gen),
                name,
                artist:    "Unknown".to_string(),
                genre:     "Unknown".to_string(),
                year:      0,
                file_path,
                is_playing: false,
            })
        })
        .collect()
}