use std::{collections::HashMap, path::PathBuf};

use super::TaskId;

pub(super) fn load_all_configs(paths: &Vec<PathBuf>) -> HashMap<TaskId, super::types::Config> {
    let mut tasks = HashMap::new();
    paths.iter().for_each(|path| {
        let config = load_config(path);
        if let Some(config) = config {
            if let Err(e) = config.validate() {
                log::error!("config file {:?} is invalid: {}", path, e);
                return;
            }
            tasks.insert(config.task_id(), config);
        }
    });
    return tasks;
}

fn load_config(path: &PathBuf) -> Option<super::types::Config> {
    if path.extension() != Some("toml".as_ref()) {
        log::warn!("config file {:?} is not a toml file", path);
        return None;
    }
    let config: Result<super::types::Config, toml::de::Error> =
        toml::from_str(&std::fs::read_to_string(path).unwrap());
    if let Err(e) = config {
        log::warn!("Failed to parse config file: {:?}, error: {}", path, e);
        return None;
    }
    return Some(config.unwrap());
}
