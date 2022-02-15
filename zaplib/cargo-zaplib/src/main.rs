mod install_deps;

#[cfg(not(target_arch = "wasm32"))]
use install_deps::*;

use std::process::{exit, Command};

use clap::{App, AppSettings, Arg};

fn main() {
    let matches = App::new("Zaplib Command Line Tool")
        .setting(AppSettings::ArgRequiredElseHelp)
        .about(env!["CARGO_PKG_DESCRIPTION"])
        .version(env!("CARGO_PKG_VERSION"))
        // When running as a cargo command, the second argument is the name of the program
        // and we want to ignore it when displaying help
        .arg(Arg::new("crate-version").hide(true))
        .subcommand(
            App::new("install-deps")
                .arg(
                    Arg::new("devel")
                        .short('D')
                        .long("devel")
                        .takes_value(false)
                        .help("Install additional dependencies for Zaplib development."),
                )
                .arg(Arg::new("ci").long("ci").takes_value(false).help("Install dependencies for CI")),
        )
        .subcommand(
            App::new("build")
                .arg(
                    Arg::new("release")
                        .short('R')
                        .long("release")
                        .takes_value(false)
                        .help("Build artifacts in release mode, with optimizations"),
                )
                .arg(Arg::new("package").short('p').long("package").takes_value(true).help("Build only the specified package."))
                .arg(Arg::new("features").long("features").takes_value(true).help("Specify feature flags."))
                .arg(Arg::new("all-targets").long("all-targets").takes_value(false).help("Build all targets."))
                .arg(Arg::new("workspace").long("workspace").takes_value(false).help("Build all members in the workspace."))
                .arg(Arg::new("simd128").long("simd128").takes_value(false).help("Use 128-bit SIMD instruction set for WASM")),
        )
        .get_matches();

    if let Some(cmd) = matches.subcommand_matches("build") {
        build(BuildOpts {
            release: cmd.is_present("release"),
            use_simd128: cmd.is_present("simd128"),
            all_targets: cmd.is_present("all-targets"),
            workspace: cmd.is_present("workspace"),
            features: cmd.value_of("features").unwrap_or("").to_string(),
            package: cmd.value_of("package").unwrap_or("").to_string(),
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(cmd) = matches.subcommand_matches("install-deps") {
        if cmd.is_present("ci") {
            install_ci_deps();
        } else {
            install_deps(cmd.is_present("devel"));
        }
    }
}

#[derive(Default, Debug)]
struct BuildOpts {
    release: bool,
    use_simd128: bool,
    all_targets: bool,
    workspace: bool,
    package: String,
    features: String,
}

fn build(opts: BuildOpts) {
    println!("    Running cargo build");

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
    println!("Running RUSTFLAGS='{rust_flags}' cargo {string_args}");
    let exit_status = Command::new("cargo")
        .env("RUSTFLAGS", &rust_flags)
        .args(args)
        .spawn()
        .expect("Failed to execute command")
        .wait()
        .unwrap();
    exit(exit_status.code().unwrap_or(1));
}
