fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/aurora.ico");
        res.set("ProductName", "Aurora");
        res.set("FileDescription", "Aurora — explorateur de fichiers");
        if let Err(e) = res.compile() {
            println!("cargo:warning=Ressource Windows non compilée : {e}");
        }
    }
}
