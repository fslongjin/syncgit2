use git2::ProxyOptions;
use log::info;

use crate::{
    config_parser::{all_tasks, TaskId},
    constant::REPO_CLONE_DIR,
};

pub fn sync_git(task_id: TaskId) -> Result<(), String> {
    let task = all_tasks().get(&task_id).ok_or("task not found")?.clone();

    let mut worker = GitSyncWorker::new(task)?;
    worker.run()?;
    return Ok(());
}

struct GitSyncWorker {
    repo: Option<git2::Repository>,
    task: crate::config_parser::types::Config,
}

impl GitSyncWorker {
    pub fn new(task: crate::config_parser::types::Config) -> Result<GitSyncWorker, String> {
        Ok(GitSyncWorker { repo: None, task })
    }

    fn repo_path(&self) -> String {
        format!("{}/{}", REPO_CLONE_DIR, self.task.task_id().as_str())
    }

    fn run(&mut self) -> Result<(), String> {
        self.update_repo()?;
        self.sync()?;
        return Ok(());
    }

    pub fn sync(&self) -> Result<(), String> {
        info!(
            "Syncing repo: {} to remote...",
            self.task.task_id().as_str()
        );
        let repo = self.repo.as_ref().unwrap();

        // 确保目标仓库存在
        if let Ok(remote) = repo.find_remote("syncto") {
            if remote.url().unwrap() != self.task.destination().url() {
                repo.remote_set_url("syncto", self.task.destination().url())
                    .map_err(|e| e.to_string())?;
            }
        } else {
            repo.remote("syncto", self.task.destination().url())
                .map_err(|e| e.to_string())?;
        }

        let auth = self.destination_auth_options().unwrap();
        let git_config = self
            .repo
            .as_ref()
            .unwrap()
            .config()
            .map_err(|e| e.to_string())?;
        let mut push_options = git2::PushOptions::new();
        let mut remote_callbacks = git2::RemoteCallbacks::new();

        remote_callbacks.credentials(auth.credentials(&git_config));
        push_options.remote_callbacks(remote_callbacks);

        let mut remote = repo.find_remote("syncto").map_err(|e| e.to_string())?;

        remote
            .push(&["refs/*:refs/*"], Some(&mut push_options))
            .map_err(|e| e.to_string())?;

        info!(
            "Syncing repo: {} to remote... done",
            self.task.task_id().as_str()
        );
        return Ok(());
    }

    fn destination_auth_options(&self) -> Result<auth_git2::GitAuthenticator, String> {
        let mut auth: auth_git2::GitAuthenticator = auth_git2::GitAuthenticator::default();
        let a = self.task.destination().auth().unwrap();
        if let Some(rsa) = a.rsa() {
            auth = auth.add_ssh_key_from_file(rsa.private_key_path(), None);
        } else if let Some(pwd) = a.pwd() {
            auth = auth.add_plaintext_credentials("*", pwd.username(), pwd.password());
        }

        return Ok(auth);
    }

    fn source_auth_options(&self) -> Result<auth_git2::GitAuthenticator, String> {
        let mut auth: auth_git2::GitAuthenticator = auth_git2::GitAuthenticator::default();
        let a = self.task.origin().auth().ok_or("origin auth not found")?;
        if let Some(rsa) = a.rsa() {
            auth = auth.add_ssh_key_from_file(rsa.private_key_path(), None);
        } else if let Some(pwd) = a.pwd() {
            auth = auth.add_plaintext_credentials("*", pwd.username(), pwd.password());
        }

        return Ok(auth);
    }

    fn update_repo(&mut self) -> Result<(), String> {
        info!("Updating repo: {}", self.task.origin().url());
        self.clone_or_open_repo()?;
        let repo = self.repo.as_ref().unwrap();
        repo.remote_set_url("origin", self.task.origin().url())
            .map_err(|e| e.to_string())?;
        let mut use_auth = true;
        let auth = self.source_auth_options().unwrap_or_else(|_| {
            use_auth = false;
            auth_git2::GitAuthenticator::default()
        });

        let git_config = self
            .repo
            .as_ref()
            .unwrap()
            .config()
            .map_err(|e| e.to_string())?;
        let mut fetch_options = git2::FetchOptions::new();
        let mut remote_callbacks = git2::RemoteCallbacks::new();
        if use_auth {
            remote_callbacks.credentials(auth.credentials(&git_config));
        }
        fetch_options.remote_callbacks(remote_callbacks);

        let mut remote = repo.find_remote("origin").map_err(|e| e.to_string())?;

        fetch_options
            .follow_redirects(git2::RemoteRedirect::All)
            .prune(git2::FetchPrune::On)
            .download_tags(git2::AutotagOption::All);
        remote
            .fetch(&["refs/*:refs/*"], Some(&mut fetch_options), None)
            .map_err(|e| e.to_string())?;
        info!("Updating repo: {}... done", self.task.task_id().as_str());
        return Ok(());
    }

    fn clone_or_open_repo(&mut self) -> Result<(), String> {
        if self.repo.is_some() {
            return Ok(());
        }

        let path = self.repo_path();

        if std::path::Path::new(&path).exists() {
            self.repo = Some(git2::Repository::open(&path).map_err(|e| e.to_string())?);
            return Ok(());
        }
        // clone
        info!("Cloning repo: {}", self.task.origin().url());
        std::process::Command::new("git")
            .args(&["clone", self.task.origin().url(), &path, "--mirror"])
            .output()
            .map_err(|e| e.to_string())?;
        let repo = git2::Repository::open(&path).map_err(|e| e.to_string())?;

        self.repo = Some(repo);
        return Ok(());
    }
}
