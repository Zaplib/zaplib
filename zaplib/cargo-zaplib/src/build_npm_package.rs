use log::info;
use std::process::Command;

/// Builds the Zaplib npm package for use in local examples and tutorials
///
/// Only runs if it detects it's inside the Zaplib repo and npm is installed
/// It installs yarn (via npm), if necessary, and then builds the Zaplib npm package
pub async fn build_npm_package(path: &str) {
    // Only run if we're in the Zaplib repo
    if !std::path::Path::new(&format!("{}/zaplib/web", path)).is_dir() {
        return;
    }

    if std::path::Path::new(&format!("{}/zaplib/web/dist/zaplib_runtime.js", path)).exists() {
        info!("Found zaplib/web/zaplib_runtime.js; skipping build");
        return;
    }

    info!("Did not find zaplib/web/zaplib_runtime.js; building...");

    let npm_status = Command::new("npm").arg("--version").stdout(std::process::Stdio::null()).status();
    if npm_status.is_err() {
        info!("npm not found; skipping build; please install npm");
        return;
    }

    let yarn_status = Command::new("yarn").arg("--version").stdout(std::process::Stdio::null()).status();
    if yarn_status.is_err() {
        info!("yarn not found; installing yarn");
        let yarn_install_status = Command::new("npm").arg("install").arg("-g").arg("yarn").status();
        if yarn_install_status.is_err() {
            info!("Installing yarn failed; skipping build; please install yarn");
            return;
        }
    }

    let packages_install_status = Command::new("yarn").current_dir(&format!("{}/zaplib/web", path)).status();
    if packages_install_status.is_err() {
        info!("`yarn` command in ./zaplib/web directory failed; skipping build");
        return;
    }

    let build_status = Command::new("yarn").arg("build-dev").current_dir(&format!("{}/zaplib/web", path)).status();
    if build_status.is_err() {
        info!("`yarn build-dev` in ./zaplib/web directory failed; skipping build");
    }
}
