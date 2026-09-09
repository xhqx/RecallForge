use recallforge::{service::Service, store::Store};
use serde_json::json;

#[test]
fn refuses_another_app_database_without_changing_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("memory.db");
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("CREATE TABLE precious(value TEXT); INSERT INTO precious VALUES ('keep');")
        .unwrap();
    drop(conn);
    let before = std::fs::read(&path).unwrap();
    assert!(Store::open(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn oversized_or_invalid_fields_are_rejected_without_model_loading() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::open(dir.path().join("data")).unwrap();
    service
        .store
        .add_project("test", "Test", dir.path())
        .unwrap();
    for entry in [
        json!({"title":"x","problem":"y","unexpected":true}),
        json!({"title":"x","problem":"y","status":"verified"}),
        json!({"title":"x","problem":"x".repeat(65000)}),
    ] {
        assert!(
            service
                .call("memory_save", json!({"project":"test","entry":entry}))
                .is_err()
        );
    }
    assert_eq!(service.store.doctor().unwrap()["entries"], 0);
}

#[test]
fn rejects_future_schema_without_resetting_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("memory.db");
    drop(Store::open(&path).unwrap());
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("PRAGMA user_version=999;").unwrap();
    drop(conn);
    assert!(Store::open(&path).is_err());
    let conn = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        999
    );
}

#[test]
fn input_chunking_keeps_unicode_and_tail_solution() {
    let text = format!("{}TAIL_SOLUTION", "Длинная запись 🦀 ".repeat(300));
    let chunks = recallforge::embedding::chunks(&text);
    assert!(chunks.len() > 1);
    assert!(chunks.last().unwrap().contains("TAIL_SOLUTION"));
    assert!(chunks.iter().all(|c| c.chars().count() <= 800));
}
