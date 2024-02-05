mod project;
mod project_directory;
use std::{fs, os::unix, path::PathBuf};

use clap::Parser;
use log::{debug, info};
use path_absolutize::*;
use project::Project;

use crate::project_directory::ProjectDirOpts;

#[derive(Parser)]
struct Cli {
    project_path: Option<PathBuf>,
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

    let project_root = args.project_path.unwrap_or(PathBuf::from("."));
    let project_root_path = PathBuf::from(&project_root);
    let project_root_abs = project_root_path.absolutize()?;

    let mut project = Project::new(
        project_root_abs.into(),
        ProjectDirOpts {
            no_node_modules: args.node_modules_skip,
            no_workspace: args.workspace_skip,
        },
    );

    project.load("".into())?;
    for (link_path, dest_path) in project.get_links() {
        let link_dir_abs = link_path.parent().unwrap();
        debug!("Ensuring dir {}", link_dir_abs.display());
        fs::create_dir_all(link_dir_abs)
            .map_err(|err| CliError::Io(err, Some(link_dir_abs.into())))?;

        if link_path.exists() || link_path.is_symlink() {
            debug!("Removing file {}", link_path.display());
            fs::remove_file(&link_path)
                .map_err(|err| CliError::Io(err, Some(link_path.clone())))?;
        }
        info!("Linking: {:?} -> {:?}", link_path, dest_path);
        unix::fs::symlink(dest_path, link_path)?;
    }

    Ok(())
}
