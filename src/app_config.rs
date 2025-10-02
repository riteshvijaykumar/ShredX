use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_url: String,
    pub enable_server: bool,
    pub auto_upload_certificates: bool,
    pub local_cert_storage: String,
    pub debug_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: "http://10.0.43.207:8080".to_string(),
            enable_server: true,
            auto_upload_certificates: true,
            local_cert_storage: "./certificates".to_string(),
            debug_mode: false,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        // Try to load from environment variables first
        if let Ok(server_url) = std::env::var("HDD_TOOL_SERVER_URL") {
            return Self {
                server_url,
                enable_server: true,
                auto_upload_certificates: true,
                local_cert_storage: "./certificates".to_string(),
                debug_mode: std::env::var("HDD_TOOL_DEBUG").is_ok(),
            };
        }
        
        // Try to load from config file
        if let Ok(config_str) = fs::read_to_string("config.json") {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&config_str) {
                return config;
            }
        }
        
        // Return default and save it
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = serde_json::to_string_pretty(self)?;
        fs::write("config.json", config_str)?;
        Ok(())
    }
    
    pub fn is_server_enabled(&self) -> bool {
        self.enable_server && !self.server_url.is_empty()
    }
    
    pub fn get_cert_storage_path(&self) -> &str {
        &self.local_cert_storage
    }
    
    pub fn ensure_cert_directory(&self) -> Result<(), std::io::Error> {
        if !Path::new(&self.local_cert_storage).exists() {
            fs::create_dir_all(&self.local_cert_storage)?;
        }
        Ok(())
    }
}