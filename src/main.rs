use std::{
    collections::HashMap,
    error::Error,
    fs,
    os::unix,
    path::{Path, PathBuf},
};

use clap::Parser;
use log::{debug, error, info, trace};
use path_absolutize::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ModuleAliases {
    #[serde(rename = "moduleAliases")]
    module_aliases: HashMap<String, String>,
}

#[derive(Parser)]
struct Cli {
    project_path: PathBuf,
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

struct Link {
    link: PathBuf,
    destination_abs: PathBuf,
}
struct ProjectDirectory {
    path: PathBuf,
    links: Vec<Link>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    env_logger::builder()
        .filter_level(args.verbosity.log_level_filter())
        .init();

    let project_root = args.project_path;
    let project_root_path = PathBuf::from(&project_root);
    let project_root_abs = project_root_path.absolutize()?;
    trace!("Project root: {project_root:?}");
    let pkg_json_path = project_root.join("package.json");
    trace!("package.json path: {pkg_json_path:?}");
    let pkg_json_content = fs::read_to_string(pkg_json_path)?;
    trace!("package.json content: {}", pkg_json_content);
    let res: ModuleAliases = serde_json::from_str(&pkg_json_content)?;

    let mut links_iter = res.module_aliases.iter();
    while let Some((link_name, dest_path)) = links_iter.next() {
        trace!("Will link path {} as {}", dest_path, link_name);
        let mut link_base_iter = [".", "node_modules"].iter();
        while let Some(link_base) = link_base_iter.next() {
            let link_path = project_root.join(link_base).join(link_name);
            let link_path = Path::new(&link_path).absolutize()?;
            if let Some(link_dir_name) = link_path.parent() {
                let link_dir_abs = link_dir_name.absolutize()?;
                debug!("Ensuring dir {}", link_dir_abs.display());
                fs::create_dir_all(link_dir_abs.clone())?;

                let idk = project_root_abs.join(dest_path);
                let dest_abs = idk.absolutize()?;

                let Some(link_content) =
                    pathdiff::diff_paths(dest_abs.clone(), link_dir_abs.clone())
                else {
                    error!("Failed to diff paths");
                    continue;
                };
                trace!(
                    "Diff between {} and {} is {}",
                    link_dir_abs.display(),
                    project_root_abs.display(),
                    link_content.display(),
                );

                if link_path.exists() || link_path.is_symlink() {
                    debug!("Removing file {}", link_path.display());
                    fs::remove_file(link_path.clone())?;
                }
                info!(
                    "Linking: {} -> {}",
                    link_path.display(),
                    link_content.display()
                );
                unix::fs::symlink(link_content, link_path)?;
            }
        }
    }

    //println!("Hello, world! {}", pkg_json_content);
    Ok(())
}
