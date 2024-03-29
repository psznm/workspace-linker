use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::project::ProjectDirs;

struct Link {
    link: PathBuf,
    destination_relative: PathBuf,
}

pub struct ProjectDirectory {
    pub path: PathBuf,
    links: Vec<Link>,
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
    fn get_links<'a>(
        &'a self,
        project_dirs: &'a ProjectDirs,
    ) -> Box<dyn Iterator<Item = &'a Link> + 'a> {
        self.imports
            .iter()
            .fold(Box::new(self.links.iter()), move |acc, import| {
                let Some(project_dir) = project_dirs.get(import) else {
                    panic!("Missing import {:?} in {:?}", import, self.path);
                };
                let import_iter = project_dir.get_links(project_dirs);
                Box::new(acc.chain(import_iter))
            })
    }

    pub fn get_paths(&self, project_dirs: &ProjectDirs) -> Paths {
        self.get_links(project_dirs)
            .fold(Paths::new(), |mut acc, item| {
                if acc.get(&item.link).is_none() {
                    acc.insert(item.link.clone(), item.destination_relative.clone());
                }
                acc
            })
    }
}
