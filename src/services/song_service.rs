use crate::domain::song::Song;
use std::collections::HashMap;

pub fn list_songs(songs: &[Song]) -> Vec<Song> {
    songs.to_vec()
}

pub fn print_songs(songs: &[Song]) {
    if songs.is_empty() {
        println!("No hay canciones.");
        return;
    }
    songs.iter().for_each(|s| {
        println!(
            "[{}] {} - {} ({}) [{}] {}",
            s.id,
            s.name,
            s.artist,
            s.genre,
            s.year,
            if s.is_playing { "[REPRODUCIENDO]" } else { "" }
        );
    });
}

pub fn add_song(songs: &mut Vec<Song>, song: Song) {
    songs.push(song);
}

pub fn remove_song(songs: Vec<Song>, id: u32) -> Vec<Song> {
    songs
        .into_iter()
        .filter(|s| s.id != id || s.is_playing)
        .collect()
}

// BÚSQUEDA 1: Por nombre usando índice invertido
// Construye un HashMap de palabra -> índices de canciones
// luego busca cada palabra de la query en ese mapa
pub fn search_by_name(songs: &[Song], query: &str) -> Vec<Song> {
    // Paso 1: construir índice invertido
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();

    songs.iter().enumerate().for_each(|(i, song)| {
        song.name
            .to_lowercase()
            .split_whitespace()
            .for_each(|word| {
                index.entry(word.to_string())
                    .or_insert_with(Vec::new)
                    .push(i);
            });
    });

    // Paso 2: buscar cada palabra de la query en el índice
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    // Paso 3: acumular índices de canciones que coincidan
    let mut indices_encontrados: Vec<usize> = query_words
        .iter()
        .flat_map(|word| {
            index.iter()
                .filter(|(key, _)| key.contains(*word))
                .flat_map(|(_, indices)| indices.clone())
        })
        .collect();

    // Eliminar duplicados manteniendo orden
    indices_encontrados.sort();
    indices_encontrados.dedup();

    indices_encontrados
        .into_iter()
        .map(|i| songs[i].clone())
        .collect()
}

// BÚSQUEDA 2: Por rango de años
// Opera sobre números con dos parámetros (desde/hasta)
// Si el usuario pone el rango al revés lo corrige automáticamente
pub fn search_by_year_range(songs: &[Song], from: u32, to: u32) -> Vec<Song> {
    let (start, end) = if from <= to {
        (from, to)
    } else {
        (to, from)
    };

    songs.iter()
        .cloned()
        .filter(|s| s.year >= start && s.year <= end)
        .collect()
}

// BÚSQUEDA 3: Por género con ranking de relevancia usando fold
// Match exacto = puntaje 2, match parcial = puntaje 1
// Devuelve ordenadas por relevancia de mayor a menor
pub fn search_by_genre_ranked(songs: &[Song], query: &str) -> Vec<Song> {
    let query_lower = query.to_lowercase();

    let mut ranked = songs.iter().fold(Vec::new(), |mut acc, song| {
        let genre_lower = song.genre.to_lowercase();

        let score = if genre_lower == query_lower {
            2 // match exacto
        } else if genre_lower.contains(&query_lower) {
            1 // match parcial
        } else {
            0
        };

        if score > 0 {
            acc.push((score, song.clone()));
        }
        acc
    });

    ranked.sort_by(|a, b| b.0.cmp(&a.0));
    ranked.into_iter().map(|(_, song)| song).collect()
}