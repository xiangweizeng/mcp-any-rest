//! Preset loader for MCP-ANY-REST
//! This module provides enhanced preset loading functionality with better error handling and validation

use super::module::{GlobalModuleConfig, ModuleConfig, AccessLevel, RateLimitConfig};
use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::clone::Clone;

/// Preset configuration definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    /// Preset name
    pub name: String,
    /// Preset description
    pub description: String,
    /// Default access level (reuse AccessLevel enum from module)
    pub default_access_level: Option<AccessLevel>,
    /// Default rate limit configuration
    pub default_rate_limit: Option<RateLimitConfig>,
    /// Module configurations - directly reuse existing module configs
    pub modules: HashMap<String, ModuleConfig>,
}

/// Preset information for index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetInfo {
    /// Unique identifier for the preset
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Preset file name (without extension)
    pub file: String,
    /// Whether this preset is enabled
    pub enabled: bool,
    /// Preset priority (lower number = higher priority)
    pub priority: u32,
}

/// Preset index containing all available presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetIndex {
    /// List of available presets
    pub presets: Vec<PresetInfo>,
    /// Default preset ID
    pub default_preset: Option<String>,
}

// Note: RateLimitConfig, ModuleConfig, MethodConfig, and ResourceConfig are now directly reused from the module module

/// Preset loader for loading and managing configuration presets
#[derive(Clone)]
pub struct PresetLoader {
    /// Preset configuration directory path
    preset_path: PathBuf,
    /// Loaded preset index
    preset_index: Option<PresetIndex>,
    /// Loaded preset configurations
    loaded_presets: HashMap<String, PresetConfig>,
}

impl PresetLoader {
    /// Create a new preset loader
    pub fn new(preset_path: impl AsRef<Path>) -> Self {
        Self {
            preset_path: preset_path.as_ref().to_path_buf(),
            preset_index: None,
            loaded_presets: HashMap::new(),
        }
    }

    /// Load preset index from file
    pub fn load_preset_index(&mut self) -> Result<&PresetIndex> {
        info!("Loading preset index from: {:?}", self.preset_path);

        // Try to load index from YAML first
        let yaml_path = self.preset_path.join("index.yaml");
        if yaml_path.exists() {
            let content = fs::read_to_string(&yaml_path)
                .with_context(|| format!("Failed to read preset index YAML: {:?}", yaml_path))?;
            
            let index: PresetIndex = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse preset index YAML: {:?}", yaml_path))?;
            
            self.preset_index = Some(index);
            info!("Successfully loaded preset index from YAML");
            return Ok(self.preset_index.as_ref().unwrap());
        }

        // Fall back to JSON
        let json_path = self.preset_path.join("index.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path)
                .with_context(|| format!("Failed to read preset index JSON: {:?}", json_path))?;
            
            let index: PresetIndex = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse preset index JSON: {:?}", json_path))?;
            
            self.preset_index = Some(index);
            info!("Successfully loaded preset index from JSON");
            return Ok(self.preset_index.as_ref().unwrap());
        }

        // Create default index if no file exists
        warn!("No preset index file found, creating default index");
        let default_index = Self::create_default_index();
        self.preset_index = Some(default_index);
        
        Ok(self.preset_index.as_ref().unwrap())
    }

