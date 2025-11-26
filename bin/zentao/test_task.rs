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
            name: "Test Task Management Client".to_string(),
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

    // Test 1: Get execution tasks list
    tracing::info!("🧪 Test 1: Getting execution tasks list...");
    let get_execution_tasks_result = client
        .call_tool(CallToolRequestParam {
            name: "task_get_execution_tasks".into(),
            arguments: serde_json::json!({
                "executionId": 1
            }).as_object().cloned(),
        })
        .await;
    match get_execution_tasks_result {
        Ok(_result) => {
            tracing::info!("✅ Execution tasks list result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get execution tasks list: {}", e);
        }
    }

    // Test 2: Create a new task
    tracing::info!("🧪 Test 2: Creating a new task...");
    let create_task_result = client
        .call_tool(CallToolRequestParam {
            name: "task_create_task".into(),
            arguments: serde_json::json!({
                "executionId": 1,
                "name": "Test Task MCP1",
                "assignedTo": ["admin"],
                "type": "design",
                "estStarted": "2025-10-23",
                "deadline": "2025-10-25"
            }).as_object().cloned(),
        })
        .await?;

    // Extract task ID from the creation result
    let task_id = if let Some(content) = create_task_result.content.first() {
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
    tracing::info!("📝 Created task ID: {}", task_id);

    // Test 3: Get task details
    tracing::info!("🧪 Test 3: Getting task details...");
    let get_task_result = client
        .call_tool(CallToolRequestParam {
            name: "task_get_task".into(),
            arguments: serde_json::json!({
                "taskId": task_id
            }).as_object().cloned(),
        })
        .await;
    match get_task_result {
        Ok(_result) => {
            tracing::info!("✅ Task details result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get task details: {}", e);
        }
    }

    // Test 4: Update a task
    tracing::info!("🧪 Test 4: Updating a task...");
    let update_task_result = client
        .call_tool(CallToolRequestParam {
            name: "task_update_task".into(),
            arguments: serde_json::json!({
                "taskId": task_id,
                "name": "Updated Test Task",
                "assignedTo": ["admin"],
                "pri": 1,
                "estimate": 4.0,
                "desc": "This task has been updated by MCP client"
            }).as_object().cloned(),
        })
        .await;
    match update_task_result {
        Ok(_result) => {
            tracing::info!("✅ Update task result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to update task: {}", e);
        }
    }

    // Test 5: Start a task
    tracing::info!("🧪 Test 5: Starting a task...");
    let start_task_result = client
        .call_tool(CallToolRequestParam {
            name: "task_start_task".into(),
            arguments: serde_json::json!({
                "taskId": task_id,
                "assignedTo": "admin",
                "left": 8.0
            }).as_object().cloned(),
        })
        .await;
    match start_task_result {
        Ok(_result) => {
            tracing::info!("✅ Start task result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to start task: {}", e);
        }
    }

    // Test 6: Pause a task
    tracing::info!("🧪 Test 6: Pausing a task...");
    let pause_task_result = client
        .call_tool(CallToolRequestParam {
            name: "task_pause_task".into(),
            arguments: serde_json::json!({
                "taskId": task_id,
                "comment": "Pausing for review"
            }).as_object().cloned(),
        })
        .await;
    match pause_task_result {
        Ok(_result) => {
            tracing::info!("✅ Pause task result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to pause task: {}", e);
        }
    }

    // Test 7: Restart a task
    tracing::info!("🧪 Test 7: Restarting a task...");
    let restart_task_result = client
        .call_tool(CallToolRequestParam {
            name: "task_restart_task".into(),
            arguments: serde_json::json!({
                "taskId": task_id,
                "assignedTo": "admin",
                "realStarted": "2025-10-25 14:04:59",
                "consumed": 2,
                "left": 6
            }).as_object().cloned(),
        })
        .await;
    match restart_task_result {
        Ok(_result) => {
            tracing::info!("✅ Restart task result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to restart task: {}", e);
        }
    }

    // Test 8: Add task logs
    tracing::info!("🧪 Test 8: Adding task logs...");
    let add_task_logs_result = client
        .call_tool(CallToolRequestParam {
            name: "task_add_task_logs".into(),
            arguments: serde_json::json!({
                "taskId": task_id,
                "version": "new",
                "newVersionBody": {
                    "date": ["2025-10-23"],
                    "consumed": [2.0],
                    "left": [4.0],
                    "work": ["Worked on task implementation"]
                }
            }).as_object().cloned(),
        })
        .await;
    match add_task_logs_result {
        Ok(_result) => {
            tracing::info!("✅ Add task logs result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to add task logs: {}", e);
        }
    }

    // Test 9: Get task logs
    tracing::info!("🧪 Test 9: Getting task logs...");
    let get_task_logs_result = client
        .call_tool(CallToolRequestParam {
            name: "task_get_task_logs".into(),
            arguments: serde_json::json!({
                "taskId": task_id
            }).as_object().cloned(),
        })
        .await;
    match get_task_logs_result {
        Ok(_result) => {
            tracing::info!("✅ Get task logs result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to get task logs: {}", e);
        }
    }

    // Test 10: Finish a task
    tracing::info!("🧪 Test 10: Finishing a task...");
    let finish_task_result = client
        .call_tool(CallToolRequestParam {
            name: "task_finish_task".into(),
            arguments: serde_json::json!({
                "taskId": task_id,
                "currentConsumed": 6,
                "finishedDate": "2025-10-26 15:45:27"
            }).as_object().cloned(),
        })
        .await;
    match finish_task_result {
        Ok(_result) => {
            tracing::info!("✅ Finish task result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to finish task: {}", e);
        }
    }

    // Test 11: Close a task
    tracing::info!("🧪 Test 11: Closing a task...");
    let close_task_result = client
        .call_tool(CallToolRequestParam {
            name: "task_close_task".into(),
            arguments: serde_json::json!({
                "taskId": task_id
            }).as_object().cloned(),
        })
        .await;
    match close_task_result {
        Ok(_result) => {
            tracing::info!("✅ Close task result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to close task: {}", e);
        }
    }

    // Test 12: Delete a task
    tracing::info!("🧪 Test 12: Deleting a task...");
    let delete_task_result = client
        .call_tool(CallToolRequestParam {
            name: "task_delete_task".into(),
            arguments: serde_json::json!({
                "taskId": task_id
            }).as_object().cloned(),
        })
        .await;
    match delete_task_result {
        Ok(_result) => {
            tracing::info!("✅ Delete task result success");
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete task: {}", e);
        }
    }

    client.cancel().await?;
    
    tracing::info!("🎉 Task management tests completed successfully!");
    Ok(())
}