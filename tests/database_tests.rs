// tests/database_tests.rs - Statistics persistence

use dicemaiden_rs::database::Database;

/// A fresh database in its own temporary file.
async fn temp_database(name: &str) -> Database {
    let path = std::env::temp_dir().join(format!("dicemaiden-{name}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let db = Database::connect(&format!("sqlite:{}", path.display()))
        .await
        .expect("open temp database");
    db.init().await.expect("create tables");
    db
}

#[tokio::test]
async fn test_restart_replaces_its_process_stats_row() {
    let db = temp_database("process-rows").await;

    // A PID-keyed row from before process IDs were stable, its replacement,
    // and another process's row that must survive.
    db.update_process_stats("process_80_4242", 80, 15, 256, 30000, 70.0)
        .await
        .unwrap();
    db.update_process_stats("process_80", 80, 15, 256, 30670, 68.7)
        .await
        .unwrap();
    db.update_process_stats("process_96", 96, 15, 256, 30500, 69.0)
        .await
        .unwrap();

    db.remove_stale_process_rows("process_80", 80)
        .await
        .unwrap();

    // A restart writes under the same ID, replacing the row
    db.update_process_stats("process_80", 80, 15, 256, 30700, 69.1)
        .await
        .unwrap();

    let mut rows: Vec<(String, i32)> = db
        .get_all_process_stats()
        .await
        .unwrap()
        .into_iter()
        .map(|s| (s.process_id, s.server_count))
        .collect();
    rows.sort();
    assert_eq!(
        rows,
        vec![
            ("process_80".to_string(), 30700),
            ("process_96".to_string(), 30500)
        ],
        "one row per shard range, so bot-info's totals count each process once"
    );
}
