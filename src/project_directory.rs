use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::project::ProjectDirs;

pub struct Link {
    link: PathBuf,
    destination_relative: PathBuf,
}

pub struct ProjectDirectory {
    pub path: PathBuf,
    pub links: Vec<Link>,
    imports: HashSet<PathBuf>,
}
pub type Paths = HashMap<PathBuf, PathBuf>;

impl ProjectDirectory {
    pub fn new(path: PathBuf) -> Self {
        ProjectDirectory {
            path,
            links: vec![],
            imports: HashSet::new(),
        }
    }

    pub fn add_import(&mut self, import: &PathBuf) {
        self.imports.insert(import.into());
    }

    pub fn add_link(&mut self, link: PathBuf, destination_relative: PathBuf) {
        self.links.push(Link {
            link,
            destination_relative,
        });
    }

    pub fn get_paths(&self, project_dirs: &ProjectDirs) -> Paths {
        let mut paths = Paths::new();
        for item in self.links.iter() {
            paths.insert(item.link.clone(), item.destination_relative.clone());
        }
        for import in self.imports.iter() {
            let Some(project_dir) = project_dirs.get(import) else {
                panic!("Missing import {:?} in {:?}", import, self.path);
            };
            let ws_relative = pathdiff::diff_paths(&project_dir.path, &self.path).unwrap();
            for item in project_dir.links.iter() {
                if paths.get(&item.link).is_none() {
                    paths.insert(
                        item.link.clone(),
                        ws_relative.join(&item.destination_relative),
                    );
                }
            }
        }
        paths
    }
}
