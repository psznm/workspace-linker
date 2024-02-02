use std::{collections::HashMap, fs, path::PathBuf, rc::Rc};

use log::trace;
use serde::{Deserialize, Serialize};

use crate::{
    project_directory::{ProjectDirOpts, ProjectDirectory},
    CliError,
};

pub struct Project {
    dirs: HashMap<PathBuf, ProjectDirectory>,
    opts: Rc<ProjectDirOpts>,
    project_path: PathBuf,
}

impl Project {
    pub fn new(project_path: PathBuf, opts: ProjectDirOpts) -> Self {
        Project {
            dirs: HashMap::new(),
            opts: Rc::new(opts),
            project_path,
        }
    }
    pub fn load(&mut self, path_relative: PathBuf) -> Result<(), CliError> {
        let dir = self.project_path.join(&path_relative);
        trace!("Project directory root: {dir:?}");
        let pkg_json_path = dir.join("package.json");
        trace!("package.json path: {pkg_json_path:?}");
        let pkg_json_content = fs::read_to_string(pkg_json_path)?;
        trace!("package.json content: {}", pkg_json_content);
        let res: PkgJson = serde_json::from_str(&pkg_json_content)?;

        let mut project_directory = ProjectDirectory::new(dir.clone(), Rc::clone(&self.opts));
        if let Some(module_links) = res.module_aliases {
            if let Some(links) = module_links.links {
                for (link_name, dest_path) in links.iter() {
                    project_directory.add_link(link_name, dest_path);
                }
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

    pub fn get_links(&self) -> impl Iterator<Item = (PathBuf, PathBuf)> + '_ {
        self.dirs.values().flat_map(|dir| dir.get_absolute_links())
    }
}

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
