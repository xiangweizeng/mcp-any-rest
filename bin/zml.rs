// ZML unified CLI: list modules and compile to JSON
// All logs and comments in English as required.

use std::io::{self, Read};
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use mcp_any_rest::config::zml_loader::ZmlModuleLoader;
use mcp_any_rest::config::preset_loader::PresetLoader;
use mcp_any_rest::config::module::{AccessLevel as ConfigAccessLevel, MethodConfig, ModuleConfig, RateLimitConfig, ResourceConfig, ResourceType as ConfigResourceType};
use mcp_any_rest::zml::ast::{AccessLevel as AstAccessLevel, RateLimit as AstRateLimit};
use mcp_any_rest::config::preset_loader::PresetConfig;
use mcp_any_rest::zml::{process_zml, process_zml_file};

/// Get executable directory path
fn get_executable_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    
    let exe_dir = current_exe.parent()
        .ok_or("Failed to get executable directory")?;
    
    Ok(exe_dir.to_path_buf())
}

/// Determine configuration directory based on command line arguments
fn determine_config_dir(config_dir: Option<String>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // If command line specifies config directory, use it
    if let Some(config_dir) = config_dir {
        let path = PathBuf::from(config_dir);
        println!("Using command line specified config directory: {:?}", path);
        return Ok(path);
    }
    
    // Otherwise, get executable directory and use config subdirectory
    let exe_dir = get_executable_dir()?;
    let config_dir = exe_dir.join("config");
    
    println!("Using executable relative config directory: {:?}", config_dir);
    Ok(config_dir)
}

/// Determine ZML directory based on configuration directory and command line arguments
fn determine_zml_dir(config_dir: &PathBuf, zml_dir: Option<String>) -> PathBuf {
    // If command line specifies ZML directory, use it
    if let Some(zml_dir) = zml_dir {
        PathBuf::from(zml_dir)
    } else {
        // Otherwise, use zml/ subdirectory under config directory
        config_dir.join("zml")
    }
}

/// Determine presets directory based on configuration directory and command line arguments
fn determine_presets_dir(config_dir: &PathBuf, presets_dir: Option<String>) -> PathBuf {
    // If command line specifies presets directory, use it
    if let Some(presets_dir) = presets_dir {
        PathBuf::from(presets_dir)
    } else {
        // Otherwise, use presets/ subdirectory under config directory
        config_dir.join("presets")
    }
}

#[derive(Parser, Debug)]
#[command(name = "zml", version, about = "ZML CLI: list modules and compile to JSON configuration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List ZML modules from a directory
    List(ListArgs),
    /// Compile ZML source to JSON configuration
    Compile(CompileArgs),
    /// Generate preset JSON from ZML directory
    Preset(PresetArgs),
}

#[derive(Args, Debug)]
struct ListArgs {
    /// Configuration directory path (default: config/)
    #[arg(short = 'c', long = "config-dir", value_name = "DIR")]
    config_dir: Option<String>,
    /// Directory containing .zml files (default: zml/ under config directory)
    #[arg(short = 'd', long = "dir", value_name = "DIR")]
    dir: Option<String>,
}

#[derive(Args, Debug)]
struct CompileArgs {
    /// Path to ZML file; if omitted, read from STDIN
    #[arg(short = 'i', long = "input", value_name = "FILE")]
    input: Option<String>,

    /// Pretty print JSON output
    #[arg(short = 'p', long = "pretty")]
    pretty: bool,
}

#[derive(Args, Debug)]
struct PresetArgs {
    /// Configuration directory path (default: config/)
    #[arg(short = 'c', long = "config-dir", value_name = "DIR")]
    config_dir: Option<String>,
    /// Directory containing .zml files (default: zml/ under config directory)
    #[arg(short = 'd', long = "dir", value_name = "DIR")]
    dir: Option<String>,
    /// Preset ID to save (default: full)
    #[arg(short = 'p', long = "preset", value_name = "ID")]
    preset: Option<String>,
    /// Output directory for presets (default: presets/ under config directory)
    #[arg(short = 'o', long = "out", value_name = "DIR")]
    out: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List(args) => list_modules(args),
        Commands::Compile(args) => compile_zml(args),
        Commands::Preset(args) => generate_preset(args),
    }
}

fn list_modules(args: ListArgs) {
    // Determine configuration directory
    let config_dir = match determine_config_dir(args.config_dir) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Failed to determine config directory: {}", e);
            std::process::exit(1);
        }
    };
    
    // Determine ZML directory
    let zml_dir = determine_zml_dir(&config_dir, args.dir);
    
    println!("=== ZML Module List ===");
    println!("Config directory: {:?}", config_dir);
    println!("ZML directory: {:?}", zml_dir);
    
    match ZmlModuleLoader::from_dir(&zml_dir) {
        Ok(loader) => {
            let names = loader.get_all_module_names();
            println!("Loaded {} ZML module(s)", names.len());
            if names.is_empty() {
                println!("No ZML modules found in {}", zml_dir.display());
            } else {
                println!("Modules: {}", names.join(", "));
            }
        }
        Err(e) => {
            eprintln!("Failed to load ZML modules: {}", e);
            std::process::exit(1);
        }
    }
}

