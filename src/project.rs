use std::{collections::HashMap, fs, path::PathBuf};

use log::trace;
use serde::{Deserialize, Serialize};

use crate::{project_directory::ProjectDirectory, CliError};

pub type ProjectDirs = HashMap<PathBuf, ProjectDirectory>;
pub struct Project {
    pub dirs: ProjectDirs,
    project_path: PathBuf,
}

impl Project {
    pub fn new<S: Into<PathBuf>>(project_path: S) -> Self {
        Project {
            dirs: HashMap::new(),
            project_path: project_path.into(),
        }
    }
    pub fn load(&mut self, path_relative: PathBuf) -> Result<(), CliError> {
        let dir = self.project_path.join(&path_relative);
        trace!("Project directory root: {dir:?}");
        let pkg_json_path = dir.join("package.json");
        trace!("package.json path: {pkg_json_path:?}");
        let pkg_json_content = fs::read_to_string(pkg_json_path)?;
        let res: PkgJson = serde_json::from_str(&pkg_json_content)?;

        let mut project_directory = ProjectDirectory::new(dir);
        if let Some(module_links) = res.module_aliases {
            if let Some(links) = module_links.links {
                for (link_name, dest_path) in links.iter() {
                    project_directory.add_link(link_name.clone(), dest_path.clone());
                }
            }
            if let Some(imports) = module_links.imports {
                imports
                    .iter()
                    .for_each(|import| project_directory.add_import(import))
            }
        }
        self.dirs.insert(path_relative, project_directory);
        if let Some(workspaces) = res.workspaces {
            for ws in workspaces {
                self.load(ws)?;
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PkgJson {
    #[serde(rename = "workspaceLinks")]
    module_aliases: Option<ModuleAliases>,
    workspaces: Option<Vec<PathBuf>>,
}
#[derive(Serialize, Deserialize, Debug)]
struct ModuleAliases {
    #[serde(rename = "local")]
    links: Option<HashMap<PathBuf, PathBuf>>,
    #[serde(rename = "imports")]
    imports: Option<Vec<PathBuf>>,
}
