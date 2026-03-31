use serde::{Deserialize, Serialize};
use crate::domain::song::Song;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command")]
pub enum Request {
    ListSongs,
    SearchByName   { query: String },
    SearchByArtist { query: String },
    SearchByYear   { year: u32     },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum Response {
    Ok    { songs: Vec<Song> },
    Error { message: String  },
}