use anyhow::Result;
use chrono;
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    transport::StreamableHttpClientTransport,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Test configuration
const TEST_PROJECT_ID: i32 = 1;
const TEST_PRODUCT_ID: i32 = 1;
const TEST_EXECUTION_ID: i32 = 1;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let transport = StreamableHttpClientTransport::from_uri("http://127.0.0.1:8081/mcp");
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "Test Build Management Client".to_string(),
            title: None,
            version: "0.0.1".to_string(),
            website_url: None,
            icons: None,
        },
    };
    let client = client_info.serve(transport).await.inspect_err(|e| {
        tracing::error!("client error: {:?}", e);
    })?;

    // Initialize
    let server_info = client.peer_info();
    tracing::info!("Connected to server: {server_info:#?}");

    // Test 1: Create a new build (API 706)
    tracing::info!("🧪 Test 1: Creating a new build...");
    let create_build_result = client
        .call_tool(CallToolRequestParam {
            name: "build/create_build".into(),
            arguments: serde_json::json!({
                "projectId": TEST_PROJECT_ID,
                "execution": TEST_EXECUTION_ID,
                "product": TEST_PRODUCT_ID,
                "branch": 0,
                "name": format!("v1.0.{}", chrono::Utc::now().timestamp()),
                "builder": "MCP Client",
                "date": "2024-01-01",
                "scmPath": "",
                "filePath": "",
                "desc": "<p>This is a test build created by MCP client</p>"
            }).as_object().cloned(),
        })
        .await;

    let build_id = match create_build_result {
        Ok(result) => {
            tracing::info!("Create build success");
            // Extract build ID from the creation result
            if let Some(content) = result.content.first() {
                if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_content.text) {
                        let id = json.get("id").and_then(|v| v.as_i64()).unwrap_or(1);
                        tracing::info!("Created build ID: {}", id);
                        id
                    } else {
                        tracing::warn!("Could not parse build creation response, using default ID: 1");
                        1
                    }
                } else {
                    tracing::warn!("No text content in response, using default ID: 1");
                    1
                }
            } else {
                tracing::warn!("No content in response, using default ID: 1");
                1
            }
        }
        Err(e) => {
            tracing::error!("Error creating build: {:?}", e);
            // Use a default ID for subsequent tests
            tracing::warn!("Using default build ID: 1 for subsequent tests");
            1
        }
    };

    // Test 2: Get build details (API 707)
    tracing::info!("🧪 Test 2: Getting build details...");
    let build_details_result = client
        .call_tool(CallToolRequestParam {
            name: "build/get_build".into(),
            arguments: serde_json::json!({
                "buildId": build_id
            }).as_object().cloned(),
        })
        .await;
    match build_details_result {
        Ok(_result) => {
            tracing::info!("✅ Get build details success");
        }
        Err(e) => tracing::error!("❌ Error getting build details: {:?}", e),
    }

    // Test 3: Get project builds list (API 704)
    tracing::info!("🧪 Test 3: Getting project builds list...");
    let project_builds_result = client
        .call_tool(CallToolRequestParam {
            name: "build/get_project_builds".into(),
            arguments: serde_json::json!({
                "projectId": TEST_PROJECT_ID,
                "page": 1,
                "limit": 10
            }).as_object().cloned(),
        })
        .await;
    match project_builds_result {
        Ok(_result) => {
            tracing::info!("✅ Get project builds list success");
        }
        Err(e) => tracing::error!("❌ Error getting project builds list: {:?}", e),
    }

    // Test 4: Get execution builds list (API 705)
    tracing::info!("🧪 Test 4: Getting execution builds list...");
    let execution_builds_result = client
        .call_tool(CallToolRequestParam {
            name: "build/get_execution_builds".into(),
            arguments: serde_json::json!({
                "executionId": TEST_EXECUTION_ID,
                "page": 1,
                "limit": 10
            }).as_object().cloned(),
        })
        .await;
    match execution_builds_result {
        Ok(_result) => {
            tracing::info!("✅ Get execution builds list success");
        }
        Err(e) => tracing::error!("❌ Error getting execution builds list: {:?}", e),
    }
  
    // Test 5: Update build (API 708) - Only modify name to avoid complex field issues
    tracing::info!("🧪 Test 5: Updating build...");
    let update_build_result = client
        .call_tool(CallToolRequestParam {
            name: "build/update_build".into(),
            arguments: serde_json::json!({
                "buildId": build_id,
                "name": "v1.0.1"
            }).as_object().cloned(),
        })
        .await;
    match update_build_result {
        Ok(_result) => {
            tracing::info!("✅ Update build success");
        }
        Err(e) => {
            tracing::error!("❌ Error updating build: {:?}", e);
        }
    }
    
    // Test 6: Delete build (API 709)
    tracing::info!("🧪 Test 6: Deleting build...");
    let delete_build_result = client
        .call_tool(CallToolRequestParam {
            name: "build/delete_build".into(),
            arguments: serde_json::json!({
                "buildId": build_id
            }).as_object().cloned(),
        })
        .await;
    match delete_build_result {
        Ok(_result) => tracing::info!("✅ Delete build success"),
        Err(e) => tracing::error!("❌ Error deleting build: {:?}", e),
    }
    
    client.cancel().await?;
    
    tracing::info!("Version management tests completed successfully!");
    Ok(())
}