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
            name: "Test Test Case Client".to_string(),
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

    // Test 1: Create a new test case
    tracing::info!("🧪 Test 1: Creating a new test case...");
    let create_testcase_result = client
        .call_tool(CallToolRequestParam {
            name: "testcase/create_testcase".into(),
            arguments: serde_json::json!({
                "productId": 1,
                "title": "Simple Test Case",
                "type": "feature",
                "pri": 1,
                "steps": [
                    {
                        "desc": "Step 1",
                        "expect": "Result 1"
                    }
                ]
            }).as_object().cloned(),
        })
        .await?;

    // Extract testcase_id from the creation result
    let testcase_id = if let Some(content) = create_testcase_result.content.first() {
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
    tracing::info!("📝 Creat ID: {}", testcase_id);

    // Test 2: Get test case details
    tracing::info!("🧪 Test 2: Getting test case details...");
    let get_testcase_result = client
        .call_tool(CallToolRequestParam {
            name: "testcase/get_testcase".into(),
            arguments: serde_json::json!({
                "testcaseId": testcase_id
            }).as_object().cloned(),
        })
        .await;
    match get_testcase_result {
        Ok(_result) => tracing::info!("✅ Get test case success"),
        Err(e) => tracing::error!("❌ Get test case error: {:?}", e),
    }

    // Test 3: Update test case
    tracing::info!("🧪 Test 3: Updating test case...");
    let update_testcase_result = client
        .call_tool(CallToolRequestParam {
            name: "testcase/update_testcase".into(),
            arguments: serde_json::json!({
                "testcaseId": testcase_id,
                "title": "Updated Test Case Title",
                "steps": [
                    {
                        "desc": "Updated Step 1: Modified setup process",
                        "expect": "Updated expected result for step 1"
                    },
                    {
                        "desc": "Updated Step 2: Enhanced main functionality",
                        "expect": "Updated expected result for step 2"
                    }
                ]
            }).as_object().cloned(),
        })
        .await;
    match update_testcase_result {
        Ok(_result) => tracing::info!("✅ Update test case success"),
        Err(e) => tracing::error!("❌ Update test case error: {:?}", e),
    }
        
  

    // Test 4: Get test cases by product ID
    tracing::info!("🧪 Test 4: Getting test cases by product ID...");
    let get_product_testcases_result = client
        .call_tool(CallToolRequestParam {
            name: "testcase/get_product_testcases".into(),
            arguments: serde_json::json!({
                "productId": 1
            }).as_object().cloned(),
        })
        .await;
    match get_product_testcases_result {
        Ok(_result) => tracing::info!("✅ Get product test cases success"),
        Err(e) => tracing::error!("❌ Get product test cases error: {:?}", e),
    }

    // Test 5: Execute test case
    tracing::info!("🧪 Test 5: Executing test case...");
    let execute_testcase_result = client
        .call_tool(CallToolRequestParam {
            name: "testcase/execute_testcase".into(),
            arguments: serde_json::json!({
                "testcaseId": testcase_id,
                "testtask": 1,
                "version": 1,
                "steps": [
                    {
                        "result": "pass",
                        "real": "Test step executed successfully"
                    },
                    {
                        "result": "fail", 
                        "real": "Test step failed due to timeout"
                    }
                ]
            }).as_object().cloned(),
        })
        .await;
    match execute_testcase_result {
        Ok(_result) => tracing::info!("✅ Execute test case success"),
        Err(e) => tracing::error!("❌ Execute test case error: {:?}", e),
    }

    // Test 6: Delete test case
    tracing::info!("🧪 Test 6: Deleting test case...");
    let delete_testcase_result = client
        .call_tool(CallToolRequestParam {
            name: "testcase/delete_testcase".into(),
            arguments: serde_json::json!({
                "testcaseId": testcase_id
            }).as_object().cloned(),
        })
        .await;
    match delete_testcase_result {
        Ok(_result) => tracing::info!("✅ Delete test case success"),
        Err(e) => tracing::error!("❌ Delete test case error: {:?}", e),
    }
    
    client.cancel().await?;
    

    tracing::info!("🎉 Test case management tests completed successfully!");
    Ok(())
}