    /// Load a specific preset configuration
    pub fn load_preset(&mut self, preset_id: &str) -> Result<()> {
        // Check if preset is already loaded
        if self.loaded_presets.contains_key(preset_id) {
            return Ok(());
        }

        // Ensure index is loaded
        if self.preset_index.is_none() {
            self.load_preset_index()?;
        }

        let preset_info = {
            let index = self.preset_index.as_ref().unwrap();
            let preset_info = index.presets
                .iter()
                .find(|p| p.id == preset_id)
                .with_context(|| format!("Preset not found: {}", preset_id))?;
            preset_info.clone()
        };

        if !preset_info.enabled {
            return Err(anyhow::anyhow!("Preset is disabled: {}", preset_id));
        }

        info!("Loading preset configuration: {} ({})", preset_info.name, preset_id);

        // Try to load from YAML first
        let yaml_path = self.preset_path.join(format!("{}.yaml", preset_info.file));
        if yaml_path.exists() {
            let content = fs::read_to_string(&yaml_path)
                .with_context(|| format!("Failed to read preset YAML: {:?}", yaml_path))?;
            
            let preset: PresetConfig = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse preset YAML: {:?}", yaml_path))?;
            
            self.loaded_presets.insert(preset_id.to_string(), preset);
            info!("Successfully loaded preset from YAML: {}", preset_id);
            return Ok(());
        }

        // Fall back to JSON
        let json_path = self.preset_path.join(format!("{}.json", preset_info.file));
        if json_path.exists() {
            let content = fs::read_to_string(&json_path)
                .with_context(|| format!("Failed to read preset JSON: {:?}", json_path))?;
            
            let preset: PresetConfig = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse preset JSON: {:?}", json_path))?;
            
            self.loaded_presets.insert(preset_id.to_string(), preset);
            info!("Successfully loaded preset from JSON: {}", preset_id);
            return Ok(());
        }

        Err(anyhow::anyhow!(
            "Preset file not found for {}: {}.yaml or {}.json",
            preset_id,
            preset_info.file,
            preset_info.file
        ))
    }

    /// Get a loaded preset configuration
    pub fn get_preset(&self, preset_id: &str) -> Option<&PresetConfig> {
        self.loaded_presets.get(preset_id)
    }

    /// Apply a preset to the global module configuration
    pub fn apply_preset(
        &mut self,
        preset_id: &str,
        global_config: &mut GlobalModuleConfig,
    ) -> Result<Vec<String>> {
        // Load preset if not already loaded
        if !self.loaded_presets.contains_key(preset_id) {
            self.load_preset(preset_id)?;
        }
        
        let preset = self.get_preset(preset_id)
            .with_context(|| format!("Preset not loaded: {}", preset_id))?;

        let mut changes = Vec::new();

        // Apply default access level (now directly using AccessLevel enum)
        if let Some(access_level) = &preset.default_access_level {
            global_config.default_access_level = access_level.clone();
            changes.push(format!("Set default access level to: {:?}", access_level));
        }

        // Apply default rate limit (now directly using RateLimitConfig)
        if let Some(rate_limit) = &preset.default_rate_limit {
            global_config.default_rate_limit = Some(rate_limit.clone());
            changes.push("Updated default rate limit configuration".to_string());
        }

        // Apply module configurations (now directly using ModuleConfig)
        for (module_name, preset_module) in &preset.modules {
            if let Some(existing_module) = global_config.modules.get_mut(module_name) {
                // Update existing module - directly merge the configurations
                existing_module.enabled = preset_module.enabled;
                
                // Update description if provided
                if let Some(description) = &preset_module.description {
                    existing_module.description = Some(description.clone());
                }
                
                // Update methods
                if let Some(preset_methods) = &preset_module.methods {
                    if existing_module.methods.is_none() {
                        existing_module.methods = Some(HashMap::new());
                    }
                    
                    if let Some(existing_methods) = existing_module.methods.as_mut() {
                        for (method_name, preset_method) in preset_methods {
                            if let Some(existing_method) = existing_methods.get_mut(method_name) {
                                // Update existing method
                                existing_method.enabled = preset_method.enabled;
                                if let Some(description) = &preset_method.description {
                                    existing_method.description = Some(description.clone());
                                }
                                if let Some(access_level) = &preset_method.access_level {
                                    existing_method.access_level = Some(access_level.clone());
                                }
                                if let Some(rate_limit) = &preset_method.rate_limit {
                                    existing_method.rate_limit = Some(rate_limit.clone());
                                }
                            } else {
                                // Create new method configuration
                                existing_methods.insert(method_name.clone(), preset_method.clone());
                            }
                        }
                    }
                }
                
                // Update resources
                if let Some(preset_resources) = &preset_module.resources {
                    if existing_module.resources.is_none() {
                        existing_module.resources = Some(HashMap::new());
                    }
                    
                    if let Some(existing_resources) = existing_module.resources.as_mut() {
                        for (resource_name, preset_resource) in preset_resources {
                            if let Some(existing_resource) = existing_resources.get_mut(resource_name) {
                                // Update existing resource
                                existing_resource.enabled = preset_resource.enabled;
                                if let Some(description) = &preset_resource.description {
                                    existing_resource.description = Some(description.clone());
                                }
                                if let Some(access_level) = &preset_resource.access_level {
                                    existing_resource.access_level = Some(access_level.clone());
                                }
                                if let Some(resource_type) = &preset_resource.resource_type {
                                    existing_resource.resource_type = Some(resource_type.clone());
                                }
                            } else {
                                // Create new resource configuration
                                existing_resources.insert(resource_name.clone(), preset_resource.clone());
                            }
                        }
                    }
                }
                
                changes.push(format!("Updated module: {}", module_name));
            } else {
                // Create new module configuration
                global_config.modules.insert(module_name.clone(), preset_module.clone());
                changes.push(format!("Created module: {}", module_name));
            }
        }

        changes.push(format!("Applied preset: {}", preset.name));
        info!("Successfully applied preset: {} with {} changes", preset_id, changes.len());
        
        Ok(changes)
    }

