use std::{collections::HashSet, path::PathBuf, rc::Rc};

use crate::project::ProjectDirs;

struct Link {
    link: PathBuf,
    destination_abs: PathBuf,
}
pub struct ProjectDirOpts {
    pub no_node_modules: bool,
    pub no_workspace: bool,
}

pub struct ProjectDirectory {
    path: PathBuf,
    links: Vec<Link>,
    imports: HashSet<PathBuf>,
    options: Rc<ProjectDirOpts>,
}

type LinkIter<'a> = Box<dyn Iterator<Item = &'a Link> + 'a>;
impl ProjectDirectory {
    pub fn new(path: PathBuf, options: Rc<ProjectDirOpts>) -> Self {
        ProjectDirectory {
            path,
            links: vec![],
            imports: HashSet::new(),
            options,
        }
    }

    pub fn add_import(&mut self, import: &PathBuf) {
        self.imports.insert(import.into());
    }

    pub fn add_link(&mut self, link: &PathBuf, destination_relative: &PathBuf) {
        let destination_abs = self.path.join(destination_relative);
        if !self.options.no_workspace {
            self.links.push(Link {
                link: link.into(),
                destination_abs: destination_abs.clone(),
            });
        }

        if !self.options.no_node_modules {
            self.links.push(Link {
                link: PathBuf::from("node_modules").join(link),
                destination_abs,
            });
        }
    }
    fn get_links<'a>(&'a self, project_dirs: &'a ProjectDirs) -> LinkIter<'a> {
        self.imports
            .iter()
            .fold(Box::new(self.links.iter()), move |acc, import| {
                let Some(project_dir) = project_dirs.get(import) else {
                    panic!("Missing import");
                };
                let import_iter = project_dir.get_links(project_dirs);
                Box::new(acc.chain(import_iter))
            })
    }

    pub fn get_absolute_links<'a>(
        &'a self,
        project_dirs: &'a ProjectDirs,
    ) -> Vec<(PathBuf, PathBuf)> {
        let links = self.get_links(project_dirs);
        let res: Vec<(PathBuf, PathBuf)> = links
            .map(|link| {
                let link_path = self.path.join(link.link.clone());
                let link_dir = link_path.parent().unwrap();
                let destination_relative =
                    pathdiff::diff_paths(link.destination_abs.clone(), link_dir).unwrap();
                (link_path, destination_relative)
            })
            .collect();
        res
    }
}
