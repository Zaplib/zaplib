use std::env;
use std::path::Path;

enum Platform {
    Windows,
    Mac,
    Linux,
}

fn get_platform() -> Platform {
    match std::env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => Platform::Windows,
        "macos" => Platform::Mac,
        "linux" => Platform::Linux,
        other => panic!("Sorry, platform \"{}\" is not supported by CEF.", other),
    }
}

fn choose_source_dir() -> Option<String> {
    if let Ok(path) = env::var("CEF_ROOT") {
        if Path::new(&path).exists() {
            return Some(path);
        }
    }

    // Check out `zaplib/main/bind/cef-sys/README.md` file for notes on the current CEF/Chromium version.
    let cef_build = "cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164";
    let default_path = match get_platform() {
        Platform::Windows => Path::new(env!("CARGO_MANIFEST_DIR")).join("deps").join(format!("{}_windows64", cef_build)),
        Platform::Mac => Path::new(env!("CARGO_MANIFEST_DIR")).join("deps").join(format!("{}_macosx64", cef_build)),
        // TODO(Dmitry): switch linux version from minimal
        Platform::Linux => Path::new(env!("CARGO_MANIFEST_DIR")).join("deps").join(format!("{}_linux64_minimal", cef_build)),
    };
    if default_path.exists() {
        return Some(default_path.to_str().unwrap().to_string());
    }
    None
}

fn main() {
    if std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap() == "wasm" {
        return;
    }

    let source_dir = choose_source_dir().expect("Failed to locate CEF lib path");

    // Inform rust what to link.
    match get_platform() {
        Platform::Windows => println!("cargo:rustc-link-lib=libcef"),
        Platform::Mac => println!("cargo:rustc-link-lib=framework=Chromium Embedded Framework"),
        Platform::Linux => println!("cargo:rustc-link-lib=cef"),
    };

    // Generate bindings.rs.
    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_arg("-I".to_string() + &source_dir)
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        .allowlist_type("cef_.*")
        .allowlist_function("cef_.*")
        .bitfield_enum(".*_mask_t")
        .generate()
        .expect("Unable to generate bindings");
    bindings.write_to_file(Path::new(&env::var("OUT_DIR").unwrap()).join("bindings.rs")).expect("Couldn't write bindings!");

    // Get the "Release" dir, which contains the resources (or "Debug" when using the "debug" feature).
    #[allow(unused_mut, unused_assignments)]
    let mut release_dir = Path::new(&source_dir).join("Release");
    #[cfg(feature = "debug")]
    {
        release_dir = Path::new(&source_dir).join("Debug");
    }

    if !release_dir.exists() {
        panic!("CEF Release directory ({}) does not exist", release_dir.to_str().unwrap_or(""));
    }

    let opts = fs_extra::dir::CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 64000, // Default
        copy_inside: true,
        depth: 0,
        content_only: false,
    };

    if let Platform::Mac = get_platform() {
        // Add the release dir as a framework search path.
        if let Some(release_dir) = release_dir.to_str() {
            println!("cargo:rustc-link-search=framework={}", release_dir);
        }

        // Copy the framework to "../Frameworks" relative to the final executable, which is where it will
        // automatically get linked to. This is pretty brittle. TODO(JP): Figure out something better here,
        // e.g. absolute paths during debugging (at least that is more stable and seems to be done for system
        // frameworks) and "@rpath" for release builds? See also https://github.com/rust-lang/cargo/issues/5077.
        // Or keep the current behavior for release builds and put everything in the correct place in the final
        // `.app`.
        let dest_path = Path::new(&env::var("OUT_DIR").unwrap()).join("../../../../Frameworks");
        if dest_path.exists() {
            fs_extra::remove_items(&[&dest_path]).unwrap();
        }
        let all_items = vec![release_dir.to_str().unwrap()];
        fs_extra::copy_items(&all_items, &dest_path, &opts).unwrap();
    } else {
        if let Some(release_dir) = release_dir.to_str() {
            println!("cargo:rustc-link-search=native={}", release_dir);
        }

        let resources_dir = Path::new(&source_dir).join("Resources");
        if !resources_dir.exists() {
            panic!("CEF Resources directory ({}) does not exist", resources_dir.to_str().unwrap_or(""));
        }

        // Copy the required Resources & Release contents to OUT_DIR so that a cargo run works.
        let dest_path = Path::new(&env::var("OUT_DIR").unwrap()).join("../../..");

        let mut release_items = fs_extra::dir::get_dir_content(&release_dir).unwrap();
        let mut resources_items = fs_extra::dir::get_dir_content(&resources_dir).unwrap();

        let mut all_items = Vec::new();
        all_items.append(&mut release_items.directories);
        all_items.append(&mut release_items.files);
        all_items.append(&mut resources_items.directories);
        all_items.append(&mut resources_items.files);

        fs_extra::copy_items(&all_items, &dest_path, &opts).unwrap();
    }
}
