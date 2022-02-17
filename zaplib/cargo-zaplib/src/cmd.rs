use clap::{App, AppSettings, Arg};

pub(crate) fn cmd() {
    // Use "info" logging level by default.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

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
        .subcommand(
            App::new("serve")
                .arg(Arg::new("path").takes_value(true).default_value(".").help("Path to files"))
                .arg(Arg::new("port").long("port").takes_value(true).default_value("3000").help("TCP port to use"))
                .arg(
                    Arg::new("ssl").long("ssl").takes_value(false).help("Start HTTPS server with a self-signed SSL certificate"),
                ),
        )
        .get_matches();

    if let Some(cmd) = matches.subcommand_matches("build") {
        crate::build::build(crate::build::BuildOpts {
            release: cmd.is_present("release"),
            use_simd128: cmd.is_present("simd128"),
            all_targets: cmd.is_present("all-targets"),
            workspace: cmd.is_present("workspace"),
            features: cmd.value_of("features").unwrap_or("").to_string(),
            package: cmd.value_of("package").unwrap_or("").to_string(),
        });
    }

    if let Some(cmd) = matches.subcommand_matches("install-deps") {
        if cmd.is_present("ci") {
            crate::install_deps::install_ci_deps();
        } else {
            crate::install_deps::install_deps(cmd.is_present("devel"));
        }
    }

    if let Some(cmd) = matches.subcommand_matches("serve") {
        crate::serve::serve(cmd.value_of_t_or_exit("path"), cmd.value_of_t_or_exit("port"), cmd.is_present("ssl"));
    }
}
