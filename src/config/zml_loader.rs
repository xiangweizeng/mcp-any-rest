//! ZML module loader and configuration helpers
//! Loads `.zml` files into ZML AST `Module` objects for service generation.
//! Also provides a lightweight config loader that produces `ModuleConfig` entries
//! to merge into `GlobalModuleConfig`.

use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::zml::ast::Module;
use crate::zml::parser::ZMLParserWrapper;
use crate::config::module::{GlobalModuleConfig, ModuleConfig};

/// Loader that parses ZML modules from a directory and caches them by name.
#[derive(Debug, Clone)]
pub struct ZmlModuleLoader {
    modules: HashMap<String, Module>,
}

impl Default for ZmlModuleLoader {
    fn default() -> Self {
        Self { modules: HashMap::new() }
    }
}

impl ZmlModuleLoader {
    /// Load all `.zml` modules from directory
    pub fn from_dir(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        info!("Loading ZML modules from {}", dir.display());

        let mut modules: HashMap<String, Module> = HashMap::new();
        let mut parser = ZMLParserWrapper::new();

        if !dir.exists() {
            warn!("ZML directory does not exist: {}", dir.display());
            return Ok(Self { modules });
        }

        for entry in fs::read_dir(dir).context("Failed to read ZML directory")? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "zml").unwrap_or(false) {
                let source = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read ZML file: {}", path.display()))?;
                match parser.parse(&source) {
                    Ok(module) => {
                        debug!("Parsed ZML module: {}", module.name);
                        modules.insert(module.name.clone(), module);
                    }
                    Err(e) => {
                        warn!("Failed to parse ZML file {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(Self { modules })
    }

    /// Get module by name
    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    /// Check if module exists
    pub fn has_module(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    /// Return all module names
    pub fn get_all_module_names(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    /// Return enabled module names according to GlobalModuleConfig
    pub fn get_enabled_modules(&self, global: &GlobalModuleConfig) -> Vec<String> {
        self.modules
            .keys()
            .filter(|name| global.is_module_enabled(name))
            .cloned()
            .collect()
    }
}

/// Output structure for ZML config loader to merge into `GlobalModuleConfig`
#[derive(Debug, Clone)]
pub struct ZmlConfigOutput {
    pub modules: HashMap<String, ModuleConfig>,
}

/// Lightweight configuration loader: reads ZML files and produces module-level configs.
#[derive(Debug, Default, Clone)]
pub struct ZmlConfigLoader {}

impl ZmlConfigLoader {
    pub fn new() -> Self { Self {} }

    /// Load ZML modules and produce module visibility configs.
    /// - Enabled defaults to `true` if not specified in ZML.
    pub fn load_from_dir(&mut self, dir: &Path) -> Result<ZmlConfigOutput> {
        let loader = ZmlModuleLoader::from_dir(dir)?;
        let mut modules_cfg: HashMap<String, ModuleConfig> = HashMap::new();
        for (name, module) in loader.modules.iter() {
            let mut cfg = ModuleConfig::new();
            cfg.enabled = module.enabled.unwrap_or(true);
            cfg.description = module.description.clone();
            modules_cfg.insert(name.clone(), cfg);
        }
        Ok(ZmlConfigOutput { modules: modules_cfg })
    }
}