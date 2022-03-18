use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Row;
// provides `try_next`
use futures::TryStreamExt;

#[async_std::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePoolOptions::new().max_connections(5).connect("cmi-pb.db").await?;
    sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;

    let mut rows = sqlx::query("SELECT * FROM foobar WHERE child LIKE ?").bind("%").fetch(&pool);

    while let Some(row) = rows.try_next().await? {
        // map the row into a user-defined domain type
        let parent: &str = row.try_get("parent")?;
        println!("FETCH {:?}", parent);
    }

    println!("---------------------");

    let rows_all =
        sqlx::query("SELECT * FROM foobar WHERE child LIKE ?").bind("%").fetch_all(&pool).await?;

    let second_row = &rows_all[1];
    let parent: &str = second_row.try_get("parent")?;
    println!("FETCH ALL {:?}", parent);

    println!("---------------------");

    let maybe_a_row = sqlx::query("SELECT * FROM foobar WHERE CHILD LIKE ?")
        .bind("z")
        .fetch_optional(&pool)
        .await?;

    match maybe_a_row {
        Some(row) => {
            let parent: &str = row.try_get("parent")?;
            println!("FETCH OPTIONAL {:?}", parent)
        }
        None => println!("FETCH OPTIONAL No row"),
    };

    println!("---------------------");

    let definitely_a_row =
        sqlx::query("SELECT * FROM foobar WHERE CHILD LIKE ?").bind("a").fetch_one(&pool).await?;

    let parent: &str = definitely_a_row.try_get("parent")?;
    println!("FETCH ONE {:?}", parent);

    println!("---------------------");

    Ok(())
}
