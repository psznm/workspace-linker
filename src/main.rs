mod project_directory;
use std::{collections::HashMap, fs, os::unix, path::PathBuf};

use clap::Parser;
use log::{debug, info, trace};
use path_absolutize::*;
use serde::{Deserialize, Serialize};

use crate::project_directory::{ProjectDirOpts, ProjectDirectory};

#[derive(Serialize, Deserialize, Debug)]
struct PkgJson {
    #[serde(rename = "moduleAliases")]
    module_aliases: Option<ModuleAliases>,
    workspaces: Option<Vec<PathBuf>>,
}
#[derive(Serialize, Deserialize, Debug)]
struct ModuleAliases {
    links: Option<HashMap<PathBuf, PathBuf>>,
    imports: Option<Vec<PathBuf>>,
}

#[derive(Parser)]
struct Cli {
    project_path: PathBuf,
    #[arg(short, long, help = "Do not link to node_modules of each workspace")]
    node_modules_skip: bool,
    #[arg(short, long, help = "Do not link to root of each workspace")]
    workspace_skip: bool,
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

#[derive(Debug)]
enum CliError {
    Io(std::io::Error, Option<PathBuf>),
    Serialization(serde_json::Error),
}
impl From<std::io::Error> for CliError {
    fn from(value: std::io::Error) -> Self {
        CliError::Io(value, None)
    }
}
impl From<serde_json::Error> for CliError {
    fn from(value: serde_json::Error) -> Self {
        CliError::Serialization(value)
    }
}

fn main() -> Result<(), CliError> {
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
    let res: PkgJson = serde_json::from_str(&pkg_json_content)?;

    let mut project_directory = ProjectDirectory::new(
        project_root_abs.clone().into(),
        ProjectDirOpts {
            no_node_modules: args.node_modules_skip,
            no_workspace: args.workspace_skip,
        },
    );
    if let Some(module_links) = res.module_aliases {
        if let Some(links) = module_links.links {
            for (link_name, dest_path) in links.iter() {
                project_directory.add_link(link_name, dest_path);
            }
        }
    }
    for (link_path, dest_path) in project_directory.get_absolute_links() {
        let link_dir_abs = link_path.parent().unwrap();
        debug!("Ensuring dir {}", link_dir_abs.display());
        fs::create_dir_all(link_dir_abs)
            .map_err(|err| CliError::Io(err, Some(link_dir_abs.into())))?;

        if link_path.exists() || link_path.is_symlink() {
            debug!("Removing file {}", link_path.display());
            fs::remove_file(link_path.clone())
                .map_err(|err| CliError::Io(err, Some(link_path.clone())))?;
        }
        info!(
            "Linking: {} -> {}",
            link_path.display(),
            dest_path.display()
        );
        unix::fs::symlink(dest_path, link_path)?;
    }

    //println!("Hello, world! {}", pkg_json_content);
    Ok(())
}
