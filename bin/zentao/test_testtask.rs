use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    transport::StreamableHttpClientTransport,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
            name: "Test TestTask Management Client".to_string(),
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

    // Test 1: Get test task list with pagination
    tracing::info!("🧪 Test 1: Getting test task list with pagination...");
    let testtask_list_result = client
        .call_tool(CallToolRequestParam {
            name: "testtask_get_testtask_list".into(),
            arguments: serde_json::json!({}).as_object().cloned(),
        })
        .await;
        match testtask_list_result {
            Ok(_result) => {
                tracing::info!("✅ Test task list retrieved successfully");
            }
            Err(e) => {
                tracing::error!("❌ Failed to get test task list: {}", e);
            }
        }

    // Test 2: Get test task details
    tracing::info!("🧪 Test 2: Getting test task details...");
    let testtask_details_result = client
        .call_tool(CallToolRequestParam {
            name: "testtask/get_testtask".into(),
            arguments: serde_json::json!({
                "testtaskId": 0
            }).as_object().cloned(),
        })
        .await;
    match testtask_details_result {
        Ok(_result) => {
            tracing::info!("✅ Test task details retrieved successfully");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get test task details: {}", e);
        }
    }

    // Test 3: Get project test tasks
    tracing::info!("🧪 Test 3: Getting project test tasks...");
    let project_testtasks_result = client
        .call_tool(CallToolRequestParam {
            name: "testtask/get_project_testtasks".into(),
            arguments: serde_json::json!({
                "projectId": 1
            }).as_object().cloned(),
        })
        .await;
    match project_testtasks_result {
        Ok(_result) => {
            tracing::info!("✅ Project test tasks retrieved successfully");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get project test tasks: {}", e);
        }
    }

    // Test 4: Get test task list with product filter
    tracing::info!("🧪 Test 4: Getting test task list with product filter...");
    let filtered_testtask_list_result = client
        .call_tool(CallToolRequestParam {
            name: "testtask/get_testtask_list".into(),
            arguments: serde_json::json!({
                "page": 1,
                "limit": 5,
                "product": 1,
                "branch": 0
            }).as_object().cloned(),
        })
        .await;
    match filtered_testtask_list_result {
        Ok(_result) => {
            tracing::info!("✅ Filtered test task list retrieved successfully");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get filtered test task list: {}", e);
        }
    }

    client.cancel().await?;
    
    tracing::info!("🎉 Test task management tests completed successfully!");
    Ok(())
}