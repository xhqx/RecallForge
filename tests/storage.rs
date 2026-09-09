use recallforge::{embedding::DIMENSIONS, model::EntryInput, store::Store};
use serde_json::json;
use tempfile::TempDir;

fn setup() -> (TempDir, Store) {
    let d = TempDir::new().unwrap();
    let s = Store::open(&d.path().join("memory.db")).unwrap();
    s.add_project("alpha", "Alpha", d.path()).unwrap();
    let other = d.path().join("other");
    std::fs::create_dir(&other).unwrap();
    s.add_project("beta", "Beta", &other).unwrap();
    (d, s)
}
fn input(title: &str) -> EntryInput {
    serde_json::from_value(json!({"title":title,"problem":"Handler dependency missing from the integration test application","cause":"Separate dependency graph","solution":"Register the service in the test container","status":"verified","verification":"Regression test passes","sources":[{"reference":"tests/container.rs"}]})).unwrap()
}
fn vector(axis: usize) -> Vec<Vec<f32>> {
    let mut v = vec![0.0; DIMENSIONS];
    v[axis] = 1.0;
    vec![v]
}

#[test]
fn durable_roundtrip_dedup_and_revisions() {
    let (d, mut s) = setup();
    let a = s.save("alpha", input("Container"), None).unwrap();
    assert_eq!(s.save("alpha", input("Container"), None).unwrap().id, a.id);
    let mut changed = a.data.clone();
    changed.id = Some(a.id.clone());
    changed.expected_revision = Some(1);
    changed.solution = "Register both services".into();
    let b = s.save("alpha", changed.clone(), None).unwrap();
    assert_eq!(b.revision, 2);
    assert!(
        s.save("alpha", changed, None)
            .unwrap_err()
            .to_string()
            .contains("revision conflict")
    );
    assert_eq!(s.history("alpha", &a.id).unwrap().len(), 2);
    drop(s);
    let s = Store::open(&d.path().join("memory.db")).unwrap();
    assert_eq!(s.get("alpha", &a.id).unwrap().revision, 2);
}
#[test]
fn project_isolation_in_reads_search_and_graph() {
    let (_d, mut s) = setup();
    let a = s
        .save("alpha", input("Container"), Some(vector(0)))
        .unwrap();
    let b = s.save("beta", input("Container"), Some(vector(0))).unwrap();
    assert!(s.get("beta", &a.id).is_err());
    assert!(s.history("beta", &a.id).is_err());
    assert!(s.link("alpha", &a.id, &b.id, "relates_to").is_err());
    for mode in ["lexical", "semantic", "hybrid"] {
        let r = s
            .search("alpha", "Container", mode, 5, false, Some(&vector(0)[0]))
            .unwrap();
        assert_eq!(r.hits.len(), 1);
        assert_eq!(r.hits[0].id, a.id);
    }
}
#[test]
fn invalid_evidence_and_vectors_leave_no_rows() {
    let (_d, mut s) = setup();
    let mut a = input("Unverified");
    a.sources.clear();
    assert!(s.save("alpha", a, None).is_err());
    assert!(
        s.save("alpha", input("Wrong dimension"), Some(vec![vec![1.0; 3]]))
            .is_err()
    );
    assert!(
        s.save(
            "alpha",
            input("NaN"),
            Some(vec![vec![f32::NAN; DIMENSIONS]])
        )
        .is_err()
    );
    assert_eq!(s.entries("alpha").unwrap().len(), 0);
}
#[test]
fn inactive_filter_runs_before_vector_top_k_and_lexical_update_clears_vectors() {
    let (_d, mut s) = setup();
    let mut stale = input("Old container");
    stale.status = "stale".into();
    for i in 0..30 {
        stale.title = format!("Old {i}");
        s.save("alpha", stale.clone(), Some(vector(0))).unwrap();
    }
    let a = s
        .save("alpha", input("Current container"), Some(vector(1)))
        .unwrap();
    let r = s
        .search(
            "alpha",
            "container",
            "semantic",
            1,
            false,
            Some(&vector(0)[0]),
        )
        .unwrap();
    assert_eq!(r.hits[0].id, a.id);
    let mut changed = a.data;
    changed.id = Some(a.id.clone());
    changed.expected_revision = Some(1);
    s.save("alpha", changed, None).unwrap();
    let r = s
        .search(
            "alpha",
            "container",
            "hybrid",
            5,
            false,
            Some(&vector(0)[0]),
        )
        .unwrap();
    assert!(r.hits[0].semantic_rank.is_none());
    assert_eq!(r.warnings.len(), 1);
}
#[test]
fn search_treats_fts_operators_as_literal_and_updates_index() {
    let (_d, mut s) = setup();
    let a = s.save("alpha", input("Uniqueterm"), None).unwrap();
    for q in [
        "\" OR *",
        "NEAR(foo)",
        "x' ; DROP TABLE entries; --",
        "тесты",
        "Container",
    ] {
        s.search("alpha", q, "lexical", 5, false, None).unwrap();
    }
    let mut changed = a.data;
    changed.id = Some(a.id.clone());
    changed.expected_revision = Some(1);
    changed.title = "Newterm".into();
    s.save("alpha", changed, None).unwrap();
    assert!(
        s.search("alpha", "Uniqueterm", "lexical", 5, false, None)
            .unwrap()
            .hits
            .is_empty()
    );
    assert_eq!(
        s.search("alpha", "Newterm", "lexical", 5, false, None)
            .unwrap()
            .hits
            .len(),
        1
    );
}
#[test]
fn links_are_idempotent_and_backup_restores() {
    let (d, mut s) = setup();
    let a = s.save("alpha", input("One"), Some(vector(0))).unwrap();
    let b = s.save("alpha", input("Two"), Some(vector(1))).unwrap();
    for _ in 0..2 {
        s.link("alpha", &a.id, &b.id, "relates_to").unwrap();
    }
    assert_eq!(
        s.neighbors("alpha", &a.id)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let backup = d.path().join("backup.db");
    s.backup(&backup).unwrap();
    assert!(s.backup(&backup).is_err());
    let restored = Store::open(&backup).unwrap();
    assert_eq!(restored.entries("alpha").unwrap().len(), 2);
    assert_eq!(restored.doctor().unwrap()["integrity"], "ok");
    assert_eq!(
        restored
            .search("alpha", "test", "semantic", 5, false, Some(&vector(0)[0]))
            .unwrap()
            .hits[0]
            .id,
        a.id
    );
}
#[test]
fn concurrent_creates_and_optimistic_updates() {
    let (d, _s) = setup();
    let path = d.path().join("memory.db");
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let path = path.clone();
            std::thread::spawn(move || {
                let mut s = Store::open(&path).unwrap();
                s.save("alpha", input("Same finding"), None).unwrap().id
            })
        })
        .collect();
    let ids: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert!(ids.iter().all(|id| id == &ids[0]));
    let s = Store::open(&path).unwrap();
    assert_eq!(s.entries("alpha").unwrap().len(), 1);
}
#[test]
fn stale_reindex_cannot_replace_newer_vectors() {
    let (_d, mut s) = setup();
    let a = s.save("alpha", input("Initial"), None).unwrap();
    let mut changed = a.data.clone();
    changed.id = Some(a.id.clone());
    changed.expected_revision = Some(1);
    changed.title = "Revised".into();
    s.save("alpha", changed, Some(vector(1))).unwrap();
    assert!(s.index_entry(&a, vector(0)).is_err());
    assert_eq!(s.get("alpha", &a.id).unwrap().revision, 2);
}
#[test]
fn project_roots_cannot_be_reassigned() {
    let (d, s) = setup();
    assert!(s.add_project("beta", "Beta", d.path()).is_err());
    assert!(s.add_project("../bad", "Bad", d.path()).is_err());
    assert_eq!(s.projects().unwrap().len(), 2);
}

#[test]
fn duplicate_semantic_save_repairs_lexical_only_entry_without_new_revision() {
    let (_d, mut s) = setup();
    let a = s.save("alpha", input("Deferred vector"), None).unwrap();
    let b = s
        .save("alpha", input("Deferred vector"), Some(vector(0)))
        .unwrap();
    assert_eq!(a.id, b.id);
    assert!(b.indexed);
    assert_eq!(b.revision, 1);
    assert_eq!(s.history("alpha", &a.id).unwrap().len(), 1);
    assert_eq!(
        s.search(
            "alpha",
            "anything",
            "semantic",
            1,
            false,
            Some(&vector(0)[0])
        )
        .unwrap()
        .hits[0]
            .id,
        a.id
    );
}
