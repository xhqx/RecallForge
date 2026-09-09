//! Explicit model-backed retrieval evaluation. No toy/hash embedding fallback.
use recallforge::service::Service;
use serde_json::json;

#[test]
#[ignore = "downloads the multilingual E5 model on first use"]
fn russian_queries_retrieve_english_lessons() {
    let dir = tempfile::tempdir().unwrap();
    // Reuse a user-selected model cache across runs while keeping all memory synthetic.
    let mut service = Service::open(dir.path().join("data")).unwrap();
    service
        .store
        .add_project("demo", "Demo", dir.path())
        .unwrap();
    let corpus = [
        (
            "Dependency registration in tests",
            "Integration tests fail because the handler service is missing from the test application container.",
            "Register the service in the test dependency injection module.",
        ),
        (
            "Database migrations before startup",
            "Database schema is missing a newly added column after upgrading the application.",
            "Apply SQL migrations before queries start.",
        ),
        (
            "Timezone conversion",
            "Calendar events appear on the wrong day when UTC timestamps are interpreted as local time.",
            "Store UTC and convert at the presentation boundary.",
        ),
        (
            "Expired authentication token",
            "API requests return unauthorized because an access token has expired.",
            "Refresh the access token and retry the request once.",
        ),
        (
            "Video audio synchronization",
            "Exported video sound drifts from the image because time bases differ.",
            "Normalize presentation timestamps before multiplexing audio and video.",
        ),
        (
            "Optimistic revision conflicts",
            "Concurrent editors overwrite each other's changes to an existing record.",
            "Use an expected revision compare-and-swap and retry after reading the latest state.",
        ),
    ];
    let mut ids = vec![];
    for (title, problem, solution) in corpus {
        let result=service.call("memory_save",json!({"project":"demo","entry":{"title":title,"problem":problem,"solution":solution,"status":"verified","verification":"Synthetic evaluation fixture","sources":[{"reference":"tests/semantic.rs"}]}})).unwrap();
        ids.push(result["id"].as_str().unwrap().to_string());
    }
    let cases = [
        (
            "Тесты падают, потому что нужный сервис не зарегистрирован",
            0,
        ),
        ("После обновления приложения в базе нет нового столбца", 1),
        (
            "Событие календаря отображается на другую дату из-за часового пояса",
            2,
        ),
        ("Срок действия токена истёк, сервер не разрешает запрос", 3),
        ("Звук отстаёт от картинки при экспорте ролика", 4),
        (
            "Два агента одновременно изменили одну запись и затёрли изменения",
            5,
        ),
    ];
    for mode in ["semantic", "hybrid"] {
        let mut top1 = 0;
        let mut top3 = 0;
        for (query, index) in cases {
            let result = service
                .call(
                    "memory_search",
                    json!({"project":"demo","query":query,"mode":mode,"limit":3}),
                )
                .unwrap();
            let hits = result["hits"].as_array().unwrap();
            if hits.first().is_some_and(|h| h["id"] == ids[index]) {
                top1 += 1;
            }
            if hits.iter().any(|h| h["id"] == ids[index]) {
                top3 += 1;
            }
            println!(
                "{mode}: {query} => {}",
                hits.first()
                    .map(|h| h["title"].as_str().unwrap())
                    .unwrap_or("no results")
            );
        }
        println!("{mode}: top1={top1}/6 top3={top3}/6");
        assert_eq!(top3, 6);
        assert!(top1 >= 5, "cross-language top1 regression");
    }
}
