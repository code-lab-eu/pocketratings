//! Integration tests for `pocketratings product` CLI.

use std::io::Cursor;

use pocketratings::cli;
use pocketratings::db;

async fn run_product(
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

async fn create_category_and_get_id(pool: &sqlx::SqlitePool, name: &str) -> String {
    let (res, stdout, _) = run_product(
        pool,
        &["category", "create", "--name", name, "--output", "json"],
    )
    .await;
    assert!(res.is_ok());
    let json: serde_json::Value =
        serde_json::from_str(stdout.lines().next().expect("line")).expect("json");
    json.get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string()
}

#[tokio::test]
async fn product_create_and_show_roundtrip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_product_create_show.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = create_category_and_get_id(&pool, "Groceries").await;

    let (create_result, create_stdout, create_stderr) = run_product(
        &pool,
        &[
            "product",
            "create",
            "--name",
            "Widget",
            "--brand",
            "Acme",
            "--category-id",
            &cat_id,
            "--output",
            "json",
        ],
    )
    .await;
    assert!(create_result.is_ok(), "stderr: {}", create_stderr);
    let line = create_stdout.lines().next().expect("line");
    let json: serde_json::Value = serde_json::from_str(line).expect("json");
    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id in response");
    assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("Widget"));
    assert_eq!(json.get("brand").and_then(|v| v.as_str()), Some("Acme"));

    let (show_result, show_stdout, show_stderr) =
        run_product(&pool, &["product", "show", id, "--output", "json"]).await;
    assert!(show_result.is_ok(), "stderr: {}", show_stderr);
    let line = show_stdout.lines().next().expect("show line");
    let show_json: serde_json::Value = serde_json::from_str(line).expect("json");
    assert_eq!(
        show_json.get("name").and_then(|v| v.as_str()),
        Some("Widget")
    );
    assert_eq!(
        show_json.get("brand").and_then(|v| v.as_str()),
        Some("Acme")
    );
}

#[tokio::test]
async fn product_list_filters_by_category_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_product_list.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat1_id = create_category_and_get_id(&pool, "Cat1").await;
    let cat2_id = create_category_and_get_id(&pool, "Cat2").await;

    let (r1, _, _) = run_product(
        &pool,
        &[
            "product",
            "create",
            "--name",
            "P1",
            "--brand",
            "B1",
            "--category-id",
            &cat1_id,
        ],
    )
    .await;
    assert!(r1.is_ok());
    let (r2, _, _) = run_product(
        &pool,
        &[
            "product",
            "create",
            "--name",
            "P2",
            "--brand",
            "B2",
            "--category-id",
            &cat1_id,
        ],
    )
    .await;
    assert!(r2.is_ok());
    let (r3, _, _) = run_product(
        &pool,
        &[
            "product",
            "create",
            "--name",
            "P3",
            "--brand",
            "B3",
            "--category-id",
            &cat2_id,
        ],
    )
    .await;
    assert!(r3.is_ok());

    let (list_res, list_stdout, list_stderr) = run_product(
        &pool,
        &[
            "product",
            "list",
            "--category-id",
            &cat1_id,
            "--output",
            "json",
        ],
    )
    .await;
    assert!(list_res.is_ok(), "stderr: {}", list_stderr);
    let line = list_stdout.lines().next().expect("list line");
    let arr: Vec<serde_json::Value> = serde_json::from_str(line).expect("json array");
    assert_eq!(arr.len(), 2);
}

