//! Integration tests for purchase DB functions.

use pocketratings::db;
use pocketratings::domain::category::Category;
use pocketratings::domain::location::Location;
use pocketratings::domain::product::Product;
use pocketratings::domain::purchase::Purchase;
use pocketratings::domain::user::User;
use rust_decimal::Decimal;
use uuid::Uuid;

async fn insert_user_product_location(pool: &sqlx::SqlitePool) -> (Uuid, Uuid, Uuid) {
    let now = 1_000_i64;

    let user_id = Uuid::new_v4();
    let user = User::new(
        user_id,
        "Test User".to_string(),
        "u@example.com".to_string(),
        "hash".to_string(),
        now,
        now,
        None,
    )
    .expect("valid user");
    db::user::insert(pool, &user).await.expect("insert user");

    let cat_id = Uuid::new_v4();
    let cat = Category::new(cat_id, None, "C".to_string(), now, now, None).expect("valid");
    db::category::insert(pool, &cat)
        .await
        .expect("insert category");

    let product_id = Uuid::new_v4();
    let product = Product::new(
        product_id,
        cat_id,
        "Brand".to_string(),
        "Product".to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::product::insert(pool, &product)
        .await
        .expect("insert product");

    let location_id = Uuid::new_v4();
    let location = Location::new(location_id, "Store".to_string(), None).expect("valid");
    db::location::insert(pool, &location)
        .await
        .expect("insert location");

    (user_id, product_id, location_id)
}

async fn insert_product(
    pool: &sqlx::SqlitePool,
    id: Uuid,
    category_id: Uuid,
    brand: &str,
    name: &str,
) {
    let now = 1_000_i64;
    let p = Product::new(
        id,
        category_id,
        brand.to_string(),
        name.to_string(),
        now,
        now,
        None,
    )
    .expect("valid");
    db::product::insert(pool, &p).await.expect("insert product");
}

async fn insert_location(pool: &sqlx::SqlitePool, id: Uuid, name: &str) {
    let loc = Location::new(id, name.to_string(), None).expect("valid");
    db::location::insert(pool, &loc)
        .await
        .expect("insert location");
}

/// Two products, two locations, one user; four purchases at ts 1000, 2000, 3000, 4000 (numbered 1–4).
/// Purchase 2 (ts 2000, product1, location2) is soft-deleted. List order is DESC by `purchased_at` (4, 3, 2, 1).
/// Returns (`user_id`, `product1_id`, `product2_id`, `location1_id`, `location2_id`, `purchase_ids`).
async fn setup_list_with_relations_data(
    pool: &sqlx::SqlitePool,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid, [Uuid; 4]) {
    let now = 1_000_i64;
    let user_id = Uuid::new_v4();
    let user = User::new(
        user_id,
        "Test User".to_string(),
        "u@example.com".to_string(),
        "hash".to_string(),
        now,
        now,
        None,
    )
    .expect("valid user");
    db::user::insert(pool, &user).await.expect("insert user");

    let cat_id = Uuid::new_v4();
    let cat = Category::new(cat_id, None, "C".to_string(), now, now, None).expect("valid");
    db::category::insert(pool, &cat)
        .await
        .expect("insert category");

    let product1_id = Uuid::new_v4();
    let product2_id = Uuid::new_v4();
    insert_product(pool, product1_id, cat_id, "Brand1", "Product1").await;
    insert_product(pool, product2_id, cat_id, "Brand2", "Product2").await;

    let location1_id = Uuid::new_v4();
    let location2_id = Uuid::new_v4();
    insert_location(pool, location1_id, "Store1").await;
    insert_location(pool, location2_id, "Store2").await;

    let price: Decimal = "1.00".parse().expect("decimal");
    let p1 = Purchase::new(
        Uuid::new_v4(),
        user_id,
        product1_id,
        location1_id,
        1,
        price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(pool, &p1).await.expect("insert");
    let p2 = Purchase::new(
        Uuid::new_v4(),
        user_id,
        product1_id,
        location2_id,
        1,
        price,
        2_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(pool, &p2).await.expect("insert");
    let p3 = Purchase::new(
        Uuid::new_v4(),
        user_id,
        product2_id,
        location1_id,
        1,
        price,
        3_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(pool, &p3).await.expect("insert");
    let p4 = Purchase::new(
        Uuid::new_v4(),
        user_id,
        product2_id,
        location2_id,
        1,
        price,
        4_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(pool, &p4).await.expect("insert");

    db::purchase::soft_delete(pool, p2.id())
        .await
        .expect("soft_delete");

    (
        user_id,
        product1_id,
        product2_id,
        location1_id,
        location2_id,
        [p1.id(), p2.id(), p3.id(), p4.id()],
    )
}

#[tokio::test]
async fn purchase_insert_and_get_by_id_roundtrip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_insert.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id, location_id) = insert_user_product_location(&pool).await;

    let purchase_id = Uuid::new_v4();
    let price: Decimal = "12.50".parse().expect("decimal");
    let purchase = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        location_id,
        2,
        price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert");

    let loaded = db::purchase::get_by_id(&pool, purchase_id)
        .await
        .expect("get_by_id")
        .expect("purchase should exist");
    assert_eq!(loaded.id(), purchase_id);
    assert_eq!(loaded.quantity(), 2);
    assert_eq!(loaded.price(), price);
}

#[tokio::test]
async fn purchase_list_with_filters_and_include_deleted() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_list.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id, location_id) = insert_user_product_location(&pool).await;

    let price: Decimal = "1".parse().expect("decimal");
    let p1 = Purchase::new(
        Uuid::new_v4(),
        user_id,
        product_id,
        location_id,
        1,
        price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &p1).await.expect("insert");

    let all = db::purchase::list(&pool, None, None, None, None, None, false)
        .await
        .expect("list");
    assert_eq!(all.len(), 1);

    let by_user = db::purchase::list(&pool, Some(user_id), None, None, None, None, false)
        .await
        .expect("list");
    assert_eq!(by_user.len(), 1);

    db::purchase::soft_delete(&pool, p1.id())
        .await
        .expect("soft_delete");

    let active = db::purchase::list(&pool, None, None, None, None, None, false)
        .await
        .expect("list");
    assert_eq!(active.len(), 0);

    let with_deleted = db::purchase::list(&pool, None, None, None, None, None, true)
        .await
        .expect("list");
    assert_eq!(with_deleted.len(), 1);
    assert!(!with_deleted[0].is_active());
}

#[tokio::test]
async fn purchase_soft_delete_sets_deleted_at_and_excludes_from_get_by_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_soft_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id, location_id) = insert_user_product_location(&pool).await;

    let purchase_id = Uuid::new_v4();
    let price: Decimal = "5".parse().expect("decimal");
    let purchase = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        location_id,
        1,
        price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert");

    db::purchase::soft_delete(&pool, purchase_id)
        .await
        .expect("soft_delete");

    let by_id = db::purchase::get_by_id(&pool, purchase_id)
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
async fn purchase_hard_delete_removes_row() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_hard_delete.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id, location_id) = insert_user_product_location(&pool).await;

    let purchase_id = Uuid::new_v4();
    let price: Decimal = "3.99".parse().expect("decimal");
    let purchase = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        location_id,
        1,
        price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert");

    db::purchase::hard_delete(&pool, purchase_id)
        .await
        .expect("hard_delete");

    let by_id = db::purchase::get_by_id(&pool, purchase_id)
        .await
        .expect("get_by_id");
    assert!(by_id.is_none());

    let with_deleted = db::purchase::list(&pool, None, None, None, None, None, true)
        .await
        .expect("list");
    assert!(with_deleted.is_empty());
}

