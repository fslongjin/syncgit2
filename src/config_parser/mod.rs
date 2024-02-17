use std::{collections::HashMap, path::PathBuf};

use log::info;

use crate::constant::REPO_CONFIG_DIR;

pub mod types;
mod validator;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TaskId {
    id: String,
}

impl TaskId {
    pub fn new(id: &str) -> TaskId {
        TaskId {
            id: id.to_string().to_ascii_lowercase(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.id
    }
}

lazy_static! {
    pub static ref ALL_TASKS: HashMap<TaskId, types::Config> = init_all_tasks();
}

fn init_all_tasks() -> HashMap<TaskId, types::Config> {
    let file_paths = all_config_paths();
    let tasks = validator::load_all_configs(&file_paths);

    info!("tasks: {:?}", tasks);
    return tasks;
}

pub fn all_tasks() -> &'static HashMap<TaskId, types::Config> {
    &ALL_TASKS
}

fn all_config_paths() -> Vec<PathBuf> {
    let mut files = Vec::new();
    // list all files in the config directory
    if !std::path::Path::new(REPO_CONFIG_DIR).exists() {
        std::fs::create_dir("config/repo").unwrap();
        return files;
    }
    std::fs::read_dir(REPO_CONFIG_DIR)
        .unwrap()
        .for_each(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            }
        });

    return files;
}
