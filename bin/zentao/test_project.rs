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
            name: "Test Project Management Client".to_string(),
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

    // Test 1: Create a new project (先增加)
    tracing::info!("🧪 Test 1: Creating a new project...");
    let create_project_result = client
        .call_tool(CallToolRequestParam {
            name: "project_create_project".into(),
            arguments: serde_json::json!({
                "name": format!("Test Project {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")),
                "code": "TEST001",
                "begin": "2024-01-01",
                "end": "2024-12-31",
                "products": [1],
                "model": "scrum",
                "plans": [1, 2]  // Test plans field
            }).as_object().cloned(),
        })
        .await?;

    // Extract project_id from the creation result
    let project_id = if let Some(content) = create_project_result.content.first() {
        if let rmcp::model::RawContent::Text(text_content) = &content.raw {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_content.text) {
                // Handle both string and number formats for id
                json.get("id")
                    .and_then(|v| {
                        if v.is_string() {
                            v.as_str().and_then(|s| s.parse::<i64>().ok())
                        } else {
                            v.as_i64()
                        }
                    })
                    .unwrap_or(1)
            } else {
                1
            }
        } else {
            1
        }
    } else {
        1
    };
    tracing::info!("📝 Created project ID: {}", project_id);

    // Test 2:  project details
    tracing::info!("🧪 Test 2: Getting project details...");
    let project_details_result = client
        .call_tool(CallToolRequestParam {
            name: "project_get_project".into(),
            arguments: serde_json::json!({
                "projectId": project_id
            })
            .as_object()
            .cloned(),
        })
        .await;
    match project_details_result {
        Ok(_result) => tracing::info!("✅ Get project details success"),
        Err(e) => tracing::error!("❌ Error getting project details: {:?}", e),
    }

    // Test 3: Get project list
    tracing::info!("🧪 Test 3: Getting project list...");
    let project_list_result = client
        .call_tool(CallToolRequestParam {
            name: "project_get_projects".into(),
            arguments: serde_json::json!({
                "page": "1",
                "limit": "1"
            })
            .as_object()
            .cloned(),
        })
        .await;
    match project_list_result {
        Ok(_result) => tracing::info!("✅ Get project list success"),
        Err(e) => tracing::error!("❌ Error getting project list: {:?}", e),
    }

    // Test 4: Update a project
    tracing::info!("🧪 Test 4: Updating a project...");
    let update_project_result = client
        .call_tool(CallToolRequestParam {
            name: "project/update_project".into(),
            arguments: serde_json::json!({
                "projectId": project_id,
                "name": "Updated Test Project",
                "budget": 100000,
                "budgetUnit": "CNY",
                "acl": "private",
                "auth": "extend",
                "plans": [3, 4]  // Test plans field update
            })
            .as_object()
            .cloned(),
        })
        .await;
    match update_project_result {
        Ok(_result) => tracing::info!("✅ Update project success"),
        Err(e) => tracing::error!("❌ Error updating project: {:?}", e),
    }

    // Test 5: Delete a project
    tracing::info!("🧪 Test 5: Deleting a project...");
    let delete_project_result = client
        .call_tool(CallToolRequestParam {
            name: "project/delete_project".into(),
            arguments: serde_json::json!({
                "projectId": project_id
            })
            .as_object()
            .cloned(),
        })
        .await;
    match delete_project_result {
        Ok(_result) => tracing::info!("✅ Delete project success"),
        Err(e) => tracing::error!("❌ Error deleting project: {:?}", e),
    }

    client.cancel().await?;

    tracing::info!("🎉 Project management tests completed successfully!");
    Ok(())
}
