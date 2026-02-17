//! CLI commands and parsing.

mod category;
mod location;
mod product;
mod purchase;
mod review;
mod user;

use std::io::Write;

use clap::{CommandFactory, Parser, Subcommand};
use sqlx::SqlitePool;

use crate::cli::category as category_cli;
use crate::cli::location as location_cli;
use crate::cli::product as product_cli;
use crate::cli::purchase as purchase_cli;
use crate::cli::review as review_cli;
use crate::cli::user as user_cli;

/// Pocket Ratings — product reviews and ratings.
#[derive(Parser)]
#[command(name = "pocketratings")]
#[command(about = "Product reviews and ratings — backend API and CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    User(UserArgs),
    Category(CategoryArgs),
    Location(LocationArgs),
    Product(ProductArgs),
    Purchase(PurchaseArgs),
    Review(ReviewArgs),
}

/// Manage user accounts: register, list, and delete users.
#[derive(clap::Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub command: UserCmd,
}

#[derive(Subcommand)]
pub enum UserCmd {
    Register(RegisterOpts),
    List(ListOpts),
    Delete(DeleteOpts),
}

/// Manage product categories: create, list, show, update, and delete.
#[derive(clap::Args)]
pub struct CategoryArgs {
    #[command(subcommand)]
    pub command: CategoryCmd,
}

#[derive(Subcommand)]
pub enum CategoryCmd {
    /// Create a category.
    Create(CategoryCreateOpts),
    /// List categories.
    List(CategoryListOpts),
    /// Show a single category.
    Show(CategoryShowOpts),
    /// Update a category.
    Update(CategoryUpdateOpts),
    /// Soft-delete a category.
    Delete(CategoryDeleteOpts),
}

/// Manage products: create, list, show, update, and delete (by category/brand/name).
#[derive(clap::Args)]
pub struct ProductArgs {
    #[command(subcommand)]
    pub command: ProductCmd,
}

#[derive(Subcommand)]
pub enum ProductCmd {
    /// Create a product.
    Create(ProductCreateOpts),
    /// List products.
    List(ProductListOpts),
    /// Show a single product.
    Show(ProductShowOpts),
    /// Update a product.
    Update(ProductUpdateOpts),
    /// Soft-delete or remove a product.
    Delete(ProductDeleteOpts),
}

