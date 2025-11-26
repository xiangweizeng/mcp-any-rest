//! Configuration validation utilities

use log::info;
use serde::{Deserialize, Serialize};

use crate::config::module::{AccessLevel, GlobalModuleConfig, MethodConfig, ModuleConfig, RateLimitConfig};

/// Validation result containing detailed information about validation issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the configuration is valid
    pub is_valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Summary of validation results
    pub summary: ValidationSummary,
}

/// Validation error with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Error message
    pub message: String,
    /// Path to the problematic configuration element
    pub path: String,
    /// Additional context information
    pub context: Option<String>,
}

/// Validation warning with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,
    /// Path to the problematic configuration element
    pub path: String,
    /// Additional context information
    pub context: Option<String>,
}

/// Validation summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total number of modules validated
    pub total_modules: usize,
    /// Total number of methods validated
    pub total_methods: usize,
    /// Total number of resources validated
    pub total_resources: usize,
    /// Number of validation errors
    pub error_count: usize,
    /// Number of validation warnings
    pub warning_count: usize,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Critical error that prevents configuration from being used
    Critical,
    /// High severity error that may cause runtime issues
    High,
    /// Medium severity error that should be addressed
    Medium,
    /// Low severity error that can be safely ignored
    Low,
}

/// Configuration validator
#[derive(Debug, Clone)]
pub struct ConfigValidator {
    /// Whether to include warnings in validation results
    include_warnings: bool,
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self {
            include_warnings: true,
        }
    }
}

impl ConfigValidator {
    /// Create a new configuration validator
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a validator with strict validation enabled
    pub fn strict() -> Self {
        Self {
            include_warnings: false,
        }
    }

    /// Validate GlobalModuleConfig
    pub fn validate_global_module_config(&self, config: &GlobalModuleConfig) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            summary: ValidationSummary {
                total_modules: 0,
                total_methods: 0,
                total_resources: 0,
                error_count: 0,
                warning_count: 0,
            },
        };

        info!("Validating GlobalModuleConfig");

        // Validate each module
        for (module_name, module_config) in &config.modules {
            result.summary.total_modules += 1;
            
            self.validate_module_config(&mut result, module_name, module_config);
        }

        // Update summary counts
        result.summary.error_count = result.errors.len();
        result.summary.warning_count = result.warnings.len();
        result.is_valid = result.errors.is_empty();

        result
    }

    /// Validate module configuration
    fn validate_module_config(
        &self,
        result: &mut ValidationResult,
        module_name: &str,
        module_config: &ModuleConfig,
    ) {
        let module_path = format!("modules.{}", module_name);

        // Validate module name
        if module_name.trim().is_empty() {
            self.add_error(
                result,
                ErrorSeverity::Critical,
                "Module name cannot be empty",
                &module_path,
                None,
            );
        }

        // Validate module enabled state
        if !module_config.enabled {
            if self.include_warnings {
                self.add_warning(
                    result,
                    &format!("Module '{}' is disabled", module_name),
                    &module_path,
                    None,
                );
            }
        }

        // Validate methods
        if let Some(methods) = &module_config.methods {
            for (method_name, method_config) in methods {
                result.summary.total_methods += 1;
                
                let method_path = format!("{}.methods.{}", module_path, method_name);
                self.validate_method_config(result, &method_path, method_name, method_config);
            }
        }

        // Validate resources
        if let Some(resources) = &module_config.resources {
            for (resource_name, resource_config) in resources {
                result.summary.total_resources += 1;
                
                let resource_path = format!("{}.resources.{}", module_path, resource_name);
                self.validate_resource_config(result, &resource_path, resource_name, resource_config);
            }
        }
    }

    
    /// Validate method configuration
    fn validate_method_config(
        &self,
        result: &mut ValidationResult,
        method_path: &str,
        method_name: &str,
        method_config: &MethodConfig,
    ) {
        // Validate method name
        if method_name.trim().is_empty() {
            self.add_error(
                result,
                ErrorSeverity::Critical,
                "Method name cannot be empty",
                method_path,
                None,
            );
        }

        // Validate method enabled state
        if !method_config.enabled {
            if self.include_warnings {
                self.add_warning(
                    result,
                    &format!("Method '{}' is disabled", method_name),
                    method_path,
                    None,
                );
            }
        }

        // Validate rate limit configuration
        if let Some(rate_limit) = &method_config.rate_limit {
            self.validate_rate_limit_config(result, method_path, rate_limit);
        }

        // Validate access level
        if let Some(access_level) = &method_config.access_level {
            match access_level {
                AccessLevel::Public => {},
                AccessLevel::Internal => {},
                AccessLevel::Private => {},
            }
        }
    }

       /// Validate rate limit configuration
    fn validate_rate_limit_config(
        &self,
        result: &mut ValidationResult,
        method_path: &str,
        rate_limit: &RateLimitConfig,
    ) {
        let rate_limit_path = format!("{}.rate_limit", method_path);

        if rate_limit.requests_per_minute == 0 {
            self.add_error(
                result,
                ErrorSeverity::High,
                "Rate limit requests_per_minute cannot be 0",
                &rate_limit_path,
                None,
            );
        }

        if rate_limit.requests_per_hour == 0 {
            self.add_error(
                result,
                ErrorSeverity::High,
                "Rate limit requests_per_hour cannot be 0",
                &rate_limit_path,
                None,
            );
        }

        if rate_limit.burst_capacity == 0 {
            self.add_error(
                result,
                ErrorSeverity::High,
                "Rate limit burst_capacity cannot be 0",
                &rate_limit_path,
                None,
            );
        }

        if rate_limit.requests_per_minute > rate_limit.requests_per_hour {
            self.add_error(
                result,
                ErrorSeverity::High,
                &format!(
                    "Rate limit requests_per_minute ({}) cannot be greater than requests_per_hour ({})",
                    rate_limit.requests_per_minute, rate_limit.requests_per_hour
                ),
                &rate_limit_path,
                None,
            );
        }
    }

      /// Validate resource configuration
    fn validate_resource_config(
        &self,
        result: &mut ValidationResult,
        resource_path: &str,
        resource_name: &str,
        resource_config: &crate::config::module::ResourceConfig,
    ) {
        // Validate resource name
        if resource_name.trim().is_empty() {
            self.add_error(
                result,
                ErrorSeverity::Critical,
                "Resource name cannot be empty",
                resource_path,
                None,
            );
        }

        // Validate resource enabled state
        if !resource_config.enabled {
            if self.include_warnings {
                self.add_warning(
                    result,
                    &format!("Resource '{}' is disabled", resource_name),
                    resource_path,
                    None,
                );
            }
        }
    }

    /// Add an error to the validation result
    fn add_error(
        &self,
        result: &mut ValidationResult,
        severity: ErrorSeverity,
        message: &str,
        path: &str,
        context: Option<&str>,
    ) {
        result.errors.push(ValidationError {
            severity,
            message: message.to_string(),
            path: path.to_string(),
            context: context.map(|s| s.to_string()),
        });
    }

    /// Add a warning to the validation result
    fn add_warning(
        &self,
        result: &mut ValidationResult,
        message: &str,
        path: &str,
        context: Option<&str>,
    ) {
        if self.include_warnings {
            result.warnings.push(ValidationWarning {
                message: message.to_string(),
                path: path.to_string(),
                context: context.map(|s| s.to_string()),
            });
        }
    }
}