#[tokio::test]
async fn purchase_update_changes_quantity_and_price() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_update.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id, location_id) = insert_user_product_location(&pool).await;

    let purchase_id = Uuid::new_v4();
    let price: Decimal = "2.00".parse().expect("decimal");
    let purchase = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        location_id,
        1,
        price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert");

    // Update quantity and price.
    let updated_price: Decimal = "3.50".parse().expect("decimal");
    let updated = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        location_id,
        3,
        updated_price,
        1_000,
        None,
    )
    .expect("valid");
    db::purchase::update(&pool, &updated).await.expect("update");

    let loaded = db::purchase::get_by_id(&pool, purchase_id)
        .await
        .expect("get_by_id")
        .expect("purchase should exist");
    assert_eq!(loaded.quantity(), 3);
    assert_eq!(loaded.price(), updated_price);
}

#[tokio::test]
async fn purchase_get_by_id_with_relations_returns_purchase_and_relation_names() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_get_with_relations.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let (user_id, product_id, location_id) = insert_user_product_location(&pool).await;

    let purchase_id = Uuid::new_v4();
    let price: Decimal = "2.99".parse().expect("decimal");
    let purchase = Purchase::new(
        purchase_id,
        user_id,
        product_id,
        location_id,
        1,
        price,
        1_700_000_000,
        None,
    )
    .expect("valid");
    db::purchase::insert(&pool, &purchase)
        .await
        .expect("insert");

    let row = db::purchase::get_by_id_with_relations(&pool, purchase_id)
        .await
        .expect("get_by_id_with_relations")
        .expect("purchase should exist");
    assert_eq!(row.id, purchase_id);
    assert_eq!(row.user_id, user_id);
    assert_eq!(row.product_id, product_id);
    assert_eq!(row.location_id, location_id);
    assert_eq!(row.quantity, 1);
    assert_eq!(row.price, "2.99");
    assert_eq!(row.purchased_at, 1_700_000_000);
    assert!(row.deleted_at.is_none());
    assert_eq!(row.user_name, "Test User");
    assert_eq!(row.product_brand, "Brand");
    assert_eq!(row.product_name, "Product");
    assert_eq!(row.location_name, "Store");
}