#[derive(clap::Args)]
pub struct ProductCreateOpts {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub brand: String,
    #[arg(long)]
    pub category_id: String,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct ProductListOpts {
    #[arg(long)]
    pub category_id: Option<String>,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
    /// Include soft-deleted products in the list.
    #[arg(long)]
    pub include_deleted: bool,
}

#[derive(clap::Args)]
pub struct ProductShowOpts {
    /// Product UUID to show.
    pub id: String,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct ProductUpdateOpts {
    /// Product UUID to update.
    pub id: String,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub brand: Option<String>,
    #[arg(long)]
    pub category_id: Option<String>,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct ProductDeleteOpts {
    /// Product UUID to delete (soft-delete unless `--force`).
    pub id: String,
    /// Remove the product row from the database instead of soft-deleting.
    #[arg(long)]
    pub force: bool,
}

/// Manage locations (stores): create, list, show, update, and delete.
#[derive(clap::Args)]
pub struct LocationArgs {
    #[command(subcommand)]
    pub command: LocationCmd,
}

#[derive(Subcommand)]
pub enum LocationCmd {
    /// Create a location.
    Create(LocationCreateOpts),
    /// List locations.
    List(LocationListOpts),
    /// Show a single location.
    Show(LocationShowOpts),
    /// Update a location.
    Update(LocationUpdateOpts),
    /// Soft-delete or remove a location.
    Delete(LocationDeleteOpts),
}

#[derive(clap::Args)]
pub struct LocationCreateOpts {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct LocationListOpts {
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
    /// Include soft-deleted locations in the list.
    #[arg(long)]
    pub include_deleted: bool,
}

#[derive(clap::Args)]
pub struct LocationShowOpts {
    /// Location UUID to show.
    pub id: String,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct LocationUpdateOpts {
    /// Location UUID to update.
    pub id: String,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct LocationDeleteOpts {
    /// Location UUID to delete (soft-delete unless `--force`).
    pub id: String,
    /// Remove the location row from the database instead of soft-deleting.
    #[arg(long)]
    pub force: bool,
}

/// Manage reviews: create, list, show, update, and delete.
#[derive(clap::Args)]
pub struct ReviewArgs {
    #[command(subcommand)]
    pub command: ReviewCmd,
}

#[derive(Subcommand)]
pub enum ReviewCmd {
    /// Create a review.
    Create(ReviewCreateOpts),
    /// List reviews.
    List(ReviewListOpts),
    /// Show a single review.
    Show(ReviewShowOpts),
    /// Update a review.
    Update(ReviewUpdateOpts),
    /// Soft-delete or remove a review.
    Delete(ReviewDeleteOpts),
}

#[derive(clap::Args)]
pub struct ReviewCreateOpts {
    #[arg(long)]
    pub product_id: String,
    #[arg(long)]
    pub rating: String,
    #[arg(long)]
    pub user_id: Option<String>,
    #[arg(long)]
    pub email: Option<String>,
    #[arg(long)]
    pub text: Option<String>,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct ReviewListOpts {
    #[arg(long)]
    pub product_id: Option<String>,
    #[arg(long)]
    pub user_id: Option<String>,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
    #[arg(long)]
    pub include_deleted: bool,
}

#[derive(clap::Args)]
pub struct ReviewShowOpts {
    pub id: String,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct ReviewUpdateOpts {
    pub id: String,
    #[arg(long)]
    pub rating: Option<String>,
    #[arg(long)]
    pub text: Option<String>,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct ReviewDeleteOpts {
    pub id: String,
    #[arg(long)]
    pub force: bool,
}

/// Manage purchases: create, list, show, and delete.
#[derive(clap::Args)]
pub struct PurchaseArgs {
    #[command(subcommand)]
    pub command: PurchaseCmd,
}

#[derive(Subcommand)]
pub enum PurchaseCmd {
    /// Create a purchase.
    Create(PurchaseCreateOpts),
    /// List purchases.
    List(PurchaseListOpts),
    /// Show a single purchase.
    Show(PurchaseShowOpts),
    /// Soft-delete or remove a purchase.
    Delete(PurchaseDeleteOpts),
}

#[derive(clap::Args)]
pub struct PurchaseCreateOpts {
    #[arg(long)]
    pub product_id: String,
    #[arg(long)]
    pub location_id: String,
    #[arg(long)]
    pub price: String,
    #[arg(long)]
    pub user_id: Option<String>,
    #[arg(long)]
    pub email: Option<String>,
    #[arg(long, default_value = "1")]
    pub quantity: i32,
    #[arg(long)]
    pub at: Option<String>,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct PurchaseListOpts {
    #[arg(long)]
    pub user_id: Option<String>,
    #[arg(long)]
    pub product_id: Option<String>,
    #[arg(long)]
    pub location_id: Option<String>,
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
    #[arg(long)]
    pub include_deleted: bool,
}

#[derive(clap::Args)]
pub struct PurchaseShowOpts {
    pub id: String,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct PurchaseDeleteOpts {
    pub id: String,
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::Args)]
pub struct RegisterOpts {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub email: String,
    #[arg(long)]
    pub password: String,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct ListOpts {
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
    /// Include soft-deleted users in the list.
    #[arg(long)]
    pub include_deleted: bool,
}

#[derive(clap::Args)]
pub struct DeleteOpts {
    /// User UUID to delete (soft-delete unless `--force`).
    pub id: String,
    /// Remove the user row from the database instead of soft-deleting.
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::Args)]
pub struct CategoryCreateOpts {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub parent_id: Option<String>,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct CategoryListOpts {
    #[arg(long)]
    pub parent_id: Option<String>,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
    /// Include soft-deleted categories in the list.
    #[arg(long)]
    pub include_deleted: bool,
}

#[derive(clap::Args)]
pub struct CategoryShowOpts {
    /// Category UUID to show.
    pub id: String,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct CategoryUpdateOpts {
    /// Category UUID to update.
    pub id: String,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub parent_id: Option<String>,
    #[arg(long, default_value = "human", value_parser = ["human", "json"])]
    pub output: String,
}

#[derive(clap::Args)]
pub struct CategoryDeleteOpts {
    /// Category UUID to delete (soft-delete unless `--force`).
    pub id: String,
    /// Remove the category row from the database instead of soft-deleting.
    #[arg(long)]
    pub force: bool,
}

/// CLI-specific errors for user-facing messages and exit codes.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("email already registered")]
    EmailAlreadyRegistered,

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl From<crate::db::DbError> for CliError {
    fn from(e: crate::db::DbError) -> Self {
        Self::Other(e.into())
    }
}

/// Parse args and dispatch to the appropriate handler.
///
/// When the command is `user register`, `pool` must be `Some`; the caller (e.g. `main`) is responsible for creating the pool and running migrations first.
///
/// # Errors
///
/// Returns [`CliError`] on parse failure, missing pool for `user register`, register handler errors, or I/O when writing help or output.
#[allow(clippy::too_many_lines)]
pub async fn run(
    args: impl Iterator<Item = impl Into<std::ffi::OsString> + Clone>,
    pool: Option<&SqlitePool>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> Result<(), CliError> {
    let cli = Cli::parse_from(args);

    match cli.command {
        Some(Commands::User(user_args)) => match user_args.command {
            UserCmd::Register(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for user register"))
                })?;
                let output_json = opts.output.as_str() == "json";
                user_cli::register(
                    pool,
                    &opts.name,
                    &opts.email,
                    &opts.password,
                    output_json,
                    stdout,
                    stderr,
                )
                .await
            }
            UserCmd::List(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for user list"))
                })?;
                let output_json = opts.output.as_str() == "json";
                user_cli::list(pool, output_json, opts.include_deleted, stdout, stderr).await
            }
            UserCmd::Delete(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for user delete"))
                })?;
                user_cli::delete(pool, &opts.id, opts.force, stdout, stderr).await
            }
        },
        Some(Commands::Category(cat_args)) => match cat_args.command {
            CategoryCmd::Create(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!(
                        "database pool required for category create"
                    ))
                })?;
                let output_json = opts.output.as_str() == "json";
                category_cli::create(
                    pool,
                    &opts.name,
                    opts.parent_id.as_deref(),
                    output_json,
                    stdout,
                    stderr,
                )
                .await
            }
            CategoryCmd::List(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for category list"))
                })?;
                let output_json = opts.output.as_str() == "json";
                category_cli::list(
                    pool,
                    opts.parent_id.as_deref(),
                    output_json,
                    opts.include_deleted,
                    stdout,
                    stderr,
                )
                .await
            }
            CategoryCmd::Show(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for category show"))
                })?;
                let output_json = opts.output.as_str() == "json";
                category_cli::show(pool, &opts.id, output_json, stdout, stderr).await
            }
            CategoryCmd::Update(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!(
                        "database pool required for category update"
                    ))
                })?;
                let output_json = opts.output.as_str() == "json";
                category_cli::update(
                    pool,
                    &opts.id,
                    opts.name.as_deref(),
                    opts.parent_id.as_deref(),
                    output_json,
                    stdout,
                    stderr,
                )
                .await
            }
            CategoryCmd::Delete(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!(
                        "database pool required for category delete"
                    ))
                })?;
                category_cli::delete(pool, &opts.id, opts.force, stdout, stderr).await
            }
        },
        Some(Commands::Location(loc_args)) => match loc_args.command {
            LocationCmd::Create(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!(
                        "database pool required for location create"
                    ))
                })?;
                let output_json = opts.output.as_str() == "json";
                location_cli::create(pool, &opts.name, output_json, stdout, stderr).await
            }
            LocationCmd::List(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for location list"))
                })?;
                let output_json = opts.output.as_str() == "json";
                location_cli::list(pool, output_json, opts.include_deleted, stdout, stderr).await
            }
            LocationCmd::Show(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for location show"))
                })?;
                let output_json = opts.output.as_str() == "json";
                location_cli::show(pool, &opts.id, output_json, stdout, stderr).await
            }
            LocationCmd::Update(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!(
                        "database pool required for location update"
                    ))
                })?;
                let output_json = opts.output.as_str() == "json";
                location_cli::update(
                    pool,
                    &opts.id,
                    opts.name.as_deref(),
                    output_json,
                    stdout,
                    stderr,
                )
                .await
            }
            LocationCmd::Delete(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!(
                        "database pool required for location delete"
                    ))
                })?;
                location_cli::delete(pool, &opts.id, opts.force, stdout, stderr).await
            }
        },
        Some(Commands::Product(prod_args)) => match prod_args.command {
            ProductCmd::Create(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for product create"))
                })?;
                let output_json = opts.output.as_str() == "json";
                product_cli::create(
                    pool,
                    &opts.name,
                    &opts.brand,
                    &opts.category_id,
                    output_json,
                    stdout,
                    stderr,
                )
                .await
            }
            ProductCmd::List(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for product list"))
                })?;
                let output_json = opts.output.as_str() == "json";
                product_cli::list(
                    pool,
                    opts.category_id.as_deref(),
                    output_json,
                    opts.include_deleted,
                    stdout,
                    stderr,
                )
                .await
            }
            ProductCmd::Show(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for product show"))
                })?;
                let output_json = opts.output.as_str() == "json";
                product_cli::show(pool, &opts.id, output_json, stdout, stderr).await
            }
            ProductCmd::Update(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for product update"))
                })?;
                let output_json = opts.output.as_str() == "json";
                product_cli::update(
                    pool,
                    &opts.id,
                    opts.name.as_deref(),
                    opts.brand.as_deref(),
                    opts.category_id.as_deref(),
                    output_json,
                    stdout,
                    stderr,
                )
                .await
            }
            ProductCmd::Delete(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for product delete"))
                })?;
                product_cli::delete(pool, &opts.id, opts.force, stdout, stderr).await
            }
        },
        Some(Commands::Purchase(pur_args)) => match pur_args.command {
            PurchaseCmd::Create(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!(
                        "database pool required for purchase create"
                    ))
                })?;
                let output_json = opts.output.as_str() == "json";
                purchase_cli::create(
                    pool,
                    &opts.product_id,
                    &opts.location_id,
                    &opts.price,
                    opts.user_id.as_deref(),
                    opts.email.as_deref(),
                    opts.quantity,
                    opts.at.as_deref(),
                    output_json,
                    stdout,
                    stderr,
                )
                .await
            }
            PurchaseCmd::List(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for purchase list"))
                })?;
                let output_json = opts.output.as_str() == "json";
                purchase_cli::list(
                    pool,
                    opts.user_id.as_deref(),
                    opts.product_id.as_deref(),
                    opts.location_id.as_deref(),
                    opts.from.as_deref(),
                    opts.to.as_deref(),
                    output_json,
                    opts.include_deleted,
                    stdout,
                    stderr,
                )
                .await
            }
            PurchaseCmd::Show(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for purchase show"))
                })?;
                let output_json = opts.output.as_str() == "json";
                purchase_cli::show(pool, &opts.id, output_json, stdout, stderr).await
            }
            PurchaseCmd::Delete(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!(
                        "database pool required for purchase delete"
                    ))
                })?;
                purchase_cli::delete(pool, &opts.id, opts.force, stdout, stderr).await
            }
        },
        Some(Commands::Review(rev_args)) => match rev_args.command {
            ReviewCmd::Create(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for review create"))
                })?;
                let output_json = opts.output.as_str() == "json";
                review_cli::create(
                    pool,
                    &opts.product_id,
                    &opts.rating,
                    opts.user_id.as_deref(),
                    opts.email.as_deref(),
                    opts.text.as_deref(),
                    output_json,
                    stdout,
                    stderr,
                )
                .await
            }
            ReviewCmd::List(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for review list"))
                })?;
                let output_json = opts.output.as_str() == "json";
                review_cli::list(
                    pool,
                    opts.product_id.as_deref(),
                    opts.user_id.as_deref(),
                    output_json,
                    opts.include_deleted,
                    stdout,
                    stderr,
                )
                .await
            }
            ReviewCmd::Show(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for review show"))
                })?;
                let output_json = opts.output.as_str() == "json";
                review_cli::show(pool, &opts.id, output_json, stdout, stderr).await
            }
            ReviewCmd::Update(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for review update"))
                })?;
                let output_json = opts.output.as_str() == "json";
                review_cli::update(
                    pool,
                    &opts.id,
                    opts.rating.as_deref(),
                    opts.text.as_deref(),
                    output_json,
                    stdout,
                    stderr,
                )
                .await
            }
            ReviewCmd::Delete(opts) => {
                let pool = pool.ok_or_else(|| {
                    CliError::Other(anyhow::anyhow!("database pool required for review delete"))
                })?;
                review_cli::delete(pool, &opts.id, opts.force, stdout, stderr).await
            }
        },
        None => {
            let mut out = Vec::new();
            Cli::command()
                .write_help(&mut out)
                .map_err(|e: std::io::Error| CliError::Other(e.into()))?;
            stdout
                .write_all(&out)
                .map_err(|e| CliError::Other(e.into()))?;
            Ok(())
        }
    }
}
