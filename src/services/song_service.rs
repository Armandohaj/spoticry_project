use crate::domain::song::Song;

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

pub fn search_by_name(songs: &[Song], query: &str) -> Vec<Song> {
    songs.iter()
        .cloned()
        .filter(|s| s.name.to_lowercase().contains(&query.to_lowercase()))
        .collect()
}

pub fn search_by_artist(songs: &[Song], query: &str) -> Vec<Song> {
    songs.iter()
        .cloned()
        .filter(|s| s.artist.to_lowercase().contains(&query.to_lowercase()))
        .collect()
}

pub fn search_by_genre(songs: &[Song], query: &str) -> Vec<Song> {
    songs.iter()
        .cloned()
        .filter(|s| s.genre.to_lowercase().contains(&query.to_lowercase()))
        .collect()
}

pub fn search_by_year(songs: &[Song], year: u32) -> Vec<Song> {
    songs.iter()
        .cloned()
        .filter(|s| s.year == year)
        .collect()
}