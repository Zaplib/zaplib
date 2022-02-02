//! Version of [`rand`] that also works in WebAssembly.
/// Public random API that has been tested on native and WASM.
///
/// TODO(Paras): This does not currently mirror the [`rand`] interface, and
/// if we find ourselves needing more rand methods, we should conform it using
/// a trait as we do in [`universal_thread`].

#[cfg(not(target_arch = "wasm32"))]
use rand::Rng;

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn randomU64() -> u64;
}

pub fn random_128() -> u128 {
    #[cfg(not(target_arch = "wasm32"))]
    return rand::thread_rng().gen();

    #[cfg(target_arch = "wasm32")]
    unsafe {
        let a = randomU64();
        let b = randomU64();
        return ((a as u128) << 64) | b as u128;
    }
}
