//! Test cases for product plan management module

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
            name: "Test Plan Management Client".to_string(),
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
            name: "product/create_product".into(),
            arguments: serde_json::json!({
                "program": 1,
                "name": format!("Test Product {}", chrono::Utc::now()),
                "code": "test_product",
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

    // Create 2 requirements
    tracing::info!("🧪 Test: Creating 2 requirements...");
    let mut requirement_ids = Vec::new();

    for i in 1..=2 {
        let create_requirement_result = client
            .call_tool(CallToolRequestParam {
                name: "story/create_story".into(),
                arguments: serde_json::json!({
                    "product": product_id,
                    "title": format!("Test Requirement {}", i),
                    "pri": 1,
                    "category": "feature",
                    "spec": "This is a test requirement specification",
                    "verify": "Test verification criteria",
                    "source": "customer",
                    "sourceNote": "Customer request",
                    "estimate": 8.0,
                    "keywords": "test,requirement"
                })
                .as_object()
                .cloned(),
            })
            .await;

        let requirement_id = match create_requirement_result {
            Ok(result) => {
                // Get requirement ID from response
                if let Some(content) = result.content.first() {
                    if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                        match serde_json::from_str::<serde_json::Value>(&text_content.text) {
                            Ok(json) => {
                                if let Some(id) = json.get("id").and_then(|v| v.as_i64()) {
                                    tracing::info!("📝 Created requirement {} ID: {}", i, id);
                                    id
                                } else {
                                    tracing::warn!(
                                        "⚠️ Could not extract requirement {} ID, using default 1",
                                        i
                                    );
                                    1
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "❌ Failed to parse response for requirement {}: {}",
                                    i,
                                    e
                                );
                                1
                            }
                        }
                    } else {
                        tracing::warn!(
                            "⚠️ Unexpected response format for requirement {}, using default ID 1",
                            i
                        );
                        1
                    }
                } else {
                    tracing::warn!(
                        "⚠️ No content in response for requirement {}, using default ID 1",
                        i
                    );
                    1
                }
            }
            Err(e) => {
                tracing::error!("❌ Failed to create requirement {}: {}", i, e);
                tracing::warn!(
                    "⚠️ Using default requirement {} ID 1 for subsequent tests",
                    i
                );
                1
            }
        };

        requirement_ids.push(requirement_id);
    }

    // Test 1: Create a new product plan (先增加)
    tracing::info!("🧪 Test 1: Creating a new product plan...");
    let create_plan_result = client
        .call_tool(CallToolRequestParam {
            name: "plan/create_plan".into(),
            arguments: serde_json::json!({
                "productId": product_id,
                "title": "Test Plan",
                "desc": "This is a test plan created via API",
                "begin": "2024-01-01",
                "end": "2024-12-31"
            })
            .as_object()
            .cloned(),
        })
        .await;

    let plan_id = match create_plan_result {
        Ok(result) => {
            // Extract plan ID from the creation result
            if let Some(content) = result.content.first() {
                if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                    match serde_json::from_str::<serde_json::Value>(&text_content.text) {
                        Ok(json) => {
                            if let Some(id) = json.get("id").and_then(|v| v.as_i64()) {
                                tracing::info!("📝 Created plan ID: {}", id);
                                id
                            } else {
                                tracing::warn!("⚠️ Could not extract plan ID, using default 1");
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
            tracing::error!("❌ Failed to create plan: {}", e);
            tracing::warn!("⚠️ Using default plan ID 1 for subsequent tests");
            1
        }
    };

    // Test 2: Get product plan list (后查询)
    tracing::info!("🧪 Test 2: Getting product plan list...");
    let plan_list_result = client
        .call_tool(CallToolRequestParam {
            name: "plan/get_plans".into(),
            arguments: serde_json::json!({
                "productId": product_id,
                "page": 1,
                "limit": 10
            })
            .as_object()
            .cloned(),
        })
        .await;

    match plan_list_result {
        Ok(result) => {
            // Add detailed JSON parsing like in comprehensive test
            if let Some(content) = result.content.first() {
                if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                    match serde_json::from_str::<serde_json::Value>(&text_content.text) {
                        Ok(json) => {
                            tracing::info!("📋 Get plans successful: page={}, total={}, limit={}, plans count={}", 
                                json.get("page").and_then(|v| v.as_i64()).unwrap_or(0),
                                json.get("total").and_then(|v| v.as_i64()).unwrap_or(0),
                                json.get("limit").and_then(|v| v.as_i64()).unwrap_or(0),
                                json.get("plans").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0));
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to parse response: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to get plan list: {}", e);
        }
    }

    // Test 3: Get product plan details (后查询)
    tracing::info!("🧪 Test 3: Getting product plan details...");
    let plan_details_result = client
        .call_tool(CallToolRequestParam {
            name: "plan/get_plan".into(),
            arguments: serde_json::json!({
                "planId": plan_id
            })
            .as_object()
            .cloned(),
        })
        .await;

    match plan_details_result {
        Ok(_result) => {
            tracing::info!("✅ Product plan details result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get plan details: {}", e);
            // Check if it's a deserialization error
            if e.to_string()
                .contains("invalid type: string \"0\", expected i32")
            {
                tracing::info!("🔍 This is the branch field deserialization error!");
            }
        }
    }

    // Test 4: Update a product plan (再修改)
    tracing::info!("🧪 Test 4: Updating a product plan...");
    let update_plan_result = client
        .call_tool(CallToolRequestParam {
            name: "plan/update_plan".into(),
            arguments: serde_json::json!({
                "planId": plan_id,
                "title": "Updated Test Plan",
                "desc": "This plan has been updated via API"
            })
            .as_object()
            .cloned(),
        })
        .await;

    match update_plan_result {
        Ok(_result) => {
            tracing::info!("✅ Update product plan result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to update plan: {}", e);
        }
    }

    // Test 5: Link stories to product plan (关联操作)
    tracing::info!("🧪 Test 5: Linking stories to product plan...");
    let link_stories_result = client
        .call_tool(CallToolRequestParam {
            name: "plan/link_stories".into(),
            arguments: serde_json::json!({
                "planId": plan_id,
                "stories": requirement_ids
            })
            .as_object()
            .cloned(),
        })
        .await;

    match link_stories_result {
        Ok(_result) => {
            tracing::info!("✅ Link stories result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to link stories: {}", e);
        }
    }

    // Test 6: Unlink stories from product plan (关联操作)
    tracing::info!("🧪 Test 6: Unlinking stories from product plan...");
    let unlink_stories_result = client
        .call_tool(CallToolRequestParam {
            name: "plan/unlink_stories".into(),
            arguments: serde_json::json!({
                "planId": plan_id,
                "stories": requirement_ids
            })
            .as_object()
            .cloned(),
        })
        .await;

    match unlink_stories_result {
        Ok(_result) => {
            tracing::info!("✅ Unlink stories result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to unlink stories: {}", e);
        }
    }

    // Test 7: Link bugs to product plan (关联操作)
    tracing::info!("🧪 Test 7: Linking bugs to product plan...");
    let link_bugs_result = client
        .call_tool(CallToolRequestParam {
            name: "plan/link_bugs".into(),
            arguments: serde_json::json!({
                "planId": plan_id,
                "bugs": [1, 2, 3]
            })
            .as_object()
            .cloned(),
        })
        .await;

    match link_bugs_result {
        Ok(_result) => {
            tracing::info!("✅ Link bugs result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to link bugs: {}", e);
        }
    }

    // Test 8: Unlink bugs from product plan (关联操作)
    tracing::info!("🧪 Test 8: Unlinking bugs from product plan...");
    let unlink_bugs_result = client
        .call_tool(CallToolRequestParam {
            name: "plan/unlink_bugs".into(),
            arguments: serde_json::json!({
                "planId": plan_id,
                "bugs": [1, 2]
            })
            .as_object()
            .cloned(),
        })
        .await;

    match unlink_bugs_result {
        Ok(_result) => {
            tracing::info!("✅ Unlink bugs result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to unlink bugs: {}", e);
        }
    }

    // Test 9: Delete product plan (最后删除)
    tracing::info!("🧪 Test 9: Deleting product plan...");
    let delete_plan_result = client
        .call_tool(CallToolRequestParam {
            name: "plan/delete_plan".into(),
            arguments: serde_json::json!({
                "planId": plan_id
            })
            .as_object()
            .cloned(),
        })
        .await;

    match delete_plan_result {
        Ok(_result) => {
            tracing::info!("✅ Delete product plan result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete plan: {}", e);
        }
    }

    // Clean up: Delete the created stories
    tracing::info!("🧪 Test 11: Cleaning up created stories...");
    for requirement_id in requirement_ids {
        let delete_requirement_result = client
            .call_tool(CallToolRequestParam {
                name: "story/delete_story".into(),
                arguments: serde_json::json!({
                    "storyId": requirement_id
                })
                .as_object()
                .cloned(),
            })
            .await;

        match delete_requirement_result {
            Ok(_result) => {
                tracing::info!("✅ Delete story result success");
            }
            Err(e) => {
                tracing::error!("❌ Failed to delete story: {}", e);
            }
        }
    }

    // Clean up: Delete the created product
    tracing::info!("🧪 Test 10: Cleaning up created product...");
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

    client.cancel().await?;
    tracing::info!("🎉 Product plan management tests completed successfully!");

    Ok(())
}
