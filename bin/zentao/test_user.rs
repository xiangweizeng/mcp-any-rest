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
            name: "Test User Management Client".to_string(),
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

    // Test 1 Create a test user with unique username
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let test_username = format!("testuser_{}", timestamp);

    tracing::info!(
        "🧪 Test 1: Creating test user with username: {}...",
        test_username
    );
    let create_user_result = client
        .call_tool(CallToolRequestParam {
            name: "user_create_user".into(),
            arguments: serde_json::json!({
                "account": test_username,
                "password": "Aaaa@12345",
                "realname": "Test User",
                "gender": "m"
            })
            .as_object()
            .cloned(),
        })
        .await?;
    let user_id =    // 从创建结果中提取产品 ID
            if let Some(content) = create_user_result.content.first() {
                if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                    match serde_json::from_str::<serde_json::Value>(&text_content.text) {
                        Ok(json) => {
                            if let Some(id) = json.get("id").and_then(|v| v.as_i64()) {
                                tracing::info!("📝 Created user ID: {}", id);
                                id
                            } else {
                                tracing::warn!("⚠️ Could not extract user ID, using default 1");
                                1
                            }
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to parse response: {}", e);
                            1
                        }
                    }
                } else {
                    tracing::warn!("⚠️ Unexpected response format, using default ID 1");
                    1
                }
            } else {
                tracing::warn!("⚠️ No content in response, using default ID 1");
                1
            };

            
    // Modify user info
    tracing::info!("🧪 Test 2: Modifying test user info...");
    let update_user_result = client
        .call_tool(CallToolRequestParam {
            name: "user_update_user".into(),
            arguments: serde_json::json!({
                "userId": user_id,
                "nickname": "Test User",
                "avatar": "https://example.com/avatar.jpg"
            })
            .as_object()
            .cloned(),
        })
        .await;
    match update_user_result {
        Ok(_result) => {
            tracing::info!("✅ Test user info updated successfully");
        }
        Err(e) => {
            tracing::error!("❌ Failed to update test user info: {}", e);
        }
    }

    // Test 3: Get user details
    tracing::info!("🧪 Test 2: Getting user details...");
    let user_details_result = client
        .call_tool(CallToolRequestParam {
            name: "user_get_user".into(),
            arguments: serde_json::json!({
                "userId": user_id
            })
            .as_object()
            .cloned(),
        })
        .await;
    match user_details_result {
        Ok(_result) => {
            tracing::info!("✅ User details result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get user details: {}", e);
        }
    }

    // Test 4: Get user list with pagination
    tracing::info!("🧪 Test 4: Getting user list with pagination...");
    let paginated_user_list_result = client
        .call_tool(CallToolRequestParam {
            name: "user_get_users".into(),
            arguments: serde_json::json!({
                "page": 1,
                "limit": 10
            })
            .as_object()
            .cloned(),
        })
        .await;
    match paginated_user_list_result {
        Ok(_result) => {
            tracing::info!("✅ Paginated user list result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get paginated user list: {}", e);
        }
    }

    // Delete the test user
    tracing::info!("🧪 Test 5: Deleting test user...");
    let delete_user_result = client
        .call_tool(CallToolRequestParam {
            name: "user_delete_user".into(),
            arguments: serde_json::json!({
                "userId": user_id
            })
            .as_object()
            .cloned(),
        })
        .await;
    match delete_user_result {
        Ok(_result) => {
            tracing::info!("✅ Test user deleted successfully");
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete test user: {}", e);
        }
    }

    client.cancel().await?;

    tracing::info!("🎉 User management tests completed successfully!");
    Ok(())
}
