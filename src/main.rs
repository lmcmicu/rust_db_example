use sqlx::Connection;
use sqlx::Row;
use sqlx::SqliteConnection;
// provides `try_next`
use futures::TryStreamExt;

#[async_std::main]
async fn main() -> Result<(), sqlx::Error> {
    let mut conn = SqliteConnection::connect("cmi-pb.db").await?;
    sqlx::query("PRAGMA foreign_keys = ON").execute(&mut conn).await?;
    let mut rows = sqlx::query("SELECT * FROM foobar WHERE child = ?")
        .bind("a")
        .fetch(&mut conn);

    while let Some(row) = rows.try_next().await? {
        // map the row into a user-defined domain type
        let parent: &str = row.try_get("parent")?;
        println!("{:?}", parent);
    }


    Ok(())
}
