use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::Digest;

use super::TaskId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    package: Package,
    origin: GitEndpoint,
    destination: GitEndpoint,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Package {
    name: String,
    /// 该值应当全局唯一, 会被用作 webhook 的url路径
    webhook_name: String,
    secret: Option<String>,
}

impl Package {
    pub fn secret(&self) -> Option<&str> {
        self.secret.as_ref().map(|s| s.as_str())
    }

    pub fn secret_sha256(&self) -> Option<String> {
        self.secret.as_ref().map(|s| {
            let mut hasher = sha2::Sha256::new();
            hasher.update(s);
            let result = hasher.finalize();
            format!("{:x}", result)
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitEndpoint {
    url: String,
    auth: Option<GitAuth>,
    endpoint_type: Option<String>,
}

impl GitEndpoint {
    pub fn is_github(&self) -> bool {
        self.endpoint_type
            .as_ref()
            .map(|s| s.to_lowercase() == "github")
            .unwrap_or(false)
    }
    pub fn is_gitee(&self) -> bool {
        self.endpoint_type
            .as_ref()
            .map(|s| s.to_lowercase() == "gitee")
            .unwrap_or(false)
    }

    pub fn is_gitlab(&self) -> bool {
        self.endpoint_type
            .as_ref()
            .map(|s| s.to_lowercase() == "gitlab")
            .unwrap_or(false)
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn auth(&self) -> Option<&GitAuth> {
        self.auth.as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitAuth {
    rsa: Option<GitRSAAuth>,
    pwd: Option<GitPasswordAuth>,
}

impl GitAuth {
    pub fn rsa(&self) -> Option<&GitRSAAuth> {
        self.rsa.as_ref()
    }

    pub fn pwd(&self) -> Option<&GitPasswordAuth> {
        self.pwd.as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitRSAAuth {
    /// 私钥路径
    private_key: String,
    /// 公钥路径
    public_key: String,
}

impl GitRSAAuth {
    pub fn private_key(&self) -> String {
        std::fs::read_to_string(&self.private_key).unwrap()
    }

    pub fn public_key(&self) -> String {
        std::fs::read_to_string(&self.public_key).unwrap()
    }

    pub fn private_key_path(&self) -> PathBuf {
        PathBuf::from(&self.private_key)
    }

    pub fn public_key_path(&self) -> PathBuf {
        PathBuf::from(&self.public_key)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitPasswordAuth {
    username: String,
    password: String,
}

impl GitPasswordAuth {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

impl Config {
    pub fn name(&self) -> &str {
        &self.package.name
    }

    pub fn task_id(&self) -> TaskId {
        TaskId::new(&self.package.webhook_name)
    }

    pub fn secret_sha256(&self) -> Option<String> {
        self.package.secret_sha256()
    }

    pub fn validate(&self) -> Result<(), String> {
        self.check_empty()?;
        Ok(())
    }

    pub fn origin(&self) -> &GitEndpoint {
        &self.origin
    }

    pub fn destination(&self) -> &GitEndpoint {
        &self.destination
    }

    fn check_empty(&self) -> Result<(), String> {
        if self.package.name.is_empty() {
            return Err("package name is empty".to_string());
        }
        if self.package.webhook_name.is_empty() {
            return Err("webhook name is empty".to_string());
        }
        if self.origin.url.is_empty() {
            return Err("origin url is empty".to_string());
        }
        if self.destination.url.is_empty() {
            return Err("destination url is empty".to_string());
        }
        if self.destination.auth.is_none() {
            return Err("destination auth is empty".to_string());
        }
        if let Some(auth) = &self.destination.auth {
            if auth.pwd.is_none() && auth.rsa.is_none() {
                return Err("destination auth is empty".to_string());
            }
        }

        return Ok(());
    }
}
