fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set by Cargo")
        == "windows"
    {
        let mut res = winresource::WindowsResource::new();

        if cfg!(target_os = "windows") {
            res.set_toolkit_path(
                "C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.26100.0\\x64",
            );
        } else {
            res.set_windres_path("x86_64-w64-mingw32-windres");
            res.set_ar_path("x86_64-w64-mingw32-ar");
        }

        res.set_icon("assets/app/main.ico");
        res.compile().expect("Couldn't compile windows resources");
    }
}
