use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Song {
    pub id: u32,
    pub name: String,
    pub artist: String,
    pub genre: String,
    pub year: u32,
    pub file_path: String,
    pub is_playing: bool,
}