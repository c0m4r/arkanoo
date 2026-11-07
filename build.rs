fn main() {
    // Check the target rather than the host so the icon is also embedded
    // when cross-compiling the Windows build from Linux.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile().unwrap();
    }
}
