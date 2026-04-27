use serde::{Deserialize, Serialize};
use crate::domain::song::Song;
use crate::domain::playlist::Playlist;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command")]
pub enum Request {
    // Canciones
    ListSongs,
    SearchByName      { query: String      },
    SearchByYearRange { from: u32, to: u32 },
    SearchByGenre     { query: String      },
    Play              { song_id: u32       },
    Stop              { song_id: u32       },

    // Playlists
    CreatePlaylist    { name: String                    },
    ListPlaylists,
    AddToPlaylist     { playlist_name: String, song_id: u32 },
    RemoveFromPlaylist{ playlist_name: String, song_id: u32 },
    GetPlaylist       { playlist_name: String           },
    FilterPlaylist    { playlist_name: String, field: String, query: String },
    SortPlaylist      { playlist_name: String, by: String },
    SummarizePlaylist { playlist_name: String           },
    DeletePlaylist    { name: String                    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum Response {
    Ok            { songs: Vec<Song>                                                },
    Error         { message: String                                                 },
    AudioStart    { sample_rate: u32, channels: u32, bits: u32, total_bytes: usize },
    AudioEnd,
    PlaylistOk    { playlist: Playlist                                              },
    PlaylistList  { playlists: Vec<String>                                          },
    PlaylistInfo  { info: String                                                    },
}