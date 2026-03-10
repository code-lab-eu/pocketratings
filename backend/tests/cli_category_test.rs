//! Integration tests for `pocketratings category` CLI.

use std::io::Cursor;

use pocketratings::cli;
use pocketratings::db;

async fn run_category(
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
async fn category_create_and_show_roundtrip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_create_show.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (create_result, create_stdout, create_stderr) = run_category(
        &pool,
        &[
            "category",
            "create",
            "--name",
            "Groceries",
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
    assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("Groceries"));

    let (show_result, show_stdout, show_stderr) =
        run_category(&pool, &["category", "show", id, "--output", "json"]).await;
    assert!(show_result.is_ok(), "stderr: {show_stderr}");
    let line = show_stdout.lines().next().expect("show line");
    let show_json: serde_json::Value = serde_json::from_str(line).expect("json");
    assert_eq!(
        show_json.get("name").and_then(|v| v.as_str()),
        Some("Groceries")
    );
}

#[tokio::test]
async fn category_list_filters_by_parent_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_list.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    // Create root category.
    let (root_res, root_stdout, _) = run_category(
        &pool,
        &["category", "create", "--name", "Root", "--output", "json"],
    )
    .await;
    assert!(root_res.is_ok());
    let root_json: serde_json::Value =
        serde_json::from_str(root_stdout.lines().next().expect("root line")).expect("json");
    let root_id = root_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("root id");

    // Create child under root.
    let (child_res, _child_stdout, _) = run_category(
        &pool,
        &[
            "category",
            "create",
            "--name",
            "Child",
            "--parent-id",
            root_id,
        ],
    )
    .await;
    assert!(child_res.is_ok());

    // List with parent filter should only show the child.
    let (list_res, list_stdout, list_stderr) = run_category(
        &pool,
        &[
            "category",
            "list",
            "--parent-id",
            root_id,
            "--output",
            "json",
        ],
    )
    .await;
    assert!(list_res.is_ok(), "stderr: {list_stderr}");
    let line = list_stdout.lines().next().expect("list line");
    let arr: Vec<serde_json::Value> = serde_json::from_str(line).expect("json array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].get("name").and_then(|v| v.as_str()), Some("Child"));
}

#[tokio::test]
async fn category_delete_soft_deletes_category() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (create_result, create_stdout, _) = run_category(
        &pool,
        &[
            "category", "create", "--name", "ToDelete", "--output", "json",
        ],
    )
    .await;
    assert!(create_result.is_ok());
    let json: serde_json::Value =
        serde_json::from_str(create_stdout.lines().next().expect("line")).expect("json");
    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id in response");

    let (del_result, _del_stdout, del_stderr) =
        run_category(&pool, &["category", "delete", id]).await;
    assert!(del_result.is_ok(), "stderr: {del_stderr}");

    let cat = db::category::get_by_id(&pool, id.parse().expect("uuid"), false)
        .await
        .expect("get_by_id");
    assert!(
        cat.is_none(),
        "soft-deleted category should not be returned by get_by_id"
    );
    let active = db::category::get_all(&pool, false).await.expect("get_all");
    assert!(
        active.iter().all(|c| c.name() != "ToDelete"),
        "soft-deleted category should not be in get_all"
    );
    let with_deleted = db::category::get_all(&pool, true)
        .await
        .expect("get_all with include_deleted");
    assert!(
        with_deleted
            .iter()
            .any(|c| c.name() == "ToDelete" && !c.is_active()),
        "deleted category should appear in get_all_with_deleted and be inactive"
    );
}

#[tokio::test]
async fn category_delete_fails_when_category_has_products() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_delete_products.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    // Create category.
    let (create_result, create_stdout, _) = run_category(
        &pool,
        &[
            "category",
            "create",
            "--name",
            "HasProducts",
            "--output",
            "json",
        ],
    )
    .await;
    assert!(create_result.is_ok());
    let json: serde_json::Value =
        serde_json::from_str(create_stdout.lines().next().expect("line")).expect("json");
    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id in response");
    let cat_id = uuid::Uuid::parse_str(id).expect("uuid");

    // Insert an active product referencing this category.
    let product_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO products (id, category_id, brand, name, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(product_id.to_string())
    .bind(cat_id.to_string())
    .bind("Brand")
    .bind("Product")
    .bind(1_000_i64)
    .bind(1_000_i64)
    .bind::<Option<i64>>(None)
    .execute(&pool)
    .await
    .expect("insert product");

    let (del_result, _del_stdout, _del_stderr) =
        run_category(&pool, &["category", "delete", id]).await;
    assert!(
        del_result.is_err(),
        "delete should fail when category has active products"
    );
}

#[tokio::test]
async fn category_delete_force_removes_row() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_delete_force.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (create_result, create_stdout, _) = run_category(
        &pool,
        &[
            "category", "create", "--name", "ToRemove", "--output", "json",
        ],
    )
    .await;
    assert!(create_result.is_ok());
    let json: serde_json::Value =
        serde_json::from_str(create_stdout.lines().next().expect("line")).expect("json");
    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id in response");

    let (del_result, _del_stdout, del_stderr) =
        run_category(&pool, &["category", "delete", id, "--force"]).await;
    assert!(del_result.is_ok(), "stderr: {del_stderr}");

    let cat = db::category::get_by_id(&pool, id.parse().expect("uuid"), false)
        .await
        .expect("get_by_id");
    assert!(cat.is_none());
    let with_deleted = db::category::get_all(&pool, true)
        .await
        .expect("get_all with include_deleted");
    assert!(
        !with_deleted.iter().any(|c| c.id().to_string() == id),
        "hard-deleted category should not appear in get_all_with_deleted"
    );
}

