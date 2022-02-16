#[cfg(not(target_arch = "wasm32"))]
mod cmd;
#[cfg(not(target_arch = "wasm32"))]
mod install_deps;
#[cfg(not(target_arch = "wasm32"))]
mod serve;

// Use an empty main() function in the wasm32 case, so you can run
// `cargo zaplib build --workspace` without crashing.
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    cmd::cmd();
}
