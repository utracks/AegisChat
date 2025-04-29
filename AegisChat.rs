use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

mod config {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AppConfig {
        pub theme: String,
        pub log_level: String,
        pub auto_connect: bool,
        pub key_rotation: u64,
    }

    #[derive(Debug)]
    pub enum ConfigError {
        Io(io::Error),
        Parse(String),
        Validation(String),
    }

    impl From<io::Error> for ConfigError {
        fn from(e: io::Error) -> Self {
            ConfigError::Io(e)
        }
    }
}

struct ConfigManager;

impl ConfigManager {
    pub fn initialize() -> Result<(), config::ConfigError> {
        Self::create_directories()?;
        Self::setup_themes()?;
        Self::setup_config()?;
        Self::rotate_backups()?;
        Ok(())
    }

    fn create_directories() -> io::Result<()> {
        let dirs = [
            "assets/themes",
            "~/.securechat/keys",
            "~/.securechat/history",
            "~/.securechat/backups",
            "~/.securechat/quarantine",
        ];
        
        for dir in dirs {
            fs::create_dir_all(shellexpand::tilde(dir).into_owned())?;
        }
        Ok(())
    }

    fn setup_themes() -> Result<(), config::ConfigError> {
        let default_themes = [
            ("dark", include_str!("../assets/default_themes/dark.json")),
            ("light", include_str!("../assets/default_themes/light.json")),
        ];

        for (name, content) in default_themes {
            let path = PathBuf::from("assets/themes").join(format!("{}.json", name));
            if !path.exists() {
                Self::validate_theme(content)?;
                fs::write(path, content)?;
            }
        }
        Ok(())
    }

    fn validate_theme(content: &str) -> Result<(), config::ConfigError> {
        #[derive(Deserialize)]
        struct Theme {
            text: String,
            background: String,
            accent: String,
            borders: String,
        }

        serde_json::from_str::<Theme>(content)
            .map_err(|e| config::ConfigError::Validation(e.to_string()))?;
        Ok(())
    }

    fn setup_config() -> Result<(), config::ConfigError> {
        let config_path = shellexpand::tilde("~/.securechat/config.ron").into_owned();
        
        if !Path::new(&config_path).exists() {
            return Self::create_default_config(&config_path);
        }

        match Self::try_load_config(&config_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                log::warn!("Config repair needed: {}", e);
                Self::repair_config(&config_path)
            }
        }
    }

    fn try_load_config(path: &str) -> Result<config::AppConfig, config::ConfigError> {
        let content = fs::read_to_string(path)?;
        let config = ron::from_str::<config::AppConfig>(&content)
            .map_err(|e| config::ConfigError::Parse(e.to_string()))?;
        
        if config.key_rotation == 0 {
            return Err(config::ConfigError::Validation(
                "Key rotation must be > 0".to_string()
            ));
        }
        
        Ok(config)
    }

    fn repair_config(config_path: &str) -> Result<(), config::ConfigError> {
        // Move broken config to quarantine
        let quarantine_dir = shellexpand::tilde("~/.securechat/quarantine").into_owned();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let quarantine_path = PathBuf::from(quarantine_dir)
            .join(format!("config_{}.ron.broken", timestamp));
        
        fs::rename(config_path, quarantine_path)?;
        
        // Try to salvage values from broken config
        let salvaged = Self::salvage_config(config_path)?;
        
        // Create new config with salvaged values
        Self::create_config(config_path, salvaged)
    }

    fn salvage_config(path: &str) -> Result<Option<config::AppConfig>, config::ConfigError> {
        let broken_content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };

        if let Ok(mut config) = ron::from_str::<config::AppConfig>(&broken_content) {
            // Fix invalid values but keep valid ones
            if config.key_rotation == 0 {
                config.key_rotation = 86400;
            }
            if config.theme.is_empty() {
                config.theme = "dark".to_string();
            }
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    fn create_config(path: &str, salvaged: Option<config::AppConfig>) -> Result<(), config::ConfigError> {
        let default_config = match salvaged {
            Some(c) => c,
            None => config::AppConfig {
                theme: "dark".to_string(),
                log_level: "info".to_string(),
                auto_connect: true,
                key_rotation: 86400,
            },
        };

        let config_str = ron::to_string(&default_config)
            .map_err(|e| config::ConfigError::Parse(e.to_string()))?;
        
        fs::write(path, config_str)?;
        Ok(())
    }

    fn create_default_config(path: &str) -> Result<(), config::ConfigError> {
        let default_config = config::AppConfig {
            theme: "dark".to_string(),
            log_level: "info".to_string(),
            auto_connect: true,
            key_rotation: 86400,
        };

        let config_str = ron::to_string(&default_config)
            .map_err(|e| config::ConfigError::Parse(e.to_string()))?;
        
        fs::write(path, config_str)?;
        Ok(())
    }

    fn backup_config(path: &str) -> io::Result<()> {
        let backup_dir = shellexpand::tilde("~/.securechat/backups").into_owned();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        
        let backup_path = PathBuf::from(backup_dir).join(format!("config_{}.ron.bak", timestamp));
        fs::copy(path, backup_path)?;
        Ok(())
    }

    fn rotate_backups() -> io::Result<()> {
        let backup_dir = shellexpand::tilde("~/.securechat/backups").into_owned();
        let mut backups: Vec<fs::DirEntry> = fs::read_dir(&backup_dir)?
            .filter_map(Result::ok)
            .collect();

        // Keep last 5 backups
        if backups.len() > 5 {
            backups.sort_by_key(|f| f.metadata().ok()?.modified().ok());
            for old_backup in backups.drain(..backups.len()-5) {
                fs::remove_file(old_backup.path())?;
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Initialize configuration
    if let Err(e) = ConfigManager::initialize() {
        log::error!("Failed to initialize config: {}", e);
        // Attempt to continue with safe defaults
    }

    // Rest of application
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main application loop
    loop {
        // Your application logic here
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}