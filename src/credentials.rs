use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".config").join("syres").join("credentials.json"))
}

pub fn load_credentials() -> Result<Option<Credentials>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&path).context("Failed to read credentials file")?;
    let creds: Credentials =
        serde_json::from_str(&data).context("Failed to parse credentials file")?;
    Ok(Some(creds))
}

pub fn save_credentials(username: &str, password: &str) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }
    let creds = Credentials {
        username: username.to_string(),
        password: password.to_string(),
    };
    let data = serde_json::to_string_pretty(&creds).context("Failed to serialize credentials")?;
    fs::write(&path, &data).context("Failed to write credentials file")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms).context("Failed to set credentials file permissions")?;
    }

    Ok(())
}

pub fn clear_credentials() -> Result<()> {
    let path = config_path()?;
    if path.exists() {
        fs::remove_file(&path).context("Failed to remove credentials file")?;
    }
    Ok(())
}
