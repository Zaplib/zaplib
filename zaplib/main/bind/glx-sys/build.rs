fn main() {
    #[cfg(target_os = "linux")] // this is the *current* os, not the target triple
    {
        // Per https://kazlauskas.me/entries/writing-proper-buildrs-scripts.html
        if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "linux" {
            println!("cargo:rustc-link-lib=GLX");
        }
    }
}