fn map_access_level(level: &AstAccessLevel) -> ConfigAccessLevel {
    match level {
        AstAccessLevel::Public => ConfigAccessLevel::Public,
        AstAccessLevel::Private => ConfigAccessLevel::Private,
        AstAccessLevel::Internal => ConfigAccessLevel::Internal,
    }
}

fn map_rate_limit(rate: &AstRateLimit) -> RateLimitConfig {
    let requests_per_minute = ((rate.requests as u64) * 60 / (rate.per_seconds as u64)).max(1) as u32;
    let requests_per_hour = requests_per_minute.saturating_mul(60);
    let burst_capacity = 10;
    RateLimitConfig { requests_per_minute, requests_per_hour, burst_capacity }
}

fn build_module_config(module: &mcp_any_rest::zml::ast::Module) -> ModuleConfig {
    let mut module_config = ModuleConfig::new();
    module_config.enabled = module.enabled.unwrap_or(true);
    module_config.description = module.description.clone();

    if !module.methods.is_empty() {
        let mut methods_map = std::collections::HashMap::new();
        for (method_name, method_def) in module.methods.iter() {
            let mut method_cfg = MethodConfig::default();
            method_cfg.enabled = true;
            method_cfg.description = method_def.description.clone();
            method_cfg.access_level = Some(map_access_level(&method_def.access_level));
            if let Some(rate) = &method_def.rate_limit {
                method_cfg.rate_limit = Some(map_rate_limit(rate));
            }
            methods_map.insert(method_name.clone(), method_cfg);
        }
        module_config.methods = Some(methods_map);
    }

    if !module.resources.is_empty() {
        let mut resources_map = std::collections::HashMap::new();
        for (_res_name, res_def) in module.resources.iter() {
            let key = if !res_def.uri.is_empty() { res_def.uri.clone() } else { res_def.name.clone() };
            let mut res_cfg = ResourceConfig::default();
            res_cfg.enabled = true;
            res_cfg.description = res_def.description.clone();
            res_cfg.access_level = Some(module
                .access_level
                .as_ref()
                .map(map_access_level)
                .unwrap_or(ConfigAccessLevel::Internal));
            res_cfg.resource_type = Some(ConfigResourceType::ApiEndpoint);
            resources_map.insert(key, res_cfg);
        }
        module_config.resources = Some(resources_map);
    }

    module_config
}

fn generate_preset(args: PresetArgs) {
    // Determine configuration directory
    let config_dir = match determine_config_dir(args.config_dir) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Failed to determine config directory: {}", e);
            std::process::exit(1);
        }
    };
    
    // Determine ZML directory
    let zml_dir = determine_zml_dir(&config_dir, args.dir);
    let preset_id = args.preset.unwrap_or_else(|| "full".to_string());
    let out_dir = determine_presets_dir(&config_dir, args.out);

    println!("=== Generate Preset ===");
    println!("Config directory: {:?}", config_dir);
    println!("ZML directory: {:?}", zml_dir);
    println!("Preset ID: {}", preset_id);
    println!("Output directory: {:?}", out_dir);

    match ZmlModuleLoader::from_dir(&zml_dir) {
        Ok(loader) => {
            let mut modules_cfg = std::collections::HashMap::new();
            for name in loader.get_all_module_names() {
                if let Some(module) = loader.get_module(&name) {
                    let module_cfg = build_module_config(module);
                    modules_cfg.insert(name.clone(), module_cfg);
                }
            }

            let preset = PresetConfig {
                name: "完整功能".to_string(),
                description: "启用所有模块和功能的完整配置".to_string(),
                default_access_level: Some(ConfigAccessLevel::Internal),
                default_rate_limit: Some(RateLimitConfig { requests_per_minute: 60, requests_per_hour: 1000, burst_capacity: 10 }),
                modules: modules_cfg,
            };

            let mut loader = PresetLoader::new(&out_dir);
            match loader.save_preset(&preset_id, &preset) {
                Ok(()) => {
                    println!("Preset '{}' saved to {}", preset_id, out_dir.display());
                }
                Err(e) => {
                    eprintln!("Failed to save preset: {}", e);
                    std::process::exit(2);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load ZML modules from {}: {}", zml_dir.display(), e);
            std::process::exit(1);
        }
    }
}

fn compile_zml(args: CompileArgs) {
    // Compile ZML to JSON
    let result = if let Some(ref input) = args.input {
        process_zml_file(input)
    } else {
        let mut buf = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buf) {
            eprintln!("Error: failed to read from STDIN: {}", e);
            std::process::exit(1);
        }
        if buf.trim().is_empty() {
            eprintln!("Error: ZML source is empty");
            std::process::exit(1);
        }
        process_zml(&buf)
    };

    match result {
        Ok(json) => {
            if args.pretty {
                println!("{}", serde_json::to_string_pretty(&json).unwrap());
            } else {
                println!("{}", serde_json::to_string(&json).unwrap());
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}