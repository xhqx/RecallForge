use recallforge::{mcp::handle, service::Service};
use serde_json::json;

#[test]
fn tool_parameter_errors_use_json_rpc_errors() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::open(dir.path().join("data")).unwrap();
    let mut initialized = true;
    for params in [
        json!({"name":"missing","arguments":{}}),
        json!({"name":"memory_get","arguments":{"project_id":"wrong","id":"x"}}),
        json!({"name":"project_list","arguments":{"surprise":true}}),
    ] {
        let response = handle(
            &mut service,
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":params}),
            &mut initialized,
        )
        .unwrap();
        assert_eq!(response["error"]["code"], -32602);
    }
}

#[test]
fn initialization_negotiates_and_notifications_have_no_reply() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::open(dir.path().join("data")).unwrap();
    let mut initialized = false;
    assert!(
        handle(
            &mut service,
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            &mut initialized
        )
        .is_none()
    );
    let response=handle(&mut service,json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"future-version","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),&mut initialized).unwrap();
    assert_eq!(response["result"]["protocolVersion"], "2025-11-25");
    let repeated = handle(
        &mut service,
        json!({"jsonrpc":"2.0","id":2,"method":"initialize"}),
        &mut initialized,
    )
    .unwrap();
    assert!(repeated.get("error").is_some());
}

#[test]
fn malformed_initialize_does_not_advance_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::open(dir.path().join("data")).unwrap();
    let mut initialized = false;
    for params in [
        json!({"protocolVersion":"2025-11-25","clientInfo":{}}),
        json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test"}}),
    ] {
        let result = handle(
            &mut service,
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":params}),
            &mut initialized,
        )
        .unwrap();
        assert_eq!(result["error"]["code"], -32602);
        assert!(!initialized);
    }
}
