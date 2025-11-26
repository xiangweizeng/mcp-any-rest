//! Dynamic configuration management for MCP-ANY-REST
//! This module provides real-time configuration management with web interface integration

use anyhow::{Context, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tokio::sync::broadcast;

use crate::config::config::Config;
use crate::config::loader::ConfigLoader;
use crate::config::module::GlobalModuleConfig;
use crate::config::module::ModuleConfig;
use crate::config::preset_loader::PresetLoader;

/// Configuration preset definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    pub name: String,
    pub description: String,
    pub modules: HashMap<String, ModuleConfig>,
    pub default_access_level: Option<String>,
    pub default_rate_limit: Option<u32>,
}

/// Preset configuration index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetIndex {
    pub presets: Vec<PresetInfo>,
}

/// Preset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub file: String,
    pub enabled: bool,
    pub priority: u32,
}

/// Dynamic configuration manager
pub struct DynamicConfigManager {
    /// Current configuration
    config: Arc<RwLock<Config>>,
    /// Configuration file path
    config_path: PathBuf,
    /// Module configuration file path
    module_config_path: PathBuf,
    /// Preset configuration directory path
    preset_config_path: PathBuf,
    /// Configuration change notifier
    change_sender: broadcast::Sender<ConfigChangeEvent>,
    /// Last modification time
    last_modified: Arc<RwLock<SystemTime>>,
    /// Configuration change history (last 100 changes)
    change_history: Arc<RwLock<VecDeque<ConfigChangeEvent>>>,
}

/// Configuration change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    pub preset: String,
    pub timestamp: SystemTime,
    pub changes: Vec<String>,
}

