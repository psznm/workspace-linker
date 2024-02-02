use std::{path::PathBuf, rc::Rc};

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
    options: Rc<ProjectDirOpts>,
}
impl ProjectDirectory {
    pub fn new(path: PathBuf, options: Rc<ProjectDirOpts>) -> Self {
        ProjectDirectory {
            path,
            links: vec![],
            options,
        }
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

    pub fn get_absolute_links(&self) -> impl Iterator<Item = (PathBuf, PathBuf)> + '_ {
        return self.links.iter().map(|link| {
            let link_path = self.path.join(link.link.clone());
            let link_dir = link_path.parent().unwrap();
            let destination_relative =
                pathdiff::diff_paths(link.destination_abs.clone(), link_dir).unwrap();
            (link_path, destination_relative)
        });
    }
}
