use serde::{Deserialize, Serialize};
use crate::domain::song::Song;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
}

pub fn create_playlist(name: &str) -> Playlist {
    Playlist {
        name: name.to_string(),
        songs: Vec::new(),
    }
}

pub fn add_song_to_playlist(pl: &Playlist, song: Song) -> Playlist {
    let new_songs = pl.songs
        .iter()
        .cloned()
        .chain(std::iter::once(song))
        .collect();

    Playlist {
        name: pl.name.clone(),
        songs: new_songs,
    }
}

pub fn remove_song_from_playlist(pl: &Playlist, id: u32) -> Playlist {
    let new_songs = pl.songs
        .iter()
        .cloned()
        .filter(|s| s.id != id)
        .collect();

    Playlist {
        name: pl.name.clone(),
        songs: new_songs,
    }
}

pub fn filter_playlist<F>(pl: &Playlist, predicate: F) -> Playlist
where
    F: Fn(&Song) -> bool,
{
    let new_songs = pl.songs
        .iter()
        .cloned()
        .filter(|s| predicate(s))
        .collect();

    Playlist {
        name: pl.name.clone(),
        songs: new_songs,
    }
}

pub fn sort_playlist_by_name(pl: &Playlist) -> Playlist {
    let mut new_songs = pl.songs.clone();
    new_songs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Playlist {
        name: pl.name.clone(),
        songs: new_songs,
    }
}

pub fn sort_playlist_by_year(pl: &Playlist) -> Playlist {
    let mut new_songs = pl.songs.clone();
    new_songs.sort_by(|a, b| a.year.cmp(&b.year));

    Playlist {
        name: pl.name.clone(),
        songs: new_songs,
    }
}