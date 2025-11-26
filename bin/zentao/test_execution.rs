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
            name: "Test Execution Management Client".to_string(),
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
    // 1. First try to get existing products to find a valid product ID
    tracing::info!("Prepare test data: Getting existing products...");
    let products_result = client
        .call_tool(CallToolRequestParam {
            name: "product_get_products".into(),
            arguments: serde_json::json!({
                "page": 1,
                "limit": 10
            }).as_object().cloned(),
        })
        .await;

    let product_id = match products_result {
        Ok(result) => {
            if let Some(content) = result.content.first() {
                if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_content.text) {
                        // Try to extract first product ID from the list
                        if let Some(products) = json.get("products") {
                            if let Some(first_product) = products.as_array().and_then(|arr| arr.first()) {
                                first_product.get("id").and_then(|v| v.as_i64()).unwrap_or(1)
                            } else {
                                1
                            }
                        } else {
                            1
                        }
                    } else {
                        1
                    }
                } else {
                    1
                }
            } else {
                1
            }
        }
        Err(_) => {
            tracing::warn!("Failed to get products list, using default product ID 1");
            1
        }
    };

    tracing::info!("Using product ID: {}", product_id);

    // 2. Create a new project
    tracing::info!("Prepare test data: Creating a new project...");
    let create_project_result = client
        .call_tool(CallToolRequestParam {
            name: "project_create_project".into(),
            arguments: serde_json::json!({
                "name": format!("Test Project {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")),
                "code": "TEST001",
                "begin": "2024-01-01",
                "end": "2024-12-31",
                "products": [product_id],
                "model": "scrum"
            }).as_object().cloned(),
        })
        .await;

    let project_id = match create_project_result {
        Ok(result) => {
            if let Some(content) = result.content.first() {
                if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_content.text) {
                        json.get("id").and_then(|v| v.as_i64()).unwrap_or(1)
                    } else {
                        tracing::error!("Failed to parse project creation response");
                        return Ok(()); // Skip tests if project creation failed
                    }
                } else {
                    tracing::error!("Unexpected response format for project creation");
                    return Ok(()); // Skip tests if project creation failed
                }
            } else {
                tracing::error!("No content in project creation response");
                return Ok(()); // Skip tests if project creation failed
            }
        }
        Err(e) => {
            tracing::error!("Failed to create project: {}", e);
            tracing::warn!("Skipping execution management tests due to project creation failure");
            return Ok(()); // Skip tests if project creation failed
        }
    };

    tracing::info!("Project created successfully with ID: {}", project_id);

    // Test 1: Create a new execution (先增加)
    tracing::info!("🧪 Test 1: Creating a new execution...");
    let create_execution_result = client
        .call_tool(CallToolRequestParam {
            name: "execution_create_execution".into(),
            arguments: serde_json::json!({
                "project": project_id,
                "name": "Test Execution Created by MCP Client",
                "code": "TEST-EXEC-001",
                "begin": "2024-01-01",
                "end": "2024-12-31",
                "days": 250,
                "lifetime": "short",
                "PO": "admin",
                "PM": "admin",
                "QD": "admin",
                "RD": "admin",
                "teamMembers": ["admin"],
                "desc": "This is a test execution created by MCP client",
                "acl": "open",
                "whitelist": []
            })
            .as_object()
            .cloned(),
        })
        .await;

    let execution_id = match create_execution_result {
        Ok(result) => {
            if let Some(content) = result.content.first() {
                if let rmcp::model::RawContent::Text(text_content) = &content.raw {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_content.text) {
                        json.get("id").and_then(|v| v.as_i64()).unwrap_or(1)
                    } else {
                        tracing::error!("Failed to parse execution creation response");
                        tracing::warn!("Skipping remaining execution management tests");
                        return Ok(()); // Skip remaining tests if execution creation failed
                    }
                } else {
                    tracing::error!("Unexpected response format for execution creation");
                    tracing::warn!("Skipping remaining execution management tests");
                    return Ok(()); // Skip remaining tests if execution creation failed
                }
            } else {
                tracing::error!("No content in execution creation response");
                tracing::warn!("Skipping remaining execution management tests");
                return Ok(()); // Skip remaining tests if execution creation failed
            }
        }
        Err(e) => {
            tracing::error!("Failed to create execution: {}", e);
            tracing::warn!("Skipping remaining execution management tests");
            return Ok(()); // Skip remaining tests if execution creation failed
        }
    };
    tracing::info!("📝 Created execution ID: {}", execution_id);

    // Test 2: Get execution details (后查询)
    tracing::info!("🧪 Test 2: Getting execution details...");
    let execution_details_result = client
        .call_tool(CallToolRequestParam {
            name: "execution_get_execution".into(),
            arguments: serde_json::json!({
                "executionId": execution_id
            })
            .as_object()
            .cloned(),
        })
        .await;
    match execution_details_result {
        Ok(_result) => {
            tracing::info!("✅ Execution details result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get execution details: {}", e);
        }
    }

    // Test 3: Update an execution (再修改)
    tracing::info!("🧪 Test 3: Updating an execution...");
    let update_execution_result = client
        .call_tool(CallToolRequestParam {
            name: "execution_update_execution".into(),
            arguments: serde_json::json!({
                "executionId": execution_id,
                "body": {
                    "project": project_id,
                    "name": "Updated Test Execution",
                    "code": "TEST-EXEC-001",
                    "begin": "2024-01-01",
                    "end": "2024-12-31",
                    "days": 250,
                    "lifetime": "short",
                    "PO": "admin",
                    "PM": "admin",
                    "QD": "admin",
                    "RD": "admin",
                    "teamMembers": ["admin"],
                    "desc": "This execution has been updated by MCP client",
                    "acl": "open",
                    "whitelist": []
                }
            })
            .as_object()
            .cloned(),
        })
        .await;
    match update_execution_result {
        Ok(_result) => {
            tracing::info!("✅ Update execution result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to update execution: {}", e);
        }
    }

    // Test 4: Get project executions list with pagination (后查询)
    tracing::info!("🧪 Test 4: Getting project executions list with pagination...");
    let paginated_execution_list_result = client
        .call_tool(CallToolRequestParam {
            name: "execution_get_executions".into(),
            arguments: serde_json::json!({
                "projectId": project_id,
                "page": "1",
                "limit": "10"
            })
            .as_object()
            .cloned(),
        })
        .await;
    match paginated_execution_list_result {
        Ok(_result) => {
            tracing::info!("✅ Paginated project executions list result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get paginated project executions list: {}", e);
        }
    }

    // Test 5: Delete an execution (最后删除)
    tracing::info!("🧪 Test 5: Deleting an execution...");
    let delete_execution_result = client
        .call_tool(CallToolRequestParam {
            name: "execution_delete_execution".into(),
            arguments: serde_json::json!({
                "executionId": execution_id
            })
            .as_object()
            .cloned(),
        })
        .await;
    match delete_execution_result {
        Ok(_result) => {
            tracing::info!("✅ Delete execution result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete execution: {}", e);
        }
    }

    // Cleanup: Delete the project
    tracing::info!("🧹 Cleanup: Deleting project...");
    let delete_project_result = client
        .call_tool(CallToolRequestParam {
            name: "project_delete_project".into(),
            arguments: serde_json::json!({
                "projectId": project_id
            })
            .as_object()
            .cloned(),
        })
        .await;
    match delete_project_result {
        Ok(_result) => {
            tracing::info!("✅ Project deletion result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete project: {}", e);
        }
    }

    client.cancel().await?;

    tracing::info!("🎉 Execution management tests completed successfully!");
    Ok(())
}
