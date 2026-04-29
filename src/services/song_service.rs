use crate::domain::song::Song;
use std::collections::{HashMap, HashSet};

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

// BÚSQUEDA 1: Por nombre usando índice invertido y ranking.
//
// Esta búsqueda no recorre simplemente cada canción con contains().
// Primero construye un índice invertido:
// palabra_normalizada -> lista de índices de canciones.
//
// Luego separa la consulta en palabras y busca coincidencias en el índice.
// Las canciones se ordenan según la cantidad de palabras coincidentes.
// Esto hace que la técnica sea distinta a las búsquedas por género y año.
pub fn search_by_name(songs: &[Song], query: &str) -> Vec<Song> {
    let query_normalized = normalize_text(query);

    if query_normalized.is_empty() {
        return songs.to_vec();
    }

    let mut inverted_index: HashMap<String, Vec<usize>> = HashMap::new();

    songs.iter().enumerate().for_each(|(index, song)| {
        tokenize(&song.name).into_iter().for_each(|word| {
            inverted_index
                .entry(word)
                .or_insert_with(Vec::new)
                .push(index);
        });
    });

    let query_words = tokenize(&query_normalized);

    if query_words.is_empty() {
        return Vec::new();
    }

    let mut scores: HashMap<usize, u32> = HashMap::new();

    query_words.iter().for_each(|query_word| {
        inverted_index
            .iter()
            .filter(|(indexed_word, _)| {
                indexed_word.contains(query_word.as_str())
                    || query_word.contains(indexed_word.as_str())
            })
            .for_each(|(_, song_indices)| {
                song_indices.iter().for_each(|song_index| {
                    let counter = scores.entry(*song_index).or_insert(0);
                    *counter += 1;
                });
            });
    });

    let mut ranked_results: Vec<(usize, u32)> = scores.into_iter().collect();

    ranked_results.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| songs[a.0].name.to_lowercase().cmp(&songs[b.0].name.to_lowercase()))
    });

    ranked_results
        .into_iter()
        .map(|(song_index, _score)| songs[song_index].clone())
        .collect()
}

// BÚSQUEDA 2: Por rango de años.
//
// Esta búsqueda es numérica, no textual.
// Trabaja con dos límites: desde y hasta.
// Si el usuario coloca el rango al revés, se corrige automáticamente.
// Luego ordena los resultados por año.
pub fn search_by_year_range(songs: &[Song], from: u32, to: u32) -> Vec<Song> {
    let start = from.min(to);
    let end = from.max(to);

    let mut results: Vec<Song> = songs
        .iter()
        .filter(|song| song.year >= start && song.year <= end)
        .cloned()
        .collect();

    results.sort_by_key(|song| song.year);

    results
}

// BÚSQUEDA 3: Por género con ranking de relevancia usando fold.
//
// Esta búsqueda trata el género como una categoría.
// Usa un sistema de puntaje:
// 3 = coincidencia exacta
// 2 = el género empieza con la consulta
// 1 = el género contiene la consulta
//
// Se implementa con fold para acumular los resultados y luego ordenarlos
// por relevancia. Es una técnica distinta al índice invertido de nombre
// y distinta a la comparación numérica por rango de años.
pub fn search_by_genre_ranked(songs: &[Song], query: &str) -> Vec<Song> {
    let query_normalized = normalize_text(query);

    if query_normalized.is_empty() {
        return songs.to_vec();
    }

    let mut ranked = songs.iter().fold(Vec::new(), |mut acc, song| {
        let genre_normalized = normalize_text(&song.genre);

        let score = if genre_normalized == query_normalized {
            3
        } else if genre_normalized.starts_with(&query_normalized) {
            2
        } else if genre_normalized.contains(&query_normalized) {
            1
        } else {
            0
        };

        if score > 0 {
            acc.push((score, song.clone()));
        }

        acc
    });

    ranked.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.genre.to_lowercase().cmp(&b.1.genre.to_lowercase()))
            .then_with(|| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()))
    });

    ranked.into_iter().map(|(_, song)| song).collect()
}

fn normalize_text(text: &str) -> String {
    text
        .trim()
        .to_lowercase()
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace('ú', "u")
        .replace('ñ', "n")
}

fn tokenize(text: &str) -> Vec<String> {
    let normalized = normalize_text(text);

    let mut seen = HashSet::new();

    normalized
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.trim().is_empty())
        .map(|word| word.trim().to_string())
        .filter(|word| seen.insert(word.clone()))
        .collect()
}