use serde_json::json;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row,
};
use std::env;
use std::process;
use std::str::FromStr;
// provides `try_next`
use futures::TryStreamExt;

async fn sqlite_demo() -> Result<(), sqlx::Error> {
    let connection_options =
        SqliteConnectOptions::from_str("sqlite://data.db")?.create_if_missing(true);
    let pool = SqlitePoolOptions::new().max_connections(5).connect_with(connection_options).await?;
    sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;

    sqlx::query(r#"DROP TABLE IF EXISTS "bar""#).execute(&pool).await?;
    sqlx::query(r#"DROP TABLE IF EXISTS "foo""#).execute(&pool).await?;

    sqlx::query(
        format!(
            r#"CREATE TABLE "foo" (
                             "child" TEXT PRIMARY KEY
                           )"#
        )
        .as_str(),
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        format!(
            r#"CREATE TABLE "bar" (
                             "parent" TEXT,
                             "child" TEXT,
                             FOREIGN KEY ("child") REFERENCES "foo"("child")
                           )"#
        )
        .as_str(),
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"INSERT INTO "foo" ("child") VALUES ($1), ($2)"#)
        .bind("a")
        .bind("b")
        .execute(&pool)
        .await?;
    sqlx::query(r#"INSERT INTO "bar" ("parent", "child") VALUES ($1, $2), ($3, $4)"#)
        .bind("x")
        .bind("a")
        .bind("y")
        .bind("b")
        .execute(&pool)
        .await?;

    let mut rows =
        sqlx::query(r#"SELECT * FROM "bar" WHERE "child" LIKE $1"#).bind("%").fetch(&pool);

    while let Some(row) = rows.try_next().await? {
        // map the row into a user-defined domain type
        let parent: &str = row.try_get("parent")?;
        println!("Sqlite result (fetch) {:?}", parent);
    }

    println!("---------------------");

    let rows_all = sqlx::query(r#"SELECT * FROM "bar" WHERE "child" LIKE $1"#)
        .bind("%")
        .fetch_all(&pool)
        .await?;

    let second_row = &rows_all[1];
    let parent: &str = second_row.try_get("parent")?;
    println!("Sqlite result (fetch_all) {:?}", parent);

    println!("---------------------");

    let maybe_a_row = sqlx::query(r#"SELECT * FROM "bar" WHERE "child" LIKE $1"#)
        .bind("z")
        .fetch_optional(&pool)
        .await?;

    match maybe_a_row {
        Some(row) => {
            let parent: &str = row.try_get("parent")?;
            println!("Sqlite result (fetch_optional) {:?}", parent)
        }
        None => println!("Sqlite result (fetch_optional) No row"),
    };

    println!("---------------------");

    let definitely_a_row = sqlx::query(r#"SELECT * FROM "bar" WHERE "child" LIKE $1"#)
        .bind("a")
        .fetch_one(&pool)
        .await?;

    let parent: &str = definitely_a_row.try_get("parent")?;
    println!("Sqlite result fetch_one {:?}", parent);

    println!("---------------------");

    sqlx::query(r#"UPDATE "bar" SET "parent" = $1 WHERE "parent" = $2"#)
        .bind("z")
        .bind("x")
        .execute(&pool)
        .await?;

    Ok(())
}

async fn postgres_demo() -> Result<(), sqlx::Error> {
    let connection_options =
        PgConnectOptions::new().database("testdb").host("/var/run/postgresql/");
    let pool = PgPoolOptions::new().max_connections(5).connect_with(connection_options).await?;

    sqlx::query(r#"DROP TABLE IF EXISTS "bar""#).execute(&pool).await?;
    sqlx::query(r#"DROP TABLE IF EXISTS "foo""#).execute(&pool).await?;

    sqlx::query(
        format!(
            r#"CREATE TABLE "foo" (
                             "child" TEXT PRIMARY KEY
                           )"#
        )
        .as_str(),
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        format!(
            r#"CREATE TABLE "bar" (
                             "parent" TEXT,
                             "child" TEXT,
                             "cousin" JSON,
                             FOREIGN KEY ("child") REFERENCES "foo"("child")
                           )"#
        )
        .as_str(),
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"INSERT INTO "foo" ("child") VALUES ($1), ($2)"#)
        .bind("a")
        .bind("b")
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"INSERT INTO "bar" ("parent", "child", "cousin") VALUES ($1, $2, $3), ($4, $5, $6)"#,
    )
    .bind("x")
    .bind("a")
    .bind(json!({"value": "shugginses", "valid": true, "messages": {}}))
    .bind("y")
    .bind("b")
    .bind(json!({"value": "pugginses", "valid": true, "messages": {}}))
    .execute(&pool)
    .await?;

    let mut rows =
        sqlx::query(r#"SELECT * FROM "bar" WHERE "child" LIKE $1"#).bind("%").fetch(&pool);

    while let Some(row) = rows.try_next().await? {
        // map the row into a user-defined domain type
        let cousin: serde_json::Value = row.try_get("cousin")?;
        match cousin {
            serde_json::Value::Object(m) => println!("Postgresql result (fetch): {:?}", m),
            _ => panic!("Programming error!"),
        };
    }

    println!("---------------------");

    let rows_all = sqlx::query(r#"SELECT * FROM "bar" WHERE "child" LIKE $1"#)
        .bind("%")
        .fetch_all(&pool)
        .await?;

    let second_row = &rows_all[1];
    let parent: &str = second_row.try_get("parent")?;
    println!("Postgresql result (fetch_all) {:?}", parent);

    println!("---------------------");

    let maybe_a_row = sqlx::query(r#"SELECT * FROM "bar" WHERE "child" LIKE $1"#)
        .bind("z")
        .fetch_optional(&pool)
        .await?;

    match maybe_a_row {
        Some(row) => {
            let parent: &str = row.try_get("parent")?;
            println!("Postgresql result (fetch_optional) {:?}", parent)
        }
        None => println!("Postgresql result (fetch_optional) No row"),
    };

    println!("---------------------");

    let definitely_a_row = sqlx::query(r#"SELECT * FROM "bar" WHERE "child" LIKE $1"#)
        .bind("a")
        .fetch_one(&pool)
        .await?;

    let parent: &str = definitely_a_row.try_get("parent")?;
    println!("Postgresql result fetch_one {:?}", parent);

    println!("---------------------");

    sqlx::query(r#"UPDATE "bar" SET "parent" = $1 WHERE "parent" = $2"#)
        .bind("z")
        .bind("x")
        .execute(&pool)
        .await?;

    Ok(())
}

#[async_std::main]
async fn main() -> Result<(), sqlx::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 || &args[1] == "--help" || &args[1] == "-h" {
        eprintln!("Usage: sqlx_example <sqlite|postgres>");
        process::exit(1);
    }
    let db_type = &args[1];
    if db_type == "sqlite" {
        println!("Running sqlite demo ...");
        return sqlite_demo().await;
    } else if db_type == "postgres" {
        println!("Running postgres demo ...");
        return postgres_demo().await;
    } else {
        panic!("Unrecognized database type: {}", db_type);
    }
}
