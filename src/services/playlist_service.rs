use crate::domain::playlist::{
    Playlist, create_playlist, add_song_to_playlist,
    remove_song_from_playlist, filter_playlist,
    sort_by_name, sort_by_year, summarize_playlist,
};
use crate::domain::song::Song;

// Crear nueva playlist si no existe ya una con ese nombre
pub fn create(playlists: &[Playlist], name: &str) -> Result<Vec<Playlist>, String> {
    if playlists.iter().any(|p| p.name == name) {
        return Err(format!("Ya existe una playlist llamada '{}'", name));
    }

    // Usar map para construir nueva lista con la playlist agregada
    let new_playlist = create_playlist(name);
    let new_playlists = playlists.iter()
        .cloned()
        .chain(std::iter::once(new_playlist))
        .collect();

    Ok(new_playlists)
}

// Eliminar una playlist por nombre
pub fn delete(playlists: &[Playlist], name: &str) -> Result<Vec<Playlist>, String> {
    if !playlists.iter().any(|p| p.name == name) {
        return Err(format!("No existe una playlist llamada '{}'", name));
    }

    let new_playlists = playlists.iter()
        .cloned()
        .filter(|p| p.name != name)
        .collect();

    Ok(new_playlists)
}

// Agregar canción a playlist — devuelve nueva lista de playlists
pub fn add_song(
    playlists: &[Playlist],
    playlist_name: &str,
    song: Song,
) -> Result<Vec<Playlist>, String> {
    if !playlists.iter().any(|p| p.name == playlist_name) {
        return Err(format!("Playlist '{}' no encontrada", playlist_name));
    }

    // map transforma cada playlist: solo modifica la que coincide
    let new_playlists = playlists.iter()
        .cloned()
        .map(|p| {
            if p.name == playlist_name {
                add_song_to_playlist(&p, song.clone())
            } else {
                p
            }
        })
        .collect();

    Ok(new_playlists)
}

// Eliminar canción de playlist — devuelve nueva lista de playlists
pub fn remove_song(
    playlists: &[Playlist],
    playlist_name: &str,
    song_id: u32,
) -> Result<Vec<Playlist>, String> {
    if !playlists.iter().any(|p| p.name == playlist_name) {
        return Err(format!("Playlist '{}' no encontrada", playlist_name));
    }

    let new_playlists = playlists.iter()
        .cloned()
        .map(|p| {
            if p.name == playlist_name {
                remove_song_from_playlist(&p, song_id)
            } else {
                p
            }
        })
        .collect();

    Ok(new_playlists)
}

// Filtrar canciones dentro de una playlist por campo y valor
pub fn filter(
    playlists: &[Playlist],
    playlist_name: &str,
    field: &str,
    query: &str,
) -> Result<Playlist, String> {
    let pl = playlists.iter()
        .find(|p| p.name == playlist_name)
        .ok_or_else(|| format!("Playlist '{}' no encontrada", playlist_name))?;

    // El closure varía según el campo — cada uno hace algo distinto
    let filtered = match field {
        "name"   => filter_playlist(pl, |s| {
            s.name.to_lowercase().contains(&query.to_lowercase())
        }),
        "artist" => filter_playlist(pl, |s| {
            s.artist.to_lowercase().contains(&query.to_lowercase())
        }),
        "genre"  => filter_playlist(pl, |s| {
            s.genre.to_lowercase().contains(&query.to_lowercase())
        }),
        "year"   => {
            let year: u32 = query.parse().unwrap_or(0);
            filter_playlist(pl, |s| s.year == year)
        },
        _ => return Err(format!("Campo '{}' no válido. Usa: name, artist, genre, year", field)),
    };

    Ok(filtered)
}

// Ordenar playlist por criterio
pub fn sort(
    playlists: &[Playlist],
    playlist_name: &str,
    by: &str,
) -> Result<Vec<Playlist>, String> {
    if !playlists.iter().any(|p| p.name == playlist_name) {
        return Err(format!("Playlist '{}' no encontrada", playlist_name));
    }

    let new_playlists = playlists.iter()
        .cloned()
        .map(|p| {
            if p.name == playlist_name {
                match by {
                    "name" => sort_by_name(&p),
                    "year" => sort_by_year(&p),
                    _      => p,
                }
            } else {
                p
            }
        })
        .collect();

    Ok(new_playlists)
}

// Obtener una playlist por nombre
pub fn get(playlists: &[Playlist], name: &str) -> Result<Playlist, String> {
    playlists.iter()
        .find(|p| p.name == name)
        .cloned()
        .ok_or_else(|| format!("Playlist '{}' no encontrada", name))
}

// Listar nombres de todas las playlists
pub fn list_names(playlists: &[Playlist]) -> Vec<String> {
    playlists.iter()
        .map(|p| p.name.clone())
        .collect()
}

// Obtener resumen de una playlist
pub fn summarize(playlists: &[Playlist], name: &str) -> Result<String, String> {
    let pl = playlists.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("Playlist '{}' no encontrada", name))?;

    Ok(summarize_playlist(pl))
}