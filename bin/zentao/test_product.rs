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
            name: "Test Product Management Client".to_string(),
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

    // Test 1: Create a new product (先增加)
    tracing::info!("🧪 Test 1: Creating a new product...");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let product_name = format!("Test Product MCP {}", timestamp);
    let product_code = format!("TPMCP{}", timestamp);

    let create_product_result = client
        .call_tool(CallToolRequestParam {
            name: "product_create_product".into(),
            arguments: serde_json::json!({
                "name": product_name,
                "code": product_code,
                "program": null,
                "line": null,
                "po": null,
                "qd": null,
                "rd": null,
                "type": "normal",
                "status": "normal",
                "acl": "open",
                "whitelist": null,
                "desc": "This is a test product created by MCP client"
            })
            .as_object()
            .cloned(),
        })
        .await?;

    // Extract product ID from the creation result
    let product_id = if let Some(content) = create_product_result.content.first() {
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
    tracing::info!("📝 Created product ID: {}", product_id);

    // Test 2: Get product list (后查询)
    tracing::info!("🧪 Test 2: Getting product list...");
    let product_list_result = client
        .call_tool(CallToolRequestParam {
            name: "product_get_product_list".into(),
            arguments: serde_json::json!({
                "page": 1,
                "limit": 20
            })
            .as_object()
            .cloned(),
        })
        .await;
    match product_list_result {
        Ok(_result) => {
            tracing::info!("✅ Product list result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get product list: {}", e);
        }
    }

    // Test 3: Get product details (后查询)
    tracing::info!("🧪 Test 3: Getting product details...");
    let product_details_result = client
        .call_tool(CallToolRequestParam {
            name: "product/get_product".into(),
            arguments: serde_json::json!({
                "productId": product_id
            })
            .as_object()
            .cloned(),
        })
        .await;
    match product_details_result {
        Ok(_result) => {
            tracing::info!("✅ Product details result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get product details: {}", e);
        }
    }

    // Test 4: Update a product (再修改)
    tracing::info!("🧪 Test 4: Updating a product...");
    let update_product_result = client
        .call_tool(CallToolRequestParam {
            name: "product/update_product".into(),
            arguments: serde_json::json!({
                "productId": product_id,
                "name": "Updated Test Product",
                "code": null,
                "type": null,
                "line": null,
                "program": null,
                "status": "closed",
                "desc": "This product has been updated by MCP client",
                "po": null,
                "qd": null,
                "rd": null,
                "acl": null,
                "whitelist": null
            })
            .as_object()
            .cloned(),
        })
        .await;
    match update_product_result {
        Ok(_result) => {
            tracing::info!("✅ Update product result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to update product: {}", e);
        }
    }

    // Test 5: Get product list with pagination (后查询)
    tracing::info!("🧪 Test 5: Getting product list with pagination...");
    let paginated_product_list_result = client
        .call_tool(CallToolRequestParam {
            name: "product/get_product_list".into(),
            arguments: serde_json::json!({
                "page": 1,
                "limit": 10
            })
            .as_object()
            .cloned(),
        })
        .await;
    match paginated_product_list_result {
        Ok(_result) => {
            tracing::info!("✅ Paginated product list result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get paginated product list: {}", e);
        }
    }

    // Test 6: Delete a product (最后删除)
    tracing::info!("🧪 Test 6: Deleting a product...");
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

    tracing::info!("🎉 Product management tests completed successfully!");
    Ok(())
}
