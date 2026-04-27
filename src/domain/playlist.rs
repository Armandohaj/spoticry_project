use serde::{Deserialize, Serialize};
use crate::domain::song::Song;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub name:  String,
    pub songs: Vec<Song>,
}

// Crear una playlist vacía — devuelve nueva instancia, sin mutar nada
pub fn create_playlist(name: &str) -> Playlist {
    Playlist {
        name:  name.to_string(),
        songs: Vec::new(),
    }
}

// Agregar canción — devuelve nueva Playlist sin mutar la original
pub fn add_song_to_playlist(pl: &Playlist, song: Song) -> Playlist {
    let new_songs = pl.songs
        .iter()
        .cloned()
        .chain(std::iter::once(song))
        .collect();

    Playlist { name: pl.name.clone(), songs: new_songs }
}

// Eliminar canción — devuelve nueva Playlist sin mutar la original
pub fn remove_song_from_playlist(pl: &Playlist, song_id: u32) -> Playlist {
    let new_songs = pl.songs
        .iter()
        .cloned()
        .filter(|s| s.id != song_id)
        .collect();

    Playlist { name: pl.name.clone(), songs: new_songs }
}

// Filtrar canciones por criterio — usa closure como parámetro
pub fn filter_playlist<F>(pl: &Playlist, predicate: F) -> Playlist
where
    F: Fn(&Song) -> bool,
{
    let new_songs = pl.songs
        .iter()
        .cloned()
        .filter(|s| predicate(s))
        .collect();

    Playlist { name: pl.name.clone(), songs: new_songs }
}

// Ordenar por nombre — devuelve nueva Playlist
pub fn sort_by_name(pl: &Playlist) -> Playlist {
    let mut new_songs = pl.songs.clone();
    new_songs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Playlist { name: pl.name.clone(), songs: new_songs }
}

// Ordenar por año — devuelve nueva Playlist
pub fn sort_by_year(pl: &Playlist) -> Playlist {
    let mut new_songs = pl.songs.clone();
    new_songs.sort_by(|a, b| a.year.cmp(&b.year));
    Playlist { name: pl.name.clone(), songs: new_songs }
}

// Obtener resumen con fold — total de canciones y artistas únicos
pub fn summarize_playlist(pl: &Playlist) -> String {
    let (total, artists) = pl.songs.iter().fold(
        (0u32, Vec::<String>::new()),
        |(count, mut artists), song| {
            if !artists.contains(&song.artist) {
                artists.push(song.artist.clone());
            }
            (count + 1, artists)
        }
    );

    format!(
        "Playlist '{}': {} canciones, {} artistas únicos",
        pl.name, total, artists.len()
    )
}

// Mapear canciones a sus nombres — uso explícito de map
pub fn get_song_names(pl: &Playlist) -> Vec<String> {
    pl.songs
        .iter()
        .map(|s| format!("{} - {}", s.name, s.artist))
        .collect()
}