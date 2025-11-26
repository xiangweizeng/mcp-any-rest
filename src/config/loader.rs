//! Configuration loader for MCP-ANY-REST
//! This module provides functionality to load and parse module configuration files with preset support

use crate::config::module::GlobalModuleConfig;
use crate::config::preset_loader::{PresetLoader};
use crate::config::validator::ConfigValidator;
use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde_json;
use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration loader
pub struct ConfigLoader {
    /// Configuration file path
    config_path: PathBuf,
    /// Preset loader for preset configurations
    preset_loader: PresetLoader,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new(config_path: impl AsRef<Path>) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            preset_loader: PresetLoader::default(),
        }
    }

    /// Create a new configuration loader with custom preset path
    pub fn with_preset_path(config_path: impl AsRef<Path>, preset_path: impl AsRef<Path>) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            preset_loader: PresetLoader::new(preset_path),
        }
    }

    /// Load configuration from file with optional preset application
    pub fn load_config(&self) -> Result<GlobalModuleConfig> {
        self.load_config_with_preset(None)
    }

    /// Load configuration from file and apply a preset
    pub fn load_config_with_preset(&self, preset_id: Option<&str>) -> Result<GlobalModuleConfig> {
        info!("Loading module configuration from: {:?}", self.config_path);

        // Check if configuration file exists
        if !self.config_path.exists() {
            debug!("Configuration file not found, using default configuration");
            
            // If preset is specified, apply it to default configuration
            if let Some(preset_id) = preset_id {
                return self.apply_preset_to_default(preset_id);
            }
            
            return Ok(GlobalModuleConfig::default());
        }

        // Read configuration file
        let config_content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("Failed to read configuration file: {:?}", self.config_path))?;

        // Determine file format based on extension
        let mut config: GlobalModuleConfig = if self.config_path.extension().map_or(false, |ext| ext == "json") {
            // Parse JSON configuration
            match serde_json::from_str(&config_content) {
                Ok(config) => config,
                Err(e) => {
                    error!("JSON parsing error: {}", e);
                    error!("JSON content preview (first 500 chars): {}", &config_content.chars().take(500).collect::<String>());
                    
                    // Try to get more detailed error information
                    match e.classify() {
                        serde_json::error::Category::Syntax => {
                            error!("Syntax error at line {}, column {}", e.line(), e.column());
                        }
                        serde_json::error::Category::Data => {
                            error!("Data error - invalid data structure");
                        }
                        serde_json::error::Category::Eof => {
                            error!("EOF error - unexpected end of file");
                        }
                        serde_json::error::Category::Io => {
                            error!("IO error - file reading issue");
                        }
                    }
                    
                    return Err(anyhow::anyhow!(
                        "Failed to parse JSON configuration file: {}. Error details: {}",
                        self.config_path.display(),
                        e
                    ));
                }
            }
        } else {
            // Parse YAML configuration (default)
            match serde_yaml::from_str(&config_content) {
                Ok(config) => config,
                Err(e) => {
                    error!("YAML parsing error: {}", e);
                    error!("YAML content preview (first 500 chars): {}", &config_content.chars().take(500).collect::<String>());
                    
                    return Err(anyhow::anyhow!(
                        "Failed to parse YAML configuration file: {}. Error details: {}",
                        self.config_path.display(),
                        e
                    ));
                }
            }
        };

        // Apply preset if specified
        if let Some(preset_id) = preset_id {
            self.apply_preset(preset_id, &mut config)?;
        }

        info!("Successfully loaded module configuration");
        debug!("Loaded {} modules", config.modules.len());

        Ok(config)
    }

    /// Apply a preset to an existing configuration
    pub fn apply_preset(&self, preset_id: &str, config: &mut GlobalModuleConfig) -> Result<Vec<String>> {
        info!("Applying preset to configuration: {}", preset_id);
        
        let mut preset_loader = self.preset_loader.clone();
        
        // Load preset index first
        preset_loader.load_preset_index()?;
        
        // Load the specific preset
        preset_loader.load_preset(preset_id)?;
        
        // Validate the preset
        preset_loader.validate_preset(preset_id)?;
        
        // Apply the preset
        let changes = preset_loader.apply_preset(preset_id, config)?;
        
        info!("Successfully applied preset: {} with {} changes", preset_id, changes.len());
        Ok(changes)
    }

    /// Apply preset to default configuration
    fn apply_preset_to_default(&self, preset_id: &str) -> Result<GlobalModuleConfig> {
        info!("Applying preset to default configuration: {}", preset_id);
        
        let mut config = GlobalModuleConfig::default();
        self.apply_preset(preset_id, &mut config)?;
        
        Ok(config)
    }

    /// Load configuration with default preset
    pub fn load_config_with_default_preset(&self) -> Result<GlobalModuleConfig> {
        info!("Loading configuration with default preset");
        
        let mut preset_loader = self.preset_loader.clone();
        
        // Load preset index
        preset_loader.load_preset_index()?;
        
        // Get default preset
        if let Some(default_preset) = preset_loader.get_default_preset()? {
            info!("Using default preset: {} ({})", default_preset.name, default_preset.id);
            let config = self.load_config_with_preset(Some(&default_preset.id))?;
            
            // Validate configuration with detailed report
            let validation_result = self.validate_config_with_report(&config)?;
            
            if !validation_result.is_valid {
                return Err(anyhow::anyhow!(
                    "Configuration validation failed with {} errors",
                    validation_result.errors.len()
                ));
            }
            
            Ok(config)
        } else {
            warn!("No default preset configured, loading configuration without preset");
            let config = self.load_config()?;
            
            // Validate configuration with detailed report
            let validation_result = self.validate_config_with_report(&config)?;
            
            if !validation_result.is_valid {
                return Err(anyhow::anyhow!(
                    "Configuration validation failed with {} errors",
                    validation_result.errors.len()
                ));
            }
            
            Ok(config)
        }
    }

    /// Get available presets
    pub fn get_available_presets(&self) -> Result<Vec<super::preset_loader::PresetInfo>> {
        let mut preset_loader = self.preset_loader.clone();
        preset_loader.load_preset_index()?;
        
        let presets = preset_loader.get_available_presets()?;
        Ok(presets.into_iter().cloned().collect())
    }

    /// Get enabled presets
    pub fn get_enabled_presets(&self) -> Result<Vec<super::preset_loader::PresetInfo>> {
        let mut preset_loader = self.preset_loader.clone();
        preset_loader.load_preset_index()?;
        
        let presets = preset_loader.get_enabled_presets()?;
        Ok(presets.into_iter().cloned().collect())
    }

    /// Get the configuration file path
    pub fn get_config_path(&self) -> &Path {
        &self.config_path
    }

    /// Save configuration to file
    pub fn save_config(&self, config: &GlobalModuleConfig) -> Result<()> {
        info!("Saving module configuration to: {:?}", self.config_path);

        // Create parent directories if they don't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directories: {:?}", parent))?;
        }

        // Determine file format based on extension
        let content = if self.config_path.extension().map_or(false, |ext| ext == "json") {
            // Serialize configuration to JSON
            serde_json::to_string_pretty(config)
                .with_context(|| "Failed to serialize configuration to JSON")?
        } else {
            // Serialize configuration to YAML (default)
            serde_yaml::to_string(config)
                .with_context(|| "Failed to serialize configuration to YAML")?
        };

        // Write to file
        fs::write(&self.config_path, content)
            .with_context(|| format!("Failed to write configuration file: {:?}", self.config_path))?;

        info!("Successfully saved module configuration");
        Ok(())
    }

    /// Create default configuration file
    pub fn create_default_config(&self) -> Result<()> {
        info!("Creating default module configuration file: {:?}", self.config_path);

        let default_config = GlobalModuleConfig::default();
        self.save_config(&default_config)?;

        info!("Default configuration file created successfully");
        Ok(())
    }

    /// Validate configuration
    pub fn validate_config(&self, config: &GlobalModuleConfig) -> Result<()> {
        info!("Validating module configuration");

        // Use the new configuration validator
        let validator = ConfigValidator::new();
        let result = validator.validate_global_module_config(config);

        if !result.is_valid {
            // Collect error messages
            let error_messages: Vec<String> = result.errors
                .iter()
                .map(|error| format!("{} at {}", error.message, error.path))
                .collect();
            
            return Err(anyhow::anyhow!(
                "Configuration validation failed with {} errors:\n{}",
                result.errors.len(),
                error_messages.join("\n")
            ));
        }

        if !result.warnings.is_empty() {
            warn!("Configuration validation completed with {} warnings", result.warnings.len());
            for warning in &result.warnings {
                warn!("Warning: {} at {}", warning.message, warning.path);
            }
        } else {
            info!("Module configuration validation completed successfully");
        }

        Ok(())
    }

    /// Validate configuration with detailed report
    pub fn validate_config_with_report(&self, config: &GlobalModuleConfig) -> Result<crate::config::validator::ValidationResult> {
        info!("Validating module configuration with detailed report");

        let validator = ConfigValidator::new();
        let result = validator.validate_global_module_config(config);

        if !result.is_valid {
            warn!("Configuration validation failed with {} errors", result.errors.len());
        } else if !result.warnings.is_empty() {
            warn!("Configuration validation completed with {} warnings", result.warnings.len());
        } else {
            info!("Module configuration validation completed successfully");
        }

        Ok(result)
    }

    /// Get configuration file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Check if configuration file exists
    pub fn config_exists(&self) -> bool {
        self.config_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::module::ModuleConfig;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_loader_creation() {
        let loader = ConfigLoader::new("config/modules.json");
        assert_eq!(loader.config_path, PathBuf::from("config/modules.json"));
    }

    #[test]
    fn test_default_config_loader() {
        let loader = ConfigLoader::new("config/modules.json");
        assert_eq!(loader.config_path, PathBuf::from("config/modules.json"));
    }

    #[test]
    fn test_validate_valid_config() {
        let loader = ConfigLoader::new("config/modules.json");
        let config = GlobalModuleConfig::default();
        
        let result = loader.validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_config() {
        let loader = ConfigLoader::new("config/modules.json");
        let mut config = GlobalModuleConfig::default();
        
        // Add invalid module with empty name
        config.modules.insert("".to_string(), ModuleConfig::default());
        
        let result = loader.validate_config(&config);
        assert!(result.is_err());
        
        // Check error message contains validation failure
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Configuration validation failed"));
    }

    #[test]
    fn test_validate_config_with_report() {
        let loader = ConfigLoader::new("config/modules.json");
        let config = GlobalModuleConfig::default();
        
        let result = loader.validate_config_with_report(&config);
        assert!(result.is_ok());
        
        let validation_result = result.unwrap();
        assert!(validation_result.is_valid);
        assert_eq!(validation_result.summary.total_modules, 0);
    }

    #[test]
    fn test_load_config_with_default_preset_no_file() {
        let loader = ConfigLoader::new("nonexistent.json");
        
        // Should work even if file doesn't exist (returns default config)
        let result = loader.load_config_with_default_preset();
        
        // The result might be Ok or Err depending on preset loading
        // We just test that the function doesn't panic
        match result {
            Ok(config) => {
                // If successful, config should be valid
                assert!(config.modules.len() > 0);
            }
            Err(e) => {
                // If error, it should be due to preset loading, not file loading
                assert!(e.to_string().contains("preset") || e.to_string().contains("Preset"));
            }
        }
    }

    #[test]
    fn test_config_exists() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.json");
        
        // Create a temporary config file
        fs::write(&config_path, "{}").unwrap();
        
        let loader = ConfigLoader::new(config_path.to_str().unwrap());
        assert!(loader.config_exists());
        
        // Test with non-existent file
        let loader2 = ConfigLoader::new("nonexistent.json");
        assert!(!loader2.config_exists());
    }

    #[test]
    fn test_apply_preset_to_default() {
        let loader = ConfigLoader::new("config/modules.json");
        let config = GlobalModuleConfig::default();
        
        // Test with non-existent preset (should return error)
        let result = loader.apply_preset_to_default("nonexistent");
        assert!(result.is_err());
        
        // Test with empty preset ID (should return error)
        let mut config_clone = config.clone();
        let result = loader.apply_preset("", &mut config_clone);
        assert!(result.is_err());
    }
}