//! ZML Language Implementation Library
//! 
//! Provides parsing, compilation, and conversion functionality for the ZML language.

pub mod ast;
pub mod compiler;
pub mod parser;

// Re-export main types
pub use ast::*;
pub use compiler::*;
pub use parser::*;

/// ZML Language Processor
pub struct ZMLProcessor {
    parser: ZMLParserWrapper,
    compiler: Compiler,
}

impl ZMLProcessor {
    pub fn new() -> Self {
        Self {
            parser: ZMLParserWrapper::new(),
            compiler: Compiler::new(),
        }
    }

    /// Process ZML source code and return JSON configuration
    pub fn process(&mut self, source: &str) -> Result<serde_json::Value, ZMLError> {
        // Parse ZML source code
        let module = self.parser.parse(source)?;
        
        // Compile to JSON configuration
        let json_config = self.compiler.compile_module(&module)?;
        
        Ok(json_config)
    }

    /// Load and process ZML from file
    pub fn process_file(&mut self, file_path: &str) -> Result<serde_json::Value, ZMLError> {
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| ZMLError::IoError { source: e })?;
        
        self.process(&source)
    }

    /// Clear all caches
    pub fn clear_cache(&mut self) {
        self.parser.clear_cache();
        self.compiler.clear_cache();
    }
}

impl Default for ZMLProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// ZML Error Type (Unified Error Handling)
#[derive(Debug, thiserror::Error)]
pub enum ZMLError {
    #[error("Parse error: {source}")]
    ParseError { source: ParseError },
    
    #[error("Compile error: {source}")]
    CompileError { source: CompileError },
    
    #[error("IO error: {source}")]
    IoError { source: std::io::Error },
}

impl From<ParseError> for ZMLError {
    fn from(source: ParseError) -> Self {
        ZMLError::ParseError { source }
    }
}

impl From<CompileError> for ZMLError {
    fn from(source: CompileError) -> Self {
        ZMLError::CompileError { source }
    }
}

impl From<std::io::Error> for ZMLError {
    fn from(source: std::io::Error) -> Self {
        ZMLError::IoError { source }
    }
}

/// Convenience function: Process ZML source code directly
pub fn process_zml(source: &str) -> Result<serde_json::Value, ZMLError> {
    let mut processor = ZMLProcessor::new();
    processor.process(source)
}

/// Convenience function: Process ZML from file
pub fn process_zml_file(file_path: &str) -> Result<serde_json::Value, ZMLError> {
    let mut processor = ZMLProcessor::new();
    processor.process_file(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zml_processor() {
        let source = r#"
module TestModule {
    version: "1.0.0"
    description: "Test Module"
    
    type User {
        id: integer
        name: string
        email: string?
    }
}
"#;

        let mut processor = ZMLProcessor::new();
        let result = processor.process(source);
        assert!(result.is_ok(), "ZML processing failed: {:?}", result.err());
        
        let json = result.unwrap();
        assert_eq!(json["name"], "TestModule");
        assert_eq!(json["version"], "1.0.0");
        assert_eq!(json["description"], "Test Module");
    }

    #[test]
    fn test_process_zml_function() {
        let source = r#"
module SimpleModule {
    version: "1.0.0"
}
"#;

        let result = process_zml(source);
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert_eq!(json["name"], "SimpleModule");
        assert_eq!(json["version"], "1.0.0");
    }
}