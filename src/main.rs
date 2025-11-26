//! MCP-ANY-REST with stdio transport

use anyhow::Result;
use clap::Parser;
use log::info;
use rmcp::{transport::stdio, ServerHandler, ServiceExt};
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, {self},
};
use mcp_any_rest::{DynamicConfigManager, ServiceComposer, WebServer};

/// Command line arguments for MCP-ANY-REST
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration directory path
    #[arg(short, long)]
    config_dir: Option<PathBuf>,

    /// Transport mode
    #[arg(long, default_value = "stdio")]
    transport: TransportMode,
}

/// Transport mode for the server
#[derive(clap::ValueEnum, Clone, Debug)]
enum TransportMode {
    /// Use stdio transport
    Stdio,
    /// Use HTTP transport
    Http,
}

/// Initialize tracing subscriber for stdio mode
fn init_stdio_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
}

/// Initialize tracing subscriber for HTTP mode
fn init_http_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "debug,rmcp=debug,mcp_any_rest=debug"
                    .to_string()
                    .into()
            }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true) // Show target module
                .with_level(true) // Show log level
                .with_thread_ids(true) // Show thread IDs
                .with_file(true) // Show file and line number
                .with_line_number(true), // Show line number
        )
        .init();
}

/// Create configuration manager with config directory
fn create_config_manager(config_dir: &PathBuf) -> Result<Arc<DynamicConfigManager>> {
    let config_path = config_dir.join("config.json");
    let modules_path = config_dir.join("modules.json");
    let presets_dir = config_dir.join("presets");

    info!("Config directory: {:?}", config_dir);
    info!("Config file: {:?}", config_path);
    info!("Modules file: {:?}", modules_path);
    info!("Presets directory: {:?}", presets_dir);

    let config_manager = Arc::new(DynamicConfigManager::new(
        config_path,
        modules_path,
        presets_dir,
    )?);

    Ok(config_manager)
}

/// Create and initialize service composer
fn create_service_composer(config_manager: &Arc<DynamicConfigManager>) -> Result<ServiceComposer> {
    let service_composer = ServiceComposer::new(config_manager.clone())?;
    info!("Service composer created successfully");

    // Print service information
    let info = service_composer.get_info();
    info!("Server Info: {:?}", info);

    Ok(service_composer)
}

/// Start server in stdio mode with web config server
async fn start_stdio_mode(config_manager: Arc<DynamicConfigManager>) -> Result<()> {
    info!("Starting MCP server in stdio mode with web configuration server...");

    let service_composer = create_service_composer(&config_manager)?;
    let web_server = WebServer::new_dynamic(config_manager.clone());

    // new thread to start web server
    let web_server_handle = tokio::spawn(async move { web_server.start().await });

    let service = service_composer.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;

    // wait for web server to shutdown
    let _ = web_server_handle.await?;

    info!("Server shutdown completed");
    Ok(())
}

/// Start server in HTTP mode
async fn start_http_mode(config_manager: Arc<DynamicConfigManager>) -> Result<()> {
    info!("Starting MCP server in HTTP mode...");

    let service_composer = create_service_composer(&config_manager)?;
    let web_server = WebServer::new_dynamic(config_manager);

    let web_server = web_server.register_service_composer(service_composer);

    match web_server.start().await {
        Ok(_) => {
            info!("HTTP server shutdown completed");
            Ok(())
        }
        Err(e) => {
            tracing::error!("HTTP server error: {:?}", e);
            Err(e)
        }
    }
}

/// Get executable directory path
fn get_executable_dir() -> Result<PathBuf> {
    let current_exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Failed to get current executable path: {}", e))?;
    
    let exe_dir = current_exe.parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get executable directory"))?;
    
    Ok(exe_dir.to_path_buf())
}

/// Determine configuration directory based on command line arguments
fn determine_config_dir(args: &Args) -> Result<PathBuf> {
    // If command line specifies config directory, use it
    if let Some(config_dir) = &args.config_dir {
        info!("Using command line specified config directory: {:?}", config_dir);
        return Ok(config_dir.clone());
    }
    
    // Otherwise, get executable directory and use config subdirectory
    let exe_dir = get_executable_dir()?;
    let config_dir = exe_dir.join("config");
    
    info!("Using executable relative config directory: {:?}", config_dir);
    Ok(config_dir)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Determine configuration directory
    let config_dir = determine_config_dir(&args)?;

    // Handle transport mode selection
    match args.transport {
        TransportMode::Stdio => {
            init_stdio_logging();
            // Create configuration manager
            let config_manager = create_config_manager(&config_dir)?;

            info!("MCP-ANY-REST with stdio transport started successfully");
            info!("Using config directory: {:?}", config_dir);
            start_stdio_mode(config_manager).await
        }
        TransportMode::Http => {
            init_http_logging();
            // Create configuration manager
            let config_manager = create_config_manager(&config_dir)?;

            info!("MCP-ANY-REST with HTTP transport started successfully");
            info!("Using config directory: {:?}", config_dir);
            start_http_mode(config_manager).await
        }
    }
}
