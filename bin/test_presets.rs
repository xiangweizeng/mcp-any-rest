use mcp_any_rest::config::loader::ConfigLoader;

fn main() {
    println!("Testing Configuration Presets...");
    
    // Create a config loader
    let loader = ConfigLoader::new("config/modules.json");
    
    // Test getting available presets
    match loader.get_available_presets() {
        Ok(presets) => {
            println!("Available presets: {} preset(s) found", presets.len());
            for preset in presets {
                println!("  - {} ({}): {}", preset.name, preset.id, preset.description);
            }
        }
        Err(e) => {
            println!("Error getting available presets: {}", e);
        }
    }
    
    // Test loading configuration with default preset
    match loader.load_config_with_default_preset() {
        Ok(config) => {
            println!("Successfully loaded configuration with default preset");
            println!("Number of modules: {}", config.modules.len());
        }
        Err(e) => {
            println!("Error loading configuration with default preset: {}", e);
        }
    }
    
    // Test applying specific preset
    match loader.load_config_with_preset(Some("full")) {
        Ok(config) => {
            println!("Successfully loaded configuration with 'full' preset");
            println!("Number of modules: {}", config.modules.len());
        }
        Err(e) => {
            println!("Error loading configuration with 'full' preset: {}", e);
        }
    }
}