#[tokio::test]
async fn purchase_get_by_id_with_relations_returns_none_for_nonexistent() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_get_with_relations_none.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");

    let missing_id = Uuid::new_v4();
    let result = db::purchase::get_by_id_with_relations(&pool, missing_id)
        .await
        .expect("get_by_id_with_relations");
    assert!(result.is_none());
}

struct ListWithRelationsCase {
    name: &'static str,
    user_id: Option<Uuid>,
    product_id: Option<Uuid>,
    location_id: Option<Uuid>,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    include_deleted: bool,
    /// Expected purchase numbers (1–4) in list order (DESC by `purchased_at`).
    expected: Vec<u8>,
}

#[allow(clippy::too_many_lines)]
fn build_list_with_relations_cases(
    user_id: Uuid,
    product1_id: Uuid,
    product2_id: Uuid,
    location1_id: Uuid,
    location2_id: Uuid,
) -> Vec<ListWithRelationsCase> {
    vec![
        ListWithRelationsCase {
            name: "no filters, active only",
            user_id: None,
            product_id: None,
            location_id: None,
            from_ts: None,
            to_ts: None,
            include_deleted: false,
            expected: vec![4, 3, 1],
        },
        ListWithRelationsCase {
            name: "no filters, include deleted",
            user_id: None,
            product_id: None,
            location_id: None,
            from_ts: None,
            to_ts: None,
            include_deleted: true,
            expected: vec![4, 3, 2, 1],
        },
        ListWithRelationsCase {
            name: "filter by user_id",
            user_id: Some(user_id),
            product_id: None,
            location_id: None,
            from_ts: None,
            to_ts: None,
            include_deleted: false,
            expected: vec![4, 3, 1],
        },
        ListWithRelationsCase {
            name: "filter by product_id (product1, one active)",
            user_id: None,
            product_id: Some(product1_id),
            location_id: None,
            from_ts: None,
            to_ts: None,
            include_deleted: false,
            expected: vec![1],
        },
        ListWithRelationsCase {
            name: "filter by product_id (product1, include deleted)",
            user_id: None,
            product_id: Some(product1_id),
            location_id: None,
            from_ts: None,
            to_ts: None,
            include_deleted: true,
            expected: vec![2, 1],
        },
        ListWithRelationsCase {
            name: "filter by product_id (product2)",
            user_id: None,
            product_id: Some(product2_id),
            location_id: None,
            from_ts: None,
            to_ts: None,
            include_deleted: false,
            expected: vec![4, 3],
        },
        ListWithRelationsCase {
            name: "filter by location_id (location1)",
            user_id: None,
            product_id: None,
            location_id: Some(location1_id),
            from_ts: None,
            to_ts: None,
            include_deleted: false,
            expected: vec![3, 1],
        },
        ListWithRelationsCase {
            name: "filter by location_id (location2, one active)",
            user_id: None,
            product_id: None,
            location_id: Some(location2_id),
            from_ts: None,
            to_ts: None,
            include_deleted: false,
            expected: vec![4],
        },
        ListWithRelationsCase {
            name: "from_ts 1500 (purchased_at >= 1500)",
            user_id: None,
            product_id: None,
            location_id: None,
            from_ts: Some(1500),
            to_ts: None,
            include_deleted: false,
            expected: vec![4, 3],
        },
        ListWithRelationsCase {
            name: "to_ts 2500 (purchased_at <= 2500)",
            user_id: None,
            product_id: None,
            location_id: None,
            from_ts: None,
            to_ts: Some(2500),
            include_deleted: false,
            expected: vec![1],
        },
        ListWithRelationsCase {
            name: "from_ts 1500 and to_ts 3500",
            user_id: None,
            product_id: None,
            location_id: None,
            from_ts: Some(1500),
            to_ts: Some(3500),
            include_deleted: false,
            expected: vec![3],
        },
        ListWithRelationsCase {
            name: "user_id + product_id",
            user_id: Some(user_id),
            product_id: Some(product2_id),
            location_id: None,
            from_ts: None,
            to_ts: None,
            include_deleted: false,
            expected: vec![4, 3],
        },
        ListWithRelationsCase {
            name: "product_id + from_ts",
            user_id: None,
            product_id: Some(product2_id),
            location_id: None,
            from_ts: Some(2500),
            to_ts: None,
            include_deleted: false,
            expected: vec![4, 3],
        },
        ListWithRelationsCase {
            name: "filter by nonexistent product_id returns empty",
            user_id: None,
            product_id: Some(Uuid::new_v4()),
            location_id: None,
            from_ts: None,
            to_ts: None,
            include_deleted: false,
            expected: vec![],
        },
    ]
}

#[tokio::test]
async fn purchase_list_with_relations_covers_all_filter_params() {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("purchase_list_with_relations.db");
    let db_path_str = db_path.to_str().expect("path UTF-8");

    let pool = db::create_pool(db_path_str).await.expect("create pool");
    db::run_migrations(&pool).await.expect("migrations");
    let (user_id, product1_id, product2_id, location1_id, location2_id, purchase_ids) =
        setup_list_with_relations_data(&pool).await;

    let cases = build_list_with_relations_cases(
        user_id,
        product1_id,
        product2_id,
        location1_id,
        location2_id,
    );

    for c in &cases {
        let list = db::purchase::list_with_relations(
            &pool,
            c.user_id,
            c.product_id,
            c.location_id,
            c.from_ts,
            c.to_ts,
            c.include_deleted,
        )
        .await
        .expect("list_with_relations");
        assert_eq!(list.len(), c.expected.len(), "case: {}", c.name);
        for (i, &num) in c.expected.iter().enumerate() {
            let expected_id = purchase_ids[(num - 1) as usize];
            assert_eq!(
                list[i].id, expected_id,
                "case: {}, position {}: expected purchase {}",
                c.name, i, num
            );
        }
    }
}