/// Utility functions for configuration validation
impl ConfigValidator {

    /// Get a detailed validation report as JSON
    pub fn get_validation_report(&self, result: &ValidationResult) -> serde_json::Value {
        serde_json::json!({
            "is_valid": result.is_valid,
            "summary": {
                "total_modules": result.summary.total_modules,
                "total_methods": result.summary.total_methods,
                "total_resources": result.summary.total_resources,
                "error_count": result.summary.error_count,
                "warning_count": result.summary.warning_count
            },
            "errors": result.errors,
            "warnings": result.warnings
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_global_module_config() {
        let validator = ConfigValidator::new();
        let config = GlobalModuleConfig::default();
        let result = validator.validate_global_module_config(&config);
        
        assert!(result.is_valid);
        assert_eq!(result.summary.total_modules, 0);
        assert_eq!(result.summary.error_count, 0);
    }

    #[test]
    fn test_validate_invalid_module_name() {
        let validator = ConfigValidator::new();
        let mut config = GlobalModuleConfig::default();
        config.modules.insert("".to_string(), ModuleConfig::default());
        
        let result = validator.validate_global_module_config(&config);
        
        assert!(!result.is_valid);
        assert_eq!(result.summary.error_count, 1);
        assert_eq!(result.errors[0].message, "Module name cannot be empty");
    }

    #[test]
    fn test_validation_report() {
        let validator = ConfigValidator::new();
        let config = GlobalModuleConfig::default();
        let result = validator.validate_global_module_config(&config);
        let report = validator.get_validation_report(&result);
        
        assert!(report["is_valid"].as_bool().unwrap());
        assert_eq!(report["summary"]["total_modules"].as_u64().unwrap(), 0);
    }
}