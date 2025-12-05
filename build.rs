fn main() {
    if cfg!(target_os = "windows") {
        // Only set icon if the file exists
        if std::path::Path::new("assets/icon.ico").exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("assets/icon.ico");
            res.compile().unwrap();
        }
    }
}

