mod project;
mod project_directory;
use std::{collections::BTreeMap, fs, os::unix, path::PathBuf};

use clap::Parser;
use log::{debug, info};
use path_absolutize::*;
use project::Project;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Cli {
    project_path: Option<PathBuf>,
    #[arg(short, long, help = "Update paths tsconfig.json")]
    tsconfig_update: bool,
    #[arg(short, long, help = "Update paths jsconfig.json")]
    jsconfig_update: bool,
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
type Other = serde_json::Map<String, serde_json::Value>;
#[derive(Serialize, Deserialize, Debug)]
struct JsTsConfig {
    #[serde(flatten)]
    other: Other,
    #[serde(rename = "compilerOptions")]
    compiler_options: CompilerOptions,
}
#[derive(Serialize, Deserialize, Debug)]
struct CompilerOptions {
    #[serde(flatten)]
    other: Other,
    paths: Option<Paths>,
}

type Paths = BTreeMap<PathBuf, Vec<PathBuf>>;

fn main() -> Result<(), CliError> {
    let args = Cli::parse();
    env_logger::builder()
        .filter_level(args.verbosity.log_level_filter())
        .init();

    let project_root = args.project_path.unwrap_or(PathBuf::from("."));
    let project_root_path = PathBuf::from(&project_root);
    let project_root_abs = project_root_path.absolutize()?;

    let mut project = Project::new(project_root_abs.clone());

    project.load("".into())?;

    for (workspace_path, links) in &project.dirs {
        let workspace_paths = links.get_paths(&project.dirs);
        let workspace_path_abs = project_root_abs.join(workspace_path);
        for (link_path, dest_path_ws_relative) in &workspace_paths {
            let link_path_abs = workspace_path_abs.join("node_modules").join(link_path);
            let link_dir_abs = link_path_abs.parent().unwrap();

            let path_from_link_to_ws =
                pathdiff::diff_paths(&workspace_path_abs, link_dir_abs).unwrap();
            let dest_path_link_relative = path_from_link_to_ws.join(dest_path_ws_relative);
            debug!("Ensuring dir {:?}", link_dir_abs);
            fs::create_dir_all(link_dir_abs)
                .map_err(|err| CliError::Io(err, Some(link_dir_abs.into())))?;

            if link_path_abs.exists() || link_path_abs.is_symlink() {
                debug!("Removing file {:?}", link_path_abs);
                fs::remove_file(&link_path_abs)
                    .map_err(|err| CliError::Io(err, Some(link_path_abs.clone())))?;
            }
            info!(
                "Linking: {:?} -> {:?}",
                link_path_abs, dest_path_link_relative
            );
            unix::fs::symlink(dest_path_link_relative, link_path_abs)?;
        }

        let links_for_configs: Paths =
            workspace_paths
                .into_iter()
                .fold(Paths::new(), |mut map, item| {
                    map.insert(item.0.join("*"), vec![item.1.join("*")]);
                    map
                });

        if args.tsconfig_update {
            update_config_json(
                &workspace_path_abs.join("tsconfig.json"),
                links_for_configs.clone(),
            )?;
        }
        if args.jsconfig_update {
            update_config_json(&workspace_path_abs.join("jsconfig.json"), links_for_configs)?;
        }
    }

    Ok(())
}

fn update_config_json(config_path: &PathBuf, paths: Paths) -> Result<(), CliError> {
    if config_path.exists() {
        info!("Updating: {:?}", config_path);
        let pkg_json_content = fs::read_to_string(config_path)?;
        let mut res: JsTsConfig = serde_json::from_str(&pkg_json_content)?;
        if paths.is_empty() {
            if res.compiler_options.paths.is_none() {
                return Ok(());
            }
            res.compiler_options.paths = None;
        } else {
            res.compiler_options.paths = Some(paths);
        }
        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        res.serialize(&mut ser).unwrap();
        let serialized = String::from_utf8(buf).unwrap();
        fs::write(config_path, format!("{}\n", serialized))?;
    }
    Ok(())
}