    /// Get all available presets
    pub fn get_available_presets(&self) -> Result<Vec<&PresetInfo>> {
        if let Some(index) = &self.preset_index {
            Ok(index.presets.iter().collect())
        } else {
            Err(anyhow::anyhow!("Preset index not loaded"))
        }
    }

    /// Get enabled presets only
    pub fn get_enabled_presets(&self) -> Result<Vec<&PresetInfo>> {
        if let Some(index) = &self.preset_index {
            Ok(index.presets.iter().filter(|p| p.enabled).collect())
        } else {
            Err(anyhow::anyhow!("Preset index not loaded"))
        }
    }

    /// Get default preset
    pub fn get_default_preset(&self) -> Result<Option<&PresetInfo>> {
        if let Some(index) = &self.preset_index {
            if let Some(default_id) = &index.default_preset {
                Ok(index.presets.iter().find(|p| p.id == *default_id))
            } else {
                Ok(None)
            }
        } else {
            Err(anyhow::anyhow!("Preset index not loaded"))
        }
    }

    /// Validate preset configuration
    pub fn validate_preset(&self, preset_id: &str) -> Result<()> {
        let preset = self.get_preset(preset_id)
            .with_context(|| format!("Preset not loaded: {}", preset_id))?;

        // Validate module configurations
        for (module_name, module_config) in &preset.modules {
            if module_name.trim().is_empty() {
                return Err(anyhow::anyhow!("Module name cannot be empty in preset"));
            }

            // Validate methods
            if let Some(methods) = &module_config.methods {
                for (method_name, method_config) in methods {
                    if method_name.trim().is_empty() {
                        return Err(anyhow::anyhow!(
                            "Method name cannot be empty in module {} in preset",
                            module_name
                        ));
                    }

                    // Validate rate limit configuration
                    if let Some(rate_limit) = &method_config.rate_limit {
                        if rate_limit.requests_per_minute == 0 {
                            return Err(anyhow::anyhow!(
                                "Invalid rate limit for method {} in module {}: requests_per_minute cannot be 0",
                                method_name, module_name
                            ));
                        }
                    }
                }
            }
        }

        debug!("Preset validation passed: {}", preset_id);
        Ok(())
    }

