use log::info;

use std::process::{exit, Command};

#[derive(Default, Debug)]
pub(crate) struct BuildOpts {
    pub(crate) release: bool,
    pub(crate) use_simd128: bool,
    pub(crate) all_targets: bool,
    pub(crate) workspace: bool,
    pub(crate) package: String,
    pub(crate) features: String,
}

pub(crate) fn build(opts: BuildOpts) {
    let mut args = vec!["+nightly-2022-01-18", "build", "--target=wasm32-unknown-unknown", "-Zbuild-std=std,panic_abort"];

    if opts.release {
        args.push("--release");
    }

    if opts.workspace {
        args.push("--workspace");
    }

    if opts.all_targets {
        args.push("--all-targets");
    }

    if !opts.package.is_empty() {
        args.push("-p");
        args.push(&opts.package);
    }

    if !opts.features.is_empty() {
        args.push("--features");
        args.push(&opts.features);
    }

    let rust_flags = {
        let mut flags = vec![];
        if opts.use_simd128 {
            flags.push("-C target-feature=+atomics,+bulk-memory,+mutable-globals,+simd128");
        } else {
            flags.push("-C target-feature=+atomics,+bulk-memory,+mutable-globals");
        }
        flags.push("-C link-arg=--max-memory=4294967296");
        flags.push("-C link-arg=--export=__stack_pointer");

        flags.join(" ")
    };

    let string_args = args.join(" ");
    info!("Running RUSTFLAGS='{rust_flags}' cargo {string_args}");
    let exit_status = Command::new("cargo")
        .env("RUSTFLAGS", &rust_flags)
        .args(args)
        .spawn()
        .expect("Failed to execute command")
        .wait()
        .unwrap();
    exit(exit_status.code().unwrap_or(1));
}
