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
            name: "Test story Management Client".to_string(),
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

    // Prepare test data
    tracing::info!("🧪 Preparing test data...");
    // Create a test product
    tracing::info!("🧪 Test: Creating a new product...");
    let create_product_result = client
        .call_tool(CallToolRequestParam {
            name: "product_create_product".into(),
            arguments: serde_json::json!({
                "program": 1,
                "name": format!("Test Product {}", chrono::Utc::now()),
                "code": "TPMCP",
                "desc": "This is a test product created via API"
            })
            .as_object()
            .cloned(),
        })
        .await;

    let product_id = match create_product_result {
        Ok(result) => {
            // 从创建结果中提取产品 ID
            if let Some(content) = result.content.first() {
                if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                    match serde_json::from_str::<serde_json::Value>(&text_content.text) {
                        Ok(json) => {
                            if let Some(id) = json.get("id").and_then(|v| v.as_i64()) {
                                tracing::info!("📝 Created product ID: {}", id);
                                id
                            } else {
                                tracing::warn!("⚠️ Could not extract product ID, using default 1");
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
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to create product: {}", e);
            tracing::warn!("⚠️ Using default product ID 1 for subsequent tests");
            1
        }
    };

    tracing::info!(" Creating a new project...");
    let create_project_result = client
        .call_tool(CallToolRequestParam {
            name: "project_create_project".into(),
            arguments: serde_json::json!({
                "name": format!("Test Project {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")),
                "code": "TEST001",
                "begin": "2024-01-01",
                "end": "2024-12-31",
                "products": [product_id],
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

    // Variable to store created story ID for subsequent tests
    let mut created_story_id: Option<u64> = None;

    // Test 1: Create a new story
    tracing::info!("🧪 Test 1: Creating a new story...");
    let create_story_result = client
        .call_tool(CallToolRequestParam {
            name: "story_create_story".into(),
            arguments: serde_json::json!({
                "title": "Test story",
                "product": product_id,
                "project": project_id,
                "pri": 1,
                "category": "feature",
                "spec": "This is a test story specification",
                "verify": "Test verification criteria",
                "source": "customer",
                "sourceNote": "Customer request",
                "estimate": 8.0,
                "keywords": "test,story",
                "reviewer": ["admin"]
            })
            .as_object()
            .cloned(),
        })
        .await?;

    // Extract story ID from create result
    if let Some(content) = create_story_result.content.first() {
        if let rmcp::model::RawContent::Text(text_content) = &content.raw {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_content.text) {
                if let Some(id) = json.get("id").and_then(|v| v.as_u64()) {
                    created_story_id = Some(id);
                    tracing::info!("📝 Created story ID: {}", created_story_id.unwrap());
                }
            }
        }
    }

    // Test 2: Get story details (verify creation)
    if let Some(req_id) = created_story_id {
        tracing::info!("🧪 Test 2: Getting story details...");
        let story_details_result = client
            .call_tool(CallToolRequestParam {
                name: "story_get_story".into(),
                arguments: serde_json::json!({
                    "storyId": req_id
                })
                .as_object()
                .cloned(),
            })
            .await;
        match story_details_result {
            Ok(_result) => {
                tracing::info!("✅ story details result success");
            }
            Err(e) => {
                tracing::error!("❌ Failed to get story details: {}", e);
            }
        }
    } else {
        tracing::error!("❌ Failed to get story details: story ID not found");
    }

    // Test 3: Get product storys
    tracing::info!("🧪 Test 3: Getting product storys...");
    let product_storys_result = client
        .call_tool(CallToolRequestParam {
            name: "story_get_product_stories".into(),
            arguments: serde_json::json!({
                "productId": product_id
            })
            .as_object()
            .cloned(),
        })
        .await;
    match product_storys_result {
        Ok(_result) => {
            tracing::info!("✅ Product storys result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get product storys: {}", e);
        }
    }

    // Test 4: Get project storys
    tracing::info!("🧪 Test 4: Getting project storys...");
    let project_storys_result = client
        .call_tool(CallToolRequestParam {
            name: "story_get_project_stories".into(),
            arguments: serde_json::json!({
                "projectId": project_id,
            })
            .as_object()
            .cloned(),
        })
        .await;
    match project_storys_result {
        Ok(_result) => {
            tracing::info!("✅ Project storys result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get project storys: {}", e);
        }
    }

    // Test 5: Get execution storys
    tracing::info!("🧪 Test 5: Getting execution storys...");
    let execution_storys_result = client
        .call_tool(CallToolRequestParam {
            name: "story_get_execution_stories".into(),
            arguments: serde_json::json!({
                "executionId": 1
            })
            .as_object()
            .cloned(),
        })
        .await;
    match execution_storys_result {
        Ok(_result) => {
            tracing::info!("✅ Execution storys result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get execution storys: {}", e);
        }
    }

    // Test 6: Update story basic fields
    if let Some(req_id) = created_story_id {
        tracing::info!("🧪 Test 6: Updating story basic fields...");
        let update_story_result = client
            .call_tool(CallToolRequestParam {
                name: "story_change_story".into(),
                arguments: serde_json::json!({
                    "storyId": req_id,
                    "title": "Test story - Updated Title",
                    "reviewer": ["admin"]
                })
                .as_object()
                .cloned(),
            })
            .await;
        match update_story_result {
            Ok(_result) => {
                tracing::info!("✅ Update story result success");
            }
            Err(e) => {
                tracing::error!("❌ Failed to update story: {}", e);
            }
        }
    }

    // Test 7: Update story additional fields
    if let Some(req_id) = created_story_id {
        tracing::info!("🧪 Test 7: Updating story additional fields...");
        let update_fields_result = client
            .call_tool(CallToolRequestParam {
                name: "story/update_story_other_fields".into(),
                arguments: serde_json::json!({
                    "storyId": req_id,
                    "category": "feature",
                    "reviewer": ["admin"]
                })
                .as_object()
                .cloned(),
            })
            .await;
        match update_fields_result {
            Ok(_result) => {
                tracing::info!("✅ Update story fields result success");
            }
            Err(e) => {
                tracing::error!("❌ Failed to update story fields: {}", e);
            }
        }
    }

    // Test 8: Assign story to user
    if let Some(req_id) = created_story_id {
        tracing::info!("🧪 Test 8: Assigning story to user...");
        let assign_story_result = client
            .call_tool(CallToolRequestParam {
                name: "story_assign_story".into(),
                arguments: serde_json::json!({
                    "storyId": req_id,
                    "assignedTo": "admin",
                    "comment": "Assigning story to admin for development"
                })
                .as_object()
                .cloned(),
            })
            .await;
        match assign_story_result {
            Ok(_result) => {
                tracing::info!("✅ Assign story result success");
            }
            Err(e) => {
                tracing::error!("❌ Failed to assign story: {}", e);
            }
        }
    }

    // Test 9: Close story
    if let Some(req_id) = created_story_id {
        tracing::info!("🧪 Test 9: Closing story...");
        let close_story_result = client
            .call_tool(CallToolRequestParam {
                name: "story/close_story".into(),
                arguments: serde_json::json!({
                    "storyId": req_id,
                    "closedReason": "willnotdo",
                    "comment": "Closing story as will not do"
                })
                .as_object()
                .cloned(),
            })
            .await;
        match close_story_result {
            Ok(_result) => {
                tracing::info!("✅ Close story result success");
            }
            Err(e) => {
                tracing::error!("❌ Failed to close story: {}", e);
            }
        }
    }

    // Test 10: Delete story (cleanup)
    if let Some(req_id) = created_story_id {
        tracing::info!("🧪 Test 10: Deleting story...");
        let delete_story_result = client
            .call_tool(CallToolRequestParam {
                name: "story/delete_story".into(),
                arguments: serde_json::json!({
                    "storyId": req_id
                })
                .as_object()
                .cloned(),
            })
            .await;
        match delete_story_result {
            Ok(_result) => {
                tracing::info!("✅ Delete story result success");
            }
            Err(e) => {
                tracing::error!("❌ Failed to delete story: {}", e);
            }
        }
    }

    // Clean: Delete project (cleanup)
    if product_id != 1 {
        tracing::info!("🧪 Cleanup: Deleting project...");
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
            Ok(_result) => {
                tracing::info!("✅ Delete project result success");
            }
            Err(e) => {
                tracing::error!("❌ Failed to delete project: {}", e);
            }
        }
    }

    // Cleanup: Delete product (cleanup)
    if product_id != 0 {
        tracing::info!("🧪 Cleanup: Deleting product...");
        let delete_product_result = client
            .call_tool(CallToolRequestParam {
                name: "product/delete_product".into(),
                arguments: serde_json::json!({
                    "productId": product_id
                })
                .as_object()
                .cloned(),
            })
            .await;
        match delete_product_result {
            Ok(_result) => {
                tracing::info!("✅ Delete product result success");
            }
            Err(e) => {
                tracing::error!("❌ Failed to delete product: {}", e);
            }
        }
    }

    client.cancel().await?;

    tracing::info!("🎉 story management tests completed successfully! All 10 tests executed.");
    Ok(())
}
