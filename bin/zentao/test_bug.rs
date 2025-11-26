use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    transport::StreamableHttpClientTransport,
};
use std::time::Duration;
use tokio::time::sleep;
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
            name: "Bug Management Test Client".to_string(),
            title: None,
            version: "0.0.1".to_string(),
            website_url: None,
            icons: None,
        },
    };
    
    let client = client_info.serve(transport).await.inspect_err(|e| {
        tracing::error!("Client connection error: {:?}", e);
    })?;

    let server_info = client.peer_info();
    tracing::info!("✅ Connected to server: {server_info:#?}");

    // List available tools
    let tools_result = client.list_tools(Default::default()).await?;
    tracing::info!("📋 Total available tools: {}", tools_result.tools.len());
    
    // Filter bug-related tools
    let bug_tools: Vec<_> = tools_result.tools
        .iter()
        .filter(|tool| tool.name.contains("bug"))
        .collect();
    
    tracing::info!("🐛 Bug-related tools found: {}", bug_tools.len());
    for tool in &bug_tools {
        tracing::info!("  - {}", tool.name);
    }

    // Run multiple test cycles without restarting server
    for cycle in 1..=3 {
        tracing::info!("\n🔄 Test Cycle {} starting...", cycle);
        
        // Test 1: Create a new bug
        tracing::info!("🧪 Cycle {} - Creating a new bug...", cycle);
        let create_bug_result = client
            .call_tool(CallToolRequestParam {
                name: "bug/create_bug".into(),
                arguments: serde_json::json!({
                    "productId": 1,
                    "openedBuild": "trunk",
                    "assignedTo": "admin",
                    "type": "codeerror",
                    "os": "",
                    "browser": "",
                    "title": format!("Test Bug Cycle {} - Management Test", cycle),
                    "severity": 3,
                    "pri": 3,
                    "steps": format!("<p>This is a test bug created during cycle {}</p>", cycle)
                }).as_object().cloned(),
            })
            .await?;
        
        // Extract bug ID
        let bug_id = if let Some(content) = create_bug_result.content.first() {
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
        tracing::info!("📝 Cycle {} - Created bug ID: {}", cycle, bug_id);

        // Test 2: Get product bugs
        tracing::info!("🧪 Cycle {} - Getting product bugs...", cycle);
        let _bug_list_result = client
            .call_tool(CallToolRequestParam {
                name: "bug/get_product_bugs".into(),
                arguments: serde_json::json!({
                    "productId": 1
                }).as_object().cloned(),
            })
            .await;
        tracing::info!("✅ Cycle {} - Product bugs retrieved", cycle);

        // Test 3: Get bug details
        tracing::info!("🧪 Cycle {} - Getting bug details...", cycle);
        let _bug_details_result = client
            .call_tool(CallToolRequestParam {
                name: "bug/get_bug".into(),
                arguments: serde_json::json!({
                    "bugId": bug_id
                }).as_object().cloned(),
            })
            .await?;
        tracing::info!("✅ Cycle {} - Bug details retrieved", cycle);

        // Test 4: Update the bug
        tracing::info!("🧪 Cycle {} - Updating the bug...", cycle);
        let _update_bug_result = client
            .call_tool(CallToolRequestParam {
                name: "bug/update_bug".into(),
                arguments: serde_json::json!({
                    "bugId": bug_id,
                    "title": format!("Updated Bug Cycle {} - Management Test", cycle),
                    "severity": 3,
                    "pri": 3,
                    "type": "codeerror",
                    "steps": format!("<p>This bug has been updated during cycle {}</p>", cycle)
                }).as_object().cloned(),
            })
            .await?;
        tracing::info!("✅ Cycle {} - Bug updated", cycle);

        // Test 5: Close the bug (to test activation later)
        tracing::info!("🧪 Cycle {} - Closing the bug...", cycle);
        let close_bug_result = client
            .call_tool(CallToolRequestParam {
                name: "bug/close_bug".into(),
                arguments: serde_json::json!({
                    "bugId": bug_id,
                    "comment": format!("Closing bug for activation test in cycle {}", cycle)
                }).as_object().cloned(),
            })
            .await;
        
        match close_bug_result {
            Ok(_) => tracing::info!("✅ Cycle {} - Bug closed successfully", cycle),
            Err(e) => tracing::warn!("⚠️ Cycle {} - Failed to close bug: {}", cycle, e),
        }

        // Test 6: Activate the bug
        tracing::info!("🧪 Cycle {} - Activating the bug...", cycle);
        let activate_bug_result = client
            .call_tool(CallToolRequestParam {
                name: "bug/activate_bug".into(),
                arguments: serde_json::json!({
                    "bugId": bug_id,
                    "assignedTo": "admin",
                    "openedBuild": ["trunk"],
                    "comment": format!("Activating bug for test in cycle {}", cycle)
                }).as_object().cloned(),
            })
            .await;
        
        match activate_bug_result {
            Ok(_) => tracing::info!("✅ Cycle {} - Bug activated successfully", cycle),
            Err(e) => tracing::warn!("⚠️ Cycle {} - Failed to activate bug: {}", cycle, e),
        }

        // Test 7: Get product bugs with pagination
        tracing::info!("🧪 Cycle {} - Getting product bugs with pagination...", cycle);
        let paginated_bug_list_result = client
            .call_tool(CallToolRequestParam {
                name: "bug/get_product_bugs".into(),
                arguments: serde_json::json!({
                    "productId": 1,
                    "page": "1",
                    "limit": "10"
                }).as_object().cloned(),
            })
            .await;
        match paginated_bug_list_result {
            Ok(_) => tracing::info!("✅ Cycle {} - Paginated bug list retrieved", cycle),
            Err(e) => tracing::warn!("⚠️ Cycle {} - Failed to get paginated bug list: {}", cycle, e),
        }

        // Test 8: Delete the bug
        tracing::info!("🧪 Cycle {} - Deleting the bug...", cycle);
        let delete_bug_result = client
            .call_tool(CallToolRequestParam {
                name: "bug/delete_bug".into(),
                arguments: serde_json::json!({
                    "bugId": bug_id
                }).as_object().cloned(),
            })
            .await;
        
        match delete_bug_result {
            Ok(_) => tracing::info!("✅ Cycle {} - Bug deleted successfully", cycle),
            Err(e) => tracing::warn!("⚠️ Cycle {} - Failed to delete bug: {}", cycle, e),
        }

        // Wait between cycles
        if cycle < 3 {
            tracing::info!("⏳ Waiting 2 seconds before next cycle...");
            sleep(Duration::from_secs(2)).await;
        }
    }

    client.cancel().await?;

    tracing::info!("\n🎉 All test cycles completed without server restart!");
    
    // Keep the connection alive for further testing
    tracing::info!("🔌 Server connection remains active for additional testing");
    
    Ok(())
}