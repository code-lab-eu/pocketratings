//! Integration tests for `pocketratings location` CLI.

use std::io::Cursor;

use pocketratings::cli;
use pocketratings::db;
use pocketratings::domain::product_variation::ProductVariation;
use pocketratings::domain::purchase::Purchase;
use rust_decimal::Decimal;

async fn run_location(
    pool: &sqlx::SqlitePool,
    args: &[&str],
) -> (Result<(), cli::CliError>, String, String) {
    let mut full: Vec<std::ffi::OsString> = Vec::with_capacity(args.len() + 1);
    full.push(std::ffi::OsString::from("pocketratings"));
    for a in args {
        full.push(std::ffi::OsString::from(a));
    }

    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let result = cli::run(full.into_iter(), Some(pool), None, &mut stdout, &mut stderr).await;
    let stdout_str = String::from_utf8(stdout.into_inner()).expect("stdout UTF-8");
    let stderr_str = String::from_utf8(stderr.into_inner()).expect("stderr UTF-8");
    (result, stdout_str, stderr_str)
}

#[tokio::test]
async fn location_create_and_show_roundtrip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_location_create_show.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (create_result, create_stdout, create_stderr) = run_location(
        &pool,
        &[
            "location",
            "create",
            "--name",
            "Warehouse",
            "--output",
            "json",
        ],
    )
    .await;
    assert!(create_result.is_ok(), "stderr: {create_stderr}");
    let line = create_stdout.lines().next().expect("line");
    let json: serde_json::Value = serde_json::from_str(line).expect("json");
    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id in response");
    assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("Warehouse"));
    assert_eq!(json.get("deleted"), Some(&serde_json::Value::Bool(false)));

    let (show_result, show_stdout, show_stderr) =
        run_location(&pool, &["location", "show", id, "--output", "json"]).await;
    assert!(show_result.is_ok(), "stderr: {show_stderr}");
    let line = show_stdout.lines().next().expect("show line");
    let show_json: serde_json::Value = serde_json::from_str(line).expect("json");
    assert_eq!(
        show_json.get("name").and_then(|v| v.as_str()),
        Some("Warehouse")
    );
}

#[tokio::test]
async fn location_list_include_deleted() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_location_list.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (_, create_stdout, _) = run_location(
        &pool,
        &[
            "location", "create", "--name", "Store A", "--output", "json",
        ],
    )
    .await;
    let id = serde_json::from_str::<serde_json::Value>(create_stdout.lines().next().expect("line"))
        .expect("json")
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (list_before, list_stdout_before, _) =
        run_location(&pool, &["location", "list", "--output", "json"]).await;
    assert!(list_before.is_ok());
    let count_before: Vec<serde_json::Value> =
        serde_json::from_str(list_stdout_before.lines().next().expect("line")).expect("json");
    assert_eq!(count_before.len(), 1);

    let (del_res, _, _) = run_location(&pool, &["location", "delete", &id]).await;
    assert!(del_res.is_ok());

    let (list_after, list_stdout_after, _) =
        run_location(&pool, &["location", "list", "--output", "json"]).await;
    assert!(list_after.is_ok());
    let count_after: Vec<serde_json::Value> =
        serde_json::from_str(list_stdout_after.lines().next().expect("line")).expect("json");
    assert_eq!(count_after.len(), 0);

    let (list_deleted, list_stdout_deleted, _) = run_location(
        &pool,
        &["location", "list", "--include-deleted", "--output", "json"],
    )
    .await;
    assert!(list_deleted.is_ok());
    let with_deleted: Vec<serde_json::Value> =
        serde_json::from_str(list_stdout_deleted.lines().next().expect("line")).expect("json");
    assert_eq!(with_deleted.len(), 1);
    assert_eq!(
        with_deleted[0].get("deleted"),
        Some(&serde_json::Value::Bool(true))
    );
}

#[tokio::test]
async fn location_update() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_location_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (_, create_stdout, _) = run_location(
        &pool,
        &[
            "location", "create", "--name", "OldName", "--output", "json",
        ],
    )
    .await;
    let id = serde_json::from_str::<serde_json::Value>(create_stdout.lines().next().expect("line"))
        .expect("json")
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (update_res, update_stdout, update_stderr) = run_location(
        &pool,
        &[
            "location", "update", &id, "--name", "NewName", "--output", "json",
        ],
    )
    .await;
    assert!(update_res.is_ok(), "stderr: {update_stderr}");
    let line = update_stdout.lines().next().expect("line");
    let json: serde_json::Value = serde_json::from_str(line).expect("json");
    assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("NewName"));

    let (show_res, show_stdout, _) =
        run_location(&pool, &["location", "show", &id, "--output", "json"]).await;
    assert!(show_res.is_ok());
    let show_json: serde_json::Value =
        serde_json::from_str(show_stdout.lines().next().expect("line")).expect("json");
    assert_eq!(
        show_json.get("name").and_then(|v| v.as_str()),
        Some("NewName")
    );
}

