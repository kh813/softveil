fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.png"); // winres usually takes .ico, but let's assume conversion or use placeholder
        res.compile().unwrap();
    }
}
