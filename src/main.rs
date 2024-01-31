use std::{collections::HashMap, fs, os::unix, path::PathBuf};

use clap::Parser;
use log::{debug, info, trace};
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
impl ProjectDirectory {
    fn new(path: PathBuf) -> Self {
        ProjectDirectory {
            path,
            links: vec![],
        }
    }

    fn add_link(&mut self, link: &String, destination_relative: &String) {
        let destination_abs = self.path.join(destination_relative);
        self.links.push(Link {
            link: link.into(),
            destination_abs: destination_abs.clone(),
        });

        self.links.push(Link {
            link: PathBuf::from("node_modules").join(link),
            destination_abs,
        });
    }

    fn get_absolute_links(&self) -> impl Iterator<Item = (PathBuf, PathBuf)> + '_ {
        return self.links.iter().map(|link| {
            let link_path = self.path.join(link.link.clone());
            let link_dir = link_path.parent().unwrap();
            let destination_relative =
                pathdiff::diff_paths(link.destination_abs.clone(), link_dir).unwrap();
            (link_path, destination_relative)
        });
    }
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
    let res: ModuleAliases = serde_json::from_str(&pkg_json_content)?;

    let mut project_directory = ProjectDirectory::new(project_root_abs.clone().into());
    for (link_name, dest_path) in res.module_aliases.iter() {
        project_directory.add_link(link_name, dest_path);
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
