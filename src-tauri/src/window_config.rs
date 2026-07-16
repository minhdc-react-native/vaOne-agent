pub fn get_window_size(id: &str) -> Option<(f64, f64)> {
    match id {
        "splash" => Some((250.0, 100.0)),
        "report" => Some((1000.0, 800.0)),
        "settings" => Some((330.0, 350.0)),
        "loginTct" => Some((380.0, 540.0)),
        "getInvoiceTct" => Some((400.0, 540.0)),
        "loginMInvoice" => Some((380.0, 520.0)),
        "loginSaveInvoice" => Some((380.0, 520.0)),
        "blank" => Some((10.0, 10.0)),
        "update" => Some((330.0, 120.0)),
        "quit" => Some((350.0, 180.0)),
        _ => None,
    }
}
