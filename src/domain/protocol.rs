use serde::{Deserialize, Serialize};
use crate::domain::song::Song;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command")]
pub enum Request {
    ListSongs,
    SearchByName   { query: String },
    SearchByArtist { query: String },
    SearchByYear   { year: u32     },
    Play           { song_id: u32  },
    Stop           { song_id: u32  },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum Response {
    Ok         { songs: Vec<Song>                                            },
    Error      { message: String                                             },
    AudioStart { sample_rate: u32, channels: u32, bits: u32, total_bytes: usize }, // ← total_bytes aquí
    AudioEnd,
}