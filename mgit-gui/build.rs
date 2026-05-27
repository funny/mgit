fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resource/logo64x64.ico");
        res.compile().unwrap();
    }
}
