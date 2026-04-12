fn main() {
    #[cfg(windows)]
    {
        let version = std::env::var("CARGO_PKG_VERSION").unwrap();
        let description = std::env::var("CARGO_PKG_DESCRIPTION").unwrap();
        let license = std::env::var("CARGO_PKG_LICENSE").unwrap();

        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app/main.ico");
        res.set("ProductName", "DAWPresence");
        res.set("ProductVersion", &version);
        res.set("FileVersion", &version);
        res.set("FileDescription", &description);
        res.set("LegalCopyright", &license);
        res.set("CompanyName", "MihaiStreames");
        res.compile().expect("couldn't compile windows resources");
    }
}