impl DynamicConfigManager {
    /// Create a new dynamic configuration manager
    pub fn new(
        config_path: PathBuf,
        module_config_path: PathBuf,
        preset_config_path: PathBuf,
    ) -> Result<Self> {
        let (change_sender, _) = broadcast::channel(100);

        // Load initial configurations
        let mut config = Self::load_config(&config_path)?;
        config.module_config = Self::load_module_config(&module_config_path)?;

        let last_modified = Arc::new(RwLock::new(SystemTime::now()));
        let change_history = Arc::new(RwLock::new(VecDeque::new()));

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            module_config_path,
            preset_config_path,
            change_sender,
            last_modified,
            change_history,
        })
    }

    /// Load configuration from file
    fn load_config(config_path: &PathBuf) -> Result<Config> {
        if !config_path.exists() {
            // Create default config if file doesn't exist
            Ok(Config::new())
        } else {
            // Load existing config
            let config_content = std::fs::read_to_string(config_path)?;
            Ok(serde_json::from_str(&config_content)?)
        }
    }

    /// Load module configuration from file
    fn load_module_config(module_config_path: &PathBuf) -> Result<GlobalModuleConfig> {
        let loader = ConfigLoader::new(module_config_path);
        loader.load_config()
    }

    /// Load preset configuration index
    pub fn load_preset_index(&self) -> Result<PresetIndex> {
        let json_path = self.preset_config_path.join("index.json");

        if json_path.exists() {
            let content = std::fs::read_to_string(&json_path)?;
            let index: PresetIndex = serde_json::from_str(&content)?;
            Ok(index)
        } else {
            Ok(PresetIndex {
                presets: Vec::new(),
            })
        }
    }

    /// Load specific preset configuration
    pub fn load_preset_config(&self, preset_id: &str) -> Result<PresetConfig> {
        let index = self.load_preset_index()?;

        let preset_info = index
            .presets
            .iter()
            .find(|p| p.id == preset_id)
            .ok_or_else(|| anyhow::anyhow!("Preset not found: {}", preset_id))?;

        // Try YAML first, then JSON for backward compatibility
        let preset_path = self.preset_config_path.join(&preset_info.file);
        let json_path = preset_path.with_extension("json");

        if json_path.exists() {
            let content = std::fs::read_to_string(&json_path)?;
            let preset: PresetConfig = serde_json::from_str(&content)?;
            Ok(preset)
        } else {
            Err(anyhow::anyhow!(
                "Preset file not found: {}",
                preset_info.file
            ))
        }
    }

    /// Get available preset configurations
    pub fn get_available_presets(&self) -> Result<Vec<PresetInfo>> {
        let index = self.load_preset_index()?;
        Ok(index.presets)
    }

    /// Get current configuration
    pub fn get_config(&self) -> Config {
        self.config.read().unwrap().clone()
    }

    /// Update configuration
    pub fn update_config(&self, new_config: Config) -> Result<()> {
        let mut config = self.config.write().unwrap();
        *config = new_config;

        // Save to file without module_config.modules to avoid duplicating module settings in config.json
        // Keep in-memory configuration intact, but write a sanitized copy to disk.
        let mut config_to_save = config.clone();
        config_to_save.module_config.modules.clear();
        config_to_save
            .save_to_file(&self.config_path)
            .map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;

        // Update modification time
        *self.last_modified.write().unwrap() = SystemTime::now();

        // Notify configuration change
        self.notify_change(
            "custom".to_string(),
            vec!["Main configuration updated".to_string()],
        )?;

        Ok(())
    }

    /// Update module configuration
    pub fn update_module_config(&self, new_module_config: GlobalModuleConfig) -> Result<()> {
        let mut config = self.config.write().unwrap();
        config.module_config = new_module_config;

        // Save to file
        let loader = ConfigLoader::new(&self.module_config_path);
        loader.save_config(&config.module_config)?;

        // Update modification time
        *self.last_modified.write().unwrap() = SystemTime::now();

        // Notify configuration change
        self.notify_change(
            "custom".to_string(),
            vec!["Module configuration updated".to_string()],
        )?;

        Ok(())
    }

    /// Apply configuration preset
    pub fn apply_preset(&self, preset: String) -> Result<()> {
        info!("Applying configuration preset: {}", preset);

        let mut changes = Vec::new();
        self.apply_preset_from_file(&preset, &mut changes)?;

        // Notify configuration change
        self.notify_change(preset, changes)?;

        Ok(())
    }

    /// Apply configuration preset from file
    fn apply_preset_from_file(&self, preset_id: &str, changes: &mut Vec<String>) -> Result<()> {
        // Create a new PresetLoader instance
        let mut preset_loader = PresetLoader::new(&self.preset_config_path);

        // Load preset index first
        preset_loader.load_preset_index()?;

        // Load the specific preset
        preset_loader.load_preset(preset_id)?;

        // Get the preset configuration
        let preset = preset_loader
            .get_preset(preset_id)
            .with_context(|| format!("Preset not loaded: {}", preset_id))?;

        // Create a completely new module configuration based on the preset
        let mut module_config = GlobalModuleConfig::default();

        // Apply default access level from preset
        if let Some(access_level) = &preset.default_access_level {
            module_config.default_access_level = access_level.clone();
            changes.push(format!("Set default access level to: {:?}", access_level));
        }

        // Apply default rate limit from preset
        if let Some(rate_limit) = &preset.default_rate_limit {
            module_config.default_rate_limit = Some(super::module::RateLimitConfig {
                requests_per_minute: rate_limit.requests_per_minute,
                requests_per_hour: rate_limit.requests_per_hour,
                burst_capacity: rate_limit.burst_capacity,
            });
            changes.push("Updated default rate limit configuration".to_string());
        }

        // Completely replace modules with preset modules
        for (module_name, preset_module) in &preset.modules {
            module_config
                .modules
                .insert(module_name.clone(), preset_module.clone().into());
            changes.push(format!("Added module: {}", module_name));
        }

        // Update the module configuration
        self.update_module_config(module_config)?;
        changes.push(format!(
            "Completely replaced configuration with preset: {}",
            preset_id
        ));

        Ok(())
    }

    /// Notify configuration change
    fn notify_change(&self, preset: String, changes: Vec<String>) -> Result<()> {
        let event = ConfigChangeEvent {
            preset,
            timestamp: SystemTime::now(),
            changes,
        };

        // Add to change history
        if let Ok(mut history) = self.change_history.write() {
            history.push_back(event.clone());
            // Keep only last 100 changes
            if history.len() > 100 {
                history.pop_front();
            }
        }

        if let Err(e) = self.change_sender.send(event) {
            warn!("Failed to send configuration change notification: {}", e);
        }

        Ok(())
    }

    /// Subscribe to configuration changes
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.change_sender.subscribe()
    }

    /// Check if configuration has been modified
    pub fn is_modified(&self) -> bool {
        if let Ok(metadata) = fs::metadata(&self.config_path) {
            if let Ok(modified_time) = metadata.modified() {
                let last_modified = *self.last_modified.read().unwrap();
                return modified_time > last_modified;
            }
        }
        false
    }

    /// Get recent configuration changes
    pub fn get_recent_changes(&self) -> Vec<ConfigChangeEvent> {
        match self.change_history.read() {
            Ok(history) => history.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Reload configuration if modified
    pub fn reload_if_modified(&self) -> Result<bool> {
        if self.is_modified() {
            info!("Configuration file modified, reloading...");

            let new_config = Self::load_config(&self.config_path)?;
            let new_module_config = Self::load_module_config(&self.module_config_path)?;

            {
                let mut config = self.config.write().unwrap();
                *config = new_config;
            }

            let mut config = self.config.write().unwrap();
            config.module_config = new_module_config;

            *self.last_modified.write().unwrap() = SystemTime::now();

            self.notify_change(
                "custom".to_string(),
                vec!["Configuration reloaded from file".to_string()],
            )?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get configuration file paths
    pub fn get_config_paths(&self) -> (PathBuf, PathBuf, PathBuf) {
        (self.config_path.clone(), self.module_config_path.clone(), self.preset_config_path.clone())
    }
}
