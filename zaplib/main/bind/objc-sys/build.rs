fn main() {
    #[cfg(target_os = "linux")] // this is the *current* os, not the target triple
    println!("cargo:rustc-link-search=/usr/GNUstep/System/Library/Libraries");
}
