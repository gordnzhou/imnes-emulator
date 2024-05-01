fn main() {
    if std::env::consts::OS == "macos" {
        println!("cargo:rustc-link-search=framework=/Library/Frameworks");
        println!("cargo:rustc-link-lib=SDL2");
    }
}