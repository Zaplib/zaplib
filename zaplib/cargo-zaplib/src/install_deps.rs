#![allow(dead_code)]

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::{
    fs::{read_dir, remove_dir_all, remove_file},
    path::Path,
};

use std::process::{Command, Output};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn install_deps(devel: bool) {
    #[cfg(target_os = "macos")]
    install_deps_macos(devel);
    #[cfg(target_os = "linux")]
    install_deps_linux(devel);
    #[cfg(target_os = "windows")]
    install_deps_windows(devel);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn install_ci_deps() {
    #[cfg(target_os = "linux")]
    {
        install_deps_linux(false);
        download_cef_ci();
    }
}

pub(crate) fn run_command(program: &str, args: &[&str], msg: &str, current_dir: Option<String>) -> Output {
    let out = Command::new(program).current_dir(current_dir.unwrap_or_else(|| ".".to_string())).args(args).output().expect(msg);
    if !out.stdout.is_empty() {
        println!("{}", std::str::from_utf8(&out.stdout).ok().unwrap());
    }
    if !out.stderr.is_empty() {
        println!("{}", std::str::from_utf8(&out.stderr).ok().unwrap());
    }
    out
}

#[cfg(target_os = "macos")]
pub(crate) fn install_deps_macos(devel: bool) {
    // Check if Xcode CLT are installed
    let out = run_command("xcode-select", &["--print-path"], "Failed to check for Xcode command line tools", None);
    if !std::str::from_utf8(&out.stdout).ok().unwrap().is_empty() || !std::str::from_utf8(&out.stdout).ok().unwrap().is_empty() {
        println!("Xcode command line tools are already installed.");
    } else {
        run_command("xcode-select", &["--install"], "Failed to install Xcode command line tools;", None);
    }

    install_rust_toolchain();
    install_wasm32();
    install_rustfmt();
    install_clippy();
    install_cargo_extensions();
    install_rust_src();

    if devel {
        download_cef_devel();
    }
}

/// NOTE: when updating this function be sure to rebuild `Dockerfile-ci`.
#[cfg(target_os = "linux")]
pub(crate) fn install_deps_linux(devel: bool) {
    install_rust_toolchain();
    install_wasm32();
    install_rustfmt();
    install_clippy();
    install_cargo_extensions();

    run_command(
        "sudo",
        &["apt", "install", "-y", "libxcursor-dev", "libx11-dev", "libgl1-mesa-dev", "cmake", "git", "libgtk-3-dev"],
        "Failed to install Linux dev tools.",
        None,
    );

    install_rust_src();

    if devel {
        download_cef_devel();
    }

    println!(
        "To link against Objective-C (e.g. for running those tests), run https://github.com/plaurent/gnustep-build for your OS"
    );
}

#[cfg(target_os = "windows")]
pub(crate) fn install_deps_windows(devel: bool) {
    install_rust_toolchain();
    install_wasm32();

    run_command("rustup", &["target", "add", "x86_64-pc-windows-msvc"], "Failed to add MSVC target", None);
    run_command("rustup", &["target", "add", "x86_64-pc-windows-gnu"], "Failed to add GNU target", None);

    install_rustfmt();
    install_clippy();
    install_cargo_extensions();
    install_rust_src();

    if devel {
        // TODO(JP): auto-download CEF here... (from https://cef-builds.spotifycdn.com/index.html#windows64)
    }
}

fn install_rust_toolchain() {
    run_command("rustup", &["toolchain", "install", "nightly-2022-01-18"], "Failed to install rust toolchain.", None);
}

fn install_wasm32() {
    run_command("rustup", &["target", "add", "wasm32-unknown-unknown"], "Failed to add WAsm32 target", None);
}

fn install_rustfmt() {
    run_command("rustup", &["component", "add", "rustfmt"], "Failed to add rustfmt component", None);
}

fn install_clippy() {
    run_command("rustup", &["component", "add", "clippy"], "Failed to add clippy component", None);
}

fn install_cargo_extensions() {
    run_command("cargo", &["install", "cargo-bundle", "mdbook"], "Failed to install cargo extensions.", None);
}

fn install_rust_src() {
    run_command("rustup", &["component", "add", "rust-src"], "Failed to add rust-src component.", None);
}

#[cfg(target_os = "linux")]
fn download_cef_ci() {
    download_cef("/tmp");
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn download_cef_devel() {
    download_cef("zaplib/main/bind/cef-sys/deps");
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn download_cef(work_dir: &str) {
    assert!(Path::new(work_dir).exists());

    #[cfg(target_os = "macos")]
    let cef_binary = "cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64.tar.bz2".to_string();

    #[cfg(target_os = "linux")]
    let cef_binary = "cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_linux64_minimal.tar.bz2".to_string();

    let cef_deps = read_dir(work_dir).unwrap();
    cef_deps.flatten().for_each(|dep| {
        if dep.path().to_str().unwrap().contains("cef_binary_") {
            println!("removing {:?}", &dep);
            let _ = remove_dir_all(dep.path());
        }
    });

    run_command(
        "curl",
        &[&format!("https://cef-builds.spotifycdn.com/{}", &cef_binary), "-o", &cef_binary],
        "Failed to download Cef binaries.",
        Some(work_dir.into()),
    );

    #[cfg(target_os = "macos")]
    let tar_opts = "-zxvf";

    #[cfg(target_os = "linux")]
    let tar_opts = "-xvjf";

    run_command("tar", &[tar_opts, &cef_binary], "Failed to extract Cef binaries.", Some(work_dir.into()));

    // Remove downloaded file
    let _ = remove_file(format!("{}/{}", work_dir, cef_binary));
}