#[tokio::test]
async fn category_delete_fails_when_category_has_children() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_delete_children.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    // Create parent category.
    let (create_result, create_stdout, _) = run_category(
        &pool,
        &["category", "create", "--name", "Parent", "--output", "json"],
    )
    .await;
    assert!(create_result.is_ok());
    let json: serde_json::Value =
        serde_json::from_str(create_stdout.lines().next().expect("line")).expect("json");
    let parent_id = json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id in response");

    // Create child category.
    let (child_result, _, _) = run_category(
        &pool,
        &[
            "category",
            "create",
            "--name",
            "Child",
            "--parent-id",
            parent_id,
            "--output",
            "json",
        ],
    )
    .await;
    assert!(child_result.is_ok(), "create child category");

    let (del_result, _del_stdout, _del_stderr) =
        run_category(&pool, &["category", "delete", parent_id]).await;
    assert!(
        del_result.is_err(),
        "delete should fail when category has child categories"
    );
}

#[tokio::test]
async fn category_create_without_json_outputs_human_readable() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_create_human.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (result, stdout, stderr) =
        run_category(&pool, &["category", "create", "--name", "Groceries"]).await;
    assert!(result.is_ok(), "stderr: {stderr}");
    assert!(
        stdout.contains("Category created:") && stdout.contains("Groceries"),
        "stdout: {stdout}"
    );
}

#[tokio::test]
async fn category_create_fails_with_empty_name() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_create_empty_name.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (result, _stdout, _stderr) =
        run_category(&pool, &["category", "create", "--name", ""]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn category_create_fails_with_invalid_parent_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_create_bad_parent.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (result, _stdout, _stderr) = run_category(
        &pool,
        &[
            "category",
            "create",
            "--name",
            "Child",
            "--parent-id",
            "not-a-uuid",
        ],
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn category_list_human_readable_and_include_deleted() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_list_human.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (create_res, create_stdout, _) = run_category(
        &pool,
        &["category", "create", "--name", "Root", "--output", "json"],
    )
    .await;
    assert!(create_res.is_ok());
    let root_id: String = create_stdout
        .lines()
        .next()
        .and_then(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .and_then(|j| j.get("id").and_then(|v| v.as_str()).map(String::from))
        .expect("root id");

    let (list_res, list_stdout, list_stderr) = run_category(&pool, &["category", "list"]).await;
    assert!(list_res.is_ok(), "stderr: {list_stderr}");
    assert!(
        list_stdout.contains("Root") && list_stdout.contains(&root_id),
        "stdout: {list_stdout}"
    );

    let (del_res, _, _) = run_category(&pool, &["category", "delete", &root_id]).await;
    assert!(del_res.is_ok());
    let (list_after_res, list_after_stdout, list_after_stderr) =
        run_category(&pool, &["category", "list", "--include-deleted"]).await;
    assert!(list_after_res.is_ok(), "stderr: {list_after_stderr}");
    assert!(
        list_after_stdout.contains("Root"),
        "include-deleted should show deleted: {list_after_stdout}"
    );
}

#[tokio::test]
async fn category_show_human_readable_and_not_found() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_show_human.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (create_res, create_stdout, _) = run_category(
        &pool,
        &["category", "create", "--name", "ShowMe", "--output", "json"],
    )
    .await;
    assert!(create_res.is_ok());
    let id = create_stdout
        .lines()
        .next()
        .and_then(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .and_then(|j| j.get("id").and_then(|v| v.as_str()).map(String::from))
        .expect("id");

    let (show_res, show_stdout, show_stderr) =
        run_category(&pool, &["category", "show", &id]).await;
    assert!(show_res.is_ok(), "stderr: {show_stderr}");
    assert!(
        show_stdout.contains("Category ") && show_stdout.contains("ShowMe"),
        "stdout: {show_stdout}"
    );

    let (show_bad_res, _stdout, _stderr) =
        run_category(&pool, &["category", "show", "not-a-uuid"]).await;
    assert!(show_bad_res.is_err());

    let nonexistent_id = uuid::Uuid::new_v4().to_string();
    let (show_missing_res, _stdout, _stderr) =
        run_category(&pool, &["category", "show", &nonexistent_id]).await;
    assert!(show_missing_res.is_err());
}

#[tokio::test]
async fn category_update_changes_name_and_outputs_human_readable() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (create_res, create_stdout, _) = run_category(
        &pool,
        &[
            "category", "create", "--name", "Original", "--output", "json",
        ],
    )
    .await;
    assert!(create_res.is_ok());
    let id: String = create_stdout
        .lines()
        .next()
        .and_then(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .and_then(|j| j.get("id").and_then(|v| v.as_str()).map(String::from))
        .expect("id");

    let (update_res, update_stdout, update_stderr) =
        run_category(&pool, &["category", "update", &id, "--name", "Updated"]).await;
    assert!(update_res.is_ok(), "stderr: {update_stderr}");
    assert!(
        update_stdout.contains("Category updated:") && update_stdout.contains("Updated"),
        "stdout: {update_stdout}"
    );

    let (show_res, show_stdout, _) =
        run_category(&pool, &["category", "show", &id, "--output", "json"]).await;
    assert!(show_res.is_ok());
    let show_json: serde_json::Value =
        serde_json::from_str(show_stdout.lines().next().expect("line")).expect("json");
    assert_eq!(
        show_json.get("name").and_then(|v| v.as_str()),
        Some("Updated")
    );
}

#[tokio::test]
async fn category_delete_invalid_id_fails() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_category_delete_invalid.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (result, _stdout, _stderr) =
        run_category(&pool, &["category", "delete", "not-a-uuid"]).await;
    assert!(result.is_err());
}
