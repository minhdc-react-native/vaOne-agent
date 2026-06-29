pub fn get_window_size(id: &str) -> Option<(f64, f64)> {
    match id {
        "splash" => Some((500.0, 200.0)),
        "settings" => Some((330.0, 350.0)),
        _ => None,
    }
}
