//! Integration tests for `pocketratings purchase` CLI.

use std::io::Cursor;

use pocketratings::cli;
use pocketratings::db;

async fn run_purchase(
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
#[allow(clippy::too_many_lines)]
async fn purchase_create_and_show_roundtrip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_purchase_create_show.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (reg_res, _, _) = run_purchase(
        &pool,
        &[
            "user",
            "register",
            "--name",
            "Alice",
            "--email",
            "alice@example.com",
            "--password",
            "secret",
            "--output",
            "json",
        ],
    )
    .await;
    assert!(reg_res.is_ok());

    let (_, cat_stdout, _) = run_purchase(
        &pool,
        &["category", "create", "--name", "C", "--output", "json"],
    )
    .await;
    let cat_json: serde_json::Value =
        serde_json::from_str(cat_stdout.lines().next().expect("line")).expect("json");
    let cat_id = cat_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (_, prod_stdout, _) = run_purchase(
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
    let prod_json: serde_json::Value =
        serde_json::from_str(prod_stdout.lines().next().expect("line")).expect("json");
    let product_id = prod_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (_, loc_stdout, _) = run_purchase(
        &pool,
        &["location", "create", "--name", "Store", "--output", "json"],
    )
    .await;
    let loc_json: serde_json::Value =
        serde_json::from_str(loc_stdout.lines().next().expect("line")).expect("json");
    let location_id = loc_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (create_result, create_stdout, create_stderr) = run_purchase(
        &pool,
        &[
            "purchase",
            "create",
            "--product-id",
            &product_id,
            "--location-id",
            &location_id,
            "--price",
            "9.99",
            "--email",
            "alice@example.com",
            "--quantity",
            "2",
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
    assert_eq!(json.get("quantity"), Some(&serde_json::Value::from(2)));
    assert_eq!(json.get("price").and_then(|v| v.as_str()), Some("9.99"));

    let (show_result, show_stdout, show_stderr) =
        run_purchase(&pool, &["purchase", "show", id, "--output", "json"]).await;
    assert!(show_result.is_ok(), "stderr: {show_stderr}");
    let line = show_stdout.lines().next().expect("show line");
    let show_json: serde_json::Value = serde_json::from_str(line).expect("json");
    assert_eq!(show_json.get("quantity"), Some(&serde_json::Value::from(2)));
    assert_eq!(
        show_json.get("price").and_then(|v| v.as_str()),
        Some("9.99")
    );
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn purchase_list_include_deleted() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_purchase_list.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (reg_res, _, _) = run_purchase(
        &pool,
        &[
            "user",
            "register",
            "--name",
            "U",
            "--email",
            "u@ex.com",
            "--password",
            "p",
            "--output",
            "json",
        ],
    )
    .await;
    assert!(reg_res.is_ok());
    let (_, cat_stdout, _) = run_purchase(
        &pool,
        &["category", "create", "--name", "C", "--output", "json"],
    )
    .await;
    let cat_json: serde_json::Value =
        serde_json::from_str(cat_stdout.lines().next().expect("line")).expect("json");
    let cat_id = cat_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();
    let (_, prod_stdout, _) = run_purchase(
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
    let prod_json: serde_json::Value =
        serde_json::from_str(prod_stdout.lines().next().expect("line")).expect("json");
    let product_id = prod_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();
    let (_, loc_stdout, _) = run_purchase(
        &pool,
        &["location", "create", "--name", "L", "--output", "json"],
    )
    .await;
    let loc_json: serde_json::Value =
        serde_json::from_str(loc_stdout.lines().next().expect("line")).expect("json");
    let location_id = loc_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (_, create_stdout, _) = run_purchase(
        &pool,
        &[
            "purchase",
            "create",
            "--product-id",
            &product_id,
            "--location-id",
            &location_id,
            "--price",
            "1",
            "--email",
            "u@ex.com",
            "--output",
            "json",
        ],
    )
    .await;
    let create_json: serde_json::Value =
        serde_json::from_str(create_stdout.lines().next().expect("line")).expect("json");
    let id = create_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (list_ok, list_stdout, _) =
        run_purchase(&pool, &["purchase", "list", "--output", "json"]).await;
    assert!(list_ok.is_ok());
    let arr: Vec<serde_json::Value> =
        serde_json::from_str(list_stdout.lines().next().expect("line")).expect("json");
    assert_eq!(arr.len(), 1);

    let (del_res, _, _) = run_purchase(&pool, &["purchase", "delete", &id]).await;
    assert!(del_res.is_ok());

    let (list_after, list_stdout_after, _) =
        run_purchase(&pool, &["purchase", "list", "--output", "json"]).await;
    assert!(list_after.is_ok());
    let arr_after: Vec<serde_json::Value> =
        serde_json::from_str(list_stdout_after.lines().next().expect("line")).expect("json");
    assert_eq!(arr_after.len(), 0);

    let (list_deleted, list_stdout_deleted, _) = run_purchase(
        &pool,
        &["purchase", "list", "--include-deleted", "--output", "json"],
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
#[allow(clippy::too_many_lines)]
async fn purchase_delete_soft_deletes() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_purchase_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (reg_res, _, _) = run_purchase(
        &pool,
        &[
            "user",
            "register",
            "--name",
            "U",
            "--email",
            "u@ex.com",
            "--password",
            "p",
            "--output",
            "json",
        ],
    )
    .await;
    assert!(reg_res.is_ok());
    let (_, cat_stdout, _) = run_purchase(
        &pool,
        &["category", "create", "--name", "C", "--output", "json"],
    )
    .await;
    let cat_json: serde_json::Value =
        serde_json::from_str(cat_stdout.lines().next().expect("line")).expect("json");
    let cat_id = cat_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();
    let (_, prod_stdout, _) = run_purchase(
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
    let prod_json: serde_json::Value =
        serde_json::from_str(prod_stdout.lines().next().expect("line")).expect("json");
    let product_id = prod_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();
    let (_, loc_stdout, _) = run_purchase(
        &pool,
        &["location", "create", "--name", "L", "--output", "json"],
    )
    .await;
    let loc_json: serde_json::Value =
        serde_json::from_str(loc_stdout.lines().next().expect("line")).expect("json");
    let location_id = loc_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (_, create_stdout, _) = run_purchase(
        &pool,
        &[
            "purchase",
            "create",
            "--product-id",
            &product_id,
            "--location-id",
            &location_id,
            "--price",
            "5",
            "--email",
            "u@ex.com",
            "--output",
            "json",
        ],
    )
    .await;
    let create_json: serde_json::Value =
        serde_json::from_str(create_stdout.lines().next().expect("line")).expect("json");
    let id = create_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();
    let id_uuid = id.parse().expect("uuid");

    let (del_result, _, del_stderr) = run_purchase(&pool, &["purchase", "delete", &id]).await;
    assert!(del_result.is_ok(), "stderr: {del_stderr}");

    let by_id = db::purchase::get_by_id(&pool, id_uuid, false)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());

    let with_deleted = db::purchase::list(&pool, None, None, None, None, None, true)
        .await
        .expect("list");
    assert_eq!(with_deleted.len(), 1);
    assert!(!with_deleted[0].is_active());
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn purchase_delete_force_removes_row() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_purchase_delete_force.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (reg_res, _, _) = run_purchase(
        &pool,
        &[
            "user",
            "register",
            "--name",
            "U",
            "--email",
            "u@ex.com",
            "--password",
            "p",
            "--output",
            "json",
        ],
    )
    .await;
    assert!(reg_res.is_ok());
    let (_, cat_stdout, _) = run_purchase(
        &pool,
        &["category", "create", "--name", "C", "--output", "json"],
    )
    .await;
    let cat_json: serde_json::Value =
        serde_json::from_str(cat_stdout.lines().next().expect("line")).expect("json");
    let cat_id = cat_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();
    let (_, prod_stdout, _) = run_purchase(
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
    let prod_json: serde_json::Value =
        serde_json::from_str(prod_stdout.lines().next().expect("line")).expect("json");
    let product_id = prod_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();
    let (_, loc_stdout, _) = run_purchase(
        &pool,
        &["location", "create", "--name", "L", "--output", "json"],
    )
    .await;
    let loc_json: serde_json::Value =
        serde_json::from_str(loc_stdout.lines().next().expect("line")).expect("json");
    let location_id = loc_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();

    let (_, create_stdout, _) = run_purchase(
        &pool,
        &[
            "purchase",
            "create",
            "--product-id",
            &product_id,
            "--location-id",
            &location_id,
            "--price",
            "3",
            "--email",
            "u@ex.com",
            "--output",
            "json",
        ],
    )
    .await;
    let create_json: serde_json::Value =
        serde_json::from_str(create_stdout.lines().next().expect("line")).expect("json");
    let id = create_json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id")
        .to_string();
    let id_uuid = id.parse().expect("uuid");

    let (del_result, _, del_stderr) =
        run_purchase(&pool, &["purchase", "delete", &id, "--force"]).await;
    assert!(del_result.is_ok(), "stderr: {del_stderr}");

    let by_id = db::purchase::get_by_id(&pool, id_uuid, false)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());
    let with_deleted = db::purchase::list(&pool, None, None, None, None, None, true)
        .await
        .expect("list");
    assert!(with_deleted.is_empty());
}

#[tokio::test]
async fn purchase_show_non_existent_fails() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("cli_purchase_show_missing.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let non_existent = uuid::Uuid::new_v4().to_string();
    let (res, _stdout, _stderr) = run_purchase(&pool, &["purchase", "show", &non_existent]).await;
    assert!(res.is_err());
}

/// Minimal setup: one user, category, product, location. Caller must keep `_dir` alive so the DB
/// path remains valid. Returns (`_dir`, pool, `user_email`, `product_id`, `location_id`).
async fn setup_purchase_prereqs(
    db_name: &str,
) -> (tempfile::TempDir, sqlx::SqlitePool, String, String, String) {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join(db_name);
    let db_path_str = db_path.to_str().expect("path UTF-8");
    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (reg_res, _, _) = run_purchase(
        &pool,
        &[
            "user",
            "register",
            "--name",
            "U",
            "--email",
            "u@ex.com",
            "--password",
            "p",
            "--output",
            "json",
        ],
    )
    .await;
    assert!(reg_res.is_ok());

    let (_, cat_stdout, _) = run_purchase(
        &pool,
        &["category", "create", "--name", "C", "--output", "json"],
    )
    .await;
    let cat_id: String =
        serde_json::from_str::<serde_json::Value>(cat_stdout.lines().next().expect("line"))
            .expect("json")
            .get("id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .expect("id");

    let (_, prod_stdout, _) = run_purchase(
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
    let product_id: String =
        serde_json::from_str::<serde_json::Value>(prod_stdout.lines().next().expect("line"))
            .expect("json")
            .get("id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .expect("id");

    let (_, loc_stdout, _) = run_purchase(
        &pool,
        &["location", "create", "--name", "L", "--output", "json"],
    )
    .await;
    let location_id: String =
        serde_json::from_str::<serde_json::Value>(loc_stdout.lines().next().expect("line"))
            .expect("json")
            .get("id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .expect("id");

    (dir, pool, "u@ex.com".to_string(), product_id, location_id)
}

#[tokio::test]
async fn purchase_create_without_json_outputs_human_readable() {
    let (_dir, pool, email, product_id, location_id) =
        setup_purchase_prereqs("cli_purchase_create_human.db").await;

    let (result, stdout, stderr) = run_purchase(
        &pool,
        &[
            "purchase",
            "create",
            "--product-id",
            &product_id,
            "--location-id",
            &location_id,
            "--price",
            "12.50",
            "--email",
            &email,
            "--quantity",
            "1",
        ],
    )
    .await;
    assert!(result.is_ok(), "stderr: {stderr}");
    assert!(
        stdout.contains("Purchase created:") && stdout.contains("12.50"),
        "stdout: {stdout}"
    );
}

#[tokio::test]
async fn purchase_create_fails_without_user_or_email() {
    let (_dir, pool, _email, product_id, location_id) =
        setup_purchase_prereqs("cli_purchase_create_no_user.db").await;

    let (result, _stdout, _stderr) = run_purchase(
        &pool,
        &[
            "purchase",
            "create",
            "--product-id",
            &product_id,
            "--location-id",
            &location_id,
            "--price",
            "1",
            "--quantity",
            "1",
        ],
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn purchase_create_fails_with_both_user_id_and_email() {
    let (_dir, pool, email, product_id, location_id) =
        setup_purchase_prereqs("cli_purchase_create_both_user.db").await;
    let (_, user_stdout, _) = run_purchase(&pool, &["user", "list", "--output", "json"]).await;
    let user_id: String =
        serde_json::from_str::<serde_json::Value>(user_stdout.lines().next().expect("line"))
            .expect("json")
            .as_array()
            .and_then(|a| a.first())
            .and_then(|o| o.get("id").and_then(|v| v.as_str()))
            .map(String::from)
            .expect("user id");

    let (result, _stdout, _stderr) = run_purchase(
        &pool,
        &[
            "purchase",
            "create",
            "--product-id",
            &product_id,
            "--location-id",
            &location_id,
            "--price",
            "1",
            "--user-id",
            &user_id,
            "--email",
            &email,
            "--quantity",
            "1",
        ],
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn purchase_create_fails_with_invalid_price() {
    let (_dir, pool, email, product_id, location_id) =
        setup_purchase_prereqs("cli_purchase_create_bad_price.db").await;

    let (result, _stdout, _stderr) = run_purchase(
        &pool,
        &[
            "purchase",
            "create",
            "--product-id",
            &product_id,
            "--location-id",
            &location_id,
            "--price",
            "not-a-number",
            "--email",
            &email,
            "--quantity",
            "1",
        ],
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn purchase_list_human_readable() {
    let (_dir, pool, email, product_id, location_id) =
        setup_purchase_prereqs("cli_purchase_list_human.db").await;

    let (create_res, _, _) = run_purchase(
        &pool,
        &[
            "purchase",
            "create",
            "--product-id",
            &product_id,
            "--location-id",
            &location_id,
            "--price",
            "7",
            "--email",
            &email,
            "--output",
            "json",
        ],
    )
    .await;
    assert!(create_res.is_ok());

    let (list_res, list_stdout, list_stderr) = run_purchase(&pool, &["purchase", "list"]).await;
    assert!(list_res.is_ok(), "stderr: {list_stderr}");
    assert!(!list_stdout.trim().is_empty(), "stdout: {list_stdout}");
}

#[tokio::test]
async fn purchase_show_human_readable_and_invalid_id() {
    let (_dir, pool, email, product_id, location_id) =
        setup_purchase_prereqs("cli_purchase_show_human.db").await;

    let (create_res, create_stdout, _) = run_purchase(
        &pool,
        &[
            "purchase",
            "create",
            "--product-id",
            &product_id,
            "--location-id",
            &location_id,
            "--price",
            "99",
            "--email",
            &email,
            "--output",
            "json",
        ],
    )
    .await;
    assert!(create_res.is_ok());
    let id: String =
        serde_json::from_str::<serde_json::Value>(create_stdout.lines().next().expect("line"))
            .expect("json")
            .get("id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .expect("id");

    let (show_res, show_stdout, show_stderr) =
        run_purchase(&pool, &["purchase", "show", &id]).await;
    assert!(show_res.is_ok(), "stderr: {show_stderr}");
    assert!(
        show_stdout.contains("Purchase:") && show_stdout.contains("99"),
        "stdout: {show_stdout}"
    );

    let (show_bad_res, _stdout, _stderr) =
        run_purchase(&pool, &["purchase", "show", "not-a-uuid"]).await;
    assert!(show_bad_res.is_err());
}

#[tokio::test]
async fn purchase_delete_invalid_id_fails() {
    let (_dir, pool, _, _, _) = setup_purchase_prereqs("cli_purchase_delete_invalid.db").await;

    let (result, _stdout, _stderr) =
        run_purchase(&pool, &["purchase", "delete", "not-a-uuid"]).await;
    assert!(result.is_err());
}
