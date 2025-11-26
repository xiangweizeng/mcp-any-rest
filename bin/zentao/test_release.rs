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
            name: "Test Release Management Client".to_string(),
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

    // Test 1: Get product releases list with product ID
    tracing::info!("🧪 Test 1: Getting product releases list with product ID...");
    let product_releases_result = client
        .call_tool(CallToolRequestParam {
            name: "release/get_product_releases".into(),
            arguments: serde_json::json!({
                "productId": 1
            }).as_object().cloned(),
        })
        .await;
    match product_releases_result {
        Ok(_result) => {
            tracing::info!("✅ Product releases result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get product releases: {}", e);
        }
    }

    // Test 2: Get project releases list with project ID
    tracing::info!("🧪 Test 2: Getting project releases list with project ID...");
    let project_releases_result = client
        .call_tool(CallToolRequestParam {
            name: "release/get_project_releases".into(),
            arguments: serde_json::json!({
                "projectId": 1
            }).as_object().cloned(),
        })
        .await;
    match project_releases_result {
        Ok(_result) => {
            tracing::info!("✅ Project releases result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get project releases: {}", e);
        }
    }

    client.cancel().await?;
    
    tracing::info!("🎉 Release management tests completed successfully!");
    Ok(())
}