#[tokio::test]
async fn product_list_include_deleted() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_product_list_deleted.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = create_category_and_get_id(&pool, "C").await;
    let (_, create_stdout, _) = run_product(
        &pool,
        &[
            "product",
            "create",
            "--name",
            "ToDelete",
            "--brand",
            "B",
            "--category-id",
            &cat_id,
            "--output",
            "json",
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
        run_product(&pool, &["product", "list", "--output", "json"]).await;
    assert!(list_before.is_ok());
    let count_before: Vec<serde_json::Value> =
        serde_json::from_str(list_stdout_before.lines().next().expect("line")).expect("json");
    assert_eq!(count_before.len(), 1);

    let (del_res, _, _) = run_product(&pool, &["product", "delete", &id]).await;
    assert!(del_res.is_ok());

    let (list_after, list_stdout_after, _) =
        run_product(&pool, &["product", "list", "--output", "json"]).await;
    assert!(list_after.is_ok());
    let count_after: Vec<serde_json::Value> =
        serde_json::from_str(list_stdout_after.lines().next().expect("line")).expect("json");
    assert_eq!(count_after.len(), 0);

    let (list_deleted, list_stdout_deleted, _) = run_product(
        &pool,
        &["product", "list", "--include-deleted", "--output", "json"],
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
async fn product_update() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_product_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = create_category_and_get_id(&pool, "C").await;
    let (_, create_stdout, _) = run_product(
        &pool,
        &[
            "product",
            "create",
            "--name",
            "OldName",
            "--brand",
            "OldBrand",
            "--category-id",
            &cat_id,
            "--output",
            "json",
        ],
    )
    .await;
    let id = serde_json::from_str::<serde_json::Value>(create_stdout.lines().next().expect("line"))
        .expect("json")
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (update_res, update_stdout, update_stderr) = run_product(
        &pool,
        &[
            "product", "update", &id, "--name", "NewName", "--brand", "NewBrand", "--output",
            "json",
        ],
    )
    .await;
    assert!(update_res.is_ok(), "stderr: {}", update_stderr);
    let line = update_stdout.lines().next().expect("line");
    let json: serde_json::Value = serde_json::from_str(line).expect("json");
    assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("NewName"));
    assert_eq!(json.get("brand").and_then(|v| v.as_str()), Some("NewBrand"));

    let (show_res, show_stdout, _) =
        run_product(&pool, &["product", "show", &id, "--output", "json"]).await;
    assert!(show_res.is_ok());
    let show_json: serde_json::Value =
        serde_json::from_str(show_stdout.lines().next().expect("line")).expect("json");
    assert_eq!(
        show_json.get("name").and_then(|v| v.as_str()),
        Some("NewName")
    );
}

#[tokio::test]
async fn product_delete_soft_deletes() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_product_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = create_category_and_get_id(&pool, "C").await;
    let (_, create_stdout, _) = run_product(
        &pool,
        &[
            "product",
            "create",
            "--name",
            "ToDelete",
            "--brand",
            "B",
            "--category-id",
            &cat_id,
            "--output",
            "json",
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
        run_product(&pool, &["product", "delete", &id]).await;
    assert!(del_result.is_ok(), "stderr: {}", del_stderr);

    let product = db::product::get_by_id(&pool, id.parse().expect("uuid"))
        .await
        .expect("get_by_id");
    assert!(
        product.is_none(),
        "soft-deleted product should not be returned by get_by_id"
    );
    let with_deleted = db::product::get_all_with_deleted(&pool)
        .await
        .expect("get_all_with_deleted");
    assert!(
        with_deleted
            .iter()
            .any(|p| p.id().to_string() == id && !p.is_active()),
        "deleted product should appear in get_all_with_deleted and be inactive"
    );
}

#[tokio::test]
async fn product_delete_force_removes_row() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_product_delete_force.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = create_category_and_get_id(&pool, "C").await;
    let (_, create_stdout, _) = run_product(
        &pool,
        &[
            "product",
            "create",
            "--name",
            "ToRemove",
            "--brand",
            "B",
            "--category-id",
            &cat_id,
            "--output",
            "json",
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
        run_product(&pool, &["product", "delete", &id, "--force"]).await;
    assert!(del_result.is_ok(), "stderr: {}", del_stderr);

    let product = db::product::get_by_id(&pool, id.parse().expect("uuid"))
        .await
        .expect("get_by_id");
    assert!(product.is_none());
    let with_deleted = db::product::get_all_with_deleted(&pool)
        .await
        .expect("get_all_with_deleted");
    assert!(
        !with_deleted.iter().any(|p| p.id().to_string() == id),
        "hard-deleted product should not appear in get_all_with_deleted"
    );
}

#[tokio::test]
async fn product_create_fails_with_invalid_category_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_product_invalid_cat.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let fake_cat_id = uuid::Uuid::new_v4().to_string();
    let (res, _stdout, _stderr) = run_product(
        &pool,
        &[
            "product",
            "create",
            "--name",
            "P",
            "--brand",
            "B",
            "--category-id",
            &fake_cat_id,
        ],
    )
    .await;
    assert!(
        res.is_err(),
        "create should fail when category does not exist"
    );
}

#[tokio::test]
async fn product_show_fails_for_nonexistent_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_product_show_missing.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let fake_id = uuid::Uuid::new_v4().to_string();
    let (res, _stdout, _stderr) = run_product(&pool, &["product", "show", &fake_id]).await;
    assert!(res.is_err(), "show should fail for non-existent product");
}

#[tokio::test]
async fn product_delete_fails_when_product_has_purchases() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_product_delete_purchases.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let cat_id = create_category_and_get_id(&pool, "C").await;
    let (_, create_stdout, _) = run_product(
        &pool,
        &[
            "product",
            "create",
            "--name",
            "WithPurchase",
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
        serde_json::from_str::<serde_json::Value>(create_stdout.lines().next().expect("line"))
            .expect("json")
            .get("id")
            .and_then(|v| v.as_str())
            .expect("id")
            .to_string();

    let user_id = uuid::Uuid::new_v4();
    let location_id = uuid::Uuid::new_v4();
    let now = 1_000_i64;
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
    sqlx::query("INSERT INTO locations (id, name, deleted_at) VALUES (?, ?, ?)")
        .bind(location_id.to_string())
        .bind("Store")
        .bind::<Option<i64>>(None)
        .execute(&pool)
        .await
        .expect("insert location");
    sqlx::query(
        "INSERT INTO purchases (id, user_id, product_id, location_id, quantity, price, purchased_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(user_id.to_string())
    .bind(&product_id)
    .bind(location_id.to_string())
    .bind(1_i32)
    .bind("9.99")
    .bind(now)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert purchase");

    let (del_result, _stdout, _stderr) =
        run_product(&pool, &["product", "delete", &product_id]).await;
    assert!(
        del_result.is_err(),
        "delete should fail when product has purchases"
    );

    let (force_result, _stdout, _stderr) =
        run_product(&pool, &["product", "delete", &product_id, "--force"]).await;
    assert!(
        force_result.is_err(),
        "delete --force should fail when product has purchases"
    );
}
