use std::io::{self, Write};

pub fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn show_menu() {
    println!("\n=== SpotiCry Server ===");
    println!("1. Listar canciones");
    println!("2. Agregar canción");
    println!("3. Eliminar canción");
    println!("4. Buscar por nombre (índice invertido)");
    println!("5. Buscar por rango de años");
    println!("6. Buscar por género (con ranking)");
    println!("7. Gestionar playlists");
    println!("8. Salir");
}