use std::process::Command;
use log::info;

pub async fn build_npm_package(path: String) {
  if !std::path::Path::new(&format!("{}/zaplib/web", path)).is_dir() {
      return;
  }

  if std::path::Path::new(&format!("{}/zaplib/web/dist/zaplib_runtime.js", path)).exists() {
      info!("Found zaplib/web/zaplib_runtime.js; skipping build");
      return;
  }

  info!("Did not find zaplib/web/zaplib_runtime; building...");

  let npm_status = Command::new("npm")
                  .arg("--version")
                  .stdout(std::process::Stdio::null())
                  .status();
  if let Err(_) = npm_status {
      info!("npm not found; skipping build; please install npm");
      return;
  }
  
  let yarn_status = Command::new("yarn")
                  .arg("--version")
                  .stdout(std::process::Stdio::null())
                  .status();
  if let Err(_) = yarn_status{
      info!("yarn not found; installing yarn");
      let yarn_install_status = Command::new("npm")
                   .arg("install")
                   .arg("-g")
                   .arg("yarn")
                  .status();
      if let Err(_) =  yarn_install_status {
          info!("yarn install failed; skipping build; please install yarn");
          return;
      }
  }

  let packages_install_status = Command::new("yarn")
                  .current_dir(&format!("{}/zaplib/web", path))
                  .status();
  if let Err(_) = packages_install_status {
      info!("yarn command in /zaplib/web directory failed; skipping build");
      return;
  }

  let build_status = Command::new("yarn")
                  .arg("build-dev")
                  .current_dir(&format!("{}/zaplib/web", path))
                  .status();
  if let Err(_) = build_status {
      info!("yarn build in /zaplib/web directory failed; skipping build");
      return;
  }
}
