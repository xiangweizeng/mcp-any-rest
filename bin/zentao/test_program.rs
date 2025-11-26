//! Test program management functionality

use anyhow::Result;
use rmcp::{
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    transport::StreamableHttpClientTransport,
    ServiceExt,
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
            name: "Test Program Management Client".to_string(),
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

    // Test 1: Create a new program
    tracing::info!("🧪 Test 1: Creating a new program...");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let program_name = format!("Test Program MCP {}", timestamp);
    let create_program_result = client
        .call_tool(CallToolRequestParam {
            name: "program_create_program".into(),
            arguments: serde_json::json!({
                "name": program_name,
                "desc": "This is a test program created by MCP client for testing purposes",
                "budget": 1000,
                "budgetUnit": "USD",
                "begin": "2024-01-01",
                "end": "2024-12-31"
            })
            .as_object()
            .cloned(),
        })
        .await?;

    // Extract program ID from the creation result
    let program_id = if let Some(content) = create_program_result.content.first() {
        if let rmcp::model::RawContent::Text(text_content) = &content.raw {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_content.text) {
                json.get("id").and_then(|v| v.as_i64()).unwrap_or(1)
            } else {
                1
            }
        } else {
            1
        }
    } else {
        1
    };
    tracing::info!("📝 Created program ID: {}", program_id);

    // Test 2: Get program list (后查询)
    tracing::info!("🧪 Test 2: Getting program list...");
    let program_list_result = client
        .call_tool(CallToolRequestParam {
            name: "program_get_program_list".into(),
            arguments: serde_json::json!({
                "order": "id_desc"
            })
            .as_object()
            .cloned(),
        })
        .await;
    match program_list_result {
        Ok(_result) => {
            tracing::info!("✅ Program list result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get program list: {}", e);
        }
    }

    // Test 3: Get program details (后查询)
    tracing::info!("🧪 Test 3: Getting program details...");
    let program_details_result = client
        .call_tool(CallToolRequestParam {
            name: "program_get_program_details".into(),
            arguments: serde_json::json!({
                "programId": program_id
            })
            .as_object()
            .cloned(),
        })
        .await;
    match program_details_result {
        Ok(_result) => {
            tracing::info!("✅ Program details result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get program details: {}", e);
        }
    }

    // Test 4: Update a program (再修改)
    tracing::info!("🧪 Test 4: Updating a program...");
    let update_program_result = client
        .call_tool(CallToolRequestParam {
            name: "program_update_program".into(),
            arguments: serde_json::json!({
                "programId": program_id,
                "name": format!("Updated Program MCP {}", timestamp),
                "desc": "This program has been updated by MCP client",
                "budget": 150000
            })
            .as_object()
            .cloned(),
        })
        .await;
    match update_program_result {
        Ok(_result) => {
            tracing::info!("✅ Update program result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to update program: {}", e);
        }
    }

    // Test 5: Get program list with ordering (后查询)
    tracing::info!("🧪 Test 5: Getting program list with ordering...");
    let ordered_program_list_result = client
        .call_tool(CallToolRequestParam {
            name: "program_get_program_list".into(),
            arguments: serde_json::json!({
                "order": "name_asc"
            })
            .as_object()
            .cloned(),
        })
        .await;
    match ordered_program_list_result {
        Ok(_result) => {
            tracing::info!("✅ Ordered program list result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get ordered program list: {}", e);
        }
    }

    // Test 6: Delete a program (最后删除)
    tracing::info!("🧪 Test 6: Deleting a program...");
    let delete_program_result = client
        .call_tool(CallToolRequestParam {
            name: "program_delete_program".into(),
            arguments: serde_json::json!({
                "programId": program_id
            })
            .as_object()
            .cloned(),
        })
        .await;
    match delete_program_result {
        Ok(_result) => {
            tracing::info!("✅ Delete program result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete program: {}", e);
        }
    }

    client.cancel().await?;

    tracing::info!("🎉 Program management tests completed successfully!");
    Ok(())
}