#[tokio::test]
async fn location_delete_soft_deletes() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_location_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (_, create_stdout, _) = run_location(
        &pool,
        &[
            "location", "create", "--name", "ToDelete", "--output", "json",
        ],
    )
    .await;
    let id = serde_json::from_str::<serde_json::Value>(create_stdout.lines().next().expect("line"))
        .expect("json")
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (del_result, _del_stdout, del_stderr) =
        run_location(&pool, &["location", "delete", &id]).await;
    assert!(del_result.is_ok(), "stderr: {del_stderr}");

    let location = db::location::get_by_id(&pool, id.parse().expect("uuid"), false)
        .await
        .expect("get_by_id");
    assert!(
        location.is_none(),
        "soft-deleted location should not be returned by get_by_id"
    );
    let with_deleted = db::location::get_all(&pool, true)
        .await
        .expect("get_all(include_deleted: true)");
    assert!(
        with_deleted
            .iter()
            .any(|l| l.id().to_string() == id && !l.is_active()),
        "deleted location should appear in get_all(include_deleted: true) and be inactive"
    );
}

#[tokio::test]
async fn location_delete_force_removes_row() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_location_delete_force.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (_, create_stdout, _) = run_location(
        &pool,
        &[
            "location", "create", "--name", "ToRemove", "--output", "json",
        ],
    )
    .await;
    let id = serde_json::from_str::<serde_json::Value>(create_stdout.lines().next().expect("line"))
        .expect("json")
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (del_result, _del_stdout, del_stderr) =
        run_location(&pool, &["location", "delete", &id, "--force"]).await;
    assert!(del_result.is_ok(), "stderr: {del_stderr}");

    let location = db::location::get_by_id(&pool, id.parse().expect("uuid"), false)
        .await
        .expect("get_by_id");
    assert!(location.is_none());
    let with_deleted = db::location::get_all(&pool, true)
        .await
        .expect("get_all(include_deleted: true)");
    assert!(
        !with_deleted.iter().any(|l| l.id().to_string() == id),
        "hard-deleted location should not appear in get_all(include_deleted: true)"
    );
}

#[tokio::test]
async fn location_show_non_existent_fails() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_location_show_missing.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let non_existent = uuid::Uuid::new_v4().to_string();
    let (res, _stdout, _stderr) = run_location(&pool, &["location", "show", &non_existent]).await;
    assert!(res.is_err(), "show should fail for non-existent location");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn location_delete_fails_when_location_has_purchases() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_location_delete_purchases.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (_, create_stdout, _) = run_location(
        &pool,
        &[
            "location",
            "create",
            "--name",
            "WithPurchase",
            "--output",
            "json",
        ],
    )
    .await;
    let location_id =
        serde_json::from_str::<serde_json::Value>(create_stdout.lines().next().expect("line"))
            .expect("json")
            .get("id")
            .and_then(|v| v.as_str())
            .expect("id")
            .to_string();

    let (_, cat_stdout, _) = run_location(
        &pool,
        &["category", "create", "--name", "C", "--output", "json"],
    )
    .await;
    let cat_id =
        serde_json::from_str::<serde_json::Value>(cat_stdout.lines().next().expect("line"))
            .expect("json")
            .get("id")
            .and_then(|v| v.as_str())
            .expect("id")
            .to_string();

    let (_, prod_stdout, _) = run_location(
        &pool,
        &[
            "product",
            "create",
            "--name",
            "P",
            "--brand",
            "B",
            "--category-id",
            &cat_id,
            "--output",
            "json",
        ],
    )
    .await;
    let product_id =
        serde_json::from_str::<serde_json::Value>(prod_stdout.lines().next().expect("line"))
            .expect("json")
            .get("id")
            .and_then(|v| v.as_str())
            .expect("id")
            .to_string();
    let product_uuid = uuid::Uuid::parse_str(&product_id).expect("product id uuid");
    let location_uuid = uuid::Uuid::parse_str(&location_id).expect("location id uuid");

    let now = 1_000_i64;
    let variation_id = {
        let existing = db::product_variation::list_by_product_id(&pool, product_uuid, false)
            .await
            .expect("list variations");
        if let Some(v) = existing.first() {
            v.id()
        } else {
            let var_id = uuid::Uuid::new_v4();
            let var = ProductVariation::new(var_id, product_uuid, "", "none", None, now, now, None)
                .expect("valid variation");
            db::product_variation::insert(&pool, &var)
                .await
                .expect("insert variation");
            var_id
        }
    };

    let user_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, name, email, password, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id.to_string())
    .bind("User")
    .bind("u@example.com")
    .bind("hash")
    .bind(now)
    .bind(now)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert user");
    let purchase = Purchase::new(
        uuid::Uuid::new_v4(),
        user_id,
        product_uuid,
        variation_id,
        location_uuid,
        1,
        "9.99".parse::<Decimal>().expect("decimal"),
        now,
        None,
    )
    .expect("valid purchase");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert purchase");

    let (del_result, _stdout, _stderr) =
        run_location(&pool, &["location", "delete", &location_id]).await;
    assert!(
        del_result.is_err(),
        "delete should fail when location has purchases"
    );

    let (force_result, _stdout, _stderr) =
        run_location(&pool, &["location", "delete", &location_id, "--force"]).await;
    assert!(
        force_result.is_err(),
        "delete --force should fail when location has purchases"
    );
}