    /// Save a preset configuration to file
    pub fn save_preset(&mut self, preset_id: &str, preset_config: &PresetConfig) -> Result<()> {
        // Ensure preset directory exists
        if !self.preset_path.exists() {
            fs::create_dir_all(&self.preset_path)
                .with_context(|| format!("Failed to create preset directory: {:?}", self.preset_path))?;
        }

        // Load or create preset index
        if self.preset_index.is_none() {
            self.load_preset_index()?;
        }

        let index = self.preset_index.as_mut().unwrap();
        
        // Check if preset exists in index
        let preset_exists = index.presets.iter().any(|p| p.id == preset_id);
        
        if !preset_exists {
            // Add new preset to index
            let preset_info = PresetInfo {
                id: preset_id.to_string(),
                name: preset_config.name.clone(),
                description: preset_config.description.clone(),
                file: preset_id.to_string(),
                enabled: true,
                priority: (index.presets.len() as u32) + 1,
            };
            index.presets.push(preset_info);
        }

        // Save preset configuration as JSON
        let json_path = self.preset_path.join(format!("{}.json", preset_id));
        let json_content = serde_json::to_string_pretty(preset_config)
            .with_context(|| format!("Failed to serialize preset to JSON: {}", preset_id))?;
        
        fs::write(&json_path, json_content)
            .with_context(|| format!("Failed to write preset JSON file: {:?}", json_path))?;

        // Save preset index
        self.save_preset_index()?;

        // Update loaded presets
        self.loaded_presets.insert(preset_id.to_string(), preset_config.clone());

        info!("Successfully saved preset: {}", preset_id);
        Ok(())
    }

    /// Save preset index to file
    pub fn save_preset_index(&self) -> Result<()> {
        if let Some(index) = &self.preset_index {

            let json_path = self.preset_path.join("index.json");
            let json_content = serde_json::to_string_pretty(index)
                .with_context(|| "Failed to serialize preset index to JSON")?;
            
            fs::write(&json_path, json_content)
                .with_context(|| format!("Failed to write preset index JSON file: {:?}", json_path))?;

            info!("Successfully saved preset index");
        }
        
        Ok(())
    }

    /// Delete a preset
    pub fn delete_preset(&mut self, preset_id: &str) -> Result<()> {
        // Load preset index
        if self.preset_index.is_none() {
            self.load_preset_index()?;
        }

        let index = self.preset_index.as_mut().unwrap();
        
        // Remove preset from index
        index.presets.retain(|p| p.id != preset_id);
        let json_path = self.preset_path.join(format!("{}.json", preset_id));
        if json_path.exists() {
            fs::remove_file(&json_path)
                .with_context(|| format!("Failed to delete preset JSON file: {:?}", json_path))?;
        }

        // Remove from loaded presets
        self.loaded_presets.remove(preset_id);

        // Save updated index
        self.save_preset_index()?;

        info!("Successfully deleted preset: {}", preset_id);
        Ok(())
    }

    /// Create default preset index
    fn create_default_index() -> PresetIndex {
        PresetIndex {
            presets: vec![
                PresetInfo {
                    id: "full".to_string(),
                    name: "Full Configuration".to_string(),
                    description: "Enable all modules and features with complete configuration".to_string(),
                    file: "full".to_string(),
                    enabled: true,
                    priority: 1,
                },
                PresetInfo {
                    id: "minimal".to_string(),
                    name: "Minimal Configuration".to_string(),
                    description: "Enable only essential modules with basic configuration".to_string(),
                    file: "minimal".to_string(),
                    enabled: true,
                    priority: 2,
                },
            ],
            default_preset: Some("full".to_string()),
        }
    }
}

impl Default for PresetLoader {
    fn default() -> Self {
        Self::new("config/presets")
    }
}

// Note: Conversion implementations are no longer needed as we directly reuse module configurations

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_loader_creation() {
        let loader = PresetLoader::new("config/presets");
        assert_eq!(loader.preset_path, PathBuf::from("config/presets"));
    }

    #[test]
    fn test_default_preset_loader() {
        let loader = PresetLoader::default();
        assert_eq!(loader.preset_path, PathBuf::from("config/presets"));
    }
}